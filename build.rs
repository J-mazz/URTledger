fn main() {
    // Compile the Slint UI file into Rust code
    slint_build::compile("ui/ui.slint").expect("failed to compile slint UI");
}
