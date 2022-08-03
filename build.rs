use std::env;
use std::path::{Path, PathBuf};
fn out_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

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

fn main() {
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

    // println!("cargo:rustc-link-lib=static=tensorflowlite_c");
    link_libs_c(build_tflite_c("tensorflow/tensorflow/lite/c"));
}
