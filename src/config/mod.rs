use crate::backend::DNSBackend;
use log::warn;
use std::collections::HashMap;
use std::fs::{metadata, read_dir, read_to_string};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::vec::Vec;

// Parse configuration files in the given directory.
// Configuration files are formatted as follows:
// The name of the file will be interpreted as the name of the network.
// The first line must be the gateway IP(s) of the network, comma-separated.
// All subsequent individual lines contain info on a single container and are
// formatted as:
// <container ID, space, IPv4 address, space, IPv6 address, space, comma-separated list of name and aliases>
// Where space is a single space character.
// Returns a complete DNSBackend struct (all that is necessary for looks) and
pub fn parse_configs(
    dir: &str,
) -> Result<(DNSBackend, Vec<Ipv4Addr>, Vec<Ipv6Addr>), std::io::Error> {
    if !metadata(dir)?.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("config directory {} must exist and be a directory", dir),
        ));
    }

    let mut network_membership: HashMap<String, Vec<String>> = HashMap::new();
    let mut container_ips: HashMap<String, Vec<IpAddr>> = HashMap::new();
    let mut network_names: HashMap<String, HashMap<String, Vec<IpAddr>>> = HashMap::new();
    let mut listen_ips_4: Vec<Ipv4Addr> = Vec::new();
    let mut listen_ips_6: Vec<Ipv6Addr> = Vec::new();

    // Enumerate all files in the directory, read them in one by one.
    // Steadily build a map of what container has what IPs and what
    // container is in what networks.
    let configs = read_dir(dir)?;
    for config in configs {
        // Each entry is a result. Interpret Err to mean the config was removed
        // while we were working; warn only, don't error.
        // Might be safer to completely restart the process, but there's also a
        // chance that, if we do that, we never finish and update the config,
        // assuming the files in question are modified at a sufficiently high
        // rate.
        match config {
            Ok(cfg) => {
                let (bind_ips, ctr_entry) = parse_config(&(cfg.path().as_path()))?;

                let network_name: String = match cfg.path().file_name() {
                    // This isn't *completely* safe, but I do not foresee many
                    // cases where our network names include non-UTF8
                    // characters.
                    Some(s) => match s.to_str() {
                        Some(st) => st.to_string(),
                        None => return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("configuration file {} name has non-UTF8 characters", s.to_string_lossy()),
                        )),
                    },
                    None => return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("configuration file {} does not have a file name, cannot identify network name", cfg.path().to_string_lossy()),
                        )),
                };

                for ip in bind_ips {
                    match ip {
                        IpAddr::V4(a) => listen_ips_4.push(a),
                        IpAddr::V6(b) => listen_ips_6.push(b),
                    }
                }

                for entry in ctr_entry {
                    // Container network membership
                    let ctr_networks = network_membership
                        .entry(entry.id.clone())
                        .or_insert(Vec::new());
                    ctr_networks.push(network_name.clone());

                    // Container IP addresses
                    let new_ctr_ips: Vec<IpAddr> = vec![IpAddr::V4(entry.v4), IpAddr::V6(entry.v6)];
                    let ctr_ips = container_ips.entry(entry.id.clone()).or_insert(Vec::new());
                    ctr_ips.append(&mut new_ctr_ips.clone());

                    // Network aliases to IPs map.
                    let network_aliases = network_names
                        .entry(network_name.clone())
                        .or_insert(HashMap::new());
                    for alias in entry.aliases {
                        let alias_entries = network_aliases.entry(alias).or_insert(Vec::new());
                        alias_entries.append(&mut new_ctr_ips.clone());
                    }
                }
            }
            Err(e) => warn!("Error reading config file for server update: {}", e),
        }
    }

    // Set up types to be returned.
    let mut ctrs: HashMap<IpAddr, Vec<String>> = HashMap::new();

    for (ctr_id, ips) in container_ips {
        match network_membership.get(&ctr_id) {
            Some(s) => {
                for ip in ips {
                    let ip_networks = ctrs.entry(ip).or_insert(Vec::new());
                    ip_networks.append(&mut s.clone());
                }
            }
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                    "Container ID {} has an entry in IPs table, but not network membership table",
                    ctr_id
                ),
                ))
            }
        }
    }

    Ok((
        DNSBackend::new(&ctrs, &network_names),
        listen_ips_4,
        listen_ips_6,
    ))
}

// A single entry in a config file
struct CtrEntry {
    id: String,
    v4: Ipv4Addr,
    v6: Ipv6Addr,
    aliases: Vec<String>,
}

// Read and parse a single given configuration file
fn parse_config(path: &std::path::Path) -> Result<(Vec<IpAddr>, Vec<CtrEntry>), std::io::Error> {
    let content = read_to_string(path)?;
    let mut is_first = true;

    let mut bind_addrs: Vec<IpAddr> = Vec::new();
    let mut ctrs: Vec<CtrEntry> = Vec::new();

    // Split on newline, parse each line
    for line in content.split("\n") {
        if is_first {
            // First line is comma-separated V4 and V6
            for ip in line.split(",") {
                let local_ip = match ip.parse() {
                    Ok(l) => l,
                    Err(e) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("error parsing IP address {}: {}", ip, e),
                        ))
                    }
                };
                bind_addrs.push(local_ip);
            }

            is_first = false;
            continue;
        }

        // Split on space
        let parts = line.split(" ").collect::<Vec<&str>>();
        if parts.len() != 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "configuration file {} line {} is improperly formatted - too few entries",
                    path.to_string_lossy(),
                    line
                ),
            ));
        }

        let v4_addr: Ipv4Addr = match parts[1].parse() {
            Ok(i) => i,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("error parsing IP address {}: {}", parts[1], e),
                ))
            }
        };
        let v6_addr: Ipv6Addr = match parts[2].parse() {
            Ok(i) => i,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("error parsing IP address {}: {}", parts[1], e),
                ))
            }
        };
        let aliases: Vec<String> = parts[3]
            .split(",")
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        if aliases.len() == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "configuration file {} line {} is improperly formatted - no names given",
                    path.to_string_lossy(),
                    line
                ),
            ));
        }

        ctrs.push(CtrEntry {
            id: parts[0].to_string(),
            v4: v4_addr,
            v6: v6_addr,
            aliases: aliases,
        });
    }

    // Must provide at least one bind address
    if bind_addrs.len() == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "configuration file {} does not provide any bind addresses",
                path.to_string_lossy()
            ),
        ));
    }

    Ok((bind_addrs, ctrs))
}
