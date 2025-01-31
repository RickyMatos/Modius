//src/build.rs
fn main() {
    slint_build::compile("src/ui/main.slint").unwrap();
    println!("cargo:rerun-if-changed=src/ui/main.slint");
    println!("cargo:rerun-if-changed=src/ui/assets/");
}
