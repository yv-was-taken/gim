use crate::config::{get_config_path, load_config, save_config};
use crate::display::{display_enhanced_commit_status, display_full_status};
use crate::git::find_git_root;
use crate::message::{
    append_instruction_comment, edit_with_configured_editor, get_message,
    read_file_extract_comments, read_file_extract_message, set_message,
};
use std::fs;
use std::fs::read_to_string;
use std::io;
use std::io::Write;
use std::io::{stdin, stdout, BufRead};

pub fn handle_config_command(args: &[String]) -> io::Result<()> {
    if args.is_empty() {
        // Show current config settings
        let config = load_config();
        let config_path = get_config_path()?;

        println!("Gim Configuration");
        println!("Location: {}", config_path.display());
        println!();
        println!("Current settings:");
        println!(
            "  default_add_as_desc = {} ({})",
            config.default_add_as_desc,
            if config.default_add_as_desc {
                "'gim add' creates descriptions by default"
            } else {
                "'gim add' adds to title by default"
            }
        );
        println!(
            "  verbose = {} ({})",
            config.verbose,
            if config.verbose {
                "shows Title:/Description: labels"
            } else {
                "clean format without labels"
            }
        );
        println!(
            "  editor = \"{}\" (editor used for 'gim edit' and 'gim config edit')",
            config.editor,
        );
        println!();
        println!("Use 'gim config edit' to modify these settings");
        return Ok(());
    }

    match args[0].as_str() {
        "edit" => {
            let config_path = get_config_path()?;

            // Ensure config file exists with current settings
            let config = load_config();
            save_config(&config)?;

            // Use the configured editor
            let config_content = read_to_string(&config_path).unwrap_or_default();

            match edit_with_configured_editor(&config_content) {
                Ok(new_content) => {
                    // Write the edited content back
                    let mut file = fs::File::create(&config_path)?;
                    write!(file, "{new_content}")?;

                    // Validate the new config by trying to parse it
                    let new_config = load_config();
                    println!("Configuration updated successfully!");
                    println!("  default_add_as_desc = {}", new_config.default_add_as_desc);
                    println!("  verbose = {}", new_config.verbose);
                    println!("  editor = \"{}\"", new_config.editor);
                }
                Err(err) => {
                    return Err(io::Error::other(format!("Failed to edit config: {err}")));
                }
            }
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Usage: gim config [edit]",
            ));
        }
    }

    Ok(())
}

pub fn handle_reorder_command() -> io::Result<()> {
    let content = match read_to_string(find_git_root()?.join(".COMMIT_MESSAGE")) {
        Ok(content) => content,
        Err(_) => {
            println!("No commit message file found.");
            return Ok(());
        }
    };

    // Extract next commits
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
        println!("No upcoming commits to reorder.");
        return Ok(());
    }

    next_commits.sort_by_key(|&(num, _)| num);

    println!("Current upcoming commits:");
    for (i, (_, message)) in next_commits.iter().enumerate() {
        println!("  {}: {}", i + 1, message);
    }

    println!("\nReorder options:");
    println!("  Enter new order as numbers separated by spaces (e.g., '2 1 3')");
    println!("  Or enter 'c' to comment out a commit (e.g., 'c 2' to comment out commit 2)");
    println!("  Or enter 'cancel' to abort");
    print!("New order: ");

    let _ = stdout().flush();
    let stdin = stdin();
    let mut line = String::new();

    if stdin.lock().read_line(&mut line).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Failed to read input",
        ));
    }

    let input = line.trim();

    if input == "cancel" {
        println!("Reorder cancelled.");
        return Ok(());
    }

    // Handle comment command
    if input.starts_with("c ") {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() == 2 {
            if let Ok(index) = parts[1].parse::<usize>() {
                if index > 0 && index <= next_commits.len() {
                    let (_, message) = &next_commits[index - 1];
                    println!("Commenting out: {message}");

                    // Remove the commit from next_commits and add as comment
                    let commented_commit = next_commits.remove(index - 1);

                    // Rebuild file content
                    rebuild_commit_file_with_reorder(
                        &other_lines,
                        &next_commits,
                        Some(&format!("# COMMENTED: {}", commented_commit.1)),
                    )?;
                    println!("Commit commented out successfully.");
                    return Ok(());
                }
            }
        }
        println!(
            "Invalid comment command. Use 'c <number>' where number is 1-{}",
            next_commits.len()
        );
        return Ok(());
    }

    // Handle reorder
    let new_order: std::result::Result<Vec<usize>, _> = input
        .split_whitespace()
        .map(|s| s.parse::<usize>())
        .collect();

    match new_order {
        Ok(order) => {
            if order.len() != next_commits.len() {
                println!(
                    "Error: Must specify exactly {} positions",
                    next_commits.len()
                );
                return Ok(());
            }

            // Check if all numbers are valid and unique
            let mut sorted_order = order.clone();
            sorted_order.sort();
            let expected: Vec<usize> = (1..=next_commits.len()).collect();

            if sorted_order != expected {
                println!(
                    "Error: Must use each number from 1 to {} exactly once",
                    next_commits.len()
                );
                return Ok(());
            }

            // Reorder the commits
            let mut reordered_commits = Vec::new();
            for &pos in &order {
                reordered_commits.push(next_commits[pos - 1].clone());
            }

            // Rebuild file
            rebuild_commit_file_with_reorder(&other_lines, &reordered_commits, None)?;

            println!("Commits reordered successfully!");
            println!("New order:");
            for (i, (_, message)) in reordered_commits.iter().enumerate() {
                println!("  {}: {}", i + 1, message);
            }
        }
        Err(_) => {
            println!("Error: Invalid input. Enter numbers separated by spaces.");
        }
    }

    Ok(())
}

pub fn rebuild_commit_file_with_reorder(
    other_lines: &[&str],
    next_commits: &[(usize, String)],
    additional_comment: Option<&str>,
) -> io::Result<()> {
    let mut new_content = String::new();

    // Add the current commit message (non-comment, non-NEXT lines)
    let mut in_instruction_section = false;
    for &line in other_lines {
        if line.starts_with("# Enter/edit the commit message") {
            in_instruction_section = true;
        }

        if !line.trim().starts_with('#')
            || (!in_instruction_section
                && line.trim().starts_with('#')
                && !line.starts_with("# Enter/edit"))
        {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    // Add additional comment if provided
    if let Some(comment) = additional_comment {
        new_content.push_str(comment);
        new_content.push('\n');
    }

    // Add renumbered next commits
    for (i, (_, message)) in next_commits.iter().enumerate() {
        new_content.push_str(&format!("# NEXT-{}: {}\n", i + 1, message));
    }

    // Add instruction comment
    new_content = append_instruction_comment(&new_content);

    // Write the updated content
    let mut file = fs::File::create(find_git_root()?.join(".COMMIT_MESSAGE"))?;
    write!(file, "{new_content}")?;

    Ok(())
}

pub fn handle_add_command(args: &[String]) -> io::Result<()> {
    let mut is_desc = false;
    let mut is_next = false;
    let mut message_parts = Vec::new();

    // Parse flags and message
    for arg in args {
        match arg.as_str() {
            "--desc" => is_desc = true,
            "--next" => is_next = true,
            _ => message_parts.push(arg.clone()),
        }
    }

    let message = message_parts.join(" ");

    if message.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no message argument provided",
        ));
    }

    if is_next {
        add_next_commit_message(&message)?;
    } else if is_desc {
        add_description(&message)?;
    } else {
        // Default behavior based on config
        let config = load_config();
        if config.default_add_as_desc {
            add_description(&message)?;
        } else {
            add_title(&message)?;
        }
    }

    println!("Message added:");
    let _ = display_enhanced_commit_status();

    Ok(())
}

pub fn handle_status_command(args: &[String]) -> io::Result<()> {
    if !args.is_empty() && args[0] == "--full" {
        display_full_status()
    } else {
        crate::display::display_status()
    }
}

pub fn add_title(new_title: &str) -> io::Result<()> {
    let current_message = read_file_extract_message(find_git_root()?.join(".COMMIT_MESSAGE")).ok();
    let current_comments =
        read_file_extract_comments(find_git_root()?.join(".COMMIT_MESSAGE")).unwrap_or_default();

    let updated_message = match current_message {
        Some(current_msg) => {
            let lines: Vec<&str> = current_msg.lines().collect();
            if lines.is_empty() {
                new_title.to_string()
            } else {
                // Update the first line (title) by appending with comma
                let updated_title = if lines[0].trim().is_empty() {
                    new_title.to_string()
                } else {
                    format!("{}, {}", lines[0], new_title)
                };

                // Keep rest of the message intact
                if lines.len() > 1 {
                    let rest: Vec<&str> = lines[1..].to_vec();
                    format!("{}\n{}", updated_title, rest.join("\n"))
                } else {
                    updated_title
                }
            }
        }
        None => new_title.to_string(),
    };

    let message_with_comments = if current_comments.is_empty() {
        updated_message
    } else {
        format!("{updated_message}\n{current_comments}")
    };

    set_message(&append_instruction_comment(&message_with_comments), false)
}

pub fn add_description(description: &str) -> io::Result<()> {
    let current_message = read_file_extract_message(find_git_root()?.join(".COMMIT_MESSAGE")).ok();
    let current_comments =
        read_file_extract_comments(find_git_root()?.join(".COMMIT_MESSAGE")).unwrap_or_default();

    let updated_message = match current_message {
        Some(current_msg) => {
            if current_msg.trim().is_empty() {
                description.to_string()
            } else {
                format!("{}\n{}", current_msg.trim(), description)
            }
        }
        None => description.to_string(),
    };

    let message_with_comments = if current_comments.is_empty() {
        updated_message
    } else {
        format!("{updated_message}\n{current_comments}")
    };

    set_message(&append_instruction_comment(&message_with_comments), false)
}

pub fn add_next_commit_message(message: &str) -> io::Result<()> {
    let current_content = match get_message(false) {
        Ok(content) => content,
        Err(_) => {
            // If no commit message exists, create a default one
            let default_msg = append_instruction_comment("");
            set_message(&default_msg, false)?;
            default_msg
        }
    };

    let next_number = find_next_commit_number(&current_content);
    let next_line = format!("# NEXT-{next_number}: {message}");

    // Find insertion point (before instruction comments)
    let lines: Vec<&str> = current_content.lines().collect();
    let mut new_lines = Vec::new();
    let mut instruction_started = false;

    for line in lines {
        if line.starts_with("# Enter/edit the commit message") {
            instruction_started = true;
            // Insert the next commit before instructions
            new_lines.push(next_line.as_str());
        }
        if !instruction_started {
            new_lines.push(line);
        }
    }

    // If we didn't find instructions, just append
    if !instruction_started {
        new_lines.push(next_line.as_str());
    }

    let updated_content = append_instruction_comment(&new_lines.join("\n"));
    set_message(&updated_content, false)
}

pub fn find_next_commit_number(content: &str) -> usize {
    let mut max_number = 0;

    for line in content.lines() {
        if let Some(captures) = line.strip_prefix("# NEXT-") {
            if let Some(colon_pos) = captures.find(':') {
                if let Ok(num) = captures[..colon_pos].parse::<usize>() {
                    max_number = max_number.max(num);
                }
            }
        }
    }

    max_number + 1
}

pub fn help() -> io::Result<()> {
    // ANSI color codes for matrix-style green
    let green = "\x1b[32m"; // Regular green
    let bright_green = "\x1b[92m"; // Bright green
    let cyan = "\x1b[36m"; // Cyan
    let bright_cyan = "\x1b[96m"; // Bright cyan
    let reset = "\x1b[0m"; // Reset color
    let bold = "\x1b[1m"; // Bold text

    println!(
        "{}",
        format_args!(
            r#"
    {green}╔═══════════════════════════════════════════════════════════════╗{reset}
    {green}║                                                               ║{reset}
    {green}║   {bright_green}{bold}██████╗ ██╗███╗   ███╗{reset}                                      {green}║{reset}
    {green}║  {bright_green}{bold}██╔════╝ ██║████╗ ████║{reset}   {cyan}Commit-Driven Development{reset}          {green}║{reset}
    {green}║  {bright_green}{bold}██║  ███╗██║██╔████╔██║{reset}   {cyan}Git CLI Utility{reset}                    {green}║{reset}
    {green}║  {bright_green}{bold}██║   ██║██║██║╚██╔╝██║{reset}   {bright_cyan}v1.0.1{reset}                             {green}║{reset}
    {green}║  {bright_green}{bold}╚██████╔╝██║██║ ╚═╝ ██║{reset}                                      {green}║{reset}
    {green}║   {bright_green}{bold}╚═════╝ ╚═╝╚═╝     ╚═╝{reset}   {cyan}Plan → Code → Commit → Push{reset}        {green}║{reset}
    {green}║                                                               ║{reset}
    {green}╚═══════════════════════════════════════════════════════════════╝{reset}
"#,
            green = green,
            bright_green = bright_green,
            cyan = cyan,
            bright_cyan = bright_cyan,
            reset = reset,
            bold = bold
        )
    );

    let help_message = r#"
`gim` provides the following commands:

### `gim set {COMMIT_MESSAGE}`

- Accepts a string argument for the planned commit message.
- The commit message is stored inside the `.COMMIT_MESSAGE` file.
    > **Note**: Don't worry about adding a `.COMMIT_MESSAGE` file yourself (or adding it to `.gitignore`), `gim` takes care of that for you!
- replaces the current commit message

### `gim edit`

- Opens system default editor to edit current commit message

### `gim add {TASK_MESSAGE}`

- Appends the `TASK_MESSAGE` to the current commit title as a comma-separated task.
- By default, messages are added as commit title components, not descriptions.

### `gim add --desc {DESCRIPTION}`

- Adds a description to the current commit message (appears below the title).

### `gim add --next {MESSAGE}`

- Adds a future commit to the planning queue. These commits will be automatically loaded after the current commit is completed.

### `gim commit`

- Equivalent to `git add -A && git commit -m $COMMIT_MESSAGE`.
- Allows optional argument for inclusion of specific files, similar to `git add $FILES`.
- Upon successful commit, advances to the next planned commit if available.
- Unlike `gim push`, this command only commits changes without pushing to remote.

### `gim push`

- Equivalent to `git add -A && git commit -m $COMMIT_MESSAGE && git push`.
- Allows optional argument for inclusion of specific files, similar to `git add $FILES`.
- Upon successful push, advances to the next planned commit if available.

### `gim status` or just `gim`

- Displays the current commit with enhanced formatting:
  - Title in bold with task breakdown
  - Description in normal text
  - Upcoming planned commits listed with numbers

### `gim status --full`

- Shows the complete `.COMMIT_MESSAGE` file including comments and next commits.

### `gim config`

- Shows current configuration settings with explanations.
- Config file location: ~/.config/gim/config.toml

### `gim config edit`

- Opens the configuration file in your default editor.
- The config file contains detailed comments explaining each option.
- Available settings:
  - `default_add_as_desc`: controls whether `gim add` creates titles or descriptions
  - `verbose`: controls whether status displays show labels or clean format

### `gim reorder`

- Interactive reordering of upcoming commits. Allows you to:
  - Reorder commits by entering new positions (e.g., '2 1 3')
  - Comment out commits with 'c <number>' (e.g., 'c 2')
  - Cancel with 'cancel'

### `gim integrate`

- Automatically adds gim integration documentation to LLM instruction files.
- Searches for common LLM files (.cursorrules, CLAUDE.md, claude.md, etc.)
- Adds comprehensive gim workflow instructions for AI assistants.
- If no LLM instruction file is found, suggests creating one first.

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
