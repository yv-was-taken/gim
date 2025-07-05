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

### Key Functions
- `find_git_root()` - Locates the git repository root directory
- `get_commit_message_path()` - Returns path to `.COMMIT_MESSAGE` file
- `ensure_gitignore()` - Manages `.gitignore` entry for commit message file
- Command handlers: `handle_set()`, `handle_edit()`, `handle_add()`, `handle_push()`, etc.

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