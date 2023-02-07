use std::env;
use std::process;
use findit::Config;

fn main() 
{
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err|
    {
        println!("Problem parsing argument: {err}");
        process::exit(1);
    });

    if let Err(e) = findit::run(&config)
    {
        println!("Application error {e}");
        process::exit(1);
    }
}

