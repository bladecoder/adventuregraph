fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = "../assets/story.ink.json";
    println!("cargo:rerun-if-changed={source}");
    let image = bladeink::image::compile_json_to_image(std::fs::File::open(source)?)?;
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or("OUT_DIR missing")?);
    std::fs::write(output.join("story.inkb"), image)?;
    for name in ["events", "defaults", "alignment"] {
        let fixture = format!("tests/fixtures/{name}.ink.json");
        println!("cargo:rerun-if-changed={fixture}");
        let image = bladeink::image::compile_json_to_image(std::fs::File::open(fixture)?)?;
        std::fs::write(output.join(format!("{name}.inkb")), image)?;
    }
    Ok(())
}
