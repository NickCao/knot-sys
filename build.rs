fn main() {
    println!("cargo:rustc-link-lib=knot");
    let bindings = bindgen::Builder::default()
        .header_contents("libknot-headers.h", "#include <libknot/libknot.h>")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .unwrap();
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}
