use std::env;
use std::process;
use findit::Config;

fn main() 
{

    let config = Config::new(env::args())
        .unwrap_or_else(|err|
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

