use colored::Colorize;
use dotenv::dotenv;
use std::io::Write;

fn main() {
    dotenv().ok();

    // Attempt to find .crt and .key files in the current directory, exit if not found
    let cert = std::env::var("CERT").expect("No .crt file found");
    let key = std::env::var("KEY").expect("No .key file found");
    std::fs::metadata(&cert).expect("No .crt file found");
    std::fs::metadata(&key).expect("No .key file found");

    std::env::var("REGEX").expect("No REGEX found");
    let flag_re = regex::Regex::new(std::env::var("REGEX").expect("No REGEX found").as_str())
        .expect("Failed to compile REGEX");
    let recon_re = regex::Regex::new(r#"This flag is "Recon (\d+)"#).unwrap();

    // Attempt to find a database file, create one if not found
    let db = "flags.db";
    let mut flags: Vec<(String, u8)> = Vec::new();
    if std::fs::metadata(db).is_err() {
        let mut file = std::fs::File::create(db).expect("Failed to create database file");
        bincode::serialize_into(&mut file, &flags).expect("Failed to write to database file");
    } else {
        let file = std::fs::File::open(db).expect("Failed to open database file");
        flags = bincode::deserialize_from(file).expect("Failed to read from database file");
    }

    loop {
        // get url, exit if empty
        print!("Enter a URL: ");
        std::io::stdout().flush().unwrap();
        let mut url = String::new();
        std::io::stdin().read_line(&mut url).unwrap();
        url = url.trim().to_string();
        if url.is_empty() {
            break;
        }

        // if URL is a number, attempt to find it in the database
        if let Ok(flag_number) = url.parse::<u8>() {
            if let Some((real, _)) = flags.iter().find(|(_, u)| u == &flag_number) {
                println!("{}: {}", "FND".bright_blue(), real,);
            }
            continue;
        }

        // if URL is "missing", print all numbers from 1 to 34, number will be green if found, red if not
        if url == "missing" {
            for i in 1..=34 {
                let s = i.to_string();
                if flags.iter().any(|(_, u)| u == &i) {
                    print!("{} ", s.green());
                } else {
                    print!("{} ", s.red());
                }
            }
            println!();
            continue;
        }

        // run curl command
        let output = std::process::Command::new("curl")
            .arg("--cert")
            .arg(&cert)
            .arg("--key")
            .arg(&key)
            .arg(format!("https://{}", url))
            .output()
            .expect("Failed to run curl command");

        // dbg!(std::str::from_utf8(&output.stdout).unwrap());

        // check for flag
        if let Some(flag) = flag_re.find(std::str::from_utf8(&output.stdout).unwrap()) {
            let flag = flag.as_str().to_string();
            if let Some(flag_number) = recon_re.find(std::str::from_utf8(&output.stdout).unwrap()) {
                let flag_number = flag_number
                    .as_str()
                    .split_whitespace()
                    .last()
                    .unwrap()
                    .parse()
                    .unwrap();
                if let Some((_, existing_flag_number)) = flags.iter().find(|(u, _)| u == &url) {
                    println!("{}: {} - {}", "OLD".yellow(), url, existing_flag_number);
                } else {
                    println!("{}: {} - {} - {}", "NEW".green(), url, flag_number, flag);
                    flags.push((url, flag_number));
                }
            } else {
                println!("{}: {} - {}", "WAT".blue(), url, flag);
            }
        } else {
            println!("{}: {}", "NIL".red(), url);
        }
    }

    // write to bincode of flags to flags.db
    let mut file = std::fs::File::create(db).expect("Failed to create database file");
    bincode::serialize_into(&mut file, &flags).expect("Failed to write to database file");
}
