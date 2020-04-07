use std::{env, path::PathBuf};

/// Add ESP-IDF native library search paths to rustc.
pub fn print_link_search() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "xtensa" {
        let esp_idf = PathBuf::from(env::var("IDF_PATH").expect("IDF_PATH environment variable must be set"));
        // Folder containing `build/` output after running `make menuconfig && make`
        let build_parent = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let build_dirs = [
            "app_update",
            "driver",
            "esp-tls",
            "esp32",
            "esp_ringbuf",
            "freertos",
            "heap",
            "log",
            "newlib",
            "pthread",
            "soc",
            "spi_flash",
            "vfs",
            "xtensa-debug-module",
        ]
        .iter()
        .map(|subdir| build_parent.join("build").join(subdir));
        let idf_components = ["esp32/", "esp32/lib", "esp32/ld", "newlib/lib"]
            .iter()
            .map(|subdir| esp_idf.join("components").join(subdir));

        for dir in build_dirs.chain(idf_components) {
            println!("cargo:rustc-link-search=native={}", dir.display());
        }
    }
}
