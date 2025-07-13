use crate::message::{advance_to_next_commit, get_message};
use std::io;
use std::io::Result;
use std::process::Command;

pub fn find_git_root() -> Result<std::path::PathBuf> {
    // Get the current directory
    let mut dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("error getting current dir: {err}"),
            ))
        }
    };
    loop {
        if dir.join(".git").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            // If `pop()` returns false, it means we reached the root directory
            break;
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "git directory not found",
    ))
}

pub fn commit(contents: Option<String>) -> io::Result<()> {
    let commit_message = get_message(true)?;

    // Handle file specification or use git add -A
    let add_result = match contents {
        Some(files) if !files.trim().is_empty() => {
            let files_list: Vec<&str> = files.split_whitespace().collect();
            let mut command = Command::new("git");
            command.arg("add");
            for file in files_list {
                command.arg(file);
            }
            command.output()
        }
        _ => Command::new("git").args(["add", "-A"]).output(),
    };

    match add_result {
        Ok(add_output) => {
            if !add_output.status.success() {
                let stderr = String::from_utf8_lossy(&add_output.stderr);
                return Err(io::Error::other(format!("Git add failed: {stderr}")));
            }
        }
        Err(err) => return Err(io::Error::other(format!("Failed to run git add: {err}"))),
    }

    // Run git commit
    let commit_output = match Command::new("git")
        .args(["commit", "-m", &commit_message])
        .output()
    {
        Ok(output) => output,
        Err(err) => return Err(io::Error::other(format!("Failed to run git commit: {err}"))),
    };

    // Check if commit failed and capture both stdout and stderr
    if !commit_output.status.success() {
        let stdout = String::from_utf8_lossy(&commit_output.stdout);
        let stderr = String::from_utf8_lossy(&commit_output.stderr);

        // Print the pre-commit hook output to help user debug
        if !stdout.is_empty() {
            println!("Git commit output:\n{stdout}");
        }
        if !stderr.is_empty() {
            println!("Git commit errors:\n{stderr}");
        }

        return Err(io::Error::other(
            "Git commit failed (possibly due to pre-commit hooks)",
        ));
    }

    // Only clear the message if the commit was successful
    if let Ok(stdout) = String::from_utf8(commit_output.stdout) {
        if !stdout.contains("nothing to commit, working tree clean") {
            advance_to_next_commit()?;
        }
    }

    println!("Changes committed.");
    Ok(())
}

pub fn push(contents: Option<String>) -> io::Result<()> {
    let commit_message = get_message(true)?;

    // Handle file specification or use git add -A
    let add_result = match contents {
        Some(files) if !files.trim().is_empty() => {
            let files_list: Vec<&str> = files.split_whitespace().collect();
            let mut command = Command::new("git");
            command.arg("add");
            for file in files_list {
                command.arg(file);
            }
            command.output()
        }
        _ => Command::new("git").args(["add", "-A"]).output(),
    };

    match add_result {
        Ok(add_output) => {
            if !add_output.status.success() {
                let stderr = String::from_utf8_lossy(&add_output.stderr);
                return Err(io::Error::other(format!("Git add failed: {stderr}")));
            }
        }
        Err(err) => return Err(io::Error::other(format!("Failed to run git add: {err}"))),
    }

    // Run git commit
    let commit_output = match Command::new("git")
        .args(["commit", "-m", &commit_message])
        .output()
    {
        Ok(output) => output,
        Err(err) => return Err(io::Error::other(format!("Failed to run git commit: {err}"))),
    };

    // Check if commit failed and capture both stdout and stderr
    if !commit_output.status.success() {
        let stdout = String::from_utf8_lossy(&commit_output.stdout);
        let stderr = String::from_utf8_lossy(&commit_output.stderr);

        // Print the pre-commit hook output to help user debug
        if !stdout.is_empty() {
            println!("Git commit output:\n{stdout}");
        }
        if !stderr.is_empty() {
            println!("Git commit errors:\n{stderr}");
        }

        return Err(io::Error::other(
            "Git commit failed (possibly due to pre-commit hooks)",
        ));
    }

    // Only clear the message if the commit was successful
    if let Ok(stdout) = String::from_utf8(commit_output.stdout) {
        if !stdout.contains("nothing to commit, working tree clean") {
            advance_to_next_commit()?;
        }
    }

    // Run git push
    let mut push_child = match Command::new("git").arg("push").spawn() {
        Ok(child) => child,
        Err(err) => {
            return Err(io::Error::other(format!(
                "Failed to run git push: {err}"
            )))
        }
    };

    // Wait for git push to complete
    let push_status = match push_child.wait() {
        Ok(status) => status,
        Err(err) => {
            return Err(io::Error::other(format!(
                "Failed to wait for git push: {err}"
            )))
        }
    };

    // Check if push succeeded
    if !push_status.success() {
        return Err(io::Error::other("Git push failed"));
    }

    println!("Changes pushed successfully.");
    Ok(())
}
