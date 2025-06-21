# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///

from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from string import Template

NMAP_FILE_PATH = Path("data/nmap-services")
RUST_FILE_PATH = Path("src/services.rs")


class Protocol(Enum):
    TCP = "tcp"
    UDP = "udp"
    SCTP = "sctp"


RUST_TEMPLATE = Template(
    """
use std::collections::HashMap;

#[derive(Debug)]
pub struct ServiceEntry {
    pub name: &'static str,
    pub comment: &'static str,
    pub prb: f32,
}

pub type ServiceMap = HashMap<u16, ServiceEntry>;

pub fn get_services() -> ServiceMap {
    HashMap::from([
$entries
    ])
}
"""
)


@dataclass
class ServiceEntry:
    name: str
    port: int
    prb: float
    comment: str = ""


def format_entry(s: ServiceEntry) -> str:
    name = s.name.replace('"', '\\"')
    comment = s.comment.replace('"', '\\"')
    return f'        ({s.port}, ServiceEntry {{ name: "{name}", comment: "{comment}", prb: {s.prb} }}),'


def generate_rust_file(services: list[ServiceEntry]) -> str:
    entries = "\n".join(format_entry(s) for s in services)
    return RUST_TEMPLATE.substitute(entries=entries)


def parse_port(port: str) -> tuple[int, Protocol]:
    tokens = port.split("/")
    if len(tokens) != 2:
        raise ValueError(f"invalid port format: {port}")

    port_num = int(tokens[0])
    proto = Protocol(tokens[1])

    return port_num, proto


def strip_comment(comment: str) -> str:
    return comment.removeprefix("# ")


def read_services(infile: Path) -> list[ServiceEntry]:
    services: list[ServiceEntry] = []
    for line in open(infile, "r"):
        line = line.strip()
        if line.startswith("#"):
            continue
        tokens = line.split("\t", maxsplit=3)
        comment = ""
        if len(tokens) == 4:
            service_name, port, prb, comment = tokens
        elif len(tokens) == 3:
            service_name, port, prb = tokens
        else:
            raise ValueError("unknown line format")

        prb = float(prb)
        port_num, proto = parse_port(port)
        if proto != Protocol.TCP or service_name == "unknown":
            continue
        comment = strip_comment(comment)
        services.append(ServiceEntry(name=service_name, port=port_num, comment=comment, prb=prb))
    return services


def write_rust_file(outfile: Path, data: str) -> None:
    with open(outfile, "w") as fout:
        fout.write(data)


if __name__ == "__main__":
    services = read_services(NMAP_FILE_PATH)
    rust_code = generate_rust_file(services)
    write_rust_file(RUST_FILE_PATH, rust_code)
