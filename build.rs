use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=gen_service.py");
    println!("cargo:rerun-if-changed=data/nmap-services");
    println!("cargo:rerun-if-changed=build.rs");
    let status = Command::new("uv")
        .args(["run", "gen_services.py"])
        .status()
        .expect("Failed to run gen_service.py with uv");

    if !status.success() {
        panic!("gen_service.py failed with status: {}", status);
    }
}
