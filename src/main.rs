use std::{
    io::{BufRead, BufReader},
    process::{self, Stdio},
    thread,
};

fn main() {
    let mut args = std::env::args().collect::<Vec<String>>();
    let bin = args.remove(0);

    if args.is_empty() {
        eprintln!("Usage: {} <any command>", bin);
        process::exit(1);
    }

    let mut child = process::Command::new(args.remove(0))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("Failed to start child process: {}", e);
            process::exit(1);
        });

    let stdout = child.stdout.take().expect("Failed to take stdout");
    let stderr = child.stderr.take().expect("Failed to take stderr");

    let out_handle = thread::spawn(move || {
        let mut buf = String::new();
        for line in BufReader::new(stdout).lines().flatten() {
            println!("{}", line);
            buf.push_str(&line);
            buf.push('\n');
        }
        buf
    });

    let err_handle = thread::spawn(move || {
        let mut buf = String::new();
        for line in BufReader::new(stderr).lines().flatten() {
            println!("{}", line);
            buf.push_str(&line);
            buf.push('\n');
        }
        buf
    });

    let status = child.wait().expect("Failed to get child status");
    let stdout = out_handle
        .join()
        .expect("Failed to join threads")
        .trim()
        .to_string();
    let stderr = err_handle
        .join()
        .expect("Failed to join threads")
        .trim()
        .to_string();

    let locale = sys_locale::get_locale()
        .unwrap_or_else(|| String::from("en-US"))
        .chars()
        .take(2)
        .collect::<String>();

    println!("Program exited with status {}", status);

    let prompt = format!(
        "Exit code: {}\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
        status, stdout, stderr
    );

    println!("{}", prompt);
}
