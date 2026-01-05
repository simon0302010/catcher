use std::process;

fn main() {
    let mut args = std::env::args().collect::<Vec<String>>();
    let bin = args.remove(0);

    if args.is_empty() {
        eprintln!("Usage: {} <any command>", bin);
        process::exit(1);
    }

    let output = process::Command::new(args.remove(0))
        .args(args)
        .output()
        .unwrap_or_else(|e| {
            eprintln!("Failed to start child process: {}", e);
            process::exit(1);
        });

    let utf8out = String::from_utf8(output.stdout)
        .expect("Failed to decode stdout")
        .trim()
        .to_string();

    let utf8err = String::from_utf8(output.stderr)
        .expect("Failed to decode stderr")
        .trim()
        .to_string();

    println!("{}", utf8err);
}
