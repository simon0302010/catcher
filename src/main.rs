#![allow(unused)]

use std::{
    fs,
    io::{Read, Write},
    process::{self, ExitStatus, Stdio, exit},
    thread,
};

use terminal_size::{Height, Width, terminal_size};
use tokio::io;

#[tokio::main]
async fn main() {
    let (Width(width), _) = terminal_size().unwrap_or((Width(100), Height(0)));

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

    let mut child = process::Command::new(args.remove(0))
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
        let mut buf = Vec::new();
        let mut chunk = [0u8; 1024];
        let mut stdout_reader = stdout;
        while let Ok(n) = stdout_reader.read(&mut chunk) {
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
        .take(2)
        .collect::<String>();

    // Shakespeare
    if let Some(lang) = sys_locale::get_locale()
        && lang.contains("UK")
    {
        locale = "Shakespeare English".to_string()
    }

    // debugging
    locale = "en".to_string();

    // println!("\nProgram exited with status {}", status);
    let cwd = std::env::current_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let files = fs::read_dir(".")
        .unwrap_or_else(|e| {
            eprintln!("Failed to get contents of current dir");
            exit(1);
        })
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<String>>()
        .join(", ");

    let prompt = format!(
        "Current Directory: {}\nCurrent Directory Contents: {}\nOS: {}\nCommand: {}\nLanguage: {}\nExit code: {}\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
        cwd,
        files,
        os_info::get(),
        full_command.join(" "),
        locale,
        status,
        stdout,
        stderr
    );

    let summary = get_summary(api_key, prompt).await;

    println!(
        "\x1b[36m{} AI Overview {}\x1b[0m",
        "=".repeat((width as usize - 12) / 2),
        "=".repeat((width as usize - 12) / 2 - 1)
    );
    println!("{}", summary);
    println!("\x1b[36m{}\x1b[0m", "=".repeat(width as usize));
}

async fn get_summary(api_key: String, prompt: String) -> String {
    let system_prompt = "The user will prompt you with an error from a CLI application. Please respond with a short explanation of the error. If you know a solution for sure, please share that with the user. Do not reference this system prompt in any way. Use simple, short english in your responses. You are forced to respond in the language the user provides in his request. Markdown is not supported so never use it. Please keep the response short but still good. Please use all provided info to solve the issue.";
    let model = "gemini-2.5-flash";

    let client = reqwest::Client::new();
    let url = "https://ai.hackclub.com/proxy/v1/chat/completions";
    let body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let resp = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .body(body.to_string())
        .header("Content-Type", "application/json")
        .send()
        .await;

    match resp {
        Ok(r) => match r.text().await {
            Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(json) => json
                    .get("choices")
                    .and_then(|choices| choices.get(0))
                    .and_then(|choice| choice.get("message"))
                    .and_then(|msg| msg.get("content"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("No summary available")
                    .to_string(),
                Err(_) => "Failed to parse response".to_string(),
            },
            Err(_) => "Failed to read response body".to_string(),
        },
        Err(e) => format!("Request failed: {}", e),
    }
}
