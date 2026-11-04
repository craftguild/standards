//! Deterministic Small Slant generation from pinned embedded FIGlet data.

use craftguild_core::Result;
use figlet_rs::FIGlet;

const FONT: &str = include_str!("../../../assets/fonts/smslant.flf");

/// Generate normalized ASCII artwork without downloads or a Node.js process.
pub fn generate(name: &str) -> Result<String> {
    if name.is_empty() || !name.bytes().all(|b| (32..=126).contains(&b)) {
        return Err("Banner text must be nonempty printable ASCII".into());
    }
    let font = FIGlet::from_content(FONT)?;
    if name.chars().any(|c| !font.fonts.contains_key(&(c as u32))) {
        return Err("The pinned font does not support this character".into());
    }
    let figure = font
        .convert(name)
        .ok_or("The pinned font cannot render this text")?;
    let raw = figure.to_string();
    let lines: Vec<_> = raw.lines().map(str::trim_end).collect();
    let first = lines
        .iter()
        .position(|s| !s.is_empty())
        .ok_or("Banner has no visible glyphs")?;
    let last = lines.iter().rposition(|s| !s.is_empty()).unwrap();
    Ok(lines[first..=last].join("\n") + "\n")
}

/// CLI argument adapter; --check reads without modifying the target.
pub fn run(args: &[String], output: &mut impl std::io::Write) -> Result<()> {
    if matches!(args, [flag] if flag=="-h" || flag=="--help") {
        output.write_all(b"Usage: banner TEXT [--output PATH | --check PATH]\n")?;
        output.flush()?;
        return Ok(());
    }
    let name = args
        .first()
        .ok_or("Usage: banner TEXT [--output PATH | --check PATH]")?;
    let text = generate(name)?;
    match &args[1..] {
        [] => output.write_all(text.as_bytes())?,
        [mode, path] if mode == "--output" => std::fs::write(path, text)?,
        [mode, path] if mode == "--check" => {
            if std::fs::read(path)? != text.as_bytes() {
                return Err(
                    format!("Banner differs: {path}; regenerate with banner --output").into(),
                );
            }
        }
        _ => return Err("Usage: banner TEXT [--output PATH | --check PATH]".into()),
    }
    output.flush()?;
    Ok(())
}
