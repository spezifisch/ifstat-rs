use anyhow::Result;
use vergen::EmitBuilder;

pub fn main() -> Result<()> {
    EmitBuilder::builder()
        .build_timestamp()
        .all_rustc()
        .all_cargo()
        .git_sha(false)
        .git_dirty(true)
        .emit()?;
    Ok(())
}
