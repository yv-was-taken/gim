use edit::edit;
use std::fs;
use std::fs::read_to_string;
use std::fs::File;
use std::io;
use std::io::Write;
use std::io::{BufRead, BufReader, Result};
use std::process::Command;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if let Some(command_input) = args.get(1) {
        let arg = if args.len() > 2 {
            Some(args[2..].join(" "))
        } else {
            None
        };

        parse_user_input(command_input, arg)
    } else {
        display_status()
    }
}

fn parse_user_input(command_input: &String, arg: Option<String>) -> io::Result<()> {
    match &*command_input.trim() {
        "set" | "edit" => {
            match get_message(false) {
                Ok(_) => (),
                //if empty, inject comment into .COMMIT_MESSAGE
                Err(_) => match clear_message(true) {
                    Ok(_) => (),
                    Err(err) => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to inject .COMMIT_MESSAGE with err: {:#?}", err),
                        ))
                    }
                },
            };
            if let Some(argument) = arg {
                set_message(&argument)
            } else {
                edit_message()
            }
        }
        "push" => push(arg),
        "status" => display_status(),
        "clear" => {
            let should_full_clear = match arg {
                Some(_arg) => match &*_arg {
                    "full" => true,
                    _ => false,
                },
                None => false,
            };

            match clear_message(should_full_clear) {
                Ok(_) => {
                    if should_full_clear {
                        println!("Commit message cleared fully.");
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
            format!("Unrecognized command: {}", command_input),
        )),
    }
}

fn display_status() -> io::Result<()> {
    match get_message(true) {
        Ok(message) => print_formatted_message(String::from("Commit message: "), message),
        Err(_) => println!("No commit message set."),
    };
    match Command::new("git").arg("status").spawn() {
        Ok(_) => Ok(()),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to retrieve status with err: {:#?}", err),
        )),
    }
}

fn set_message(message: &str) -> io::Result<()> {
    let mut file = match fs::File::create(".COMMIT_MESSAGE") {
        Ok(file) => file,
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to create .COMMIT_MESSAGE with err: {:#?}", err),
            ))
        }
    };
    match write!(file, "{}", message) {
        Ok(_) => (),
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write to .COMMIT_MESSAGE with err: {:#?}", err),
            ))
        }
    };
    //renaming for console log at the end of this function.
    let message_to_log_to_console = match get_message(true) {
        Ok(message) => message,
        Err(err) => return Err(err),
    };

    match read_to_string(".gitignore") {
        Ok(content) => {
            let mut does_gitignore_contain_commit_message = false;
            for line in content.lines().filter(|line| !line.is_empty()) {
                if line == ".COMMIT_MESSAGE" {
                    does_gitignore_contain_commit_message = true;
                }
            }
            if !does_gitignore_contain_commit_message {
                let mut file = match fs::File::create(".gitignore") {
                    Ok(file) => file,
                    Err(err) => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to create .gitignore with err: {:#?}", err),
                        ))
                    }
                };
                match writeln!(file, "{}", content) {
                    Ok(_) => (),
                    Err(err) => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to write to .gitignore with err: {:#?}", err),
                        ))
                    }
                };
                match writeln!(file, ".COMMIT_MESSAGE") {
                    Ok(_) => (),
                    Err(err) => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to write to .gitignore with err: {:#?}", err),
                        ))
                    }
                };
            }
        }
        Err(_) => {
            let mut file = match fs::File::create(".gitignore") {
                Ok(file) => file,
                Err(err) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to create .gitignore with err: {:#?}", err),
                    ))
                }
            };
            match writeln!(file, ".COMMIT_MESSAGE") {
                Ok(_) => (),
                Err(err) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to create .env with err: {:#?}", err),
                    ))
                }
            };
        }
    };
    let message_string = String::from(message);
    if message_string.trim().is_empty() {
        println!("Commit message was empty, aborted.");
    } else {
        print_formatted_message(
            String::from("Commit message set:"),
            message_to_log_to_console,
        );
    }

    Ok(())
}

fn print_formatted_message(message_title: String, message: String) {
    let bold_text_start = "\x1b[1m";
    let indented_message: String = format!(r#"{message}"#)
        .lines()
        .map(|line| format!("{}{}", "    ", line))
        .collect::<Vec<String>>()
        .join("\n");
    let bold_text_end = "\x1b[0m";

    println!("{}", &message_title);
    println!("{}{}{}", bold_text_start, indented_message, bold_text_end);
}

fn edit_message() -> io::Result<()> {
    let current_commit_message = match get_message(false) {
        Ok(message) => Some(message),
        Err(_) => None,
    };
    match current_commit_message {
        Some(message) => match edit(message) {
            Ok(m) => set_message(&m),
            Err(err) => Err(err),
        },
        None => match edit(" ") {
            Ok(m) => set_message(&m),
            Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidInput, err)),
        },
    }
}

fn get_message(ignore_comments: bool) -> io::Result<String> {
    if ignore_comments {
        match read_file_ignore_comments(".COMMIT_MESSAGE") {
            Ok(content) => {
                if content.trim().is_empty() {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        "No commit message found",
                    ))
                } else {
                    Ok(content)
                }
            }
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to read .env with err: {:#?}", err),
                ))
            }
        }
    } else {
        match read_to_string(".COMMIT_MESSAGE") {
            Ok(content) => {
                if content.trim().is_empty() {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        "No commit message found",
                    ))
                } else {
                    Ok(content)
                }
            }
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to read .env with err: {:#?}", err),
                ))
            }
        }
    }
}

fn clear_message(is_full_clear: bool) -> io::Result<()> {
    if is_full_clear {
        let mut file = match fs::File::create(".COMMIT_MESSAGE") {
            Ok(file) => file,
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to create .COMMIT_MESSAGE with err: {:#?}", err),
                ))
            }
        };
        match write!(
            file,
            r#"
# Enter/edit the commit message for your changes.
# Lines starting with '#' are considered comments, therefore are ignored, and will sustain through commits.
"#
        ) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    } else {
        let comments = match read_file_extract_comments(".COMMIT_MESSAGE") {
            Ok(file) => format!(r#"{file}"#),
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to read .COMMIT_MESSAGE with err: {:#?}", err),
                ))
            }
        };
        let mut file = match fs::File::create(".COMMIT_MESSAGE") {
            Ok(file) => file,
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to create .COMMIT_MESSAGE with err: {:#?}", err),
                ))
            }
        };
        match write!(file, r#"{comments}"#) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

fn read_file_ignore_comments(file_path: &str) -> Result<String> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => return Err(err),
    };
    let reader = BufReader::new(file);
    let mut content = String::new();

    for line in reader.lines() {
        let line = match line {
            Ok(text) => text,
            Err(err) => return Err(err),
        };
        if !line.trim_start().starts_with('#') {
            content.push_str(&line);
            content.push('\n');
        }
    }

    Ok(content)
}

fn read_file_extract_comments(file_path: &str) -> Result<String> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => return Err(err),
    };
    let reader = BufReader::new(file);
    let mut content = String::new();

    for line in reader.lines() {
        let line = match line {
            Ok(text) => text,
            Err(err) => return Err(err),
        };
        if line.trim_start().starts_with('#') {
            content.push_str(&line);
            content.push('\n');
        }
    }

    Ok(content)
}

fn push(contents: Option<String>) -> io::Result<()> {
    let commit_message = match get_message(true) {
        Ok(message) => message,
        Err(err) => return Err(err),
    };

    let files_to_push = match contents {
        Some(x) => x,
        None => String::from("."),
    };
    match Command::new("git")
        .arg("add")
        .arg(files_to_push.as_str())
        .status()
    {
        Ok(_) => (),
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to add files with err: {:#?}", err),
            ))
        }
    };

    let mut commit_message_header_and_body = commit_message.split("\n").into_iter();

    //can unwrap header since we know there is some text present, if empty, function would have returned err already and would have not been able to reach this far downstream.
    let commit_message_header = commit_message_header_and_body.next().unwrap();
    //cannot unwrap body as there may or may not be further text present.
    let commit_message_body = commit_message_header_and_body
        .collect::<Vec<&str>>()
        .join("\n")
        .to_string();

    let mut commit_command_args = Vec::new();
    commit_command_args.push("commit");
    commit_command_args.push("-m");
    commit_command_args.push(&commit_message_header);
    if !commit_message_body.is_empty() {
        commit_command_args.push("-m");
        commit_command_args.push(&commit_message_body);
    }

    let commit_command_output = match Command::new("git").args(commit_command_args).output() {
        Ok(console_output) => match String::from_utf8(console_output.stdout) {
            Ok(output) => {
                println!("{output}");
                output
            }
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to parse commit stdout with err: {:#?}", err),
                ))
            }
        },
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to commit changes with err: {:#?}", err),
            ))
        }
    };

    if !commit_command_output.contains("nothing to commit, working tree clean") {
        match clear_message(false) {
            Ok(_) => (),
            Err(err) => return Err(err),
        }
    };

    match Command::new("git").arg("push").status() {
        Ok(_) => Ok(()),
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to push changes with err: {:#?}", err),
            ))
        }
    }
}

fn help() -> io::Result<()> {
    let help_message = r#"
`gim` provides the following commands:

### `gim set {COMMIT_MESSAGE}`

- Accepts a string argument for the planned commit message.
- The commit message is stored inside the `.COMMIT_MESSAGE` file.
    > **Note**: Don't worry about adding a `.COMMIT_MESSAGE` file yourself (or adding it to `.gitignore`), `gim` takes care of that for you!

### `gim edit`

- Opens system default editor to edit current commit message

### `gim push`

- Equivalent to `git add . && git commit -m $COMMIT_MESSAGE && git push`.
- Allows optional inclusion of files after push, similar to `git add $FILE`. Defaults to `.`
- Upon a successful push, the `.COMMIT_MESSAGE` file is cleared.
### `gim status` or just `gim`

- Displays the current `gim` planned commit message at the top of the normal `git status` output.

### `gim clear`

- Clears the stored commit message.

### `gim clear full`

- Fully clears the stored commit message, comments included.

### `gim help`

- Prints the command descriptions to the console.
"#;
    println!("{help_message}");
    Ok(())
}
