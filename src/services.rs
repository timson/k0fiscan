use std::collections::HashMap;

pub struct ServiceEntry {
    pub name: &'static str,
    pub comment: &'static str,
    pub prb: f32,
}

pub type ServiceMap = HashMap<u16, ServiceEntry>;

pub fn get_services() -> ServiceMap {
    static RAW: &str = include_str!("../data/nmap-services");
    let mut map = HashMap::new();

    for line in RAW.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (fields_part, comment_part) = match line.split_once('#') {
            Some((f, c)) => (f.trim_end(), c.trim()),
            None => (line, ""), // no comment column
        };

        let mut fields = fields_part.split_whitespace();
        let name = match fields.next() {
            Some(s) => s,
            None => continue,
        };
        let port_proto = match fields.next() {
            Some(s) => s,
            None => continue,
        };
        let prb_str = match fields.next() {
            Some(s) => s,
            None => continue,
        };

        let (port_str, _proto) = match port_proto.split_once('/') {
            Some((p, "tcp")) => (p, "tcp"),
            _ => continue,
        };

        let port: u16 = match port_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let prb: f32 = match prb_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let comment = comment_part.trim_start_matches('#').trim();

        map.insert(port, ServiceEntry { name, comment, prb });
    }

    map
}
