extern crate artsy;

use std::io::{self, Write};

use artsy::Trie;

fn main(){
    let mut trie: Trie<String> = Trie::for_utf8();

    loop {
        // Use the `>` character as the prompt.  Need to explicitly flush this to ensure it prints
        // before `read_line`.
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // Everything after the first whitespace character is interpreted as command-line
        // arguments.
        let mut parts = input.trim().split_whitespace();
        let command = parts.next();
        let args: Vec<_> = parts.collect();

        match command {
            None | Some("exit") => return,
            Some("get") => {
                for key in args {
                    if let Some(value) = trie.get(key.as_bytes()).unwrap() {
                        println!("{} = \"{}\"", key, value);
                    } else {
                        println!("{} = not found", key);
                    }
                }
            },
            Some("put") => {
                if args.len() != 2 {
                    eprintln!("expected 2 arguments, got: {}", args.len());
                } else {
                    let (key, value) = (args[0], args[1]);
                    if let Some(previous_value) = trie.insert(key.as_bytes(), value.to_string()).unwrap() {
                        println!("{} was \"{}\"", key, previous_value);
                    }
                }
            }
            Some(command) => {
                eprintln!("unknown command: {}", command);
            }
        }
    }
}

