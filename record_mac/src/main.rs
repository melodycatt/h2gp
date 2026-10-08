use std::{
    fs::{self, OpenOptions},
    io::{self, BufRead, Write},
};

// espflash prints a line like:
//   [00:00:01] Flash Complete! MAC address: AB:CD:EF:01:23:45
// or sometimes:
//   MAC address: ab:cd:ef:01:23:45
fn parse_mac(line: &str) -> Option<[u8; 6]> {
    let tag = "MAC address: ";
    let start = line.find(tag)? + tag.len();
    let mac_str = line[start..].trim();

    let parts: Vec<&str> = mac_str.split(':').collect();
    if parts.len() != 6 {
        return None;
    }

    let mut bytes = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        bytes[i] = u8::from_str_radix(part, 16).ok()?;
    }
    Some(bytes)
}

fn mac_to_line(mac: [u8; 6]) -> String {
    format!("[{}]", mac.map(|b| b.to_string()).join(","))
}

fn already_recorded(path: &str, mac: [u8; 6]) -> bool {
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    let target = mac_to_line(mac);
    contents.lines().any(|l| l.trim() == target)
}

fn main() {
    let macs_path = &std::env::var("HYDRO_MAC_FILE").unwrap();
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line.expect("failed to read line");
        println!("{line}\r"); // pass through espflash output

        if let Some(mac) = parse_mac(&line) {
            if already_recorded(macs_path, mac) {
                eprintln!("MAC already recorded: {}", mac_to_line(mac));
            } else {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(macs_path)
                    .expect("failed to open macs.txt");
                writeln!(file, "{}", mac_to_line(mac)).expect("failed to write");
                eprintln!("Recorded new MAC: {}", mac_to_line(mac));
            }
        }
    }
}
