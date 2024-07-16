
# Gim

## Commit Driven Development Git-CLI Command Line Utility

### Installation

1. Ensure you have [Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git) and [Rust](https://www.rust-lang.org/tools/install) installed.
2. Install `gim` using Cargo:
    ```sh
    cargo install gim
    ```

### Functionality

`gim` provides the following commands:

#### `gim set {COMMIT_MESSAGE}`

- Accepts a string argument for the pre-planned commit message.
- The commit message is stored inside the `.env` file within the `COMMIT_MESSAGE` variable.

#### `gim push`

- Equivalent to `git add . && git commit -m $COMMIT_MESSAGE && git push`.
- Allows optional inclusion of files after push, similar to `git add $FILE`. Defaults to `.`.
- Upon a successful push, the `COMMIT_MESSAGE` variable inside `.env` is cleared.

#### `gim status`

- Displays the current `gim` plan at the top of the normal `git status` output.
