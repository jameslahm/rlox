pub mod op_code;
pub mod chunk;
pub mod vm;

pub fn say_hello(){
    println!("Say Hello");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
