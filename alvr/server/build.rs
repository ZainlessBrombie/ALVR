use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let cpp_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("cpp");

    let cpp_paths = walkdir::WalkDir::new("cpp")
        .into_iter()
        .filter_map(|maybe_entry| maybe_entry.ok())
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();

    let source_files_paths = cpp_paths.iter().filter(|path| {
        path.extension()
            .filter(|ext| {
                let ext_str = ext.to_string_lossy();
                ext_str == "c" || ext_str == "cpp"
            })
            .is_some()
    });

    let mut build = cc::Build::new();
    build
        .debug(false) // This is because we cannot link to msvcrtd (see below)
        .cpp(true)
        .files(source_files_paths)
        .include("cpp/alvr_server")
        .include("cpp/shared")
        .include("cpp/openvr/headers")
        .include("cpp/alvr_server/include")
        .include("cpp/libswresample/include")
        .include("cpp/ALVR-common")
        .define("_WINDLL", None)
        .define("NOMINMAX", None)
        .define("_WINSOCKAPI_", None)
        .define("_MBCS", None)
        .define("_MT", None)
        .define("_DLL", None);

    #[cfg(debug_assertions)]
    build.define("ALVR_DEBUG_LOG", None);

    build.compile("bindings");

    bindgen::builder()
        .clang_arg("-xc++")
        .header("cpp/alvr_server/bindings.h")
        .derive_default(true)
        .generate()
        .expect("bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("bindings.rs");

    // Many lib dependencies of alvr_server code are already included by rust libraries
    // Reenable some linkage in case of linking errors
    if cfg!(windows) {
        // println!("cargo:rustc-link-lib=kernel32");
        // println!("cargo:rustc-link-lib=user32");
        // println!("cargo:rustc-link-lib=gdi32");
        // println!("cargo:rustc-link-lib=winspool");
        // println!("cargo:rustc-link-lib=comdlg32");
        // println!("cargo:rustc-link-lib=advapi32");
        // println!("cargo:rustc-link-lib=shell32");
        // println!("cargo:rustc-link-lib=ole32");
        // println!("cargo:rustc-link-lib=oleaut32");
        // println!("cargo:rustc-link-lib=uuid");
        // println!("cargo:rustc-link-lib=odbc32");
        // println!("cargo:rustc-link-lib=odbccp32");
        // println!("cargo:rustc-link-lib=winmm");
        // println!("cargo:rustc-link-lib=ws2_32");
        // println!("cargo:rustc-link-lib=userenv");
        println!("cargo:rustc-link-lib=avrt");

        // This is the library that is linked when using the /MD flag.
        // For debug builds, /MDd should be used (that links msvcrtd), but msvcrtd clashes with
        // spirv_cross, that always use msvcrt (release version). So here the C++ code is always
        // built as release and only msvcrt is linked
        println!("cargo:rustc-link-lib=msvcrt");

        println!(
            "cargo:rustc-link-search=native={}/libswresample/lib",
            cpp_dir.to_string_lossy()
        );
        println!(
            "cargo:rustc-link-search=native={}/openvr/lib",
            cpp_dir.to_string_lossy()
        );
        println!("cargo:rustc-link-lib=swresample");
        println!("cargo:rustc-link-lib=avutil");
        println!("cargo:rustc-link-lib=openvr_api");
    }

    for path in cpp_paths {
        println!("cargo:rerun-if-changed={}", path.to_string_lossy());
    }
}
