use std::{fs::File, io::Read};

pub mod chunk;
pub mod error;
pub mod op_code;
pub mod vm;
pub mod scanner;
pub mod compiler;
pub mod token;
pub mod util;

pub fn repl() {}

pub fn run_file(filename: &String) {
    let mut file = File::open(filename).expect(format!("Could not open file {}\n", filename).as_str());
    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("Could not read file");

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
