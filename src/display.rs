use crate::config::load_config;
use crate::git::find_git_root;
use crate::message::get_message;
use std::fs::read_to_string;
use std::io;
use std::process::Command;

pub fn display_status() -> io::Result<()> {
    display_enhanced_commit_status()?;
    display_next_commits()?;

    // Show git status
    match Command::new("git").arg("status").spawn() {
        Ok(_) => {
            println!("\n");
            Ok(())
        }
        Err(err) => Err(io::Error::other(format!(
            "Failed to retrieve status with err: {err:#?}"
        ))),
    }
}

pub fn display_enhanced_commit_status() -> io::Result<()> {
    let config = load_config();

    match get_message(true) {
        Ok(message) => {
            let lines: Vec<&str> = message.trim().lines().collect();
            if lines.is_empty() {
                println!("No commit message set.");
                return Ok(());
            }

            let bold_start = "\x1b[1m";
            let bold_end = "\x1b[0m";
            let italic_start = "\x1b[3m";
            let italic_end = "\x1b[0m";

            if config.verbose {
                // Verbose format: show labels
                println!("Current commit:");
                println!("    {}Title: {}{}", bold_start, lines[0], bold_end);

                // Display description if present
                if lines.len() > 1 {
                    let description: Vec<&str> = lines[1..]
                        .iter()
                        .filter(|line| !line.trim().is_empty())
                        .cloned()
                        .collect();

                    if !description.is_empty() {
                        println!("    Description:");
                        for line in description {
                            println!("        {}{}{}", italic_start, line.trim(), italic_end);
                        }
                    }
                }
            } else {
                // Clean format: no labels, just styling
                println!("Current commit:");
                println!("    {}{}{}", bold_start, lines[0], bold_end);

                // Display description if present
                if lines.len() > 1 {
                    let description: Vec<&str> = lines[1..]
                        .iter()
                        .filter(|line| !line.trim().is_empty())
                        .cloned()
                        .collect();

                    if !description.is_empty() {
                        for line in description {
                            println!("    {}{}{}", italic_start, line.trim(), italic_end);
                        }
                    }
                }
            }
        }
        Err(_) => println!("No commit message set."),
    }
    Ok(())
}

pub fn display_next_commits() -> io::Result<()> {
    let content = match read_to_string(find_git_root()?.join(".COMMIT_MESSAGE")) {
        Ok(content) => content,
        Err(_) => return Ok(()),
    };

    let mut next_commits = Vec::new();
    for line in content.lines() {
        if let Some(captures) = line.strip_prefix("# NEXT-") {
            if let Some(colon_pos) = captures.find(':') {
                if let Ok(num) = captures[..colon_pos].parse::<usize>() {
                    let message = captures[colon_pos + 1..].trim();
                    next_commits.push((num, message));
                }
            }
        }
    }

    if !next_commits.is_empty() {
        next_commits.sort_by_key(|&(num, _)| num);
        println!("\nUpcoming commits:");
        for (num, message) in next_commits {
            println!("    {num}: {message}");
        }
    }

    Ok(())
}

pub fn display_full_status() -> io::Result<()> {
    match get_message(false) {
        Ok(message) => {
            println!("Complete .COMMIT_MESSAGE file contents:");
            println!("{message}");
        }
        Err(_) => println!("No commit message file found."),
    }
    Ok(())
}
