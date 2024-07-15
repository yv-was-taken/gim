# Gim
###  commit driven development git-cli command line utility 

------------------------------------------------ 

#### gim functionality consists of the commands below:

- **gim plan**
  - Accepts a string as an argument for the message. If no string is provided, it opens the default editor.
  - The commit message file is stored inside the `.env` file inside the directory, inside the `COMMIT_MESSAGE` variable.
   
- **gim push**
  - Equivalent to `git add . && git commit -m $COMMIT_MESSAGE && git push`.
  - Allows optional inclusion of files after push, similar to `git add $FILE`. Defaults to `.`.
  - upon a successful push, the `COMMIT_MESSAGE` variable inside `.env` is emptied.

- **gim status**
  - Displays the current gim plan at the top of the normal `git status` output display.
