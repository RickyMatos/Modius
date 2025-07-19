//src/build.rs
fn main() {
    slint_build::compile("src/ui/slint/main.slint").unwrap();
    println!("cargo:rerun-if-changed=src/ui/slint/main.slint");
    println!("cargo:rerun-if-changed=src/ui/assets/");
}
