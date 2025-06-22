# ☕ k0fiscan

Fast, async TCP port scanner brewed with **Tokio** (Rust) — because your coffee break is short and your networks are large.

```

k0fi --network 192.168.1.0/24 --port-range 22:443 --max-tasks 800

````

## Features/Limitations
- **IPv4 / IPv6** support
- TCP only
- nmap-services included + port statistics for fast scan
- CIDR, single host, custom IP ranges, IP(s) list
- Concurrency limiter (`--max-tasks`)
- Output as **table** or **JSON** (`--output table|json`)
- Graceful _Ctrl+C_ — stops immediately and prints partial results

## Install

To install k0fiscan, run the following command:

```bash
curl -sSL https://timson.github.io/k0fiscan/install.sh | bash
```


## Build from source

Clone the repository and build it:

```bash
cargo build --release
```


## Usage
```bash
# scan one host, top 100 ports
k0fi --target 10.0.0.5 --port-range 1:100

# scan entire /24
k0fi --network 172.16.0.0/24 --port-range 1:1024

# scan an IP range
k0fi --start-ip 10.1.1.10 --end-ip 10.1.1.20 --output json

# scan an IP list with 30% of top ports according port statistic from nmap-services
k0fi --list 192.168.2.1,192.168.2.2 -x 30
````
You can combine json output with using **jq** tool, it may be useful for further processing.

Some examples:

Gives you the number of open ports

```bash
k0fi -n 192.168.2.0/24 -m 2000 -o json | jq 'length'
```
Gives you the number of unique IPs with open ports
```bash
k0fi -n 192.168.2.0/24 -m 2000 -o json | jq '[.[] | .ip] | unique | length'
```
Shows you IPs with open ports and the number of ports
```bash
k0fi -n 192.168.2.0/24 -m 2000 -o json | jq 'group_by(.ip) | map({ip: .[0].ip, ports: length})'
```

Shows most common open ports in the network
```bash
k0fi -n 192.168.2.0/24 -m 2000 -o json | jq '[.[] | .port] | group_by(.) | map({port: .[0], count: length}) | sort_by(-.count)'
```

## License
k0fiscan is distributed under the **MIT License**.
