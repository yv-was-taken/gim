# Gim

![Crates.io License](https://img.shields.io/crates/l/gim)
![Crates.io Version](https://img.shields.io/crates/v/gim)

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
- The commit message is stored inside the `.env` file within the `COMMIT_MESSAGE` variable.
    > **Note**: Don't worry about adding a `.env` file yourself (or adding it to `.gitignore`), `gim` takes care of that for you!

### `gim push`

- Equivalent to `git add . && git commit -m $COMMIT_MESSAGE && git push`.
- Allows optional inclusion of files after push, similar to `git add $FILE`. Defaults to `.`
- Upon a successful push, the `COMMIT_MESSAGE` variable inside `.env` is cleared.
### `gim status` or just `gim`

- Displays the current `gim` planned commit message at the top of the normal `git status` output.
