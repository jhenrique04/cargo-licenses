# cargo-licenses
A command-line tool that scans your Cargo.toml for direct dependencies (optionally including dev-dependencies and build-dependencies), fetches their license info from crates.io, and generates a concise license report in either Markdown or JSON.

## Features
- **Direct Dependency Parsing**  
  Reads `[dependencies]`, `[dev-dependencies]`, and `[build-dependencies]` from your Cargo.toml.
- **Optional Dependencies**  
  Choose whether to skip dependencies marked `optional = true`.
- **Semver Matching**  
  Handles version constraints like "0.12" (or unspecified) by finding the newest matching version on crates.io.
- **Flexible Output**  
  Generates `.license_report.md` (Markdown) or `.license_report.json` (JSON).
- **CLI Flags**  
    - `--dev` to include dev-dependencies  
    - `--build` to include build-dependencies  
    - `--skip-optional` to skip optional dependencies  
    - `--format [md|json]` to choose the report format

## Installation
1. Clone this repository (or download the code).
2. In the project root, run:
    ```bash
    cargo build
    ```
    to build the binary.

    For a globally installed binary, run:
    ```bash
    cargo install --path .
    ```

    This places cargo-licenses in ~/.cargo/bin, so you can run it from any project folder.
    Usage

## Usage
Within the project directory (or any Rust project you want to analyze):

```bash
# Generate a Markdown report (.license_report.md)
cargo run generate

# Generate a JSON report (.license_report.json)
cargo run generate --format json

# Include dev-dependencies and build-dependencies as well
cargo run generate --dev --build

# Skip optional dependencies
cargo run generate --skip-optional
```

## Other Commands
List direct dependencies and their version constraints (without fetching license info):

```bash    
cargo run list [--dev] [--build] [--skip-optional]
```

Show the tool version:

```bash
cargo run version
```

or (for globally installed binary) 

```bash  
cargo-licenses version
```

## Example
Suppose your Cargo.toml has:
```bash
[dependencies]
reqwest = "0.12"

[dev-dependencies]
tempfile = "3.3.0"

[build-dependencies]
rand = "0.8"

[dependencies.mycrate]
version = "1.0"
optional = true
```
By default, on `generate` or `list` commands, the tool reports only [dependencies]:
```bash
reqwest (0.12)
mycrate (1.0)
```
Passing `--dev` adds `tempfile`, `--build` adds `rand` and `--skip-optional` excludes mycrate (since it’s optional=true).

The resulting .license_report.md or .license_report.json shows each crate’s resolved version and its license info from crates.io.

## Contributing
Fork this repo and clone locally.
Create a new branch for your feature or bug fix.
Make your changes and run cargo build && cargo test to ensure everything works.
Submit a pull request describing your changes.

## License

Licensed under either of:

    Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
    
    MIT license (http://opensource.org/licenses/MIT)

at your option.

For more information, visit the documentation on [crates.io](https://crates.io/crates/cargo-licenses).
