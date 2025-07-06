use crate::config::load_config;
use crate::display::display_enhanced_commit_status;
use crate::git::find_git_root;
use std::fs;
use std::fs::read_to_string;
use std::io;
use std::io::Result;
use std::io::Write;
use std::process::Command;

pub fn set_message(message_to_set: &str, print_status_update: bool) -> io::Result<()> {
    let mut file = match fs::File::create(find_git_root()?.join(".COMMIT_MESSAGE")) {
        Ok(file) => file,
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("failed to create .COMMIT_MESSAGE file: {err}"),
            ))
        }
    };

    match write!(file, "{}", message_to_set) {
        Ok(_) => (),
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("failed to write to .COMMIT_MESSAGE file: {err}"),
            ))
        }
    };

    if print_status_update {
        println!("Commit message set:");
        let _ = display_enhanced_commit_status();
    }

    Ok(())
}

pub fn edit_with_configured_editor(content: &str) -> io::Result<String> {
    let config = load_config();

    // Create a temporary file
    let temp_file = std::env::temp_dir().join(format!("gim_edit_{}.tmp", std::process::id()));

    // Write content to temp file
    fs::write(&temp_file, content)?;

    // Launch the configured editor
    let status = Command::new(&config.editor).arg(&temp_file).status()?;

    if !status.success() {
        fs::remove_file(&temp_file).ok(); // Clean up on failure
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Editor '{}' exited with non-zero status", config.editor),
        ));
    }

    // Read the edited content
    let edited_content = fs::read_to_string(&temp_file)?;

    // Clean up temp file
    fs::remove_file(&temp_file).ok();

    Ok(edited_content)
}

pub fn edit_message() -> io::Result<()> {
    let current_commit_message = get_message(false).ok();
    match current_commit_message {
        Some(message) => match edit_with_configured_editor(&message) {
            Ok(m) => set_message(&m, true),
            Err(err) => Err(err),
        },
        None => match edit_with_configured_editor(" ") {
            Ok(m) => set_message(&append_instruction_comment(&m), true),
            Err(err) => Err(err),
        },
    }
}

pub fn get_message(ignore_comments: bool) -> io::Result<String> {
    if ignore_comments {
        match read_file_extract_message(find_git_root()?.join(".COMMIT_MESSAGE")) {
            Ok(content) => {
                if content.trim().is_empty() {
                    Err(io::Error::other("No commit message found"))
                } else {
                    Ok(content)
                }
            }
            Err(err) => Err(err),
        }
    } else {
        match read_to_string(find_git_root()?.join(".COMMIT_MESSAGE")) {
            Ok(content) => {
                if content.trim().is_empty() {
                    Err(io::Error::other("No commit message found"))
                } else {
                    Ok(content)
                }
            }
            Err(err) => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("failed to read .COMMIT_MESSAGE file: {err}"),
            )),
        }
    }
}

pub fn clear_message(is_full_clear: bool) -> io::Result<()> {
    if is_full_clear {
        match fs::remove_file(find_git_root()?.join(".COMMIT_MESSAGE")) {
            Ok(_) => Ok(()),
            Err(err) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("failed to clear .COMMIT_MESSAGE file: {err}"),
            )),
        }
    } else {
        let user_added_comments =
            read_file_extract_comments(find_git_root()?.join(".COMMIT_MESSAGE"))
                .unwrap_or_default();

        match set_message(&append_instruction_comment(&user_added_comments), false) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

pub fn append_instruction_comment(message: &str) -> String {
    format!(
        r#"{}

# Enter/edit the commit message for your changes.
# Lines starting with '#' are considered comments, therefore are ignored, and will not be cleared after pushing commits.
#
# Special comment keywords for commit planning:
#   NEXT-N: <message>    - Planned upcoming commits that auto-advance after current commit
#                          Numbers are automatically managed, use 'gim add --next' to add
#                          Use 'gim reorder' to reorder or comment out planned commits
#   COMMENTED: <message> - Previously planned commits that were commented out for safekeeping
#                          These won't auto-advance but are preserved for reference
#
# Examples:
#   # NEXT-1: implement user registration
#   # NEXT-2: add password validation
#   # COMMENTED: old feature that was deprioritized
"#,
        message.trim()
    )
}

pub fn read_file_extract_message(file_path: std::path::PathBuf) -> Result<String> {
    let content = read_to_string(file_path)?;

    let non_comment_lines: Vec<&str> = content
        .lines()
        .filter(|line| !line.trim().starts_with('#'))
        .collect();

    Ok(non_comment_lines.join("\n").trim().to_string())
}

pub fn read_file_extract_comments(file_path: std::path::PathBuf) -> Result<String> {
    let content = read_to_string(file_path)?;

    // Extract only user comments (not the instruction comments)
    let mut user_comments = Vec::new();
    let mut in_instruction_section = false;

    for line in content.lines() {
        if line.starts_with("# Enter/edit the commit message") {
            in_instruction_section = true;
            continue;
        }

        if line.trim().starts_with('#') && !in_instruction_section {
            user_comments.push(line);
        }
    }

    Ok(user_comments.join("\n"))
}

pub fn advance_to_next_commit() -> io::Result<()> {
    let content = match read_to_string(find_git_root()?.join(".COMMIT_MESSAGE")) {
        Ok(content) => content,
        Err(_) => {
            // No file exists, just clear
            clear_message(false)?;
            return Ok(());
        }
    };

    // Find the lowest numbered NEXT commit
    let mut next_commits = Vec::new();
    let mut other_lines = Vec::new();

    for line in content.lines() {
        if let Some(captures) = line.strip_prefix("# NEXT-") {
            if let Some(colon_pos) = captures.find(':') {
                if let Ok(num) = captures[..colon_pos].parse::<usize>() {
                    let message = captures[colon_pos + 1..].trim();
                    next_commits.push((num, message.to_string()));
                } else {
                    other_lines.push(line);
                }
            } else {
                other_lines.push(line);
            }
        } else {
            other_lines.push(line);
        }
    }

    if next_commits.is_empty() {
        // No next commits, just clear normally
        clear_message(false)?;
        return Ok(());
    }

    // Sort by number and take the first one
    next_commits.sort_by_key(|&(num, _)| num);
    let (_, next_message) = next_commits.remove(0);

    // Update the file with the next commit as current, and remaining nexts renumbered
    let mut new_content = String::new();

    // Set the next commit as the current message
    new_content.push_str(&next_message);
    new_content.push('\n');

    // Renumber remaining next commits
    for (i, (_, message)) in next_commits.iter().enumerate() {
        new_content.push_str(&format!("\n# NEXT-{}: {}", i + 1, message));
    }

    // Add back other comments (keeping user comments)
    for line in other_lines {
        if line.trim().starts_with('#') && !line.starts_with("# Enter/edit the commit message") {
            new_content.push('\n');
            new_content.push_str(line);
        }
    }

    // Add instruction comment
    new_content = append_instruction_comment(&new_content);

    // Write the updated content
    let mut file = fs::File::create(find_git_root()?.join(".COMMIT_MESSAGE"))?;
    write!(file, "{}", new_content)?;

    println!("Advanced to next commit: {}", next_message);
    Ok(())
}
