fn main() {
    println!("Hello, world!");
}


fn 

// gim plan
// - accepts string as argument for message, if no string provided, open default editor
// - commit message file is stored inside the binary directory aka /usr/bin/gim/project-dir-title
//     - that way, don't have to worry about any sort of committing unneeded files in local git dir, can use multiple gim plans across different repos if needed.
// gim push
//   - equivalent to `git add . && git commit -m $COMMIT_MESSAGE && git push` 
//       - allow optionality for including files after push equivalent to git add $FILE, defaults to `.`
// gim status
// 
