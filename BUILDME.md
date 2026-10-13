# Procrustes a.k.a. Damastes

Audio album builder; copy and edit on the fly.

## Development

- [Format Rust code](https://github.com/rust-lang/rustfmt)
- [TagLib](https://github.com/taglib/taglib) library, release build, required on the system;
  [Rust bindings](https://github.com/ebassi/taglib-rust)
- Arch Linux family:

```
yay -S taglib
```

- Ubuntu/Debian:

```
sudo apt-get install libtagc0-dev
```

- Fedora:

```
sudo dnf install taglib-devel
```

### Toolchain

- Install `nightly` (or `stable`)

```
rustup toolchain install nightly
```

- Set default

```
rustup default nightly
```

- Better still, select inside your project root,
  then check `.rustup/settings.toml`

```
rustup override set nightly
```

Don't bother if the project has `rust-toolchain.toml`

- Manipulate overrides

```
rustup override list
```

```
rustup override unset
```

- Update all toolchains

```
rustup update
```

### Build

```
cargo build [--release]
```

### Format

```
cargo fmt
```

### Test

```
cargo test
```

### Install

Install to `~/.cargo/bin`:

```
cargo install --locked --path . [--debug]
```

### Publish

```
TODO
```
