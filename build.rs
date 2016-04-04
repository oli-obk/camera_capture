extern crate pkg_config;

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    let v4l2 = pkg_config::Config::new().statik(true).probe("libv4l2").expect("library `v4l2` not found");
}
