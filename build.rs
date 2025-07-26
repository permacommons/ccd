use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("ccd_shell_function.rs");

    // Read the shell script
    let shell_script = fs::read_to_string("ccd.sh").expect("Failed to read ccd.sh");

    // Generate Rust code that includes the shell script as a string constant
    let rust_code = format!(
        "pub const CCD_SHELL_FUNCTION: &str = r#\"{}\"#;",
        shell_script
    );

    fs::write(&dest_path, rust_code).expect("Failed to write shell function");

    // Tell cargo to rerun if ccd.sh changes
    println!("cargo:rerun-if-changed=ccd.sh");
}
