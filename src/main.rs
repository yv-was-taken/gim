use dotenvy::dotenv_iter;
use edit::edit;
use std::collections::HashMap;
use std::fs;
use std::fs::read_to_string;
use std::io;
use std::io::Write;
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

fn parse_user_input(command_input: &String, arg: Option<String>) -> Result<(), io::Error> {
    match &*command_input.trim() {
        "set" => {
            if let Some(argument) = arg {
                set_message(&argument)
            } else {
                edit_message()
            }
        }
        "edit" => edit_message(),
        "push" => push(arg),
        "status" => display_status(),
        "clear" => clear_message(),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Unrecognized command: {}", command_input),
        )),
    }
}

fn display_status() -> io::Result<()> {
    match get_message() {
        Ok(message) => println!("Commit message: {:?}", message),
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
    let mut env_vars: HashMap<String, String> = match dotenv_iter() {
        Ok(env) => env.filter_map(Result::ok).collect(),
        Err(_) => HashMap::new(),
    };

    env_vars.insert(String::from("COMMIT_MESSAGE"), message.to_string());

    let mut file = match fs::File::create(".env") {
        Ok(file) => file,
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to create .env with err: {:#?}", err),
            ))
        }
    };
    for (k, v) in &env_vars {
        match writeln!(file, "{}={}", k, v) {
            Ok(_) => (),
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to write to .env with err: {:#?}", err),
                ))
            }
        };
    }

    match read_to_string(".gitignore") {
        Ok(content) => {
            let mut does_gitignore_contain_dotenv = false;
            for line in content.lines().filter(|line| !line.is_empty()) {
                if line == ".env" {
                    does_gitignore_contain_dotenv = true;
                }
            }
            if !does_gitignore_contain_dotenv {
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
                match writeln!(file, ".env") {
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
            match writeln!(file, ".env") {
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
    println!("Commit message set: {:?}", String::from(message));

    Ok(())
}

fn edit_message() -> io::Result<()> {
    let current_commit_message = match get_message() {
        Ok(message) => Some(message),
        Err(_) => None,
    };
    match current_commit_message {
        Some(message) => match edit(message) {
            Ok(m) => set_message(&m.trim()),
            Err(err) => Err(err),
        },
        None => match edit(" ") {
            Ok(m) => set_message(&m.trim()),
            Err(err) => return Err(err),
        },
    }
}

fn get_message() -> Result<String, io::Error> {
    match read_to_string(".env") {
        Ok(content) => {
            for line in content.lines().filter(|line| !line.is_empty()) {
                if let Some((key, value)) = line.split_once('=') {
                    if key == "COMMIT_MESSAGE" {
                        let message = String::from(value);
                        if !message.is_empty() {
                            return Ok(message);
                        } else {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                "No COMMIT_MESSAGE found",
                            ));
                        }
                    }
                }
            }
            Err(io::Error::new(
                io::ErrorKind::Other,
                "No COMMIT_MESSAGE found",
            ))
        }
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read .env with err: {:#?}", err),
            ))
        }
    }
}

fn clear_message() -> io::Result<()> {
    let mut env_vars: HashMap<String, String> = match dotenv_iter() {
        Ok(dot) => dot.filter_map(Result::ok).collect(),
        Err(_) => HashMap::new(),
    };
    env_vars.insert(String::from("COMMIT_MESSAGE"), String::new());

    let mut file = match fs::File::create(".env") {
        Ok(file) => file,
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to create .env with err: {:#?}", err),
            ))
        }
    };
    for (k, v) in &env_vars {
        match writeln!(file, "{}={}", k, v) {
            Ok(()) => (),
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to write to .env with err: {:#?}", err),
                ))
            }
        };
    }
    Ok(())
}

fn push(contents: Option<String>) -> io::Result<()> {
    let commit_message = match get_message() {
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

    let commit_command_output = match Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&commit_message)
        .output()
    {
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
        match clear_message() {
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
