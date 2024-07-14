use std::io;

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let mut input_iter = input.split_whitespace();
    let command = input_iter.next();
    let arg = input_iter.next();

    if !input_iter.next().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Too many inputs!",
        ));
    } else {
        match command {
            Some(value) => match value {
                "plan" => plan(&arg.unwrap()),
                "push" => push(),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unrecognized command",
                )),
            },

            None => {
                println!("standard git status type thing here");
                Ok(())
            }
        }
    }
}

pub fn plan(arg: &str) -> io::Result<()> {
    println!("your plan is: {arg}");
    Ok(())
}

pub fn push() -> io::Result<()> {
    // read commit file, push it
    println!("Ahh, push it");
    Ok(())
}
