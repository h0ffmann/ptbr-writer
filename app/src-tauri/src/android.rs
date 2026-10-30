//! The overlay's JNI entry points: `io.github.piabrota.gramatico.Ptbr` (gen/android) calls them
//! from the accessibility service's worker thread. Each one turns Java strings into Rust ones and
//! back; the work is overlay.rs's. A panic becomes an `{"error"}`, never an unwind into the JVM.

use crate::overlay;
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};
use std::panic::{UnwindSafe, catch_unwind};
use std::path::Path;

fn guarded(f: impl FnOnce() -> String + UnwindSafe) -> String {
    catch_unwind(f).unwrap_or_else(|_| r#"{"error":"Erro interno do gramatico."}"#.into())
}

fn string(env: &mut JNIEnv, s: &JString) -> String {
    env.get_string(s).map(String::from).unwrap_or_default()
}

fn out(env: &mut JNIEnv, s: String) -> jstring {
    env.new_string(s).map(|j| j.into_raw()).unwrap_or(std::ptr::null_mut())
}

/// `Ptbr.check(dataDir, text)`: dataDir is the folder Tauri's app_config_dir is on Android.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_piabrota_gramatico_Ptbr_check<'l>(
    mut env: JNIEnv<'l>,
    _: JClass<'l>,
    dir: JString<'l>,
    text: JString<'l>,
) -> jstring {
    let (dir, text) = (string(&mut env, &dir), string(&mut env, &text));
    let r = guarded(move || overlay::check(Path::new(&dir), &text));
    out(&mut env, r)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_piabrota_gramatico_Ptbr_accept<'l>(
    mut env: JNIEnv<'l>,
    _: JClass<'l>,
    id: jint,
) -> jstring {
    let r = guarded(move || overlay::accept(id.max(0) as usize));
    out(&mut env, r)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_piabrota_gramatico_Ptbr_ignore<'l>(
    mut env: JNIEnv<'l>,
    _: JClass<'l>,
    id: jint,
) -> jstring {
    let r = guarded(move || overlay::ignore(id.max(0) as usize));
    out(&mut env, r)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_piabrota_gramatico_Ptbr_clear<'l>(_: JNIEnv<'l>, _: JClass<'l>) {
    let _ = catch_unwind(overlay::clear);
}
