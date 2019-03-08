use std::env;
mod input;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);

    let result = input::parse_args(args);
    println!("result = {:?}", result);
}
