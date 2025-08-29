use anyhow::Result;
use chu_engine::run;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;


fn main() -> Result<()> {
    run()
}
