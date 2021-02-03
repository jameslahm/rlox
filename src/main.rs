use std::env;

fn main() {
    let args: Vec<String>= env::args().collect();
    if args.len() ==1 {
        rlox::repl();
    } else if args.len() == 2 {
        rlox::run_file(&args[1]);
    } else {
        println!("Usage: rlox [path]");
    }
}
