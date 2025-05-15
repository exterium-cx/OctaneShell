use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::process::{Command, Child};
use git2::Repository;
use colored::*;

use std::sync::{Arc, Mutex};

fn get_git_branch() -> Option<(String, bool)> {
    let repo = Repository::discover(".").ok()?;
    let head = repo.head().ok()?;
    if !head.is_branch() {
        return None;
    }
    let branch_name = head.shorthand()?.to_string();

    let statuses = repo.statuses(None).ok()?;
    let dirty = statuses.iter().any(|entry| {
        let s = entry.status();
        s.is_index_new() || s.is_index_modified() || s.is_wt_modified() || s.is_wt_new()
    });

    Some((branch_name, dirty))
}

fn expand_env_vars(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            let mut var = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    var.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            if let Ok(val) = env::var(&var) {
                result.push_str(&val);
            } else {
                result.push_str(&format!("${}", var));
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn run_builtin(cmd: &str, args: &[&str], bg_processes: &Arc<Mutex<HashMap<u32, Child>>>) -> bool {
    match cmd {
        "calc" => {
            if args.is_empty() {
                println!("Usage: calc <expression>");
                return true;
            }
            let expression = args.join(" ");
            match meval::eval_str(&expression) {
                Ok(result) => println!("{}", result),
                Err(e) => println!("Error evaluating expression: {}", e),
            }
            true
        }
        "exit" => {
            std::process::exit(0);
        }
        "cd" => {
            let new_dir = args.get(0).cloned().unwrap_or(".");
            if let Err(e) = env::set_current_dir(new_dir) {
                println!("Error: {}", e);
            }
            true
        }
        "pwd" => {
            match env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(e) => println!("Error: {}", e),
            }
            true
        }
        "clear" => {
            if cfg!(windows) {
                Command::new("cmd").args(["/C", "cls"]).status().unwrap();
            } else {
                Command::new("clear").status().unwrap();
            }
            true
        }
"jobs" => {
    let bg = bg_processes.lock().unwrap();
    if bg.is_empty() {
        println!("No background jobs");
    } else {
        for (pid, _child) in bg.iter() {
            println!("PID {} - Running", pid);
        }
    }
    true
}

        "kill" => {
            if args.is_empty() {
                println!("Usage: kill <pid>");
                return true;
            }
            let pid = match args[0].parse::<u32>() {
                Ok(p) => p,
                Err(_) => {
                    println!("Invalid PID");
                    return true;
                }
            };
            let mut bg = bg_processes.lock().unwrap();
            if let Some(mut child) = bg.remove(&pid) {
                match child.kill() {
                    Ok(_) => println!("Killed process {}", pid),
                    Err(e) => println!("Failed to kill {}: {}", pid, e),
                }
            } else {
                println!("No such background process: {}", pid);
            }
            true
        }
        _ => false,
    }
}

fn main() {
    let mut aliases = HashMap::new();
    aliases.insert("ll", "ls -la");
    aliases.insert("..", "cd ..");
    aliases.insert("h", "cd ~");

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Console::{
            GetConsoleMode, SetConsoleMode, GetStdHandle,
            ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_OUTPUT_HANDLE,
        };
    
        unsafe {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            let mut mode = 0;
            if GetConsoleMode(handle, &mut mode) != 0 {
                SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }    

    let bg_processes: Arc<Mutex<HashMap<u32, Child>>> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
        let cwd_str = cwd.to_string_lossy();

        let git_info = get_git_branch();

        let prompt = match git_info {
            Some((branch, true)) => format!(
                "{}{} ({branch}*) $ ",
                "octane:".blue().bold(),
                cwd_str
            ),
            Some((branch, false)) => format!(
                "{}{} ({branch}) $ ",
                "octane:".blue().bold(),
                cwd_str
            ),
            None => format!(
                "{}{} $ ",
                "octane:".blue().bold(),
                cwd_str
            ),
        };               

        print!("{prompt}");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let mut input = input.trim().to_string();
        if input.is_empty() {
            continue;
        }

        input = expand_env_vars(&input);

        if let Some(replacement) = aliases.get(input.as_str()) {
            input = replacement.to_string();
        }

        let mut parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let background = if parts.last() == Some(&"&") {
            parts.pop();
            true
        } else {
            false
        };

        let cmd = parts[0];
        let args = &parts[1..];

        if run_builtin(cmd, args, &bg_processes) {
            continue;
        }

        match Command::new(cmd).args(args).spawn() {
            Ok(mut child) => {
                if background {
                    let pid = child.id();
                    println!("Started background job with PID {}", pid);
                    bg_processes.lock().unwrap().insert(pid, child);
                } else {
                    match child.wait() {
                        Ok(_status) => {}
                        Err(e) => {
                            println!("Error waiting on process: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error running command: {}", e);
            }
        }
    }
}

