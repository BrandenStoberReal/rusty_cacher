use std::{fs::{self, DirEntry}, io, path::{self, Path}, thread, time::Duration};
use clap::Parser;

fn createdir(path: &str) -> std::io::Result<()> {
    if !Path::new(path).exists() {
        fs::create_dir(path)?;
    }
    Ok(())
}

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

    if args.cachepath != "null" {
        cachepath = args.cachepath;
    } else {
        #[cfg(unix)]
        let app_data = std::env::var("HOME").expect("No HOME directory");
        #[cfg(windows)]
        let app_data: String = std::env::var("APPDATA").expect("No APPDATA directory");

        cachepath = app_data + "\\..\\LocalLow\\VRChat\\VRChat\\Cache-WindowsPlayer";
    }

    createdir(&args.backuppath).expect("Error creating the directory.");
    backup_path = args.backuppath;

    println!("{}", String::from("Cache Directory: ") + path::absolute(&cachepath).unwrap().display().to_string().as_str());
    println!("{}", String::from("Backup Directory: ") + path::absolute(&backup_path).unwrap().display().to_string().as_str());
    loop {
        println!("Beginning backup...");
        let paths = fs::read_dir(&cachepath).unwrap();
        for path in paths {
            // Clone our variables hehe
            let clonedbackup: String = (&backup_path).clone().to_owned();
            let refpath:Result<&DirEntry, &io::Error> = (&path).as_ref();

            // Set variables
            let filename: String = path::absolute((&refpath).clone().unwrap().path()).unwrap().file_name().unwrap().to_str().unwrap().to_owned();
            let dest: String = clonedbackup + "/" + &filename;

            // Copy files
            if !Path::new(&dest).exists() || &filename == "__info" {
                println!("Copying: {}", path::absolute((&refpath).clone().unwrap().path()).unwrap().display());
                println!("Destination: {}", &dest);
                if (&refpath).unwrap().metadata().unwrap().is_file() {
                    let _ = fs::copy(&refpath.unwrap().path(), &dest);
                } else {
                    copy_dir_all(path::absolute(&refpath.unwrap().path()).unwrap(), path::absolute(&dest).unwrap()).expect("Couldn't copy directory!");
                }
            }
        }
        println!("Sleeping {}s...", args.interval);
        thread::sleep(Duration::from_secs(args.interval));
    }
}