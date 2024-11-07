use std::{fs::{self, DirEntry}, io, path::{self, Path}, thread, time::Duration};
use clap::Parser;
use colored::Colorize;

// Creates a folder.
fn create_dir(path: &str) -> std::io::Result<()> {
    if !Path::new(path).exists() {
        fs::create_dir(path)?;
    }
    Ok(())
}

// Clones a directory and all subdirectories/files.
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Simple program to back up your VRChat cache.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path of your VRChat cache directory.
    #[arg(short, long, default_value_t = String::from("null"))]
    cachepath: String,

    /// Path of the directory to backup the cache to.
    #[arg(short, long, default_value_t = String::from("./backup"))]
    backuppath: String,

    // Time between cache scans in seconds.
    #[arg(short, long, default_value_t = 30)]
    interval: u64,
}

fn main() {
    let args: Args = Args::parse();
    let cachepath: String;
    let backup_path: String;

    // Handle default arguments and stuff
    if args.cachepath != "null" {
        cachepath = args.cachepath;
    } else {
        #[cfg(unix)]
        let app_data = std::env::var("HOME").expect("No HOME directory");
        #[cfg(windows)]
        let app_data: String = std::env::var("APPDATA").expect("No APPDATA directory");

        cachepath = app_data + "\\..\\LocalLow\\VRChat\\VRChat\\Cache-WindowsPlayer"; // Expressions are experimental for attributes so unix users should shove in their own path with the CLI flag
    }

    // Create backup dir if it doesn't exist already
    create_dir(&args.backuppath).expect("Error creating the directory.");
    backup_path = args.backuppath;

    println!("[{}] {}: {}", "OPTION".purple(), "Cache Directory".cyan(), &path::absolute(&cachepath).unwrap().display().to_string().red());
    println!("[{}] {}: {}", "OPTION".purple(), "Backup Directory".cyan(), &path::absolute(&backup_path).unwrap().display().to_string().red());
    loop {
        println!("[{}] {}", "CACHE BACKUP".purple(), "Beginning backup...".green());
        let paths = fs::read_dir(&cachepath).unwrap();
        for path in paths {
            // Clone our variables (?????)
            let clonedbackup: String = (&backup_path).clone().to_owned(); // Backup location clone for borrowing
            let refpath:Result<&DirEntry, &io::Error> = (&path).as_ref(); // No idea what this means

            // Set variables
            let filename: String = path::absolute(&refpath.clone().unwrap().path()).unwrap().file_name().unwrap().to_str().unwrap().to_owned(); // Somehow this is valid rust code
            let mut destPath: path::PathBuf = Path::new(&clonedbackup).to_path_buf(); // Mutable path buffer for backup location
            destPath.push(&filename); // This appends filename so I don't have to hardcode
            let dest: String = destPath.display().to_string();

            // Copy files
            if !Path::new(&dest).exists() || &filename == "__info" {
                println!("[{}] {}: {}", "CACHE BACKUP".purple(), "Cache Location".green(), path::absolute(&refpath.clone().unwrap().path()).unwrap().display().to_string().red()); // These really should be a variable but fuck it
                println!("[{}] {}: {}", "CACHE BACKUP".purple(), "Backup Destination".green(), path::absolute(&dest).unwrap().display().to_string().red());
                if (&refpath).unwrap().metadata().unwrap().is_file() {
                    let _ = fs::copy(&refpath.unwrap().path(), &dest); // This is really only used for __info file
                } else {
                    copy_dir_all(path::absolute(&refpath.unwrap().path()).unwrap(), path::absolute(&dest).unwrap()).expect("Couldn't copy directory!");
                }
            }
        }
        println!("[{}] {} {}{}{}", "TIMEOUT".purple(), "Sleeping".green(), args.interval.to_string().red(), "s".red(), "...".green());
        thread::sleep(Duration::from_secs(args.interval));
    }
}