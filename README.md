# Gim
##  commit driven development git-cli command line utility 

### Installlation
- Make sure you have [git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git) and [rust](https://www.rust-lang.org/tools/install) installed.
- `cargo install gim`
- that's it!

### gim functionality consists of the commands below:

- **gim set**
  - Accepts a string as an argument for the commit message. 
  If no string is provided, it opens the default editor. (<- todo)
  - The commit message file is stored inside the `.env` file inside the directory, inside the `COMMIT_MESSAGE` variable.
   
- **gim push**
  - Equivalent to `git add . && git commit -m $COMMIT_MESSAGE && git push`.
  - Allows optional inclusion of files after push, similar to `git add $FILE`. Defaults to `.`.
  - upon a successful push, the `COMMIT_MESSAGE` variable inside `.env` is emptied.

- **gim status**
  - Displays the current gim plan at the top of the normal `git status` output display.
