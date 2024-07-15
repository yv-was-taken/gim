use dotenvy::dotenv_iter;
use std::collections::HashMap;
use std::fs;
use std::fs::read_to_string;
use std::io;
use std::io::Write;
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
        Some("set") => {
            if let Some(argument) = arg {
                set_message(&argument)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Argument is missing or invalid!",
                ))
            }
        }
        Some("push") => push(),
        Some("status") | None => {
            let message = get_message()?;

            println!("current commit message: {:}", message);
            Command::new("git")
                .arg("status")
                .spawn()
                .expect("user should have git installed");
            Ok(())
        }
        Some(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unrecognized command",
        )),
    }
}

pub fn set_message(message: &str) -> io::Result<()> {
    let mut env_vars: HashMap<String, String> = match dotenv_iter() {
        Ok(dot) => dot.filter_map(Result::ok).collect(),
        Err(_) => HashMap::new(),
    };
    //write commit message to .env
    env_vars.insert(String::from("COMMIT_MESSAGE"), message.to_string());

    let mut file = fs::File::create(".env")?;
    for (k, v) in &env_vars {
        writeln!(file, "{}={}", k, v)?;
    }
    //add .env to .gitignore if not already there
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

    Ok(())
}

pub fn get_message() -> Result<String, io::Error> {
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
        Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
    }
}

pub fn push() -> io::Result<()> {
    let commit_message = get_message()?;

    let add = Command::new("git")
        .arg("add")
        .arg(".")
        .status()
        .expect("command should be able to call git add");

    assert!(add.success());

    let commit = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(wrap_in_quotes(&commit_message))
        .status()
        .expect("gim should be able to call git commit and commit message should be populated");

    assert!(commit.success());

    let push = Command::new("git")
        .arg("push")
        .status()
        .expect("gim should be able to call git push");

    assert!(push.success());

    set_message("")?;
    Ok(())
}

fn wrap_in_quotes(input: &str) -> String {
    format!("\"{}\"", input)
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
