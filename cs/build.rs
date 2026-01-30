use csbindgen::Builder;
use std::path::Path;

fn main() {
    // Generate C# bindings from Rust extern "C" functions
    let csharp_dir = Path::new("FugleMarketData");

    // Create output directory if it doesn't exist
    if !csharp_dir.exists() {
        std::fs::create_dir_all(csharp_dir).expect("Failed to create FugleMarketData directory");
    }

    Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("fugle_marketdata")
        .csharp_class_name("NativeMethods")
        .csharp_namespace("Fugle.MarketData.Native")
        .csharp_use_function_pointer(true) // .NET 5+ function pointers
        .generate_csharp_file("FugleMarketData/NativeMethods.g.cs")
        .unwrap();

    // Standard Rust library build
    println!("cargo:rerun-if-changed=src/");
}
