//! Regression test for https://github.com/makepad/stitch/issues/1
//!
//! Stitch limits the size of a single Wasm function body. That limit used to be 128 KiB,
//! which is far below the WebAssembly JS-API implementation-defined limit (7,654,321 bytes)
//! and caused large modules such as FFmpeg to fail to decode. These tests pin the behavior:
//! a body larger than the old limit but within the standard limit must now decode, while a
//! body exceeding the standard limit must still be rejected.

use makepad_stitch::{Engine, Module};

/// Build a module with a single exported function whose body consists of `nop_count` `nop`
/// instructions. Each `nop` encodes to a single byte, so the function body is roughly
/// `nop_count` bytes (plus a few bytes of framing), giving us precise control over its size.
fn module_with_nops(nop_count: usize) -> Vec<u8> {
    let mut wat = String::from("(module (func (export \"f\")");
    wat.reserve(nop_count * 5);
    for _ in 0..nop_count {
        wat.push_str(" nop");
    }
    wat.push_str("))");

    let buf = wast::parser::ParseBuffer::new(&wat).unwrap();
    let mut wat: wast::Wat = wast::parser::parse(&buf).unwrap();
    wat.encode().unwrap()
}

#[test]
fn body_larger_than_old_128kib_limit_decodes() {
    // 200_000 `nop`s => ~200 KB body: above the old 128 KiB cap, below the 7,654,321 limit.
    let bytes = module_with_nops(200_000);
    let engine = Engine::new();
    assert!(
        Module::new(&engine, &bytes).is_ok(),
        "a ~200 KB function body should decode after bumping MAX_FUNC_BODY_SIZE"
    );
}

#[test]
fn body_exceeding_standard_limit_is_rejected() {
    // Just over the 7,654,321-byte standard limit: must still be rejected at decode.
    let bytes = module_with_nops(7_654_322);
    let engine = Engine::new();
    let err = Module::new(&engine, &bytes).unwrap_err();
    assert!(
        err.to_string().contains("function body too large"),
        "an oversized function body should be rejected, got: {err}"
    );
}
