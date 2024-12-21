use anyhow::Result;
use lib_wgpu_learn::run;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;


fn main() -> Result<()> {
    run()
}
