use std::io::Result;

fn main() -> Result<()> {
    let _ = sugondese::WebApi::new("172.17.0.2:6080").run()?;

    Ok(())
}
