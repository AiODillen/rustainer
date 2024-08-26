use std::{env::current_dir, fs, path::Path, process::{Command, Stdio}};
use clap::Parser;


#[derive(Parser, Debug, Clone)]
#[command(about, long_about = None)]
struct Args {
    /// Directory to store the container
    #[arg(short, long, default_value = "./container")] // Set default value to an empty string
    directory: String,

    /// List of CPU Threads to allocate (ex. 1,2,3,4  or 1 5 7 10)
    #[arg(short, long, default_value_t = 1)]
    cpus: u8,

    /// Maximum memory to allocate (ex. 512M, 1G, 300K)
    #[arg(short, long, default_value = "512M")]
    memory: String,
}

fn create_directory(path: &str) -> Result<(), String> {

    let path = Path::new(path);

    if path.exists() {
        return Ok(());
    }

    match fs::create_dir_all(&path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to create directory '{}': {}", path.display(), e)),
    }
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
        args.directory.push_str("/container") // Set the current directory if the directory field is empty
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

    // Set the working directory
    cmd.arg(format!("--working-directory={}", args.directory));

    // Set the memory limit
    let re_mem = regex::Regex::new(r"(\d+)([KMG])").unwrap();
    if !re_mem.is_match(&args.memory) {
        eprintln!("Invalid memory format. Use a number followed by K, M, or G.");
        std::process::exit(1);
    }
    cmd.arg("-p").arg(format!("MemoryMax={}", args.memory));

    // Set the CPU limit
    cmd.arg("-p").arg(format!("AllowedCPUs={}", args.cpus));

    // isolate pid namespace
    cmd.arg("bash").arg("-c").arg("unshare --mount --pid --fork --mount-proc");
    cmd.arg("bash");

    // Setup the container directory
    match create_directory(&args.directory) {
        Ok(_) => println!("Directory '{}' created successfully.", args.directory),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

    

    //Run the container
    cmd.stdin(Stdio::inherit())  // Inherit standard input
        .stdout(Stdio::inherit()) // Inherit standard output
        .stderr(Stdio::inherit()); // Inherit standard error
        
    clearscreen::clear().expect("Failed to clear screen");

    println!("Running container with the following settings:\n");
    println!("Directory: {}", args.directory);
    println!("CPUs: {}", args.cpus);
    println!("Memory: {}\n\n", args.memory);
    
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