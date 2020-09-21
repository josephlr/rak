fn main() {
    println!("cargo:rerun-if-changed=x86_64-none-uk.json");
    println!("cargo:rerun-if-changed=layout.ld");
}
