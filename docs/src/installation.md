# Installation

## With Cargo

Requires a Rust toolchain. Install from the repository:

```sh
cargo install --git https://github.com/pwnxpl0it/spark
```

On Linux, the clipboard feature pulls in a Wayland dependency; install
`libwayland-dev` first, or build without the clipboard (see below).

## Precompiled binaries

Every release ships binaries for Linux (`x86_64`, `aarch64`), macOS (Intel and
Apple Silicon), and Windows (`x86_64`) on the [Releases][releases] page, each
with a `.sha256` checksum. Archives are named
`spark-<VERSION>-<TARGET>.tar.gz` (or `.zip` on Windows):

```sh
# Linux / macOS
tar -xzf spark-<VERSION>-<TARGET>.tar.gz
sudo install -m 755 spark-<VERSION>-<TARGET>/spark /usr/local/bin/spark
```

On Windows, extract the `.zip` and put `spark.exe` somewhere on your `PATH`.

Replace `<VERSION>` with the release tag (for example `v2.0.0`) and `<TARGET>`
with your platform, such as `x86_64-unknown-linux-gnu` or
`aarch64-apple-darwin`.

## Verify

```sh
spark --version
```

## Without the clipboard feature

Clipboard support (`clipboard://`) is a default Cargo feature backed by the
system clipboard. Build without it to drop the X11/Wayland dependencies:

```sh
cargo install --git https://github.com/pwnxpl0it/spark --no-default-features
```

`clipboard://` targets then return an error at write time instead of copying.

[releases]: https://github.com/pwnxpl0it/spark/releases
