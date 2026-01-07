# Catcher
***Catch errors so you don't have to***

Catcher helps you better understand errors of applications you run -- allowing you to troubleshoot more efficently. Catcher is meant to be ran before your intended command, `./catcher [problematic command]`. Catcher then sends your error message along with a snippet of your system information to an LLM model with a `ai.hackclub.com` API key.

## Installation

You can install Catcher using Cargo:
```bash
cargo install --git https://github.com/simon0302010/catcher
```

## Usage

Just start catcher with any command as an argument:
```bash
catcher <Any Terminal Command>
```
