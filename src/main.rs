use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::net::Ipv4Addr;
use std::str::FromStr;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RawIpRange {
    start_ip: String,
    end_ip: String,
    country: String,
}

#[derive(Debug)]
struct IpRange {
    start: u32,
    end: u32,
    country: String,
}

fn ip_to_u32(ip: &str) -> Option<u32> {
    Ipv4Addr::from_str(ip).ok().map(u32::from)
}

fn load_ip_ranges(path: &str) -> Result<Vec<IpRange>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut ranges = Vec::new();

    for result in rdr.deserialize() {
        let raw: RawIpRange = result?;
        if let (Some(start), Some(end)) = (ip_to_u32(&raw.start_ip), ip_to_u32(&raw.end_ip)) {
            ranges.push(IpRange {
                start,
                end,
                country: raw.country,
            });
        }
    }

    // Sort ranges by start IP for binary search
    ranges.sort_by_key(|r| r.start);
    Ok(ranges)
}

fn lookup_country<'a>(ip_str: &str, ranges: &'a [IpRange]) -> Option<&'a str> {
    let ip = ip_to_u32(ip_str)?;

    // Binary search for the IP range that contains the IP
    let mut low = 0;
    let mut high = ranges.len();

    while low < high {
        let mid = (low + high) / 2;
        let range = &ranges[mid];

        if ip < range.start {
            high = mid;
        } else if ip > range.end {
            low = mid + 1;
        } else {
            return Some(&range.country);
        }
    }

    None
}

fn main() -> Result<(), Box<dyn Error>> {
    let ranges = load_ip_ranges("dbip-country-lite.csv")?;

    print!("Enter an IP address: ");
    std::io::stdout().flush()?; // show prompt

    let mut ip = String::new();
    std::io::stdin().read_line(&mut ip)?;
    let ip = ip.trim();

    match lookup_country(ip, &ranges) {
        Some(country) => println!("{} → {}", ip, country),
        None => println!("Country not found for {}", ip),
    }

    Ok(())
}
