use std::env;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

extern crate chrono;
extern crate homedir;
use chrono::prelude::*;
use homedir::my_home;
fn main() -> std::io::Result<()> {
    let mut args = env::args();
    let argslength = args.len();
    if argslength != 2 {
        println!("A simple program to create new typst file, watch, edit and show them with sioyek and neovim.\n");
        println!("Usage: {}[EXE] <PATH>", args.nth(0).unwrap());
        return Ok(());
    }
    let pathstring = args.last().unwrap();
    let mut file = File::create(&pathstring)?;
    let date = Local::now();
    let selected = std::path::PathBuf::from(&pathstring);
    let template = std::fs::read_to_string(
        my_home().unwrap().unwrap().to_str().unwrap().to_owned() + "/.config/ntpdf/ntpdf.typ",
    )?
    .replace(
        "{FILE_NAME}",
        &selected
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .replace("_", " "),
    )
    .replace("{CURRENT_DATE}", &date.format("%d/%m/%Y").to_string());
    file.write_all(template.as_bytes())?;
    if !selected.is_file() {
        println!("No file selected.");
        return Ok(());
    }
    let pdf_file = selected.with_extension("pdf");
    println!("{:?}", pdf_file.as_os_str());
    Command::new("typst")
        .arg("compile")
        .arg(&selected)
        .output()
        .expect("Failed to compile file");
    while !pdf_file.exists() {
        sleep(Duration::from_secs(1));
        println!("Waiting for typst compilation");
    }
    Command::new("typst")
        .arg("watch")
        .arg(&selected)
        .stdout(Stdio::from(File::create("/dev/null").unwrap()))
        .stderr(Stdio::from(File::create("/dev/null").unwrap()))
        .spawn()
        .expect("Failed to watch file");
    Command::new("sioyek")
        .arg(&pdf_file)
        .arg("--page")
        .arg("2")
        .stdin(Stdio::null())
        .stdout(Stdio::from(File::create("/dev/null").unwrap()))
        .stderr(Stdio::from(File::create("/dev/null").unwrap()))
        .spawn()
        .expect("Failed to open PDF file");
    let mut nvim = Command::new("helix")
        .arg(&selected)
        .spawn()
        .expect("Failed to open MD file in Neovim");
    let result = nvim.wait().expect("Failed to wait for Neovim");
    if result.success() {
        println!("Neovim exited");
    } else {
        println!("Neovim exited with an error");
    }

    Ok(())
}
