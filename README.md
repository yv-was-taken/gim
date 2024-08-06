# Gim

[![Crates.io License](https://img.shields.io/crates/l/gim)](https://opensource.org/licenses/MIT)
[![Crates.io Version](https://img.shields.io/crates/v/gim)](https://crates.io/crates/gim)

Commit Driven Development Git-CLI Command Line Utility

## Installation

1. Ensure you have [Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git) and [Rust](https://www.rust-lang.org/tools/install) installed.
2. Install `gim` using Cargo:
    ```sh
    cargo install gim
    ```

## Functionality

`gim` provides the following commands:

### `gim set {COMMIT_MESSAGE}`

- Accepts a string argument for the planned commit message.
- The commit message is stored inside the `.COMMIT_MESSAGE` file.
    > **Note**: Don't worry about adding a `.COMMIT_MESSAGE` file yourself (or adding it to `.gitignore`), `gim` takes care of that for you!
- replaces the current commit message

### `gim edit`

- Opens system default editor to edit current commit message

### `gim add {ADDED_MESSAGE}`

- Appends the `ADDED_MESSAGE` to the current commit message. Used for multiline commits

### `gim push`

- Equivalent to `git add . && git commit -m $COMMIT_MESSAGE && git push`.
- Allows optional argument for inclusion of specific files, similar to `git add $FILES`.
- Upon a successful push, the `.COMMIT_MESSAGE` file is cleared, excluding comments.
### `gim status` or just `gim`

- Displays the current `gim` planned commit message at the top of the normal `git status` output.

### `gim clear`

- Clears the stored commit message.

### `gim clear full`

- Fully clears the stored commit message, comments included.

### `gim help`

- Prints the command descriptions to the console.
