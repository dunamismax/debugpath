use std::env;
use std::path::PathBuf;

fn main() {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        usage_and_exit();
    };

    match command.as_str() {
        "validate-cases" => {
            let root = args.next().unwrap_or_else(|| "cases".to_owned());
            validate_cases(PathBuf::from(root));
        }
        _ => usage_and_exit(),
    }
}

fn validate_cases(root: PathBuf) {
    match debugpath_content::load_cases(&root) {
        Ok(cases) => {
            println!("validated {} case(s) under {}", cases.len(), root.display());
            for case in cases {
                println!("- {}", case.metadata.slug);
            }
        }
        Err(error) => {
            eprintln!("case validation failed: {error}");
            std::process::exit(1);
        }
    }
}

fn usage_and_exit() -> ! {
    eprintln!("usage: cargo run -p xtask -- validate-cases [cases-dir]");
    std::process::exit(2);
}
