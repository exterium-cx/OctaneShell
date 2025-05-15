# Octane Shell

A lightweight Rust-based shell with built-in commands, Git branch integration, background job management, and more.

---

## Features

- **Custom Prompt**
  - Displays the current working directory.
  - Shows the current Git branch if inside a Git repository.
  - Indicates if the Git repository has uncommitted changes (marked with `*`).

- **Built-in Commands**
  - `calc <expression>`: Evaluate mathematical expressions (powered by `meval`).
  - `exit`: Exit the shell.
  - `cd <dir>`: Change the current directory.
  - `pwd`: Print the current working directory.
  - `clear`: Clear the terminal screen.
  - `jobs`: List all currently running background processes.
  - `kill <pid>`: Kill a background process by its PID.

- **Background Process Support**
  - Run commands in the background by appending `&` at the end.
  - Background jobs are tracked with their process IDs (PIDs).
  - Manage background jobs using `jobs` and `kill`.

- **Aliases**
  - Built-in aliases for common commands:
    - `ll` → `ls -la`
    - `..` → `cd ..`
    - `h` → `cd ~`

- **Environment Variable Expansion**
  - Supports expanding environment variables using `$VAR` syntax in commands.

- **Cross-Platform Clear Command**
  - Supports both Windows (`cls`) and Unix (`clear`) systems for clearing the terminal.

- **Windows Virtual Terminal Support**
  - Enables ANSI escape code support for colored prompts on Windows terminals.

---

## Usage

1. **Start the shell**

   Run the compiled binary:

   ```bash
   ./octane-shell
