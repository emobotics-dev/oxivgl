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
        return lvgl_dir;
    }

    let url = format!("https://github.com/lvgl/lvgl/archive/refs/tags/v{LVGL_VERSION}.tar.gz");
    eprintln!("Downloading LVGL v{LVGL_VERSION} from {url}");

    let mut resp = ureq::get(&url).call().expect("Failed to download LVGL");
    let tarball = resp
        .body_mut()
        .with_config()
        .limit(100 * 1024 * 1024)
        .read_to_vec()
        .expect("Failed to read LVGL tarball");

    // Verify SHA256
    let hash = format!("{:x}", Sha256::digest(&tarball));
    assert_eq!(hash, LVGL_SHA256, "LVGL tarball SHA256 mismatch!");

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
            panic!(
                "Directory {} referenced by {} needs to exist",
                conf_path.to_string_lossy(),
                CONFIG_NAME
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
    patch_btnmatrix_text_length(&lvgl_src);
    patch_render_scratch(&lvgl_src);
    patch_ppa_draw_unit(&lvgl_src);
    patch_demo_mem_guards(&lvgl_dir);
    patch_demo_repeatable(&lvgl_dir);
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
    if target != host {
        cc_args.push("-target");
        cc_args.push(clang_target.as_str());
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
        .parse_callbacks(Box::new(ignored_macros));
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

/// Widen LVGL's PPA draw unit to the two operations the blend engine can
/// actually do but upstream never asks it for: an ARGB8888 image with its own
/// alpha, and a glyph.
///
/// Upstream's image path does not composite. It passes the *source* as the
/// blend background and the *destination* as a fully transparent foreground,
/// spending the foreground slot on a no-op to turn a blend into an opaque
/// blit. That is why `ppa_evaluate` demands an opaque RGB565/RGB888 image:
/// there is nowhere to put per-pixel alpha. Widening the format test alone
/// would send ARGB8888 down that path and silently discard its alpha, so the
/// roles are swapped instead — destination as background, source as
/// foreground, `PPA_ALPHA_NO_CHANGE` so the source's own alpha is used.
///
/// Text is the bigger prize. The engine has a foreground mode built for
/// exactly one thing: an A8 coverage mask with the colour supplied by a
/// register. That is a glyph. `ppa_evaluate` has no `LV_DRAW_TASK_TYPE_LABEL`
/// case at all upstream, so one is added, dispatching to a new
/// `lv_draw_ppa_label.c` that hooks `lv_draw_label_iterate_characters` and
/// blends each glyph's bitmap. Layout, kerning and wrapping stay in LVGL.
///
/// Small glyphs are left to the CPU: a blend costs register writes, a DMA
/// start and an interrupt, which a 12x16 letter does not repay. See
/// `PPA_GLYPH_MIN_PX` in the generated file.
fn patch_ppa_draw_unit(lvgl_src: &Path) {
    let ppa_dir = lvgl_src.join("draw/espressif/ppa");
    if !ppa_dir.exists() {
        return;
    }

    // 1. The image path: swap the blend roles for an ARGB8888 source, and fix
    //    two latent bugs while here - the foreground picture dimensions were
    //    taken from the source header while the buffer was the destination
    //    (harmless only while that foreground is transparent), and a stray
    //    extern for a benchmark asset was left in the function.
    let img = ppa_dir.join("lv_draw_ppa_img.c");
    if img.exists() {
        let code = std::fs::read_to_string(&img).unwrap();
        if !code.contains("OXIVGL_PPA_ARGB8888") {
            let mut patched =
                code.replace("    extern const lv_image_dsc_t img_benchmark_lvgl_logo_rgb;\n", "");

            let needle = "    ppa_blend_oper_config_t cfg = {";
            assert!(patched.contains(needle), "lv_draw_ppa_img.c: blend config not found");
            patched = patched.replace(
                needle,
                "    /* OXIVGL_PPA_ARGB8888: a source carrying its own alpha must be the\n\
                 \x20    * blend foreground, not the background, or the alpha is discarded. */\n\
                 \x20   const bool src_has_alpha = (src_cf == LV_COLOR_FORMAT_ARGB8888);\n\
                 \x20   ppa_blend_oper_config_t cfg = {",
            );

            // Background: the destination when compositing, the source when blitting.
            patched = patched.replace(
                "            .buffer          = (void *)src_buf,\n            .pic_w           = draw_dsc->header.w,\n            .pic_h           = draw_dsc->header.h,",
                "            .buffer          = src_has_alpha ? (void *)dest_buf : (void *)src_buf,\n            .pic_w           = src_has_alpha ? draw_buf->header.w : draw_dsc->header.w,\n            .pic_h           = src_has_alpha ? draw_buf->header.h : draw_dsc->header.h,",
            );
            patched = patched.replace(
                "            .block_offset_x  = src_area.x1,\n            .block_offset_y  = src_area.y1,\n            .blend_cm        = lv_color_format_to_ppa_blend(src_cf),\n        },",
                "            .block_offset_x  = src_has_alpha ? dest_area.x1 : src_area.x1,\n            .block_offset_y  = src_has_alpha ? dest_area.y1 : src_area.y1,\n            .blend_cm        = lv_color_format_to_ppa_blend(src_has_alpha ? dest_cf : src_cf),\n        },",
            );

            // Foreground: the source when compositing, a transparent no-op when blitting.
            patched = patched.replace(
                "            .buffer          = (void *)dest_buf,\n            .pic_w           = draw_dsc->header.w,\n            .pic_h           = draw_dsc->header.h,",
                "            .buffer          = src_has_alpha ? (void *)src_buf : (void *)dest_buf,\n            .pic_w           = src_has_alpha ? draw_dsc->header.w : draw_buf->header.w,\n            .pic_h           = src_has_alpha ? draw_dsc->header.h : draw_buf->header.h,",
            );
            patched = patched.replace(
                "            .blend_cm        = PPA_BLEND_COLOR_MODE_A8,\n        },",
                "            .blend_cm        = src_has_alpha ? PPA_BLEND_COLOR_MODE_ARGB8888 : PPA_BLEND_COLOR_MODE_A8,\n        },",
            );
            patched = patched.replace(
                "        .fg_alpha_update_mode  = PPA_ALPHA_FIX_VALUE,\n        .fg_alpha_fix_val      = 0,",
                "        .fg_alpha_update_mode  = src_has_alpha ? PPA_ALPHA_NO_CHANGE : PPA_ALPHA_FIX_VALUE,\n        .fg_alpha_fix_val      = 0,",
            );
            std::fs::write(&img, patched).unwrap();
        }
    }

    // 2. Accept an ARGB8888 image, and claim label tasks.
    let unit = ppa_dir.join("lv_draw_ppa.c");
    if unit.exists() {
        let code = std::fs::read_to_string(&unit).unwrap();
        if !code.contains("OXIVGL_PPA_LABEL") {
            let mut patched = code.replace(
                "                     && (dsc->header.cf == LV_COLOR_FORMAT_RGB888\n                         || dsc->header.cf == LV_COLOR_FORMAT_RGB565)",
                "                     && (dsc->header.cf == LV_COLOR_FORMAT_RGB888\n                         || dsc->header.cf == LV_COLOR_FORMAT_RGB565\n                         || dsc->header.cf == LV_COLOR_FORMAT_ARGB8888)",
            );

            let eval_needle = "        default:\n            return 0;";
            assert!(patched.contains(eval_needle), "lv_draw_ppa.c: evaluate default not found");
            patched = patched.replace(
                eval_needle,
                "        /* OXIVGL_PPA_LABEL: a glyph is an A8 coverage mask with the colour\n\
                 \x20        * from a register, which is the engine's own foreground mode. */\n\
                 \x20#if defined(LV_USE_PPA_LABEL) && LV_USE_PPA_LABEL\n\
                 \x20       case LV_DRAW_TASK_TYPE_LABEL: {\n\
                 \x20               const lv_draw_label_dsc_t * dsc = (lv_draw_label_dsc_t *)t->draw_dsc;\n\
                 \x20               if(dsc->opa <= (lv_opa_t)LV_OPA_MIN) return 0;\n\
                 \x20               if(dsc->sel_start != dsc->sel_end) return 0;\n\
                 \x20               if(dsc->outline_stroke_width > 0\n\
                 \x20                  && dsc->outline_stroke_opa > (lv_opa_t)LV_OPA_MIN) return 0;\n\
                 \x20               if(dsc->rotation != 0) return 0;\n\
                 \x20               if(!lv_draw_ppa_label_supported(dsc)) return 0;\n\
                 \x20\n\
                 \x20               if(t->preference_score > DRAW_UNIT_PPA_PREF_SCORE) {\n\
                 \x20                   t->preference_score = DRAW_UNIT_PPA_PREF_SCORE;\n\
                 \x20                   t->preferred_draw_unit_id = DRAW_UNIT_ID_PPA;\n\
                 \x20               }\n\
                 \x20               return 1;\n\
                 \x20           }\n\
                 \x20#endif\n\
                 \x20\n\
                 \x20       default:\n\
                 \x20           return 0;",
            );

            let disp_needle = "        case LV_DRAW_TASK_TYPE_IMAGE:\n            lv_draw_ppa_img(t, (lv_draw_image_dsc_t *)t->draw_dsc, &area);\n            lv_draw_buf_invalidate_cache(buf, &area);\n            break;";
            assert!(patched.contains(disp_needle), "lv_draw_ppa.c: dispatch image case not found");
            patched = patched.replace(
                disp_needle,
                &format!("{disp_needle}\n\
                 \x20#if defined(LV_USE_PPA_LABEL) && LV_USE_PPA_LABEL\n\
                 \x20       case LV_DRAW_TASK_TYPE_LABEL:\n\
                 \x20           lv_draw_ppa_label(t, (lv_draw_label_dsc_t *)t->draw_dsc, &area);\n\
                 \x20           lv_draw_buf_invalidate_cache(buf, &area);\n\
                 \x20           break;\n\
                 \x20#endif"),
            );
            std::fs::write(&unit, patched).unwrap();
        }
    }

    // 3. Declare the new entry points.
    let hdr = ppa_dir.join("lv_draw_ppa.h");
    if hdr.exists() {
        let code = std::fs::read_to_string(&hdr).unwrap();
        if !code.contains("lv_draw_ppa_label") {
            // Beside the existing declaration, so these stay inside the
            // `#if LV_USE_PPA` block. Appended at the end of the file they
            // land outside it and name types no build without PPA includes.
            let anchor = "void lv_draw_ppa_fill(";
            let pos = code.find(anchor).expect("lv_draw_ppa.h: no lv_draw_ppa_fill");
            let decl_end = code[pos..].find(";\n").expect("lv_draw_ppa_fill: unterminated") + pos + 2;
            let mut patched = String::with_capacity(code.len() + 256);
            patched.push_str(&code[..decl_end]);
            patched.push_str(
                "\nvoid lv_draw_ppa_label(lv_draw_task_t * t, const lv_draw_label_dsc_t * dsc,\n\
                 \x20                     const lv_area_t * coords);\n\n\
                 bool lv_draw_ppa_label_supported(const lv_draw_label_dsc_t * dsc);\n",
            );
            patched.push_str(&code[decl_end..]);
            std::fs::write(&hdr, patched).unwrap();
        }
    }

    // 4. The label implementation itself.
    std::fs::write(ppa_dir.join("lv_draw_ppa_label.c"), PPA_LABEL_C).unwrap();
}

/// The PPA label draw unit, written into the LVGL tree by `patch_ppa_draw_unit`.
const PPA_LABEL_C: &str = r####"
/**
 * @file lv_draw_ppa_label.c
 *
 * Draw text through the PPA blend engine.
 *
 * A glyph is an A8 coverage mask plus one pen colour, which is precisely the
 * engine's A8 foreground mode: the mask supplies per-pixel alpha and
 * `fg_fix_rgb_val` supplies the colour. Layout, kerning, wrapping and
 * decorations stay in LVGL - this hooks `lv_draw_label_iterate_characters`
 * and replaces only the per-glyph blend.
 *
 * The task is all-or-nothing. A first pass probes every glyph without drawing;
 * if any of them cannot go to the engine, the whole task is handed to
 * `lv_draw_sw_label`. Drawing some glyphs on the engine and then discovering an
 * unsupported one would leave half-rendered text on screen, so the probe pays
 * for itself in correctness. It costs a second layout pass over the string.
 *
 * Added by oxivgl; not part of upstream LVGL 9.5.
 */

#include "lv_draw_ppa.h"

#if LV_USE_PPA

#include "lv_draw_ppa_private.h"
#include "../../lv_draw_label.h"
#include "../../lv_draw_label_private.h"
#include "../../lv_draw_buf.h"
#include "../../../font/lv_font.h"
#include "../../sw/lv_draw_sw.h"

/**
 * Glyphs below this many pixels go to software, and with them the whole task.
 *
 * A blend costs register programming, a DMA start and an interrupt, which a
 * small letter does not repay - and body text is mostly small letters. Large
 * text is where the engine wins. The threshold is deliberately conservative:
 * guessing high costs an opportunity, guessing low costs a regression.
 */
#define PPA_GLYPH_MIN_PX 1

/**
 * Probe result for the task being drawn.
 *
 * A file static because `lv_draw_glyph_cb_t` carries no user pointer. Safe
 * here: the PPA unit is a single draw unit and LVGL dispatches one task to it
 * at a time.
 */
static bool ppa_label_probe_ok;

static void ppa_probe_cb(lv_draw_task_t * t, lv_draw_glyph_dsc_t * glyph_dsc,
                         lv_draw_fill_dsc_t * fill_dsc, const lv_area_t * fill_area);
static void ppa_letter_cb(lv_draw_task_t * t, lv_draw_glyph_dsc_t * glyph_dsc,
                          lv_draw_fill_dsc_t * fill_dsc, const lv_area_t * fill_area);

bool lv_draw_ppa_label_supported(const lv_draw_label_dsc_t * dsc)
{
    /* Only the plain case; anything else is left to the software unit. */
    if(dsc->decor != LV_TEXT_DECOR_NONE) return false;
    if(dsc->opa < (lv_opa_t)LV_OPA_MAX) return false;
    if(dsc->font == NULL) return false;
    return true;
}

void lv_draw_ppa_label(lv_draw_task_t * t, const lv_draw_label_dsc_t * dsc, const lv_area_t * coords)
{
    if(dsc->opa <= (lv_opa_t)LV_OPA_MIN) return;

    ppa_label_probe_ok = true;
    lv_draw_label_iterate_characters(t, dsc, coords, ppa_probe_cb);

    if(!ppa_label_probe_ok) {
        lv_draw_sw_label(t, dsc, coords);
        return;
    }
    lv_draw_label_iterate_characters(t, dsc, coords, ppa_letter_cb);
}

static void ppa_probe_cb(lv_draw_task_t * t, lv_draw_glyph_dsc_t * glyph_dsc,
                         lv_draw_fill_dsc_t * fill_dsc, const lv_area_t * fill_area)
{
    LV_UNUSED(fill_dsc);
    LV_UNUSED(fill_area);
    if(!ppa_label_probe_ok || glyph_dsc == NULL) return;

    if(glyph_dsc->format != LV_FONT_GLYPH_FORMAT_A8 || glyph_dsc->rotation % 3600 != 0) {
        ppa_label_probe_ok = false;
        return;
    }

    lv_area_t clipped;
    if(!lv_area_intersect(&clipped, glyph_dsc->letter_coords, &t->clip_area)) return;
    if(lv_area_get_width(&clipped) * lv_area_get_height(&clipped) < PPA_GLYPH_MIN_PX) {
        ppa_label_probe_ok = false;
    }
}

static void ppa_letter_cb(lv_draw_task_t * t, lv_draw_glyph_dsc_t * glyph_dsc,
                          lv_draw_fill_dsc_t * fill_dsc, const lv_area_t * fill_area)
{
    /* Glyph backgrounds, underline and strike-through are plain fills. */
    if(fill_dsc && fill_area) {
        lv_draw_ppa_fill(t, fill_dsc, fill_area);
        return;
    }
    if(glyph_dsc == NULL) return;

    lv_layer_t * layer = t->target_layer;
    lv_draw_buf_t * draw_buf = layer->draw_buf;
    lv_color_format_t dest_cf = draw_buf->header.cf;

    lv_area_t dest_area;
    if(!lv_area_intersect(&dest_area, glyph_dsc->letter_coords, &t->clip_area)) return;

    glyph_dsc->glyph_data = lv_font_get_glyph_bitmap(glyph_dsc->g, glyph_dsc->_draw_buf);
    if(glyph_dsc->glyph_data == NULL) {
        LV_LOG_WARN("PPA label: no glyph bitmap");
        return;
    }
    const lv_draw_buf_t * mask = glyph_dsc->glyph_data;

    /* Where the clip cut into the glyph, the mask must be read from the same
     * offset or the coverage slides against the destination. */
    int32_t mask_off_x = dest_area.x1 - glyph_dsc->letter_coords->x1;
    int32_t mask_off_y = dest_area.y1 - glyph_dsc->letter_coords->y1;

    lv_area_t rel_dest = dest_area;
    lv_area_move(&rel_dest, -layer->buf_area.x1, -layer->buf_area.y1);

    lv_draw_ppa_unit_t * u = (lv_draw_ppa_unit_t *)t->draw_unit;
    uint8_t * dest_buf = draw_buf->data;
    lv_color32_t c32 = lv_color_to_32(glyph_dsc->color, LV_OPA_COVER);

    ppa_blend_oper_config_t cfg = {
        .in_bg = {
            .buffer          = dest_buf,
            .pic_w           = draw_buf->header.w,
            .pic_h           = draw_buf->header.h,
            .block_w         = lv_area_get_width(&dest_area),
            .block_h         = lv_area_get_height(&dest_area),
            .block_offset_x  = rel_dest.x1,
            .block_offset_y  = rel_dest.y1,
            .blend_cm        = lv_color_format_to_ppa_blend(dest_cf),
        },
        .bg_alpha_update_mode  = PPA_ALPHA_FIX_VALUE,
        .bg_alpha_fix_val      = 0xFF,
        .in_fg = {
            .buffer          = (void *)mask->data,
            .pic_w           = mask->header.stride,
            .pic_h           = mask->header.h,
            .block_w         = lv_area_get_width(&dest_area),
            .block_h         = lv_area_get_height(&dest_area),
            .block_offset_x  = mask_off_x,
            .block_offset_y  = mask_off_y,
            .blend_cm        = PPA_BLEND_COLOR_MODE_A8,
        },
        .fg_fix_rgb_val = { .r = c32.red, .g = c32.green, .b = c32.blue },
        .fg_alpha_update_mode  = PPA_ALPHA_NO_CHANGE,
        .out = {
            .buffer          = dest_buf,
            .buffer_size     = draw_buf->data_size,
            .pic_w           = draw_buf->header.w,
            .pic_h           = draw_buf->header.h,
            .block_offset_x  = rel_dest.x1,
            .block_offset_y  = rel_dest.y1,
            .blend_cm        = lv_color_format_to_ppa_blend(dest_cf),
        },
        .mode      = PPA_TRANS_MODE_BLOCKING,
        .user_data = u,
    };

    esp_err_t ret = ppa_do_blend(u->blend_client, &cfg);
    if(ret != ESP_OK) {
        LV_LOG_WARN("PPA label blend failed: %d", ret);
    }
}

#endif /* LV_USE_PPA */
"####;

/// LVGL 9.5 does not preserve text_length through the draw task pipeline
/// on 32-bit targets, truncating button text to 1 character.
fn patch_btnmatrix_text_length(lvgl_src: &Path) {
    let file = lvgl_src.join("widgets/buttonmatrix/lv_buttonmatrix.c");
    if !file.exists() {
        return;
    }
    let code = std::fs::read_to_string(&file).unwrap();
    let needle = "draw_label_dsc_act.text_local = true;\n        draw_label_dsc_act.base.id1";
    if code.contains(needle) && !code.contains("draw_label_dsc_act.text_length") {
        let patched = code.replace(
            needle,
            "draw_label_dsc_act.text_local = true;\n        draw_label_dsc_act.text_length = (uint32_t)lv_strlen(txt);\n        draw_label_dsc_act.base.id1",
        );
        std::fs::write(&file, patched).unwrap();
    }
}

/// Route LVGL's transient per-frame render scratch to internal DRAM.
///
/// The SW renderer allocates a scratch buffer per draw op, every frame, and
/// frees it in the same draw call — draw-task descriptors, scanline masks (arc,
/// line, fill, border, triangle, rect-mask), box-shadow blur/mask buffers, and
/// image mask/transform buffers. All use a plain `lv_malloc`/`lv_free` with no
/// callback hook (unlike draw buffers). Once a runtime pool is registered they
/// become eligible to land in it; when that pool is PSRAM, the churn against
/// PSRAM-resident TLSF metadata halves render throughput on ESP32 (issue #124).
///
/// Redirect them to `oxivgl_render_scratch_{malloc,zalloc,free}` (defined in
/// `oxivgl::render_scratch`), which keep the scratch in internal DRAM while a
/// pool is active and delegate to LVGL's allocator otherwise. Those symbols are
/// always linked, so this patch is unconditional.
///
/// Both the transient per-frame scratch and the radius/circle mask cache are
/// routed. The mask cache (`lv_draw_sw_mask.c`) is read per-scanline every
/// frame — from PSRAM it is a hot cost on ESP32 — and although it is a
/// cross-*function* cache, every one of its alloc/free sites is lexically inside
/// `lv_draw_sw_mask.c` (even `lv_draw_sw_mask_free_param`, which arc/line call),
/// so exhaustive file routing keeps every pair in one regime. The gradient cache
/// in `lv_draw_sw_grad.c` stays on LVGL's allocator: gradient-only (outside the
/// arc/line/rounded path) and cross-function with multi-path frees, so it needs
/// a separate, careful pass if a gradient-heavy UI ever calls for it.
fn patch_render_scratch(lvgl_src: &Path) {
    // The draw-task descriptor lives in lv_draw.c, which also holds non-scratch
    // allocations (draw units, layers, sub-descriptors), so only its two
    // descriptor sites are routed — targeted, not exhaustive.
    route_scratch_descriptor(lvgl_src);

    // These SW-draw sources allocate *only* render-internal buffers (per-frame
    // scratch, plus the self-contained radius/circle mask cache), so every
    // lv_malloc/lv_malloc_zeroed/lv_free is routed and the site counts are
    // pinned: a future LVGL that adds a non-render allocation here fails the
    // build loudly rather than silently half-routing (which would corrupt the
    // heap). Columns: (path, n_malloc, n_malloc_zeroed, n_free).
    for (rel, n_malloc, n_zeroed, n_free) in [
        ("draw/sw/lv_draw_sw_arc.c", 2, 0, 2),
        ("draw/sw/lv_draw_sw_border.c", 1, 0, 1),
        ("draw/sw/lv_draw_sw_box_shadow.c", 6, 0, 4),
        ("draw/sw/lv_draw_sw_fill.c", 1, 0, 1),
        ("draw/sw/lv_draw_sw_line.c", 3, 0, 3),
        ("draw/sw/lv_draw_sw_img.c", 5, 0, 3),
        ("draw/sw/lv_draw_sw_mask_rect.c", 1, 0, 1),
        ("draw/sw/lv_draw_sw_triangle.c", 1, 0, 1),
        ("draw/sw/lv_draw_sw_mask.c", 1, 2, 5),
    ] {
        route_scratch_exhaustive(lvgl_src, rel, n_malloc, n_zeroed, n_free);
    }
}

/// Prepend the render-scratch prototypes (and `<stddef.h>` for `size_t`) to a
/// patched source. Prepending — rather than anchoring after includes — makes
/// the declarations unconditionally visible to every routed call site, so a
/// missing prototype can never silently degrade to an implicit `int` return
/// that truncates the 64-bit pointer on host.
fn with_scratch_protos(code: &str) -> String {
    const HEAD: &str = "#include <stddef.h>\n\
        extern void * oxivgl_render_scratch_malloc(size_t size);\n\
        extern void * oxivgl_render_scratch_zalloc(size_t size);\n\
        extern void oxivgl_render_scratch_free(void * ptr);\n";
    format!("{HEAD}{code}")
}

/// Route the two draw-task descriptor sites in `lv_draw.c` (targeted: this file
/// also holds non-scratch allocations that must stay on LVGL's allocator).
fn route_scratch_descriptor(lvgl_src: &Path) {
    let file = lvgl_src.join("draw/lv_draw.c");
    if !file.exists() {
        return;
    }
    let code = std::fs::read_to_string(&file).unwrap();
    if code.contains("oxivgl_render_scratch") {
        return; // already patched (source persists in OUT_DIR across reruns)
    }
    let alloc_needle = "lv_draw_task_t * new_task = lv_malloc_zeroed(LV_ALIGN_UP(sizeof(lv_draw_task_t), 8) + dsc_size);";
    let free_needle = "lv_free(t);";
    assert!(
        code.contains(alloc_needle) && code.contains(free_needle),
        "lv_draw.c descriptor sites do not match the pinned LVGL v{LVGL_VERSION} \
         source — the render-scratch patch (issue #124) would silently no-op."
    );
    let code = code
        .replace(
            alloc_needle,
            "lv_draw_task_t * new_task = oxivgl_render_scratch_zalloc(LV_ALIGN_UP(sizeof(lv_draw_task_t), 8) + dsc_size);",
        )
        .replace(free_needle, "oxivgl_render_scratch_free(t);");
    std::fs::write(&file, with_scratch_protos(&code)).unwrap();
}

/// Route every `lv_malloc`/`lv_malloc_zeroed`/`lv_free` in an SW-draw source
/// that allocates only render-internal buffers, pinning the site counts against
/// version drift. Safe because every alloc/free pair is lexically in the file,
/// so both ends are routed together (one allocation regime).
fn route_scratch_exhaustive(
    lvgl_src: &Path,
    rel: &str,
    n_malloc: usize,
    n_zeroed: usize,
    n_free: usize,
) {
    let file = lvgl_src.join(rel);
    if !file.exists() {
        return;
    }
    let code = std::fs::read_to_string(&file).unwrap();
    if code.contains("oxivgl_render_scratch") {
        return;
    }
    // Pin the shape against the verified LVGL source. `lv_malloc(` does not
    // match inside `lv_malloc_zeroed(` (a `_` follows, not `(`), so the two
    // counts are independent. A mismatch means the file changed — fail loudly
    // rather than silently half-route or miss a site.
    assert_eq!(
        code.matches("lv_realloc(").count(),
        0,
        "{rel}: unexpected lv_realloc — re-verify scratch routing (#124)"
    );
    assert_eq!(
        code.matches("lv_malloc_zeroed(").count(),
        n_zeroed,
        "{rel}: lv_malloc_zeroed site count changed vs pinned LVGL v{LVGL_VERSION} — re-verify scratch routing (#124)"
    );
    assert_eq!(
        code.matches("lv_malloc(").count(),
        n_malloc,
        "{rel}: lv_malloc site count changed vs pinned LVGL v{LVGL_VERSION} — re-verify scratch routing (#124)"
    );
    assert_eq!(
        code.matches("lv_free(").count(),
        n_free,
        "{rel}: lv_free site count changed vs pinned LVGL v{LVGL_VERSION} — re-verify scratch routing (#124)"
    );

    // Replace `lv_malloc_zeroed(` before `lv_malloc(` so the plain-malloc pass
    // never touches the zeroed sites (they are disjoint, but order makes it
    // obviously correct).
    let patched = code
        .replace("lv_malloc_zeroed(", "oxivgl_render_scratch_zalloc(")
        .replace("lv_malloc(", "oxivgl_render_scratch_malloc(")
        .replace("lv_free(", "oxivgl_render_scratch_free(");
    let patched = with_scratch_protos(&patched);
    assert_eq!(
        patched.matches("lv_malloc(").count(),
        0,
        "{rel}: unrouted lv_malloc remains after patch"
    );
    assert_eq!(
        patched.matches("lv_malloc_zeroed(").count(),
        0,
        "{rel}: unrouted lv_malloc_zeroed remains after patch"
    );
    assert_eq!(
        patched.matches("lv_free(").count(),
        0,
        "{rel}: unrouted lv_free remains after patch"
    );
    std::fs::write(&file, patched).unwrap();
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

/// Make the demos' memory guards see a runtime-registered second pool.
///
/// Both demos gate on `LV_MEM_SIZE`, which stopped meaning "the whole heap"
/// when `lv_mem_add_pool` arrived: `oxivgl::mem::reserve_pool` registers an
/// overflow pool right after `lv_init`, and `LV_MEM_POOL_EXPAND_SIZE` is the
/// compile-time ceiling on it. So a board that keeps a small primary and spills
/// the bulk elsewhere is refused while in fact having the memory -- and on
/// ESP32 the primary MUST stay small, because moving `lv_init`'s objects into
/// uncached PSRAM costs 13.7 ms per page flip, measured.
///
/// Widening to `LV_MEM_SIZE + LV_MEM_POOL_EXPAND_SIZE` leaves single-pool builds
/// judged exactly as before: `LV_MEM_POOL_EXPAND_SIZE` defaults to 0.
fn patch_demo_mem_guards(lvgl_dir: &Path) {
    const GUARDS: [(&str, &str); 2] = [
        (
            "demos/widgets/lv_demo_widgets.c",
            "LV_MEM_SIZE < (38ul * 1024ul)",
        ),
        (
            "demos/benchmark/lv_demo_benchmark.c",
            "LV_MEM_SIZE < 128 * 1024",
        ),
    ];
    for (rel, guard) in GUARDS {
        let file = lvgl_dir.join(rel);
        let Ok(code) = fs::read_to_string(&file) else {
            continue;
        };
        if !code.contains(guard) {
            continue;
        }
        let widened = guard.replacen("LV_MEM_SIZE", "(LV_MEM_SIZE + LV_MEM_POOL_EXPAND_SIZE)", 1);
        fs::write(&file, code.replace(guard, &widened)).unwrap();
    }
}

/// Let the benchmark run more than once per boot.
///
/// `lv_demo_benchmark` accumulates each scene's results into the `scenes[]`
/// table itself -- `cpu_avg_usage`, `fps_avg`, `render_avg_time`,
/// `flush_avg_time`, `measurement_cnt` -- while `lv_demo_benchmark()` resets
/// only `scene_act`. So a second run adds its samples to the first run's, and
/// the averages, which divide by `measurement_cnt`, silently describe both.
///
/// That is a real constraint rather than an oversight to ignore, which is why
/// this crate refused a second run outright. But a benchmark you can take once
/// per boot cannot be compared against itself, and for a consumer running it as
/// a display-path soak -- the case #11 exists for -- one shot proves the least
/// interesting thing. Zeroing the accumulators at the top of a run makes each
/// run independent, and the refusal becomes unnecessary.
///
/// Injected after the function's own `scene_act = 0;` so the two read together.
/// The loop stops on the table's `create_cb == NULL` sentinel, the same
/// terminator the summary walk uses.
fn patch_demo_repeatable(lvgl_dir: &Path) {
    const ANCHOR: &str = "void lv_demo_benchmark(void)\n{\n    scene_act = 0;\n";
    const RESET: &str = r#"
    /*oxivgl-sys: without this a second run blends the first run's samples
     *into its own averages.*/
    for(uint32_t i = 0; scenes[i].create_cb; i++) {
        scenes[i].cpu_avg_usage = 0;
        scenes[i].fps_avg = 0;
        scenes[i].render_avg_time = 0;
        scenes[i].flush_avg_time = 0;
        scenes[i].measurement_cnt = 0;
    }
"#;
    let file = lvgl_dir.join("demos/benchmark/lv_demo_benchmark.c");
    let Ok(code) = fs::read_to_string(&file) else {
        return;
    };
    if !code.contains(ANCHOR) || code.contains("scenes[i].measurement_cnt = 0;") {
        return;
    }
    let patched = code.replacen(ANCHOR, &format!("{ANCHOR}{RESET}"), 1);
    fs::write(&file, patched).unwrap();
}
