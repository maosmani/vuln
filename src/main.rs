use rusqlite::{Connection, Result, params};
use std::process::Command;

fn run_command(cmd: &str, args: &[&str]) -> String {
    let output = Command::new(cmd).args(args).output();

    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).into_owned(),
        Err(_) => String::new(),
    }
}

fn main() -> Result<()> {
    let conn = Connection::open("software_report.db")?;

    // Create table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS software (
            id INTEGER PRIMARY KEY,
            name TEXT,
            version TEXT
        )",
        [],
    )?;

    // === Debian packages ===
    let packages = run_command("apt", &["list", "--installed"]);
    for line in packages.lines().skip(1) {
        // skip header line
        if let Some((name, version)) = line.split_once('/') {
            let parts: Vec<&str> = version.split_whitespace().collect();
            if parts.len() > 0 {
                conn.execute(
                    "INSERT INTO software (name, version) VALUES (?1, ?2)",
                    params![name, parts[0]],
                )?;
            }
        }
    }

    // === Flatpak apps ===
    let flatpak = run_command("flatpak", &["list"]);
    for line in flatpak.lines() {
        let parts: Vec<&str> = line.split('\t').collect(); // flatpak uses tab-separated
        if parts.len() >= 2 {
            conn.execute(
                "INSERT INTO software (name, version) VALUES (?1, ?2)",
                params![parts[0], parts[1]],
            )?;
        }
    }

    // === Snap apps ===
    let snap = run_command("snap", &["list"]);
    for (i, line) in snap.lines().enumerate() {
        if i == 0 {
            continue; // skip header row
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            conn.execute(
                "INSERT INTO software (name, version) VALUES (?1, ?2)",
                params![parts[0], parts[1]],
            )?;
        }
    }

    println!("Software (name + version) saved into software_report.db");

    Ok(())
}
