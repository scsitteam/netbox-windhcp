use std::{collections::HashMap, fs::File, io::{self, BufRead}, net::IpAddr};

use chrono::NaiveDate;
use log::{debug, error, info};
use glob::glob;
use netbox_windhcp::Config;

use netbox_windhcp::sync::netbox::NetboxApi;


fn main() {
    let config = match Config::load_from_file() {
        Ok(config) => config,
        Err(e) => {
            println!("Error reading config: {}", e);
            std::process::exit(exitcode::CONFIG);
        }
    };
    
    let dir = match config.sync.logs.dir {
        Some(dir) => dir,
        None => {
            error!("No log dir configure");
            return;
        },
    };
    let pattern = format!("{}\\DhcpSrvLog-*.log", dir.to_str().unwrap()); 

    config.log.setup("log");

    debug!("Parse Logfiles: {:?}", pattern);

    let mut last_lease = HashMap::new();

    for file in glob("logs\\DhcpSrvLog-*.log").expect("Failed to read glob pattern") {
        match file {
            Ok(filename) => {
                debug!("Parse Logfile: {:?}", filename.display());
                let file = File::open(filename).unwrap();

                for line in io::BufReader::new(file).lines() {
                    let line = line.unwrap();
                    let values: Vec<&str> = line.split(",").collect();
                    if values[0] != "10" && values[0] != "11" {
                        continue;
                    }

                    let ip: IpAddr = values[4].parse().unwrap();
                    let date = NaiveDate::parse_from_str(values[1], "%m/%d/%y").unwrap();

                    let old_entry = last_lease.get(&ip);

                    match old_entry {
                        Some(olddate) if (&date > &olddate) => {
                            last_lease.insert(ip, date);
                        },
                        Some(_) => {},
                        None => {
                            last_lease.insert(ip, date);
                        },
                    }
                }
            },
            Err(e) => println!("{:?}", e),
        }
    }

    let api = NetboxApi::new(&config.sync.netbox);
    for ip in api.get_reservations().unwrap() {
        let addr: IpAddr = std::net::IpAddr::V4(ip.address());
        let last = last_lease.get(&addr);

        match (last, ip.dhcp_reservation_last_active()) {
            (None, _) => {
                debug!("{:?} no lease action found", addr);
            },
            (Some(dhcp_date), Some(netbox_date)) if dhcp_date > &netbox_date => {
                info!("{:?} update dhcp_reservation_last_active to {}", addr, dhcp_date);
                match api.set_ip_last_active(&ip, dhcp_date) {
                    Ok(_) => info!("{:?} update dhcp_reservation_last_active to {}", addr, dhcp_date),
                    Err(e) => error!(" Reservation {}: Updating last used. {}", addr, e),
                }
            },
            (Some(_), _) => {
                debug!("{:?} no lease action found", addr);
            },
        }
    }
    
}
