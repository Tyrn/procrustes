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

- Various checks

```
rustup show active-toolchain
```

```
rustup which rust-analyzer
```

On error/mismatch

```
rustup component add rust-analyzer
```

### Build

```
cargo build
```

- If you ever want a lean build with only one backend, you can still do:

```
cargo build                                       # both backends, runtime choice
cargo build --no-default-features --features backend-taglib   # lean, taglib only
cargo build --no-default-features --features backend-lofty    # lean, lofty only
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
cargo install --locked --path .
```

### Publish

```
TODO
```
