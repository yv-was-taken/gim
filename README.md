# Gim - Commit-Driven Development Git CLI

[![Crates.io License](https://img.shields.io/crates/l/gim)](https://opensource.org/licenses/MIT)
[![Crates.io Version](https://img.shields.io/crates/v/gim)](https://crates.io/crates/gim)

```
    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║   ██████╗ ██╗███╗   ███╗                                     ║
    ║  ██╔════╝ ██║████╗ ████║   Commit-Driven Development         ║
    ║  ██║  ███╗██║██╔████╔██║   Git CLI Utility                   ║
    ║  ██║   ██║██║██║╚██╔╝██║   v1.0.1                            ║
    ║  ╚██████╔╝██║██║ ╚═╝ ██║                                     ║
    ║   ╚═════╝ ╚═╝╚═╝     ╚═╝   Plan → Code → Commit → Push     ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝
```

**Gim** is a powerful commit-driven development CLI utility for Git that revolutionizes your workflow. Plan your commits before coding, maintain structured commit messages, and seamlessly integrate with AI assistants.

## 🚀 Key Features

- **📝 Commit-Driven Development**: Plan your work through commit messages before implementation
- **🔄 Smart Commit Queue**: Automatically advance through planned commits
- **🎨 Enhanced Display**: Beautiful formatting with bold titles and italic descriptions  
- **⚙️ Configurable**: Customize editor, display format, and behavior
- **🤖 AI Integration**: Auto-configure LLM instruction files for AI-assisted development
- **📦 Zero Dependencies**: Pure Rust implementation with no external crates

## 📦 Installation

1. Ensure you have [Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git) and [Rust](https://www.rust-lang.org/tools/install) installed.
2. Install `gim` using Cargo:
    ```sh
    cargo install gim
    ```

## 🎯 Quick Start

```bash
# Set your commit message
gim set "feat: add user authentication"

# Add implementation details
gim add --desc "Implement JWT token validation and user session management"

# Plan future commits
gim add --next "feat: add password reset functionality"
gim add --next "test: add auth integration tests"

# Check your commit queue
gim status

# Commit without pushing (great for multiple commits)
gim commit

# Or commit and push in one command
gim push
```

## 📚 Commands

### Core Commands

#### `gim set {MESSAGE}`
Sets the current commit message, replacing any existing message.

#### `gim add {MESSAGE}` 
Adds to the current commit title (comma-separated by default).

#### `gim add --desc {DESCRIPTION}`
Adds a detailed description below the commit title.

#### `gim add --next {MESSAGE}`
Plans a future commit that will auto-load after the current commit.

#### `gim edit`
Opens your configured editor to edit the current commit message.

#### `gim commit`
Commits changes without pushing (allows multiple local commits).

#### `gim push`
Commits and pushes changes in one command.

### Status & Display

#### `gim status` or `gim`
Shows formatted current commit and upcoming commits with git status.

#### `gim status --full`
Displays the complete `.COMMIT_MESSAGE` file with all comments.

### Configuration

#### `gim config`
Shows current configuration settings.

#### `gim config edit`
Opens the configuration file in your editor.

**Available Settings:**
- `editor` - Default editor (vim, nano, code, etc.)
- `default_add_as_desc` - Make `gim add` create descriptions by default
- `verbose` - Show labels in status display

### Advanced Features

#### `gim reorder`
Interactive reordering of upcoming commits:
- Reorder by entering new positions (e.g., '2 1 3')
- Comment out commits with 'c <number>'
- Cancel with 'cancel'

#### `gim integrate`
Automatically adds gim workflow documentation to AI instruction files:
- Searches for `.cursorrules`, `CLAUDE.md`, etc.
- Adds comprehensive gim usage instructions
- Helps AI assistants understand your workflow

#### `gim clear`
Clears the current commit message (preserves comments).

#### `gim clear full`
Completely clears the commit file including all comments.

## 🔧 Configuration

Gim stores its configuration at `~/.config/gim/config.toml`:

```toml
# Controls default behavior of 'gim add'
default_add_as_desc = false

# Controls status display format  
verbose = false

# Default editor for 'gim edit' and 'gim config edit'
editor = "vim"
```

## 💡 Workflow Example

```bash
# 1. Plan your feature
gim set "feat: implement shopping cart"
gim add --desc "Add cart state management and UI components"

# 2. Plan upcoming work
gim add --next "test: add shopping cart unit tests"
gim add --next "docs: update API documentation for cart endpoints"

# 3. Implement and commit
# ... write your code ...
gim commit  # Creates local commit, advances to next

# 4. Continue with tests
# ... write tests ...
gim commit  # Another local commit

# 5. Push all commits
git push  # Push both commits together
```

## 🤝 AI Assistant Integration

Run `gim integrate` in any project to automatically configure AI assistants (Claude, Cursor, etc.) to use gim for commit-driven development. This adds instructions for the AI to:

- Use `gim status` to understand current work
- Plan commits with `gim add --next`
- Follow structured commit practices
- Break down features into incremental commits

## 📄 License

MIT License - see [LICENSE](https://opensource.org/licenses/MIT) for details.

## 🌟 Contributing

Contributions are welcome! Feel free to submit issues and pull requests on [GitHub](https://github.com/yv-was-taken/gim).

---

*Made with ❤️ for developers who plan before they code*