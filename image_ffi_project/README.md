# Image FFI Project — Image Processing via Rust Plugins

CLI application that loads an image, converts it to the RGBA8 format, invokes a `process_image` function from a dynamically loaded Rust plugin, and saves the result as a PNG file.

## Requirements

- Linux (on Linux, plugins are compiled into `target/debug/libmirror.so` and `target/debug/libblur.so`)
- Rust and Cargo **1.85+** (edition 2024). Install via [rustup](https://rustup.rs/).

## Structure

- `image_processor/` — CLI, image processing logic, `.so` loading, error handling
- `plugin_interface/` — shared C ABI signature
- `mirror_plugin/` — image mirroring (`cdylib`)
- `blur_plugin/` — box blur effect (`cdylib`)
- `params/` — example parameter files
- `input.png` — sample input image

## Building and Testing

Run from the workspace root:

```bash
cargo build --workspace
cargo test --workspace
```

## Start

```bash
cargo run -p image_processor -- \
  --input input.png \
  --output out_mirror.png \
  --plugin mirror \
  --params params/mirror.txt

cargo run -p image_processor -- \
  --input input.png \
  --output out_blur.png \
  --plugin blur \
  --params params/blur.txt
```

The `--plugin-path` parameter defaults to `target/debug` but can be changed (e.g., `--plugin-path /home/user/plugins`). Specify the plugin name **without** the `lib` prefix or the `.so` extension. Note that `cargo run` does not automatically build the plugins; please run `cargo build --workspace` first. 

```bash
cargo run -p image_processor -- --help
```

```bash
cargo run -p image_processor -- --help
```

## Parameter Format

Parameters are specified as `key=value` pairs, separated by commas (`,`) or newlines. Empty strings or missing keys default to predefined values.

**mirror:** `horizontal=false`, `vertical=false` (boolean `true`/`false`).

**blur:** `radius=1`, `iterations=1` (non-negative integers; capped at 100). Implements a box blur: the value of each RGBA channel is replaced by the average of its neighbors within a square defined by `radius`. Repeating this process `iterations` times results in a stronger blur.

If parameter parsing fails, the plugin writes an error message to `stderr` and leaves the image unchanged. Per the assignment requirements, the `process_image` ABI has a `void` return type; therefore, the plugin cannot signal failure to the CLI via a return code. Errors related to file access, image format, library loading, or saving the PNG file will result in a non-zero exit code for the CLI.

## Security

- `Library`, `Symbol`, `CString`, and the RGBA buffer persist for the duration of the plugin call.
- The buffer size is validated before the call; potential overflows are checked using `checked_mul`.
- Each plugin’s exported function is wrapped in `catch_unwind` to prevent Rust panics from propagating across the FFI boundary. If an error occurs during processing, the buffer is restored from a backup copy.
- Use only trusted plugins: `unsafe extern "C"` provides no safety guarantees for arbitrary third-party dynamic libraries.