fn main() {
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=strategy.toml");
}
