use chrono::Local;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use dirs::home_dir;
use indicatif::{ProgressBar, ProgressStyle};
use std::ffi::OsStr;
use std::{
    fs,
    io::{self, Read, Write},
};
use walkdir::WalkDir;
use zip::{read::ZipArchive, write::FileOptions, CompressionMethod, ZipWriter};

#[derive(Parser)]
#[command(name = "nvimTimeMachine", version)]
#[command(about = "Manage Neovim time capsules", long_about = None)]
struct Cli {
    /// Create a new capsule
    #[arg(short = 'c', long)]
    create_capsule: bool,

    /// List existing capsules
    #[arg(short = 'l', long)]
    list_capsules: bool,

    /// Restore from a capsule
    #[arg(short = 'r', long)]
    restore_capsule: bool,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    if args.create_capsule {
        create_capsule()?;
    } else if args.list_capsules {
        list_capsules()?;
    } else if args.restore_capsule {
        restore_capsule()?;
    }

    Ok(())
}

fn create_capsule() -> io::Result<()> {
    let home = home_dir().expect("Could not find HOME");
    let sources = [
        home.join(".local/share/nvim"),
        home.join(".config/nvim"),
        home.join(".cache/nvim"),
    ];
    let capsule_dir = home.join(".nvim_capsules");
    fs::create_dir_all(&capsule_dir)?;

    let timestamp = Local::now().format("%Y%m%d%H%M%S");
    let zip_path = capsule_dir.join(format!("nvim_backup_{}.zip", timestamp));

    // count files
    let total = sources
        .iter()
        .flat_map(|d| WalkDir::new(d).into_iter().filter_map(Result::ok))
        .filter(|e| e.file_type().is_file())
        .count() as u64;

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .expect("invalid progress bar template"),
    );
    let file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);
    for dir in &sources {
        for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                pb.inc(1);
                // build the path inside the zip so it’s relative
                let rel = path.strip_prefix(&home).unwrap();
                zip.start_file_from_path(rel, options)?;
                let mut f = fs::File::open(path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                zip.write_all(&buf)?;
            }
        }
    }

    zip.finish()?;
    pb.finish_with_message("🕒 Capsule created!");
    Ok(())
}

fn list_capsules() -> io::Result<()> {
    let capsule_dir = home_dir().expect("HOME not set").join(".nvim_capsules");

    if !capsule_dir.exists() {
        println!("No capsules found.");
        return Ok(());
    }

    // gather all .zip files
    let mut entries: Vec<_> = fs::read_dir(&capsule_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension() == Some(OsStr::new("zip")))
        .collect();

    // sort by modified time (oldest first)
    entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

    // now enumerate them
    for (i, entry) in entries.iter().enumerate() {
        let idx = i + 1;

        // pull the OsString into a local so .to_string_lossy()
        // doesn't borrow from a temporary
        let os_name = entry.file_name();
        let name = os_name.to_string_lossy();

        println!("[\x1b[33m\x1b[0m ]:\x1b[32m({})\x1b[0m: \"{}\"", idx, name);
    }

    Ok(())
}
fn restore_capsule() -> io::Result<()> {
    let home = home_dir().expect("HOME not set");
    let capsule_dir = home.join(".nvim_capsules");
    if !capsule_dir.exists() {
        println!("No capsules found.");
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(&capsule_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension() == Some(OsStr::new("zip")))
        .collect();
    entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

    let names: Vec<String> = entries
        .iter()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a capsule to restore")
        .items(&names)
        .default(0)
        .interact()
        .unwrap();
    let selected_path = entries[selection].path();

    let backup = Confirm::new()
        .with_prompt("Backup existing Neovim directories (rename with timestamp)?")
        .default(true)
        .interact()
        .unwrap();

    let ts = Local::now().format("%Y%m%d%H%M%S");
    let targets = [
        home.join(".local/share/nvim"),
        home.join(".config/nvim"),
        home.join(".cache/nvim"),
    ];
    for dir in &targets {
        if dir.exists() {
            if backup {
                let backup_path = dir.with_file_name(format!("nvim{}", ts));
                fs::rename(dir, backup_path)?;
            } else {
                fs::remove_dir_all(dir)?;
            }
        }
    }

    let file = fs::File::open(&selected_path)?;
    let mut archive = ZipArchive::new(file)?;
    let total = archive.len() as u64;
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .expect("Invalid progress template"),
    );

    for i in 0..archive.len() {
        let mut zip_file = archive.by_index(i)?;
        let outpath = home.join(zip_file.mangled_name());
        if zip_file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut zip_file, &mut outfile)?;
        }
        pb.inc(1);
    }
    pb.finish_with_message("🕒 Restoration complete!");

    Ok(())
}
