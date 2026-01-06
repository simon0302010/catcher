#![allow(unused)]

use std::{
    fs,
    io::{Read, Write},
    process::{self, ExitStatus, Stdio, exit},
    thread,
};

mod llm_api;

use llm_api::get_summary;
use terminal_size::{Height, Width, terminal_size};
use tokio::io;

#[tokio::main]
async fn main() {
    let (Width(width), _) = terminal_size().unwrap_or((Width(100), Height(0))); // better response formatting

    let api_key = std::env::var("HACKCLUB_API_KEY").unwrap_or_else(|_| {
        eprintln!("Please set HACKCLUB_API_KEY to your api key");
        exit(1);
    });

    let mut args = std::env::args().collect::<Vec<String>>();
    let bin = args.remove(0);

    let full_command = args.clone();

    if args.is_empty() {
        eprintln!("Usage: {} <any command>", bin);
        exit(1);
    }

    let mut child = process::Command::new(args.remove(0)) // run entered command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("Failed to start child process: {}", e);
            exit(1);
        });

    let stdout = child.stdout.take().expect("Failed to take stdout");
    let stderr = child.stderr.take().expect("Failed to take stderr");

    let out_handle = thread::spawn(move || {
        // read command output
        let mut buf = Vec::new();
        let mut chunk = [0u8; 1024];
        let mut stdout_reader = stdout;
        while let Ok(n) = stdout_reader.read(&mut chunk) {
            // buffered reader for stdout
            if n == 0 {
                break;
            }
            std::io::stdout().write_all(&chunk[..n]).ok();
            std::io::stdout().flush().ok();
            buf.extend_from_slice(&chunk[..n]);
        }
        String::from_utf8_lossy(&buf).to_string()
    });

    let err_handle = thread::spawn(move || {
        // buffered reader for stderr
        let mut buf = Vec::new();
        let mut chunk = [0u8; 1024];
        let mut stderr_reader = stderr;
        while let Ok(n) = stderr_reader.read(&mut chunk) {
            if n == 0 {
                break;
            }
            std::io::stderr().write_all(&chunk[..n]).ok();
            std::io::stderr().flush().ok();
            buf.extend_from_slice(&chunk[..n]);
        }
        String::from_utf8_lossy(&buf).to_string()
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

    if status.code().unwrap_or(1) == 0 {
        println!("The program exited with status 0. No summary will be created.");
        exit(0);
    }

    let mut locale = sys_locale::get_locale()
        .unwrap_or_else(|| String::from("en-US"))
        .chars()
        .take(2) // just the language code without country
        .collect::<String>();

    // Shakespeare
    if let Some(lang) = sys_locale::get_locale()
        && lang.to_uppercase().contains("UK")
    {
        locale = String::from("Shakespeare English");
    }

    let cwd = std::env::current_dir() // current working directory
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let files = fs::read_dir(".")
        .unwrap_or_else(|e| {
            eprintln!("Failed to get contents of current dir");
            exit(1);
        })
        .filter_map(|entry| entry.ok()) // only get valid entries
        .filter_map(|entry| entry.file_name().into_string().ok()) // only get valid filenames
        .collect::<Vec<String>>() // bring everything into a new Vec<String>
        .join(", "); // combine filenames with commas

    let prompt = format!(
        "Current Directory: \"{}\"\nCurrent Directory Contents: \"{}\"\nOS: \"{}\"\nCommand: \"{}\"\nLanguage: \"{}\"\nExit code: \"{}\"\n\nSTDOUT:\n\"{}\"\n\nSTDERR:\n\"{}\"",
        cwd,
        files,
        os_info::get(),
        full_command.join(" "),
        locale,
        status,
        stdout,
        stderr
    );

    println!("prompt: {}", prompt);

    let summary = get_summary(api_key, prompt).await;

    println!(
        "\x1b[36m{} AI Overview {}\x1b[0m",
        "=".repeat((width as usize - 12) / 2),
        "=".repeat((width as usize - 12) / 2 - 1)
    );
    println!("{}", summary);
    println!("\x1b[36m{}\x1b[0m", "=".repeat(width as usize));
}
