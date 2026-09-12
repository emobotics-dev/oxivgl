// SPDX-License-Identifier: MIT OR Apache-2.0
use cc::Build;
#[cfg(feature = "drivers")]
use std::collections::HashSet;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use sha2::{Digest, Sha256};

/// Xtensa ESP *and* RISC-V ESP (`riscv32imafc-unknown-none-elf` on S31).
fn is_esp_target(target: &str) -> bool {
    target.starts_with("xtensa-") || (target.starts_with("riscv32") && target.contains("none-elf"))
}

/// libclang does not understand cargo's `riscv32imafc-unknown-none-elf` triple.
/// Espressif's GCC is `riscv32-esp-elf`; use that so newlib `include_next` works.
fn bindgen_clang_target(target: &str) -> String {
    if target.starts_with("riscv32imafc") {
        "riscv32-esp-elf".into()
    } else {
        target.into()
    }
}

fn which(bin: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|d| d.join(bin))
            .find(|p| p.is_file())
    })
}

/// `riscv32-esp-elf-gcc` (newlib). Not host clang; not a crate-local stdint shim.
fn find_riscv32_esp_elf_gcc() -> Option<PathBuf> {
    if let Ok(p) = env::var("RISCV32_ESP_ELF_GCC") {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }
    if let Some(p) = which("riscv32-esp-elf-gcc") {
        return Some(p);
    }
    let mut roots = Vec::new();
    if let Ok(p) = env::var("IDF_TOOLS_PATH") {
        roots.push(PathBuf::from(p).join("tools/riscv32-esp-elf"));
    }
    if let Ok(h) = env::var("HOME") {
        roots.push(PathBuf::from(h).join(".espressif/tools/riscv32-esp-elf"));
    }
    roots.push(PathBuf::from("/opt/riscv32-esp-elf"));
    for root in roots {
        let direct = root.join("bin/riscv32-esp-elf-gcc");
        if direct.is_file() {
            return Some(direct);
        }
        if let Ok(rd) = fs::read_dir(&root) {
            for ent in rd.flatten() {
                let gcc = ent.path().join("riscv32-esp-elf/bin/riscv32-esp-elf-gcc");
                if gcc.is_file() {
                    return Some(gcc);
                }
            }
        }
    }
    None
}

fn gcc_print_sysroot(gcc: &Path) -> PathBuf {
    let out = Command::new(gcc)
        .arg("-print-sysroot")
        .output()
        .unwrap_or_else(|e| panic!("{} -print-sysroot: {e}", gcc.display()));
    assert!(
        out.status.success(),
        "{} -print-sysroot failed: {}",
        gcc.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(!s.is_empty(), "{} -print-sysroot was empty", gcc.display());
    PathBuf::from(s)
}

const LVGL_VERSION: &str = "9.5.0";
/// Names the patch set an extracted tree already carries. Lives in the tree,
/// not in `OUT_DIR`, so it travels with the thing it describes.
const PATCH_STAMP: &str = ".oxivgl-patches";
const LVGL_SHA256: &str = "34a955cdf3a2d005507b704e87357af669a114523b6d3f77b5344fdc68717bc6";

/// Built-in LVGL font faces that the application may disable in its `lv_conf.h`.
/// Each entry is the `lv_font_<NAME>` suffix; a face is "available" only when
/// its `LV_FONT_*` option is enabled, in which case bindgen emits the matching
/// `extern static` and we surface a `font_<NAME>` flag for `oxivgl` to gate on.
/// Keep in sync with the list in `oxivgl/build.rs` and the consts in
/// `oxivgl/src/fonts.rs`.
const GATED_FONTS: &[&str] = &[
    "montserrat_8",
    "montserrat_10",
    "montserrat_12",
    "montserrat_14",
    "montserrat_16",
    "montserrat_18",
    "montserrat_20",
    "montserrat_22",
    "montserrat_24",
    "montserrat_26",
    "montserrat_28",
    "montserrat_30",
    "montserrat_32",
    "montserrat_34",
    "montserrat_36",
    "montserrat_38",
    "montserrat_40",
    "montserrat_42",
    "montserrat_44",
    "montserrat_46",
    "montserrat_48",
    "dejavu_16_persian_hebrew",
    "source_han_sans_sc_14_cjk",
    "source_han_sans_sc_16_cjk",
];

/// Inspect the generated `bindings.rs` and emit a `cargo:font_<NAME>=1`
/// metadata value for every built-in font whose `extern static` is present.
/// Via `links = "lv"` this reaches `oxivgl`'s build script as
/// `DEP_LV_FONT_<NAME>`, which it turns into a `font_<NAME>` cfg. Using the
/// generated bindings as the source of truth means the flag matches symbol
/// availability exactly — including faces left at their `lv_conf_internal.h`
/// default rather than spelled out in the app's `lv_conf.h`.
fn emit_font_flags(bindings_path: &Path) {
    let src = std::fs::read_to_string(bindings_path).unwrap_or_default();
    for name in GATED_FONTS {
        if contains_ident(&src, &format!("lv_font_{name}")) {
            println!("cargo:font_{name}=1");
        }
    }
}

/// Emit `cargo:stdlib_malloc=builtin` when the application's `lv_conf.h`
/// selects LVGL's built-in TLSF allocator. Via `links = "lv"` this reaches
/// `oxivgl`'s build script as `DEP_LV_STDLIB_MALLOC`, which turns it into the
/// `lvgl_builtin_malloc` cfg.
///
/// Gating is not cosmetic: runtime memory pools only *work* under
/// `LV_STDLIB_BUILTIN`. The CLIB backend still exports `lv_mem_add_pool`
/// (`lv_mem_core_clib.c`), but as a no-op returning NULL — so an ungated call
/// links cleanly and silently adds nothing. Gating turns that into a compile
/// error instead.
fn emit_stdlib_flags(bindings_path: &Path) {
    let src = std::fs::read_to_string(bindings_path).unwrap_or_default();
    let selected = bindgen_const(&src, "LV_USE_STDLIB_MALLOC");
    let builtin = bindgen_const(&src, "LV_STDLIB_BUILTIN");
    if selected.is_some() && selected == builtin {
        println!("cargo:stdlib_malloc=builtin");
    }
}

/// Emit `cargo:demo_benchmark=1` when the application's `lv_conf.h` enabled
/// `LV_USE_DEMO_BENCHMARK` and the demo's declarations therefore reached the
/// bindings. Via `links = "lv"` this arrives at `oxivgl`'s build script as
/// `DEP_LV_DEMO_BENCHMARK`, which turns it into the `demo_benchmark` cfg that
/// gates `oxivgl::demo`.
///
/// The generated bindings are the source of truth for the same reason the font
/// flags use them: the demo is enabled by an `lv_conf.h` define owned by the
/// application, not by a cargo feature, so nothing in the build script's own
/// inputs can predict it. `lv_demo_benchmark_set_end_cb` is the probe because
/// it is the one symbol the wrapper cannot work without.
fn emit_demo_flags(bindings_path: &Path) {
    let src = std::fs::read_to_string(bindings_path).unwrap_or_default();
    if contains_ident(&src, "lv_demo_benchmark_set_end_cb") {
        println!("cargo:demo_benchmark=1");
    }
}

/// Value of a bindgen-emitted object-like macro. Returns `None` if the constant
/// is absent or is not a plain integer.
///
/// Tolerates both spacings bindgen produces — `LV_MEM_SIZE : u32 = 1 ;` when the
/// output is raw token stream, `LV_MEM_SIZE: u32 = 1;` once rustfmt has run —
/// which differ between host and cross builds. Matching only one silently
/// yields `None` on the other target.
fn bindgen_const(src: &str, name: &str) -> Option<u64> {
    let mut from = 0;
    while let Some(pos) = src[from..].find(name) {
        let start = from + pos;
        let end = start + name.len();
        from = end;

        // Reject a partial match inside a longer identifier.
        let before = src[..start].chars().next_back().unwrap_or(' ');
        if before.is_alphanumeric() || before == '_' {
            continue;
        }
        let rest = src[end..].trim_start();
        let Some(rest) = rest.strip_prefix(':') else {
            continue; // e.g. a mention in a type position, not a definition
        };
        let Some((_ty, tail)) = rest.split_once('=') else {
            continue;
        };
        let Some((value, _)) = tail.split_once(';') else {
            continue;
        };
        if let Ok(v) = value.trim().parse() {
            return Some(v);
        }
    }
    None
}

/// True if `ident` occurs in `src` as a whole identifier — i.e. not
/// immediately followed by another identifier character. Robust to bindgen's
/// spacing (`lv_font_x :` vs `lv_font_x:`) and collision-free across numeric
/// suffixes (`montserrat_4` does not match `montserrat_40`).
fn contains_ident(src: &str, ident: &str) -> bool {
    let bytes = src.as_bytes();
    let mut from = 0;
    while let Some(pos) = src[from..].find(ident) {
        let end = from + pos + ident.len();
        let next = bytes.get(end).copied().unwrap_or(b' ');
        if !next.is_ascii_alphanumeric() && next != b'_' {
            return true;
        }
        from = end;
    }
    false
}

/// Download and extract LVGL source tree into `out_dir/lvgl-{version}/`.
/// Returns the path to the extracted LVGL root.
/// Respects `LVGL_SRC_DIR` env var override for local development.
fn ensure_lvgl_source(out_dir: &Path) -> PathBuf {
    // User override: use local LVGL source
    if let Ok(dir) = env::var("LVGL_SRC_DIR") {
        let p = PathBuf::from(dir);
        if p.join("lv_version.h").exists() {
            return p;
        }
        panic!("LVGL_SRC_DIR={} does not contain lv_version.h", p.display());
    }

    let lvgl_dir = out_dir.join(format!("lvgl-{LVGL_VERSION}"));
    if lvgl_dir.join("lv_version.h").exists() {
        // A diff cannot be applied twice, nor over a tree holding a different
        // patch set, so a tree whose stamp does not match is discarded rather
        // than patched again.
        let stamp = fs::read_to_string(lvgl_dir.join(PATCH_STAMP)).unwrap_or_default();
        if stamp.trim() == patch_set_hash() {
            return lvgl_dir;
        }
        fs::remove_dir_all(&lvgl_dir).expect("Failed to remove stale LVGL tree");
    }

    // Kept beside the tree, so re-extracting after a patch edit costs no download.
    let cached = out_dir.join(format!("lvgl-{LVGL_VERSION}.tar.gz"));
    let tarball = match fs::read(&cached) {
        Ok(bytes) if format!("{:x}", Sha256::digest(&bytes)) == LVGL_SHA256 => bytes,
        _ => {
            let url =
                format!("https://github.com/lvgl/lvgl/archive/refs/tags/v{LVGL_VERSION}.tar.gz");
            eprintln!("Downloading LVGL v{LVGL_VERSION} from {url}");

            let mut resp = ureq::get(&url).call().expect("Failed to download LVGL");
            let bytes = resp
                .body_mut()
                .with_config()
                .limit(100 * 1024 * 1024)
                .read_to_vec()
                .expect("Failed to read LVGL tarball");

            // Verify SHA256
            let hash = format!("{:x}", Sha256::digest(&bytes));
            assert_eq!(hash, LVGL_SHA256, "LVGL tarball SHA256 mismatch!");
            let _ = fs::write(&cached, &bytes);
            bytes
        }
    };

    // Extract
    let decoder = flate2::read::GzDecoder::new(&tarball[..]);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(out_dir)
        .expect("Failed to extract LVGL tarball");

    assert!(
        lvgl_dir.join("lv_version.h").exists(),
        "LVGL extraction failed"
    );
    lvgl_dir
}

static CONFIG_NAME: &str = "DEP_LV_CONFIG_PATH";

// See https://github.com/rust-lang/rust-bindgen/issues/687#issuecomment-450750547
#[cfg(feature = "drivers")]
#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);
#[cfg(feature = "drivers")]
impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

fn main() {
    // docs.rs has no network access, so we cannot download LVGL or run
    // bindgen. Use a pre-generated host (x86_64-linux) bindings file and
    // skip both the C compilation and bindgen pipelines entirely.
    if env::var("DOCS_RS").is_ok() {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        let bindings_path = out_path.join("bindings.rs");
        std::fs::copy(manifest_dir.join("bindings_docsrs.rs"), &bindings_path)
            .expect("failed to install bundled bindings_docsrs.rs");
        emit_font_flags(&bindings_path);
        emit_stdlib_flags(&bindings_path);
        emit_demo_flags(&bindings_path);
        return;
    }

    let project_dir = canonicalize(PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()));
    let shims_dir = project_dir.join("shims");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lvgl_dir = ensure_lvgl_source(&out_path);
    let lvgl_src = lvgl_dir.join("src");

    #[cfg(feature = "rust_timer")]
    let timer_shim = shims_dir.join("timer");

    let font_extra_src: Option<PathBuf>;
    if let Ok(v) = env::var("PWD") {
        let current_dir = canonicalize(PathBuf::from(v));
        font_extra_src = {
            if let Ok(p) = env::var("LVGL_FONTS_DIR") {
                Some(canonicalize(PathBuf::from(p)))
            } else if current_dir.join("fonts").exists() {
                Some(current_dir.join("fonts"))
            } else {
                None
            }
        };
    } else {
        font_extra_src = None
    }

    // Some basic defaults; SDL2 is the only driver enabled in the provided
    // driver config by default
    #[cfg(feature = "drivers")]
    let incl_extra =
        env::var("LVGL_INCLUDE").unwrap_or("/usr/include,/usr/local/include".to_string());

    let cflags_extra_string = env::var("LVGL_CFLAGS").unwrap_or_default();

    let cflags_extra = if cflags_extra_string.is_empty() {
        None
    } else {
        Some(cflags_extra_string.split(','))
    };

    #[cfg(feature = "drivers")]
    let link_extra = env::var("LVGL_LINK").unwrap_or("SDL2".to_string());

    #[cfg(feature = "drivers")]
    let drivers = project_dir.join("lv_drivers");

    // Without this, switching config reuses the previous one's artifacts and
    // reports success — a stale pass.
    println!("cargo:rerun-if-env-changed={CONFIG_NAME}");

    let lv_config_dir = {
        let conf_path = env::var(CONFIG_NAME)
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // On docs.rs the workspace .cargo/config.toml is unavailable, so
                // fall back to the bundled default config to allow doc rendering.
                if env::var("DOCS_RS").is_ok() {
                    return project_dir.join("default-conf");
                }
                panic!(
                    "The environment variable {} is required to be defined",
                    CONFIG_NAME
                );
            });

        if !conf_path.exists() {
            // A build script's cwd is its own package dir, so a
            // workspace-relative value resolves one level too deep.
            let hint = if conf_path.is_relative() {
                format!(
                    " (resolved to {}: a relative value is taken from this build \
                     script's package directory, not the workspace root — pass an \
                     absolute path)",
                    conf_path
                        .canonicalize()
                        .unwrap_or_else(|_| project_dir.join(&conf_path))
                        .display()
                )
            } else {
                String::new()
            };
            panic!(
                "Directory {} referenced by {} needs to exist{}",
                conf_path.to_string_lossy(),
                CONFIG_NAME,
                hint
            );
        }
        if !conf_path.is_dir() {
            panic!("{} needs to be a directory", CONFIG_NAME);
        }
        if !conf_path.join("lv_conf.h").exists() {
            panic!(
                "Directory {} referenced by {} needs to contain a file called lv_conf.h",
                conf_path.to_string_lossy(),
                CONFIG_NAME
            );
        }
        #[cfg(feature = "drivers")]
        if !conf_path.join("lv_drv_conf.h").exists() {
            panic!(
                "Directory {} referenced by {} needs to contain a file called lv_drv_conf.h",
                conf_path.to_string_lossy(),
                CONFIG_NAME
            );
        }

        if let Some(p) = &font_extra_src {
            println!("cargo:rerun-if-changed={}", p.to_str().unwrap())
        }

        println!(
            "cargo:rerun-if-changed={}",
            conf_path.join("lv_conf.h").to_str().unwrap()
        );
        // The directory, not just lv_conf.h: `add_c_files` compiles every `.c`
        // in here, and those were not being watched.
        println!("cargo:rerun-if-changed={}", conf_path.to_str().unwrap());
        #[cfg(feature = "drivers")]
        println!(
            "cargo:rerun-if-changed={}",
            conf_path.join("lv_drv_conf.h").to_str().unwrap()
        );
        conf_path
    };

    #[cfg(feature = "drivers")]
    {
        println!("cargo:rerun-if-env-changed=LVGL_INCLUDE");
        println!("cargo:rerun-if-env-changed=LVGL_LINK");
    }

    let mut cfg = Build::new();
    let target_str = env::var("TARGET").unwrap_or_default();
    if target_str.starts_with("xtensa-") {
        cfg.flag("-mlongcalls");
    }
    let riscv_gcc = if target_str.starts_with("riscv32") {
        find_riscv32_esp_elf_gcc()
    } else {
        None
    };
    if target_str.starts_with("riscv32") {
        let gcc = riscv_gcc.clone().unwrap_or_else(|| {
            panic!(
                "riscv32-esp-elf-gcc not found (PATH, RISCV32_ESP_ELF_GCC, \
                 ~/.espressif/tools/riscv32-esp-elf, /opt/riscv32-esp-elf). \
                 Install Espressif's RISC-V GCC (newlib); do not use host clang."
            )
        });
        println!("cargo:rerun-if-env-changed=RISCV32_ESP_ELF_GCC");
        println!("cargo:rerun-if-env-changed=IDF_TOOLS_PATH");
        cfg.compiler(&gcc);
        cfg.flag("-march=rv32imafc");
        cfg.flag("-mabi=ilp32f");
    }
    if let Some(p) = &font_extra_src {
        add_c_files(&mut cfg, p)
    }
    apply_lvgl_patches(&lvgl_dir);
    println!("cargo:SRC_DIR={}", lvgl_dir.display());
    add_c_files(&mut cfg, &lvgl_src);
    add_c_files(&mut cfg, &lv_config_dir);
    add_c_files(&mut cfg, &shims_dir);
    #[cfg(feature = "drivers")]
    add_c_files(&mut cfg, &drivers);

    // Host (non-ESP) builds: SDL2 include path so LVGL's SDL driver compiles.
    if !is_esp_target(&target_str) {
        if let Ok(lib) = pkg_config::probe_library("sdl2") {
            for p in &lib.include_paths {
                cfg.include(p);
            }
        }
        println!("cargo:rustc-link-lib=SDL2");
    }

    cfg.define("LV_CONF_INCLUDE_SIMPLE", Some("1"))
        .include(&lvgl_dir)
        .include(&lvgl_src)
        .warnings(false)
        .include(&lv_config_dir);
    if let Some(p) = &font_extra_src {
        cfg.include(p);
    }
    #[cfg(feature = "rust_timer")]
    cfg.include(&timer_shim);
    #[cfg(feature = "drivers")]
    cfg.include(&drivers);
    #[cfg(feature = "drivers")]
    cfg.includes(incl_extra.split(','));

    if let Some(ref cflags_extra) = cflags_extra {
        cflags_extra.clone().for_each(|e| {
            let mut it = e.split('=');
            cfg.define(it.next().unwrap(), it.next().unwrap_or_default());
        });
    }

    let mut cc_args = vec![
        "-DLV_CONF_INCLUDE_SIMPLE=1",
        "-I",
        lv_config_dir.to_str().unwrap(),
        "-I",
        lvgl_dir.to_str().unwrap(),
        "-fvisibility=default",
    ];

    // For Xtensa targets, auto-detect the ESP-capable clang if LIBCLANG_PATH
    // doesn't already point to one. The system clang doesn't understand Xtensa.
    let target = env::var("TARGET").expect("Cargo build scripts always have TARGET");
    if target.starts_with("xtensa-") {
        let current = env::var("LIBCLANG_PATH").unwrap_or_default();
        if !current.contains("esp") {
            // Search common ESP clang locations (devcontainer, CI image).
            let suffix = "toolchains/esp/xtensa-esp32-elf-clang/esp-20.1.1_20250829/esp-clang/lib";
            let candidates = [
                format!("{}/.rustup/{suffix}", env::var("HOME").unwrap_or_default()),
                format!("{}/{suffix}", env::var("RUSTUP_HOME").unwrap_or_default()),
            ];
            for path in &candidates {
                if std::path::Path::new(path).exists() {
                    env::set_var("LIBCLANG_PATH", path);
                    break;
                }
            }
        }
    }

    // Set correct target triple for bindgen when cross-compiling
    let host = env::var("HOST").expect("Cargo build scripts always have HOST");
    let clang_target = bindgen_clang_target(&target);
    let riscv_sysroot = riscv_gcc.as_ref().map(|g| gcc_print_sysroot(g));
    let riscv_sysroot_s = riscv_sysroot
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned());
    let riscv_include_s = riscv_sysroot
        .as_ref()
        .map(|p| p.join("include").to_string_lossy().into_owned());
    // Name the target even when it is the host. Without this bindgen parses with
    // whatever the loaded libclang defaults to, so a host build is correct only
    // when that happens to be a host clang: an esp-clang LIBCLANG_PATH defaults
    // to Xtensa and the host build dies on `left: 4, right: 8`.
    cc_args.push("-target");
    cc_args.push(clang_target.as_str());
    if target != host {
        if target.starts_with("riscv32imafc") {
            cc_args.push("-march=rv32imafc");
            cc_args.push("-mabi=ilp32f");
            if let (Some(sys), Some(inc)) = (riscv_sysroot_s.as_deref(), riscv_include_s.as_deref())
            {
                cc_args.push("--sysroot");
                cc_args.push(sys);
                cc_args.push("-isystem");
                cc_args.push(inc);
            }
        }
    }

    let mut additional_args = Vec::new();
    // Add SDL2 include paths for bindgen on host builds
    if !is_esp_target(&target) {
        if let Ok(lib) = pkg_config::probe_library("sdl2") {
            for p in &lib.include_paths {
                additional_args.push("-I".to_string());
                additional_args.push(p.to_str().unwrap().to_string());
            }
        }
    }
    if target.ends_with("emscripten") {
        match env::var("EMSDK") {
            Ok(em_path) => {
                additional_args.push("-I".to_string());
                additional_args.push(format!(
                    "{}/upstream/emscripten/system/include/libc",
                    em_path
                ));
                additional_args.push("-I".to_string());
                additional_args.push(format!(
                    "{}/upstream/emscripten/system/lib/libc/musl/arch/emscripten",
                    em_path
                ));
                additional_args.push("-I".to_string());
                additional_args.push(format!(
                    "{}/upstream/emscripten/system/include/SDL",
                    em_path
                ));
            }
            Err(_) => panic!(
                "The EMSDK environment variable is not set. Has emscripten been properly initialized?"
            ),
        }
    }

    #[cfg(feature = "drivers")]
    let ignored_macros = IgnoreMacros(
        vec![
            "FP_INFINITE".into(),
            "FP_NAN".into(),
            "FP_NORMAL".into(),
            "FP_SUBNORMAL".into(),
            "FP_ZERO".into(),
            "IPPORT_RESERVED".into(),
        ]
        .into_iter()
        .collect(),
    );

    let bindings =
        bindgen::Builder::default().header(shims_dir.join("lvgl_sys.h").to_str().unwrap());
    let bindings = add_font_headers(bindings, &font_extra_src);
    #[cfg(feature = "drivers")]
    let bindings = bindings
        .header(shims_dir.join("lvgl_drv.h").to_str().unwrap())
        .parse_callbacks(Box::new(ignored_macros))
        // Watches every header bindgen opens, so an `lv_conf.h` that pulls in a
        // shared fragment is rebuilt when that fragment changes. The explicit
        // `rerun-if-changed` above covers only the config directory itself.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));
    #[cfg(feature = "rust_timer")]
    let bindings = bindings.header(shims_dir.join("rs_timer.h").to_str().unwrap());

    let extra_clang_args: Vec<String> = env::var("BINDGEN_EXTRA_CLANG_ARGS")
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_owned)
        .collect();

    let bindings = bindings
        .generate_comments(false)
        .derive_default(true)
        .layout_tests(false)
        .use_core()
        .ctypes_prefix("core::ffi")
        .clang_args(&cc_args)
        .clang_args(&additional_args)
        .clang_args(
            cflags_extra
                .map(|s| s.collect::<Vec<_>>())
                .unwrap_or(Vec::new()),
        )
        .clang_args(&extra_clang_args)
        .wrap_unsafe_ops(true)
        .wrap_static_fns(true)
        .wrap_static_fns_path(out_path.join("static_fns.c"))
        .generate()
        .expect("Unable to generate bindings");

    let bindings_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Can't write bindings!");

    // bindgen 0.72 emits `transmute` for signed↔unsigned bitfield casts;
    // newer rustc warns (unnecessary_transmutes). Patch to use direct casts.
    fix_bindgen_transmutes(&bindings_path);

    // Surface which built-in fonts actually made it into the bindings so
    // `oxivgl` can gate its font consts and avoid forcing every face on.
    emit_font_flags(&bindings_path);

    // Likewise for the allocator backend, which decides whether runtime memory
    // pools are usable at all.
    emit_stdlib_flags(&bindings_path);

    // Likewise for the benchmark demo, which only exists when the application
    // asked for it in its lv_conf.h.
    emit_demo_flags(&bindings_path);

    // From the bindings, not a scan of lv_conf.h: the value may arrive via an
    // #include. Getting it wrong is quiet — the sources are absent and the
    // first sign is an undefined reference at link.
    if bindgen_const(&fs::read_to_string(&bindings_path).unwrap_or_default(), "LV_BUILD_DEMOS")
        == Some(1)
    {
        add_c_files(&mut cfg, &lvgl_dir.join("demos"));
    }

    cfg.file(out_path.join("static_fns.c"));
    cfg.compile("lvgl");

    #[cfg(feature = "drivers")]
    link_extra.split(',').for_each(|a| {
        println!("cargo:rustc-link-lib={a}");
        //println!("cargo:rustc-link-search=")
    })
}

fn add_font_headers(
    bindings: bindgen::Builder,
    dir: &Option<impl AsRef<Path>>,
) -> bindgen::Builder {
    if let Some(p) = dir {
        let mut temp = bindings;
        for e in p.as_ref().read_dir().unwrap() {
            let e = e.unwrap();
            let path = e.path();
            if !e.file_type().unwrap().is_dir()
                && path.extension().and_then(|s| s.to_str()) == Some("h")
            {
                temp = temp.header(path.to_str().unwrap());
            }
        }
        temp
    } else {
        bindings
    }
}

fn add_c_files(build: &mut cc::Build, path: impl AsRef<Path>) {
    let skip_host = is_esp_target(&env::var("TARGET").unwrap_or_default());
    for e in path.as_ref().read_dir().unwrap() {
        let e = e.unwrap();
        let path = e.path();
        if e.file_type().unwrap().is_dir() {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            // Host-only LVGL backends — not for bare-metal ESP.
            if skip_host
                && matches!(
                    name,
                    "sdl" | "wayland" | "x11" | "windows" | "nuttx" | "evdev" | "libinput" | "glfw"
                )
            {
                continue;
            }
            add_c_files(build, e.path());
        } else if path.extension().and_then(|s| s.to_str()) == Some("c") {
            build.file(&path);
        }
    }
}

/// Replace unnecessary `transmute` calls in bindgen bitfield accessors and
/// strip `unsafe` blocks that become safe after removal.
/// bindgen 0.72 uses transmute for integer casts that rustc now warns about.
fn fix_bindgen_transmutes(path: &Path) {
    let mut code = std::fs::read_to_string(path).unwrap();

    // Phase 1: Replace `::core::mem::transmute(INNER)` → `(INNER) as _`.
    // Uses paren-matching to handle multi-line expressions.
    // Support both spaced (`:: core :: mem :: transmute (`) and compact
    // (`::core::mem::transmute(`) formats emitted by different bindgen versions.
    let needles = [":: core :: mem :: transmute (", "::core::mem::transmute("];
    while let Some((start, needle_len)) = needles
        .iter()
        .filter_map(|n| code.find(n).map(|pos| (pos, n.len())))
        .min_by_key(|(pos, _)| *pos)
    {
        let inner_start = start + needle_len;
        let mut depth: u32 = 1;
        let mut end = inner_start;
        for ch in code[inner_start..].chars() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            end += ch.len_utf8();
        }
        let inner = code[inner_start..end].to_string();
        let replacement = format!("({}) as _", inner);
        code = format!("{}{}{}", &code[..start], replacement, &code[end + 1..]);
    }

    // Phase 2: Strip `unsafe { ... }` blocks that no longer contain unsafe ops.
    // Keep blocks containing `raw_get`, `raw_set`, or `addr_of` (raw-pointer ops).
    let unsafe_kw = "unsafe {";
    let mut result = String::with_capacity(code.len());
    let mut pos = 0;
    let bytes = code.as_bytes();
    while pos < code.len() {
        if let Some(rel) = code[pos..].find(unsafe_kw) {
            let block_start = pos + rel;
            let brace_start = block_start + unsafe_kw.len() - 1; // position of '{'
            // Find matching '}'
            let mut depth: u32 = 1;
            let mut end = brace_start + 1;
            while end < code.len() && depth > 0 {
                match bytes[end] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                end += 1;
            }
            let body = &code[brace_start + 1..end - 1]; // between { and }
            // Only strip unsafe from blocks whose body is purely safe after
            // transmute removal: bitfield get/set and simple casts.
            let is_safe_body = !body.contains("unsafe")
                && !body.contains("raw_get")
                && !body.contains("raw_set")
                && !body.contains("addr_of")
                && !body.contains("write_bytes")
                && !body.contains("assume_init")
                && !body.contains("from_raw")
                && !body.contains("as_ptr")
                && !body.contains("read_unaligned")
                && !body.contains("write_unaligned")
                && !body.contains("copy_nonoverlapping")
                && (body.contains("_bitfield_1") || body.contains("as _"));
            let needs_unsafe = !is_safe_body;

            // Copy text before `unsafe`
            result.push_str(&code[pos..block_start]);

            if needs_unsafe {
                // Keep the entire `unsafe { ... }` block
                result.push_str(&code[block_start..end]);
            } else {
                // Strip `unsafe { }`, keep the body with adjusted whitespace.
                // Single-line: `unsafe { EXPR }` → `EXPR`
                // Multi-line: preserve inner indentation as-is.
                let trimmed = body.trim();
                if !body.contains('\n') {
                    result.push_str(trimmed);
                } else {
                    result.push_str(body);
                }
            }
            pos = end;
        } else {
            result.push_str(&code[pos..]);
            break;
        }
    }

    std::fs::write(path, result).unwrap();
}

fn canonicalize(path: impl AsRef<Path>) -> PathBuf {
    let canonicalized = path.as_ref().canonicalize().unwrap();
    let canonicalized = &*canonicalized.to_string_lossy();

    PathBuf::from(canonicalized.strip_prefix(r"\\?\").unwrap_or(canonicalized))
}

/// The LVGL patch series, applied in this order.
///
/// A flat numbered series, as quilt and the distro build systems use it: the
/// number is the apply order and the name says what the patch is for. Each
/// patch names the file it targets in its own `+++` header, so nothing here
/// restates a path that could drift out of step with the diff.
///
/// Diffs rather than string surgery: the change reviews as a diff, its context
/// lines do the anchoring, and a hunk that stops matching fails the build
/// instead of silently doing nothing. `include_str!` is deliberate -- if a
/// patch ever falls out of the published package, build.rs fails to compile
/// rather than quietly shipping unpatched LVGL.
const LVGL_PATCHES: &[(&str, &str)] = &[
    ("0001-buttonmatrix-text-length.patch", include_str!("patches/0001-buttonmatrix-text-length.patch")),
    ("0002-render-scratch-to-internal-dram.patch", include_str!("patches/0002-render-scratch-to-internal-dram.patch")),
    ("0003-ppa-argb8888-images-and-glyphs.patch", include_str!("patches/0003-ppa-argb8888-images-and-glyphs.patch")),
    ("0004-demo-guards-see-a-second-pool.patch", include_str!("patches/0004-demo-guards-see-a-second-pool.patch")),
    ("0005-benchmark-reset-scene-accumulators.patch", include_str!("patches/0005-benchmark-reset-scene-accumulators.patch")),
];

/// Hash of the whole patch set, stamped into an extracted tree so a later build
/// knows whether that tree already carries exactly these patches.
fn patch_set_hash() -> String {
    let mut h = Sha256::new();
    for (rel, text) in LVGL_PATCHES {
        h.update(rel.as_bytes());
        h.update(text.as_bytes());
    }
    format!("{:x}", h.finalize())
}

/// Split a patch covering several files into one unified diff per file.
///
/// A patch here is one *change*, which may touch several files, the way a quilt
/// series is normally organised. `diffy::Patch` is deliberately single-file and
/// refuses a concatenated diff outright rather than parsing the first and
/// dropping the rest, so the segments have to be handed to it one at a time.
///
/// Only the segmenting is done here; diffy still does the patching. A split in
/// the wrong place yields a segment diffy cannot parse, so a mistake here fails
/// the build instead of applying part of a change.
fn split_patch(text: &str) -> Vec<&str> {
    let lines: Vec<(usize, &str)> = {
        let mut v = Vec::new();
        let mut at = 0;
        for line in text.split_inclusive('\n') {
            v.push((at, line));
            at += line.len();
        }
        v
    };
    let starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(i, (_, l))| {
            l.starts_with("--- ")
                && lines.get(i + 1).is_some_and(|(_, n)| n.starts_with("+++ "))
        })
        .map(|(_, (at, _))| *at)
        .collect();
    starts
        .iter()
        .enumerate()
        .map(|(i, &from)| &text[from..starts.get(i + 1).copied().unwrap_or(text.len())])
        .collect()
}

/// Apply [`LVGL_PATCHES`] to an extracted LVGL tree, once.
fn apply_lvgl_patches(lvgl_dir: &Path) {
    let stamp = lvgl_dir.join(PATCH_STAMP);
    let want = patch_set_hash();
    if fs::read_to_string(&stamp).is_ok_and(|s| s.trim() == want) {
        return;
    }
    for (name, text) in LVGL_PATCHES {
        println!("cargo:rerun-if-changed=patches/{name}");
        let segments = split_patch(text);
        assert!(
            !segments.is_empty(),
            "patches/{name}: no `---`/`+++` file header, so it patches nothing"
        );
        for segment in segments {
            let patch = diffy::Patch::from_str(segment)
                .unwrap_or_else(|e| panic!("patches/{name} is not a valid unified diff ({e})"));
            // The `+++ b/<path>` header is the single source of truth for the
            // target, so a patch cannot be pointed at the wrong file by a table.
            let rel = patch.modified().and_then(|p| p.strip_prefix("b/")).unwrap_or_else(|| {
                panic!("patches/{name}: no `+++ b/<path>` header naming its target")
            });
            let file = lvgl_dir.join(rel);
            // `--- /dev/null` is how a unified diff says "this file is new", so
            // creating one needs no mechanism of its own.
            let original = if patch.original() == Some("/dev/null") {
                String::new()
            } else {
                fs::read_to_string(&file)
                    .unwrap_or_else(|e| panic!("patches/{name}: cannot read {rel} to patch it ({e})"))
            };
            let patched = diffy::apply(&original, &patch).unwrap_or_else(|e| {
                panic!(
                    "patches/{name} does not apply to {rel} in LVGL v{LVGL_VERSION} ({e}) -- \
                     LVGL moved, so re-cut the patch against the new source"
                )
            });
            fs::write(&file, patched)
                .unwrap_or_else(|e| panic!("patches/{name}: cannot write {rel} ({e})"));
        }
    }
    fs::write(&stamp, want).expect("cannot write the patch stamp");
}
