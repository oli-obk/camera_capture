extern crate pkg_config;

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    if std::env::var("TARGET").unwrap().contains("linux") {
        pkg_config::Config::new().statik(true).probe("libv4l2").expect("library `v4l2` not found");
    }
}
