mod config;
mod git;
mod message;
mod display;
mod commands;
mod integration;

use std::io;
use crate::commands::{
    handle_config_command, handle_add_command, handle_status_command, 
    handle_reorder_command, help
};
use crate::git::{commit, push};
use crate::message::{set_message, edit_message, clear_message, append_instruction_comment, read_file_extract_comments};
use crate::display::display_status;
use crate::integration::handle_integrate_command;
use crate::git::find_git_root;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if let Some(command_input) = args.get(1) {
        parse_user_input(command_input, &args[2..])
    } else {
        display_status()
    }
}

fn parse_user_input(command_input: &String, args: &[String]) -> io::Result<()> {
    match command_input.trim() {
        "set" => {
            let message = args.join(" ");
            if message.trim().is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "no message argument provided",
                ));
            }

            let user_added_comments =
                read_file_extract_comments(find_git_root()?.join(".COMMIT_MESSAGE"))
                    .unwrap_or_default();

            set_message(&append_instruction_comment(
                &(message + &user_added_comments),
            ))
        }
        "edit" => edit_message(),
        "add" => handle_add_command(args),
        "push" => {
            let files = args.join(" ").trim().to_string();
            push(if files.is_empty() { None } else { Some(files) })
        }
        "commit" => {
            let files = args.join(" ").trim().to_string();
            commit(if files.is_empty() { None } else { Some(files) })
        }
        "status" => handle_status_command(args),
        "config" => handle_config_command(args),
        "integrate" => handle_integrate_command(),
        "reorder" => handle_reorder_command(),
        "clear" => {
            let should_full_clear = args.get(0).map_or(false, |arg| arg == "full");

            match clear_message(should_full_clear) {
                Ok(_) => {
                    if should_full_clear {
                        println!("Commit message fully cleared.");
                    } else {
                        println!("Commit message cleared.");
                    }
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }

        "help" => help(),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Unrecognized command: {command_input}"),
        )),
    }
}