pub fn parse_port(port_arg: &str) -> Result<(u16, u16), String> {
    let (start_str, end_str) = port_arg
        .split_once(':')
        .ok_or("port_range should be <start:end>")?;

    let start = match start_str.parse::<u16>() {
        Ok(n) => n,
        Err(_) => return Err(format!("start_str {} not a number", start_str)),
    };

    let end = match end_str.parse::<u16>() {
        Ok(n) => n,
        Err(_) => return Err(format!("end_str {} not a number", end_str)),
    };

    if start == 0 || end == 0 {
        return Err("port 0 not valid".to_string());
    }

    if start >= end {
        return Err("start_port should be less than end_port".to_string());
    }

    return Ok((start, end));
}

// tests
#[cfg(test)]
mod tests {
    use super::parse_port;

    #[test]
    fn ok_range() {
        assert_eq!(parse_port("80:2000"), Ok((80, 2000)));
    }

    #[test]
    fn wrong_delimiter() {
        assert!(parse_port("80-2000").is_err());
    }

    #[test]
    fn non_numeric() {
        assert!(parse_port("abc:80").is_err());
    }

    #[test]
    fn start_gt_end() {
        assert!(parse_port("2000:80").is_err());
    }

    #[test]
    fn port_zero() {
        assert!(parse_port("0:10").is_err());
    }
}
