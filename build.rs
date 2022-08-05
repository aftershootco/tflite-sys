extern crate bindgen;

#[cfg(any(feature = "bindgen", feature = "build"))]
fn clone(out_dir: impl AsRef<Path>) {
    use std::process::{Command, Stdio};
    let out_dir = out_dir.as_ref();
    let tf = "https://github.com/tensorflow/tensorflow";
    let _ = Command::new("git")
        .arg("clone")
        .arg("-b")
        .arg("r2.6")
        .arg("--depth")
        .arg("1")
        .arg(tf)
        .arg(out_dir.join("tensorflow"))
        .stdout(Stdio::inherit())
        .output()
        .expect("Failed to clone tensorflow");
}

#[cfg(feature = "build")]
fn build_tflite_c<P: AsRef<Path>>(tf_src_path: P) -> PathBuf {
    if cfg!(target_arch = "aarch64") && cfg!(target_os = "macos") {
        cmake::Config::new(tf_src_path)
            .define("TFLITE_C_BUILD_SHARED_LIBS", "OFF")
            .define("CMAKE_OSX_ARCHITECTURES", "arm64")
            .build()
    } else {
        cmake::Config::new(tf_src_path)
            .define("TFLITE_C_BUILD_SHARED_LIBS", "OFF")
            .build()
    }
}

#[cfg(feature = "build")]
fn link_libs_c<P: AsRef<Path>>(target_dir: P) {
    let build_dir = target_dir.as_ref().join("build");
    let search_paths = vec![
        "ruy-build",
        "fft2d-build",
        "xnnpack-build",
        "farmhash-build",
        "flatbuffers-build",
    ];

    for p in search_paths {
        println!(
            "cargo:rustc-link-search=native={}",
            build_dir.join("_deps").join(p).display()
        );
    }

    let search_paths = vec!["pthreadpool", "cpuinfo", "tensorflow-lite", "clog"];
    for p in search_paths {
        println!(
            "cargo:rustc-link-search=native={}",
            build_dir.join(p).display()
        );
    }
}

#[cfg(feature = "bindgen")]
fn generate_bindings(tf_src_path: impl AsRef<Path>) {
    let tf_src_path = tf_src_path.as_ref();
    let builder = bindgen::Builder::default()
        .header(
            tf_src_path
                .join("tensorflow/lite/c/c_api.h")
                .to_str()
                .unwrap(),
        )
        .header(
            tf_src_path
                .join("tensorflow/lite/delegates/xnnpack/xnnpack_delegate.h")
                .to_str()
                .unwrap(),
        );
    let bindgen = builder
        .clang_arg(format!("-I{}", tf_src_path.to_str().unwrap()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");
    let home = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src");
    bindgen
        .write_to_file(home.join("bindings.rs"))
        .expect("Failed to write bindings");
}

fn main() {
    // println!("cargo:rustc-link-lib=static=tensorflowlite_c");

    #[cfg(any(feature = "bindgen", feature = "build"))]
    {
        use std::env;
        use std::path::{Path, PathBuf};
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        clone(&out_dir);
        let tf_src_path = out_dir.join("tensorflow");
    }
    #[cfg(feature = "build")]
    {
        link_libs_c(build_tflite_c(
            tf_src_path.join("tensorflow").join("lite").join("c"),
        ));
        println!("cargo:rustc-link-lib=static=farmhash");
        println!("cargo:rustc-link-lib=static=ruy");
        println!("cargo:rustc-link-lib=static=cpuinfo");
        println!("cargo:rustc-link-lib=static=XNNPACK");
        println!("cargo:rustc-link-lib=static=tensorflow-lite");
        println!("cargo:rustc-link-lib=static=pthreadpool");
        println!("cargo:rustc-link-lib=static=fft2d_fftsg");
        println!("cargo:rustc-link-lib=static=flatbuffers");
        println!("cargo:rustc-link-lib=static=clog");
        println!("cargo:rustc-link-lib=static=fft2d_fftsg2d"); // println!("cargo:rustc-link-lib=tensorflowlite_c");
    }

    #[cfg(feature = "bindgen")]
    generate_bindings(tf_src_path);
}
