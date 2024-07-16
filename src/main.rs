use dotenvy::dotenv_iter;
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
            let msg = Some(args[2..].join(" "));
            if msg.is_none() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(" Invalid message format. {}", args[2..].join(" ")),
                ));
            }
            msg
        } else {
            None
        };

        match &*command_input.trim() {
            "set" => {
                if let Some(argument) = arg {
                    set_message(&argument)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Argument is missing or invalid!",
                    ))
                }
            }
            "push" => push(arg),
            "status" => display_status(),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unrecognized command: {}", command_input),
            )),
        }
    } else {
        display_status()
    }
}

pub fn display_status() -> io::Result<()> {
    match get_message() {
        Ok(message) => println!("planned commit message: {}", message),
        Err(_) => println!("no planned commit message. "),
    };
    Command::new("git")
        .arg("status")
        .spawn()
        .expect("user should have git installed");
    Ok(())
}
pub fn set_message(message: &str) -> io::Result<()> {
    let mut env_vars: HashMap<String, String> = match dotenv_iter() {
        Ok(dot) => dot.filter_map(Result::ok).collect(),
        Err(_) => HashMap::new(),
    };

    env_vars.insert(String::from("COMMIT_MESSAGE"), message.to_string());

    let mut file = fs::File::create(".env")?;
    for (k, v) in &env_vars {
        writeln!(file, "{}={}", k, v)?;
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
                let mut file = fs::File::create(".gitignore")?;
                writeln!(file, "{}", content)?;
                writeln!(file, ".env")?;
            }
        }
        Err(_) => {
            let mut file = fs::File::create(".gitignore")?;
            writeln!(file, ".env")?;
        }
    }
    println!("planned commit message set: {:?}", String::from(message));

    Ok(())
}

pub fn get_message() -> Result<String, io::Error> {
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
        Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
    }
}

fn clear_message() -> io::Result<()> {
    let mut env_vars: HashMap<String, String> = match dotenv_iter() {
        Ok(dot) => dot.filter_map(Result::ok).collect(),
        Err(_) => HashMap::new(),
    };
    env_vars.insert(String::from("COMMIT_MESSAGE"), String::new());

    let mut file = fs::File::create(".env")?;
    for (k, v) in &env_vars {
        writeln!(file, "{}={}", k, v)?;
    }
    Ok(())
}

pub fn push(contents: Option<String>) -> io::Result<()> {
    let commit_message = get_message()?;

    let files_to_push = match contents {
        Some(x) => x,
        None => String::from("."),
    };
    let add = Command::new("git")
        .arg("add")
        .arg(files_to_push.as_str())
        .status()
        .expect("command should be able to call git add");
    assert!(add.success());

    let commit = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&commit_message)
        .status()
        .expect("gim should be able to call git commit and commit message should be populated");
    assert!(commit.success());

    let push = Command::new("git")
        .arg("push")
        .status()
        .expect("gim should be able to call git push");
    assert!(push.success());

    clear_message()?;
    Ok(())
}
