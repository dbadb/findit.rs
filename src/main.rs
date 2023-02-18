use std::env;
use std::process;
use findit::Config;
use colored::*;

fn main() 
{

    let config = Config::new(env::args())
        .unwrap_or_else(|err|
        {
            println!("Problem parsing argument: {err}");
            process::exit(1);
        });

    let mut n_dirs: u32 = 0;
    let mut n_files: u32 = 0;
    let mut n_lines: u32 = 0;

    println!("\n{}\n", config.summarize().green());

    if let Err(e) = findit::run(&config, 
                    &mut n_dirs, &mut n_files, &mut n_lines)
    {
        println!("Application error {e}");
        process::exit(1);
    }
    else
    {
        let x = format!("\n---- searched {n_lines} lines in {n_files} files from {n_dirs} dirs. -----");
        println!("{}", x.green());
    }
}

