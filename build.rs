use ethos_zero::{Actualizing, File, Generating, Potential};
fn main() {
    let root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("manifest"));
    let source = std::fs::read_to_string(root.join("ethos/signal.ethos")).expect("source");
    let file = match Potential::<File>::from(source).actualize() {
        Ok(file) => file,
        Err(_) => panic!("read"),
    };
    let generated = match file.generate() {
        Ok(generated) => generated,
        Err(_) => panic!("generate"),
    };
    assert_eq!(
        generated,
        std::fs::read_to_string(root.join("src/generated/signal.rs")).expect("generated")
    );
}
