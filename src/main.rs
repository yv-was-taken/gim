use dotenv;
use std::fs::{read_to_string, write};
use std::io;
use std::process::Command;
use std::str::FromStr;

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let mut input_iter = input.split_whitespace();
    let command_input = input_iter.next();
    let rest_of_input = input_iter.collect::<Vec<&str>>().join(" ");

    let arg = extract_quoted_string(&rest_of_input);

    if command_input.is_none() || (arg.is_none() && rest_of_input != "") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid input format!",
        ));
    }

    match command_input {
        Some("plan") => {
            if let Some(argument) = arg {
                set_plan(&argument)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Argument is missing or invalid!",
                ))
            }
        }
        Some("push") => push(),
        Some(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unrecognized command",
        )),
        None => {
            println!("standard git status type thing here");
            Ok(())
        }
    }
}

fn extract_quoted_string(input: &str) -> Option<String> {
    if input.starts_with('"') && input.ends_with('"') {
        Some(String::from_str(&input[1..input.len() - 1]).unwrap())
    } else if input.starts_with('\'') && input.ends_with('\'') {
        Some(String::from_str(&input[1..input.len() - 1]).unwrap())
    } else {
        None
    }
}

pub fn set_plan(arg: &str) -> io::Result<()> {
    // @TODO write input arg as commit message, to `.env`

    println!("your plan is: {}", arg);
    Ok(())
}

pub fn get_plan() -> Result<String, io::Error> {
    match read_to_string(".env") {
        Ok(content) => {
            for line in content.lines().filter(|line| !line.is_empty()) {
                if let Some((key, value)) = line.split_once('=') {
                    if key == "COMMIT_MESSAGE" {
                        return Ok(String::from(value));
                    }
                }
            }
            Err(io::Error::new(
                io::ErrorKind::Other,
                "No COMMIT_MESSAGE found",
            ))
        }
        Err(err) => Err(err),
    }
}

fn wrap_in_quotes(input: &str) -> String {
    format!("\"{}\"", input)
}

pub fn push() -> io::Result<()> {
    let commit_message = get_plan()?;
    Command::new("git")
        .arg("add")
        .arg(".")
        .arg("&&")
        .arg("commit")
        .arg("-m")
        .arg(wrap_in_quotes(&commit_message))
        .spawn()
        .expect("this command should have executed, but something went wrong. are you sure you set the commit message and have git installed?.");

    println!("Ahh, push it");
    Ok(())
}
