mod whereis;

use std::path::PathBuf;

use clap::{App, Arg};
use whereis::whereis;

fn main() {
    let matches = App::new("whereis")
        .arg(
            Arg::new("FILE")
                .takes_value(true)
                .help("A file name"),
        )
        .about(concat!(
            "A simple program to print where a file is (Only search in dir in system environment variables)\n", 
            "Please input a FULL name, ex. ping.exe\n"
        ))
        .author("朕与将军解战袍, 1393323447@qq.com")
        .version("0.1.0")
        .get_matches();

    let file_name = matches
        .value_of("FILE")
        .expect("Please provide a file name");
    let paths = whereis(file_name);

    if paths.is_empty() {
        println!("Could not found {}, please check your input.", file_name);
    } else {
        paths.into_iter().for_each(print_path);
    }
}

fn print_path(path: PathBuf) {
    println!("{}", path.to_string_lossy());
}
