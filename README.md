# anim2rbx

A Rust library and CLI tool for converting animation files to Roblox KeyframeSequence format.

## Quick Start

### CLI Usage
```bash
# Basic conversion
anim2rbx animation.fbx

# With options
anim2rbx animation.fbx -o output.rbxm --verbose
```

### Library Usage
```rust
use anim2rbx::AnimationConverter;

let converter = AnimationConverter::default();
let kfs_dom = converter.convert_file_to_weakdom("animation.fbx")?;
```

## Installation

### via cargo
```bash
cargo install anim2rbx
```

### via rokit
```bash
rokit add jiwonz/anim2rbx
```

## Supported Formats

Supports [many formats](https://github.com/assimp/assimp/blob/master/doc/Fileformats.md) via Assimp including:
- **FBX** (.fbx) - Recommended for animations
- **COLLADA** (.dae) - Open standard
- **glTF** (.gltf, .glb) - Modern format
- **3ds Max** (.3ds), **Maya** (.ma/.mb)
- And many more

## Configuration

```rust
let converter = AnimationConverter::new(true, 1e-5)
    .with_filter_identical_bones(false)
    .with_epsilon(0.001);
```

Options:
- `--verbose` - Enable debug logging
- `--no-filter` - Keep identical poses
- `--epsilon` - Precision threshold

## License

MIT
