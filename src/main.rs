// test
use std::fs;
use std::fs::read_to_string;
use std::fs::File;
use std::io;
use std::io::Write;
use std::io::{stderr, stdout};
use std::io::{BufRead, BufReader, Result};
use std::process::Command;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if let Some(command_input) = args.get(1) {
        parse_user_input(command_input, &args[2..])
    } else {
        display_status()
    }
}

fn find_git_root() -> Result<std::path::PathBuf> {
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

fn parse_user_input(command_input: &String, args: &[String]) -> io::Result<()> {
    match command_input.trim() {
        "set" => {
            let message = args.join(" ");
            if message.trim().is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "no message argument provided",
                ));
            }

            let user_added_comments =
                read_file_extract_comments(find_git_root()?.join(".COMMIT_MESSAGE"))
                    .unwrap_or_default();

            set_message(&append_instruction_comment(
                &(message + &user_added_comments),
            ))
        }
        "edit" => edit_message(),
        "add" => handle_add_command(args),
        "push" => {
            let files = args.join(" ").trim().to_string();
            push(if files.is_empty() { None } else { Some(files) })
        }
        "commit" => {
            let files = args.join(" ").trim().to_string();
            commit(if files.is_empty() { None } else { Some(files) })
        }
        "status" => handle_status_command(args),
        "config" => handle_config_command(args),
        "integrate" => handle_integrate_command(),
        "reorder" => handle_reorder_command(),
        "clear" => {
            let should_full_clear = args.get(0).map_or(false, |arg| arg == "full");

            match clear_message(should_full_clear) {
                Ok(_) => {
                    if should_full_clear {
                        println!("Commit message fully cleared.");
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
            format!("Unrecognized command: {command_input}"),
        )),
    }
}

fn display_status() -> io::Result<()> {
    display_enhanced_commit_status()?;
    display_next_commits()?;

    // Show git status
    match Command::new("git").arg("status").spawn() {
        Ok(_) => Ok(()),
        Err(err) => Err(io::Error::other(format!(
            "Failed to retrieve status with err: {err:#?}"
        ))),
    }
}

fn display_enhanced_commit_status() -> io::Result<()> {
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

fn display_next_commits() -> io::Result<()> {
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

fn handle_config_command(args: &[String]) -> io::Result<()> {
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
            "  editor = \"{}\" ({})",
            config.editor, "editor used for 'gim edit' and 'gim config edit'"
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
                    write!(file, "{}", new_content)?;

                    // Validate the new config by trying to parse it
                    let new_config = load_config();
                    println!("Configuration updated successfully!");
                    println!("  default_add_as_desc = {}", new_config.default_add_as_desc);
                    println!("  verbose = {}", new_config.verbose);
                    println!("  editor = \"{}\"", new_config.editor);
                }
                Err(err) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to edit config: {}", err),
                    ));
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

fn handle_integrate_command() -> io::Result<()> {
    let git_root = find_git_root()?;
    
    // Define possible LLM instruction files to search for
    let llm_files = [
        ".cursorrules",
        "CLAUDE.md", 
        "claude.md",
        "Claude.md",
        ".claude.md",
        "CODER.md",
        "coder.md",
        "AI.md",
        "ai.md",
        ".ai.md"
    ];
    
    let mut found_file = None;
    
    // Search for existing LLM instruction files
    for file_name in &llm_files {
        let file_path = git_root.join(file_name);
        if file_path.exists() {
            found_file = Some(file_path);
            break;
        }
    }
    
    let target_file = match found_file {
        Some(file_path) => file_path,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No LLM instruction file found (.cursorrules, CLAUDE.md, etc.)\nConsider running '/init' in Claude Code or creating an instruction file first before running 'gim integrate'."
            ));
        }
    };
    
    // Read existing content
    let existing_content = read_to_string(&target_file).unwrap_or_default();
    
    // Check if gim integration is already present
    if existing_content.contains("## Gim Integration") || existing_content.contains("# Gim Integration") {
        println!("Gim integration already present in {}", target_file.file_name().unwrap().to_string_lossy());
        return Ok(());
    }
    
    // Prepare the gim integration section
    let gim_section = r#"
## Gim Integration

This project uses `gim` for commit-driven development. When working with this codebase:

### Core Workflow
- Use `gim status` to see current commit message and upcoming planned commits
- Use `gim add "task description"` to add tasks to the current commit title
- Use `gim add --desc "detailed description"` to add descriptions below the title
- Use `gim add --next "future commit message"` to plan upcoming commits
- Use `gim commit` to commit without pushing (allows multiple commits before push)
- Use `gim push` to commit and push all changes

### Development Approach
- **Commit-driven development**: Plan work through commit messages before implementation
- **Upcoming commits**: View NEXT-N comments in `gim status` as your todo list for future work
- **Structured commits**: Use clear, descriptive commit titles with optional detailed descriptions
- **Incremental progress**: Make small, focused commits that build toward larger features

### Configuration
- Run `gim config` to view current settings
- Run `gim config edit` to customize behavior (default editor, verbosity, etc.)
- Use `gim reorder` to reorganize planned upcoming commits

### Integration Notes
- Always check `gim status` before starting work to understand current context
- Use the upcoming commits (NEXT-N) as a roadmap for implementation
- When approaching issues, break them down into planned commits using `gim add --next`
- Prefer multiple small commits over large monolithic ones
"#;
    
    // Append the gim section to the file
    let new_content = if existing_content.trim().is_empty() {
        gim_section.trim().to_string()
    } else {
        format!("{}\n{}", existing_content.trim(), gim_section)
    };
    
    // Write the updated content
    let mut file = fs::File::create(&target_file)?;
    write!(file, "{}", new_content)?;
    
    println!("Gim integration added to {}", target_file.file_name().unwrap().to_string_lossy());
    println!("LLMs working with this repository will now be instructed to use gim for commit-driven development.");
    
    Ok(())
}

fn handle_reorder_command() -> io::Result<()> {
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

    use std::io::{stdin, BufRead};
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
                    println!("Commenting out: {}", message);

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

fn rebuild_commit_file_with_reorder(
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
    write!(file, "{}", new_content)?;

    Ok(())
}

#[derive(Debug)]
struct GimConfig {
    default_add_as_desc: bool,
    verbose: bool,
    editor: String,
}

impl Default for GimConfig {
    fn default() -> Self {
        Self {
            default_add_as_desc: false, // Default behavior: add as title, not description
            verbose: false,             // Default behavior: no labels, clean display
            editor: "vim".to_string(),  // Default editor
        }
    }
}

fn get_config_path() -> io::Result<std::path::PathBuf> {
    let home = match std::env::var("HOME") {
        Ok(home) => std::path::PathBuf::from(home),
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find HOME directory",
            ))
        }
    };

    let config_dir = home.join(".config").join("gim");

    // Create config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir.join("config.toml"))
}

fn load_config() -> GimConfig {
    match get_config_path() {
        Ok(config_path) => {
            if let Ok(content) = read_to_string(&config_path) {
                parse_config(&content)
            } else {
                GimConfig::default()
            }
        }
        Err(_) => GimConfig::default(),
    }
}

fn parse_config(content: &str) -> GimConfig {
    let mut config = GimConfig::default();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');

            match key {
                "default_add_as_desc" => {
                    config.default_add_as_desc = value.parse().unwrap_or(false);
                }
                "verbose" => {
                    config.verbose = value.parse().unwrap_or(false);
                }
                "editor" => {
                    config.editor = value.to_string();
                }
                _ => {} // Ignore unknown keys
            }
        }
    }

    config
}

fn save_config(config: &GimConfig) -> io::Result<()> {
    let config_path = get_config_path()?;
    let content = format!(
        r#"# Gim Configuration File
# Edit this file to customize gim behavior
# Location: ~/.config/gim/config.toml
# Use 'gim config edit' to edit this file

# ============================================================================
# COMMIT MESSAGE BEHAVIOR
# ============================================================================

# Controls default behavior of 'gim add' command
# 
# When true:  'gim add "message"' adds to description (below title)
# When false: 'gim add "message"' adds to title (comma-separated)
# 
# You can always override with:
#   'gim add --desc "description"' to add to description
#   'gim add "title"' to add to title (default behavior)
#
# Default: false
default_add_as_desc = {}

# ============================================================================
# DISPLAY FORMATTING
# ============================================================================

# Controls status display format
#
# When true:  Shows labels and structure
#             Current commit:
#                 Title: feat/feature-name, task1, task2
#                 Description:
#                     Detailed description here
#
# When false: Clean format without labels
#             Current commit:
#                 feat/feature-name, task1, task2    (bold)
#                 Detailed description here          (italic)
#
# Default: false 
verbose = {}

# ============================================================================
# EDITOR SETTINGS
# ============================================================================

# Default editor for 'gim edit' and 'gim config edit' commands
#
# This should be the command name or full path to your preferred text editor.
# The editor will be launched with the file path as an argument.
#
# Default: "vim"
editor = "{}"
"#,
        config.default_add_as_desc, config.verbose, config.editor
    );

    let mut file = fs::File::create(config_path)?;
    write!(file, "{content}")?;
    Ok(())
}

fn advance_to_next_commit() -> io::Result<()> {
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

fn set_message(message_to_set: &str) -> io::Result<()> {
    let mut file = match fs::File::create(find_git_root()?.join(".COMMIT_MESSAGE")) {
        Ok(file) => file,
        Err(err) => {
            return Err(io::Error::other(format!(
                "Failed to create .COMMIT_MESSAGE with err: {err:#?}"
            )))
        }
    };

    let current_message = read_file_extract_message(find_git_root()?.join(".COMMIT_MESSAGE")).ok();

    let message_with_added_message = match current_message {
        Some(m) => m + message_to_set,
        None => String::from(message_to_set),
    };

    let current_message_comments =
        read_file_extract_comments(find_git_root()?.join(".COMMIT_MESSAGE")).ok();

    let message_with_comments = match current_message_comments {
        Some(comments) => message_with_added_message + &comments,
        None => message_with_added_message,
    };

    match write!(file, "{message_with_comments}") {
        Ok(_) => (),
        Err(err) => {
            return Err(io::Error::other(format!(
                "Failed to write to .COMMIT_MESSAGE with err: {err:#?}"
            )))
        }
    };

    match read_to_string(find_git_root()?.join(".gitignore")) {
        Ok(content) => {
            let mut does_gitignore_contain_commit_message = false;
            for line in content.lines().filter(|line| !line.is_empty()) {
                if line == ".COMMIT_MESSAGE" {
                    does_gitignore_contain_commit_message = true;
                }
            }
            if !does_gitignore_contain_commit_message {
                let mut file = match fs::File::create(find_git_root()?.join(".gitignore")) {
                    Ok(file) => file,
                    Err(err) => {
                        return Err(io::Error::other(format!(
                            "Failed to create .gitignore with err: {err:#?}"
                        )))
                    }
                };
                match writeln!(file, "{content}") {
                    Ok(_) => (),
                    Err(err) => {
                        return Err(io::Error::other(format!(
                            "Failed to write to .gitignore with err: {err:#?}"
                        )))
                    }
                };
                match writeln!(file, ".COMMIT_MESSAGE") {
                    Ok(_) => (),
                    Err(err) => {
                        return Err(io::Error::other(format!(
                            "Failed to write to .gitignore with err: {err:#?}",
                        )))
                    }
                };
            }
        }
        Err(_) => {
            let mut file = match fs::File::create(find_git_root()?.join(".gitignore")) {
                Ok(file) => file,
                Err(err) => {
                    return Err(io::Error::other(format!(
                        "Failed to create .gitignore with err: {err:#?}"
                    )))
                }
            };
            match writeln!(file, ".COMMIT_MESSAGE") {
                Ok(_) => (),
                Err(err) => {
                    return Err(io::Error::other(format!(
                        "Failed to create .env with err: {err:#?}"
                    )))
                }
            };
        }
    };

    println!("Commit message set:");
    let _ = display_enhanced_commit_status();

    Ok(())
}

fn edit_with_configured_editor(content: &str) -> io::Result<String> {
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

fn edit_message() -> io::Result<()> {
    let current_commit_message = get_message(false).ok();
    match current_commit_message {
        Some(message) => match edit_with_configured_editor(&message) {
            Ok(m) => set_message(&m),
            Err(err) => Err(err),
        },
        None => match edit_with_configured_editor(" ") {
            Ok(m) => set_message(&append_instruction_comment(&m)),
            Err(err) => Err(err),
        },
    }
}

fn get_message(ignore_comments: bool) -> io::Result<String> {
    if ignore_comments {
        match read_file_extract_message(find_git_root()?.join(".COMMIT_MESSAGE")) {
            Ok(content) => {
                if content.trim().is_empty() {
                    Err(io::Error::other("No commit message found"))
                } else {
                    Ok(content)
                }
            }
            Err(err) => Err(io::Error::other(format!(
                "Failed to read .env with err: {err:#?}",
            ))),
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
            Err(err) => Err(io::Error::other(format!(
                "Failed to read .env with err: {err:#?}",
            ))),
        }
    }
}

fn clear_message(is_full_clear: bool) -> io::Result<()> {
    if is_full_clear {
        match fs::File::create(find_git_root()?.join(".COMMIT_MESSAGE")) {
            Ok(_) => (),
            Err(err) => {
                return Err(io::Error::other(format!(
                    "Failed to create .COMMIT_MESSAGE with err: {err:#?}",
                )))
            }
        };
        set_message(&append_instruction_comment(""))
    } else {
        let comments = match read_file_extract_comments(find_git_root()?.join(".COMMIT_MESSAGE")) {
            Ok(file) => &append_instruction_comment(&file.to_string()),
            Err(err) => {
                return Err(io::Error::other(format!(
                    "Failed to read .COMMIT_MESSAGE with err: {err:#?}",
                )))
            }
        };
        let mut file = match fs::File::create(find_git_root()?.join(".COMMIT_MESSAGE")) {
            Ok(file) => file,
            Err(err) => {
                return Err(io::Error::other(format!(
                    "Failed to create .COMMIT_MESSAGE with err: {err:#?}",
                )))
            }
        };
        match write!(file, r#"{comments}"#) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

fn append_instruction_comment(message: &str) -> String {
    format!(
        r#"{message}

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
"#
    )
}

fn read_file_extract_message(file_path: std::path::PathBuf) -> Result<String> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut content = String::new();

    for line in reader.lines() {
        let line = line?;
        if !line.is_empty() && !line.trim().starts_with('#') {
            content.push_str(&line);
            content.push('\n');
        }
    }

    Ok(content)
}

fn read_file_extract_comments(file_path: std::path::PathBuf) -> Result<String> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut content = String::new();

    let mut is_end_of_user_added_comments = false;
    for line in reader.lines() {
        let line = line?;
        if line.starts_with("# Enter/edit the commit message for your changes.") {
            is_end_of_user_added_comments = true;
        }
        if !line.is_empty() && line.trim().starts_with('#') && !is_end_of_user_added_comments {
            content.push('\n');
            content.push_str(&line);
        }
    }

    Ok(content)
}

fn commit(contents: Option<String>) -> io::Result<()> {
    let commit_message = get_message(true)?;

    let files_to_add = match contents {
        Some(x) => x,
        None => String::from("-A"),
    };
    match Command::new("git")
        .arg("add")
        .arg(files_to_add.as_str())
        .status()
    {
        Ok(_) => (),
        Err(err) => {
            return Err(io::Error::other(format!(
                "Failed to add files with err: {err:#?}",
            )))
        }
    };

    let mut commit_message_header_and_body = commit_message.split("\n");

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
    commit_command_args.push(commit_message_header);
    if !commit_message_body.is_empty() {
        commit_command_args.push("-m");
        commit_command_args.push(&commit_message_body);
    }

    let commit_output = match Command::new("git").args(commit_command_args).output() {
        Ok(output) => output,
        Err(err) => {
            return Err(io::Error::other(format!(
                "Failed to execute git commit: {err:#?}",
            )))
        }
    };

    // Print stdout
    if let Ok(stdout_str) = String::from_utf8(commit_output.stdout.clone()) {
        if !stdout_str.is_empty() {
            print!("{stdout_str}");
            let _ = stdout().flush();
        }
    }

    // Print stderr (this includes pre-commit hook output)
    if let Ok(stderr_str) = String::from_utf8(commit_output.stderr.clone()) {
        if !stderr_str.is_empty() {
            eprint!("{stderr_str}");
            let _ = stderr().flush();
        }
    }

    // Check if the commit was successful
    if !commit_output.status.success() {
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

    Ok(())
}

fn push(contents: Option<String>) -> io::Result<()> {
    let commit_message = get_message(true)?;

    let files_to_push = match contents {
        Some(x) => x,
        None => String::from("-A"),
    };
    match Command::new("git")
        .arg("add")
        .arg(files_to_push.as_str())
        .status()
    {
        Ok(_) => (),
        Err(err) => {
            return Err(io::Error::other(format!(
                "Failed to add files with err: {err:#?}",
            )))
        }
    };

    let mut commit_message_header_and_body = commit_message.split("\n");

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
    commit_command_args.push(commit_message_header);
    if !commit_message_body.is_empty() {
        commit_command_args.push("-m");
        commit_command_args.push(&commit_message_body);
    }

    let commit_output = match Command::new("git").args(commit_command_args).output() {
        Ok(output) => output,
        Err(err) => {
            return Err(io::Error::other(format!(
                "Failed to execute git commit: {err:#?}",
            )))
        }
    };

    // Print stdout
    if let Ok(stdout_str) = String::from_utf8(commit_output.stdout.clone()) {
        if !stdout_str.is_empty() {
            print!("{stdout_str}");
            let _ = stdout().flush();
        }
    }

    // Print stderr (this includes pre-commit hook output)
    if let Ok(stderr_str) = String::from_utf8(commit_output.stderr.clone()) {
        if !stderr_str.is_empty() {
            eprint!("{stderr_str}");
            let _ = stderr().flush();
        }
    }

    // Check if the commit was successful
    if !commit_output.status.success() {
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

    match Command::new("git").arg("push").status() {
        Ok(_) => Ok(()),
        Err(err) => Err(io::Error::other(format!(
            "Failed to push changes with err: {err:#?}"
        ))),
    }
}

fn help() -> io::Result<()> {
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

fn handle_add_command(args: &[String]) -> io::Result<()> {
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
        add_next_commit_message(&message)
    } else {
        // Load config to determine default behavior
        let config = load_config();
        let should_add_as_desc = is_desc || (!is_desc && config.default_add_as_desc);

        if should_add_as_desc {
            add_description(&message)
        } else {
            add_title(&message)
        }
    }
}

fn handle_status_command(args: &[String]) -> io::Result<()> {
    let show_full = args.contains(&"--full".to_string());

    if show_full {
        display_full_status()
    } else {
        display_status()
    }
}

fn add_title(new_title: &str) -> io::Result<()> {
    let current_message = read_file_extract_message(find_git_root()?.join(".COMMIT_MESSAGE")).ok();

    let updated_message = match current_message {
        Some(current) => {
            // Split existing message into lines
            let lines: Vec<&str> = current.trim().lines().collect();
            if lines.is_empty() {
                new_title.to_string()
            } else {
                // First line is the title, append with comma
                let first_line = lines[0];
                let mut new_first_line = if first_line.is_empty() {
                    new_title.to_string()
                } else {
                    format!("{}, {}", first_line.trim(), new_title.trim())
                };

                // Keep the rest of the lines (descriptions, etc.)
                if lines.len() > 1 {
                    new_first_line.push('\n');
                    new_first_line.push_str(&lines[1..].join("\n"));
                }

                new_first_line
            }
        }
        None => new_title.to_string(),
    };

    let user_added_comments =
        read_file_extract_comments(find_git_root()?.join(".COMMIT_MESSAGE")).unwrap_or_default();

    set_message(&append_instruction_comment(
        &(updated_message + &user_added_comments),
    ))
}

fn add_description(description: &str) -> io::Result<()> {
    let current_message = read_file_extract_message(find_git_root()?.join(".COMMIT_MESSAGE")).ok();

    let updated_message = match current_message {
        Some(current) => {
            let lines: Vec<&str> = current.trim().lines().collect();
            if lines.is_empty() {
                format!("\n\n{}", description)
            } else {
                // Keep the title (first line) and add description
                let mut result = lines[0].to_string();
                result.push('\n');
                result.push('\n');
                result.push_str(description);

                // Add any existing descriptions
                if lines.len() > 1 {
                    result.push('\n');
                    result.push_str(&lines[1..].join("\n"));
                }

                result
            }
        }
        None => format!("\n\n{}", description),
    };

    let user_added_comments =
        read_file_extract_comments(find_git_root()?.join(".COMMIT_MESSAGE")).unwrap_or_default();

    set_message(&append_instruction_comment(
        &(updated_message + &user_added_comments),
    ))
}

fn add_next_commit_message(message: &str) -> io::Result<()> {
    let current_content =
        read_to_string(find_git_root()?.join(".COMMIT_MESSAGE")).unwrap_or_default();

    // Find the next available number for the next commit
    let next_number = find_next_commit_number(&current_content);
    let next_comment = format!("\n# NEXT-{}: {}", next_number, message);

    let updated_content = current_content + &next_comment;

    let mut file = fs::File::create(find_git_root()?.join(".COMMIT_MESSAGE"))?;
    write!(file, "{}", updated_content)?;

    println!("Added next commit #{}: {}", next_number, message);
    Ok(())
}

fn find_next_commit_number(content: &str) -> usize {
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

fn display_full_status() -> io::Result<()> {
    match read_to_string(find_git_root()?.join(".COMMIT_MESSAGE")) {
        Ok(content) => {
            println!("Full commit message file:");
            println!("{}", content);
        }
        Err(_) => println!("No commit message file found."),
    }

    // Also show git status
    match Command::new("git").arg("status").spawn() {
        Ok(_) => Ok(()),
        Err(err) => Err(io::Error::other(format!(
            "Failed to retrieve status with err: {err:#?}"
        ))),
    }
}
