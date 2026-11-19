use std::collections::HashSet;
use std::net::IpAddr;
use std::env;

pub fn get_allowed_ips() -> Option<HashSet<IpAddr>> {
    let value = env::var("FCGI_WEB_SERVER_ADDRS").ok()?;

    let ips: HashSet<IpAddr> = value
        .split(',')
        .filter_map(|s| s.trim().parse::<IpAddr>().ok())
        .collect();

    if ips.is_empty() { None }
    else { Some(ips) }
}

pub fn get_allowed_paths() -> Option<HashSet<String>> {
    let value = env::var("FCGI_WEB_SERVER_ADDRS").ok()?;

    let paths: HashSet<String> = value
        .split(',')
        .map(|s| String::from(s))
        .collect();

    Some(paths)
}