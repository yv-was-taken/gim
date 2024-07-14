use std::io;
use std::str::FromStr;

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let mut input_iter = input.split_whitespace();
    let command = input_iter.next();
    let rest_of_input = input_iter.collect::<Vec<&str>>().join(" ");

    let arg = extract_quoted_string(&rest_of_input);

    if command.is_none() || (arg.is_none() && rest_of_input != "") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid input format!",
        ));
    }

    match command {
        Some("plan") => {
            if let Some(argument) = arg {
                plan(&argument)
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

pub fn plan(arg: &str) -> io::Result<()> {
    // @TODO write input arg as commit message, to repo-specific filename
    println!("your plan is: {}", arg);
    Ok(())
}

pub fn push() -> io::Result<()> {
    // @TODO read commit file, push it
    println!("Ahh, push it");
    Ok(())
}
