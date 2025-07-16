//! Example usage of the anim2rbx library

use anim2rbx::AnimationConverter;
use std::{fs::File, io::BufWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing anim2rbx library API...");

    // Example 1: Default settings using Default trait
    let converter_default = AnimationConverter::default();
    println!("Default converter: filter={}, epsilon={}",
             converter_default.filter_identical_bones, converter_default.epsilon);

    // Example 2: Custom settings using new()
    let converter_custom = AnimationConverter::new(true, 1e-5);

    // Example 3: Builder pattern with with_ prefix (most flexible)
    let converter_builder = AnimationConverter::default()
        .with_filter_identical_bones(true)
        .with_epsilon(1e-5);

    // Convert file to keyframes using the builder pattern converter
    let keyframes = converter_builder.convert_file_to_keyframes("myanim.fbx")?;
    println!("Extracted {} keyframes from animation", keyframes.len());

    // Convert directly to WeakDom
    let kfs_dom = converter_custom.convert_file_to_weakdom("myanim.fbx")?;

    // Write to file
    let output = BufWriter::new(File::create("library_test.rbxm")?);
    rbx_binary::to_writer(output, &kfs_dom, &[kfs_dom.root_ref()])?;

    println!("Successfully created library_test.rbxm using library API");

    Ok(())
}
