use std::{env, path::PathBuf, process, sync::mpsc, thread, time};

/// Runs make/cmake to ensure native components are built.
/// Even once built this will add a few seconds to your build time.
pub fn build_native() -> std::thread::Result<()> {
    let start = time::Instant::now();
    //let spinner = indicatif::ProgressBar::new_spinner();
    let (tx, rx) = mpsc::sync_channel(1);
    let make = thread::spawn(move || {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        if cfg!(feature = "build_idf") {
            process::Command::new("idf.py")
                .arg("build")
                .current_dir(&std::path::Path::new(&manifest_dir))
                .status()
                .unwrap();
        } else if cfg!(feature = "build_make") {
            process::Command::new("make")
                .current_dir(&std::path::Path::new(&manifest_dir))
                .status()
                .unwrap();
        } else {
            println!("cargo:warning=No build feature enabled.  Build skipped.");
        }

        tx.send(()).unwrap();
    });
    loop {
        match rx.try_recv() {
            Err(mpsc::TryRecvError::Disconnected) | Ok(_) => break,
            Err(mpsc::TryRecvError::Empty) => {}
        }
        // NOTE: no point in printing anything because it's buffered and no way to flush it.
        //println!("cargo:warning=Building...");
        //spinner.tick();
        std::thread::sleep_ms(1000);
    }
    let ret = make.join();
    //spinner.finish();
    println!("cargo:warning=Build time {:?}", start.elapsed());
    ret
}

/// Add ESP-IDF native library search paths to rustc.
pub fn print_link_search() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "xtensa" {
        let esp_idf =
            PathBuf::from(env::var("IDF_PATH").expect("IDF_PATH environment variable must be set"));
        // Folder containing `build/` output after running `make menuconfig && make`
        let build_subdir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("build");
        if glob::glob(&build_subdir.join("*.bin").to_string_lossy())
            .unwrap()
            .next()
            .is_none()
        {
            panic!("No .bin files, did you run `make menuconfig && make`?");
        }

        let build_dirs = [
            "app_trace",
            "app_update",
            "asio",
            "bootloader_support",
            "bt",
            "coap",
            "console",
            "cxx",
            "driver",
            "efuse",
            "esp-tls",
            "esp32",
            "esp_adc_cal",
            "esp_common",
            "esp_eth",
            "esp_event",
            "esp_gdbstub",
            "esp_http_client",
            "esp_http_server",
            "esp_https_ota",
            "esp_local_ctrl",
            "esp_ringbuf",
            "esp_rom",
            "esp_websocket_client",
            "esp_wifi",
            "espcoredump",
            "expat",
            "fatfs",
            "freemodbus",
            "freertos",
            "heap",
            "idf_test",
            "jsmn",
            "json",
            "libsodium",
            "log",
            "lwip",
            "main",
            "mbedtls",
            "mdns",
            "mqtt",
            "newlib",
            "nghttp",
            "nvs_flash",
            "openssl",
            "protobuf-c",
            "protocomm",
            "pthread",
            "sdmmc",
            "soc",
            "spi_flash",
            "spiffs",
            "tcp_transport",
            "tcpip_adapter",
            "ulp",
            "unity",
            "vfs",
            "wear_levelling",
            "wifi_provisioning",
            "wpa_supplicant",
            "xtensa",
        ]
        .iter()
        .map(|subdir| build_subdir.join(subdir));
        let idf_components = [
            "esp32/ld",
            "esp_rom/esp32/ld",
            "esp_wifi/lib_esp32",
            "xtensa/esp32",
        ]
        .iter()
        .map(|subdir| esp_idf.join("components").join(subdir));

        for dir in build_dirs.chain(idf_components) {
            println!("cargo:rustc-link-search=native={}", dir.display());
        }
    }
}

/// Writes script file that uses esptool.py to create an application image from
/// the Rust ELF.
pub fn esptool_write_script() -> std::io::Result<()> {
    use std::io::Write;
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut file = std::fs::File::create(root.join("image.sh"))?;

    // Make it executable
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(0o744);
    file.set_permissions(perms)?;
    let cmd = format!(
        r#""$IDF_PATH/components/esptool_py/esptool/esptool.py" \
    --chip esp32 \
    elf2image \
    -o build/esp-app.bin \
    target/{}/{}/{}"#,
        env::var("TARGET").unwrap(),
        env::var("PROFILE").unwrap(),
        env::var("CARGO_PKG_NAME").unwrap()
    );

    // Write script with she-bang (#!)
    write!(file, "#!/usr/bin/env bash\n\n{}", cmd)
}
