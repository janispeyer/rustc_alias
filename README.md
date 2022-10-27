# rustc_alias

## Requirements

The optimisations developed here require the extension to the rust compiler introduced here: [https://github.com/janispeyer/rust](https://github.com/janispeyer/rust)

Make sure to build it with the component `rustc_driver`. This component is used for the extension.

## Example Setup

1. Clone both repositories

    ```bash
    git clone git@github.com:janispeyer/rust.git
    git clone git@github.com:janispeyer/rustc_alias.git
    ```

2. Build Rust (detailed documentation here: [https://rustc-dev-guide.rust-lang.org/building/how-to-build-and-run.html](https://rustc-dev-guide.rust-lang.org/building/how-to-build-and-run.html))

3. Create custom rustup toolchain

    ```bash
    rustup toolchain link stage1 build/<host-triple>/stage1
    ```

    Source: [https://rustc-dev-guide.rust-lang.org/building/how-to-build-and-run.html#creating-a-rustup-toolchain](https://rustc-dev-guide.rust-lang.org/building/how-to-build-and-run.html#creating-a-rustup-toolchain)

4. Build and run rustc_alias with custom toolchain

    Do the following once inside the folder rustc_alias to set the toolchain for the project:

    ```bash
    rustup override set stage1
    ```

    Now everytime the project is built or run, the correct rustc will be used. The project functions as kind of rustc wrapper with some functionality injected. You should be able to pass all rustc command line arguments, as if you were running rustc directly.

    ```bash
    cargo build
    cargo run -- -Zmir-emit-retag <file>
    ```

## Auto Formatting and Tooling

Note: It would be best to build and add components like clippy and fmt into the custom toolchains.

A simpler workaround is to use another toolchain for formatting:

```bash
cargo +nightly fmt
```

Example configuration for auto-formatting in Visual Studio Code:

```json
"rust-analyzer.rustfmt.overrideCommand": [
    "rustup",
    "run",
    "nightly",
    "--",
    "rustfmt",
    "--edition",
    "2021",
    "--"
],
"editor.formatOnSave": true,
```

This this not work for clippy as it will complain about the custom injection point not existing. (And rightly so, because on toolchains like stable or nightly the injection point does not exist.)

### `rust-analyzer` Settings

In the `Cargo.toml` file you'll find the lines:

```toml
[package.metadata.rust-analyzer]
rustc_private = true
```

These are required by rust-anylzer, when `#![feature(rustc_private)]` is used.

An additional requirement to make rust-analyzer work is to set the setting `rust-analyzer.rustc.source` to the correct path. More information can be found in the [rust-analyzer manual](https://rust-analyzer.github.io/manual.html).
