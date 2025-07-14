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

### Following Planned Commits Workflow
When working through planned commits (NEXT-N items):

1. **Check current status**: Run `gim status` to see the current commit and all planned commits
2. **Work on current commit**: Implement the feature/fix described in the current commit message
3. **Commit when done**: Use `gim commit` to commit the completed work
4. **Advance to next**: After committing, the next planned commit (NEXT-1) automatically becomes current
5. **Repeat the cycle**: Continue implementing each planned commit in sequence

Example workflow:
```bash
# View current and planned commits
gim status

# Work on the current commit task...
# (implement the feature described)

# Commit the completed work
gim commit

# The NEXT-1 commit is now current, ready to work on
gim status
```

### Configuration
- Run `gim config` to view current settings
- Run `gim config edit` to customize behavior (default editor, verbosity, etc.)
- Use `gim reorder` to reorganize planned upcoming commits

### Integration Notes
- Always check `gim status` before starting work to understand current context
- Use the upcoming commits (NEXT-N) as a roadmap for implementation
- When approaching issues, break them down into planned commits using `gim add --next`
- Prefer multiple small commits over large monolithic ones
- Follow the planned commits sequentially - each commit builds on the previous work

### LLM Instructions for Task Planning
When creating a todo list or planning tasks:
- **Ask the user**: "Would you like me to add these tasks as planned commits using `gim add --next`?"
- If yes, use `gim add --next "commit message"` for each major task
- This creates a commit-driven roadmap that both you and the user can follow
- The planned commits will appear in `gim status` as NEXT-N items

Example interaction:
```
User: "Help me refactor the authentication system"
LLM: "I'll help refactor the authentication system. Here's my planned approach:
1. Extract auth logic into separate module
2. Add JWT token validation
3. Implement refresh token mechanism
4. Update tests for new auth flow

Would you like me to add these as planned commits using `gim add --next`?"
```

### LLM Productivity Enhancements

#### Periodic Status Checks
- Run `gim status` at the start of each conversation to understand current context
- Check status periodically during long tasks to stay aligned with the plan
- Use status checks before and after making significant changes

#### Commit Message Best Practices
When writing commit messages with gim:
- Use imperative mood: "Add feature" not "Added feature"
- Keep titles under 50 characters when possible
- Use `gim add --desc` for detailed explanations of complex changes
- Include "why" in descriptions, not just "what"

Example:
```bash
gim set "Add user authentication middleware"
gim add --desc "Implements JWT-based authentication to secure API endpoints. This addresses the security requirement from issue #123."
```

#### Working with Partial Progress
- If interrupted mid-task, use `gim add --desc` to document progress
- Add TODOs or notes about what remains using `gim add --desc "TODO: ..."`
- This preserves context for resuming work later

#### Error Recovery Workflow
When encountering build/test failures:
1. Document the error in the current commit: `gim add --desc "ERROR: [error details]"`
2. Fix the issue
3. Update the commit message to reflect the fix
4. This creates a useful history of problem-solving

#### Multi-Session Context
- At conversation end, run `gim status` to show the user their current state
- Suggest using `gim add --next` for any unfinished work
- This helps maintain continuity across LLM sessions
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
