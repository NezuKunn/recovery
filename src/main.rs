use std::error::Error;
use std::io::{self, Read};

mod lib_gen;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut input = String::new();
    println!("Enter a publick key");
    io::stdin().read_to_string(&mut input).unwrap();
    
    let mut private: String = String::new();
    println!("Enter a private key path");
    io::stdin().read_to_string(&mut private).unwrap();

    let _result = lib_gen::start(input, &private as &str).await;

    Ok(())
}
