use std::{env::current_dir, fs, path::Path, process::{Command, Stdio}};
use clap::Parser;
use copy_dir::copy_dir;


#[derive(Parser, Debug, Clone)]
#[command(about, long_about = None)]
struct Args {
    /// Directory to store the container
    #[arg(short, long, default_value = "")] // Set default value to an empty string
    directory: String,

    /// List of CPU Threads to allocate (ex. 1,2,3,4)
    #[arg(short, long, default_value = "All Threads")]
    cpus: String,

    /// Maximum memory to allocate (ex. 512M, 1G, 300K)
    #[arg(short, long, default_value = "No Limit")]
    memory: String,
}

fn cleanup_dirs(path: &str) -> Result<(), String> {
    let path = Path::new(path);

    if !path.exists() {
        return Ok(());
    }

    match fs::remove_dir_all(&path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to remove directory '{}': {}", path.display(), e)),
    }
}

fn main() {
    let mut args = Args::parse();

    if args.directory.is_empty() {
        args.directory = current_dir().unwrap().to_string_lossy().to_string();
        args.directory.push_str("/container/") // Set the current directory if the directory field is empty
    }

    let mut cmd = Command::new("systemd-run");

    // Get a tty
    cmd.arg("--tty");

    // Be quiet
    cmd.arg("-q");

    //delete if failure
    cmd.arg("--collect");

    // Set the service name
    cmd.arg("-u").arg("rustainer");

    // Set the memory limit
    if args.memory != "No Limit" {
        let re_mem = regex::Regex::new(r"(\d+)([KMG])").unwrap();
        if !re_mem.is_match(&args.memory) {
            eprintln!("Invalid memory format. Use a number followed by K, M, or G.");
            std::process::exit(1);
        }
        cmd.arg("-p").arg(format!("MemoryMax={}", args.memory));
    }

    // Set the CPU limit
    if args.cpus != "All Threads" {
        let re_cpu = regex::Regex::new(r"(\d+)([ ,]\d+)*").unwrap();
        if !re_cpu.is_match(&args.cpus) {
            eprintln!("Invalid CPU format. Use a list of numbers separated by commas or spaces.");
            std::process::exit(1);
        }
        cmd.arg("-p").arg(format!("AllowedCPUs={}", args.cpus));
    }

    // Setup the container directory
    match copy_dir("./minifs", &args.directory) {
        Ok(_) => println!("Files copied successfully."),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

    // isolate pid namespace
    cmd.arg("bash").arg("-c")
    .arg(format!("unshare --mount --pid --fork --mount-proc --root={} -- /bin/busybox sh", args.directory));

    //Run the container
    cmd.stdin(Stdio::inherit())  // Inherit standard input
        .stdout(Stdio::inherit()) // Inherit standard output
        .stderr(Stdio::inherit()); // Inherit standard error
        
    clearscreen::clear().expect("Failed to clear screen");

    println!("Running container with the following settings:\n");
    println!("Directory: {}", args.directory);
    println!("CPUs: {}", args.cpus);
    println!("Memory: {}\n\n", args.memory);
    println!("To use standard commands, like 'ls', 'ps', 'top', etc., you neeed to call /bin/busybox <command>.\n");
    
    let mut cmd = cmd.spawn().expect("failed to start container");

    // Wait for the child process to finish
    cmd.wait().expect("failed to wait on child");
    
    match cleanup_dirs(&args.directory) {
        Ok(_) => println!("\nDirectory '{}' removed successfully.", args.directory),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}