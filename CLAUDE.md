# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**gim** (Git Interactive Message) is a Rust CLI utility for commit-driven development. It allows storing commit messages before actually committing, enabling better planning and editing of commits.

## Common Development Commands

### Build & Run
```bash
# Build the project
cargo build
cargo build --release    # Optimized build

# Run the application
cargo run -- [command]   # e.g., cargo run -- set "Initial commit"
cargo run --release -- [command]

# Check compilation without building
cargo check
```

### Code Quality
```bash
# Format code
cargo fmt

# Run linter
cargo clippy
cargo clippy -- -D warnings    # Fail on warnings

# Check formatting without modifying
cargo fmt -- --check
```

### Testing
```bash
# Run tests (note: no tests currently exist)
cargo test
```

### Release & Publishing
```bash
# Package verification before publishing
cargo publish --dry-run

# Publish to crates.io
cargo publish

# Install locally
cargo install --path .
```

## Architecture & Code Structure

### Single File Architecture
The entire application is implemented in `src/main.rs` (574 lines) with the following key components:

1. **Command Parsing**: Manual argument parsing without external CLI libraries
2. **File Management**: 
   - Stores messages in `.COMMIT_MESSAGE` at git root
   - Automatically manages `.gitignore` entries
3. **Git Integration**: Uses `std::process::Command` to execute git commands
4. **Error Handling**: Custom error types with descriptive messages

### Module Organization

The codebase has been modularized for better organization:

**src/main.rs** - Entry point and command parsing
- `main()` - Entry point
- `parse_user_input()` - Routes commands to appropriate handlers

**src/config.rs** - Configuration management
- `GimConfig` struct - Stores configuration settings
- `load_config()` - Loads config from ~/.config/gim/config.toml
- `save_config()` - Saves configuration
- `parse_config()` - Parses TOML configuration

**src/git.rs** - Git operations
- `find_git_root()` - Locates the git repository root directory
- `commit()` - Git add and commit without push
- `push()` - Git add, commit, and push

**src/message.rs** - Message handling
- `get_message()` - Retrieves commit message
- `set_message()` - Sets commit message
- `clear_message()` - Clears message (with optional full clear)
- `edit_message()` - Opens configured editor
- `advance_to_next_commit()` - Advances to next planned commit
- `append_instruction_comment()` - Adds help comments

**src/display.rs** - Display functions
- `display_status()` - Shows full status with git status
- `display_enhanced_commit_status()` - Shows formatted current commit
- `display_next_commits()` - Shows upcoming commits
- `display_full_status()` - Shows complete .COMMIT_MESSAGE file

**src/commands.rs** - Command handlers
- `handle_config_command()` - Config viewing/editing
- `handle_add_command()` - Adding to messages
- `handle_status_command()` - Status display
- `handle_reorder_command()` - Interactive reordering
- `help()` - Help documentation

**src/integration.rs** - LLM integration
- `handle_integrate_command()` - Auto-adds gim docs to LLM files

### Command Flow
1. **gim set**: Writes message to `.COMMIT_MESSAGE`
2. **gim edit**: Opens system editor with current message
3. **gim add**: Appends to existing message
4. **gim push**: Executes `git add -A && git commit -m "message" && git push`
5. **gim status**: Shows stored message and git status
6. **gim clear**: Removes stored message

## Development Notes

- No external CLI framework used - all argument parsing is manual
- Uses `edit` crate for opening system default editor
- Error messages aim to be helpful with recovery suggestions
- Supports multi-line commit messages and comment lines (starting with #)
- The `.COMMIT_MESSAGE` file is always created at the git repository root, not the current directory

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
