# Image FFI Project

A CLI application that loads PNG images, applies processing plugins, and saves the result.

## Project Structure

```
image_ffi_project/
├── Cargo.toml              # Workspace
├── image_processor/        # CLI application (binary)
├── mirror_plugin/          # Mirror/flip plugin (cdylib)
├── blur_plugin/            # Blur plugin (cdylib)
└── README.md
```

## Build

```bash
cargo build
```

Plugins are built to `target/debug/`:
- `libmirror_plugin.so` (Linux) / `libmirror_plugin.dylib` (macOS) / `mirror_plugin.dll` (Windows)
- `libblur_plugin.so` (Linux) / `libblur_plugin.dylib` (macOS) / `blur_plugin.dll` (Windows)

## Usage

```bash
./target/debug/image-processor \
  --input input.png \
  --output output.png \
  --plugin mirror_plugin \
  --params params.json \
  --plugin-path target/debug
```

## Arguments

| Argument | Description |
|----------|-------------|
| `-i, --input` | Path to the input PNG image |
| `-o, --output` | Path to save the processed image |
| `-p, --plugin` | Plugin name without extension (e.g. `mirror_plugin`, `blur_plugin`) |
| `--params` | Path to a JSON file with plugin parameters |
| `-P, --plugin-path` | Directory containing the plugin library (default: `target/debug`) |

## Plugin Parameters (JSON)

### mirror_plugin

```json
{"horizontal": true, "vertical": false}
```

### blur_plugin

```json
{"radius": 5, "iterations": 2}
```

## Examples

Horizontal mirror:
```bash
echo '{"horizontal": true, "vertical": false}' > params.json
cargo run -p image_processor -- -i input.png -o mirrored.png -p mirror_plugin --params params.json
```

Blur:
```bash
echo '{"radius": 3, "iterations": 1}' > params.json
cargo run -p image_processor -- -i input.png -o blurred.png -p blur_plugin --params params.json
```

## Testing

```bash
cargo test
```
