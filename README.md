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
    - `--deny` [LICENSE] to block crates with specified licenses.
    - `--allow` [LICENSE] to only allow crates with specified licenses.
    - Supports complex expressions like `"MIT OR Apache-2.0"` for flexible rules.    
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

### Arch Linux
  On Arch Linux a package is available from the AUR repositories. To install it, simply run:
  ```bash
  paru -S cargo-licenses
  # Or, if you use yay:
  yay -S cargo-licenses
  ```

### NetBSD
  On NetBSD a package is available from the official repositories. To install it, simply run:
  ```bash
  pkgin install cargo-licenses
  ```

### Pop!_OS

  On Pop!_OS, a package is available from the official repositories. To install it, run:
  ```bash
  sudo apt install cargo-licenses
  ```

## Usage
To generate a report:

```bash
# Generate a Markdown report (.license_report.md)
cargo-licenses generate

# Generate a JSON report (.license_report.json)
cargo-licenses generate --format json

# Include dev-dependencies and build-dependencies as well
cargo-licenses generate --dev --build

# Skip optional dependencies
cargo-licenses generate --skip-optional

```
Check licenses against user-defined policies:
```bash
# Check licenses against a deny list
cargo-licenses check --deny MIT --deny Apache-2.0

# Check licenses against an allow list
cargo-licenses check --allow MIT --allow BSD-3-Clause

# Check licenses by parsing expressions
cargo-licenses check --deny "MIT OR Apache-2.0"
```

List direct dependencies and their version constraints (without fetching license info):

```bash    
cargo-licenses list [--dev] [--build] [--skip-optional]
```

Show the tool version:

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
