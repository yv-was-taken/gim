use crate::git::find_git_root;
use std::fs;
use std::fs::read_to_string;
use std::io;
use std::io::Write;

pub fn handle_integrate_command() -> io::Result<()> {
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
        ".ai.md",
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
    if existing_content.contains("## Gim Integration")
        || existing_content.contains("# Gim Integration")
    {
        println!(
            "Gim integration already present in {}",
            target_file.file_name().unwrap().to_string_lossy()
        );
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
    write!(file, "{new_content}")?;

    println!(
        "Gim integration added to {}",
        target_file.file_name().unwrap().to_string_lossy()
    );
    println!("LLMs working with this repository will now be instructed to use gim for commit-driven development.");

    Ok(())
}
