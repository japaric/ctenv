//! CTEnv (Name Subject to Change), compile time [.env] style configuration
//!
//! [.env]: https://crates.io/crates/dotenv
//!
//! # How to use?
//!
//! - Write this in the build script of the dependency:
//!
//! ```
//! // crate: foo
//! // file: build.rs
//!
//! fn main() -> Result<(), Box<Error>> {
//!     ctenv::run()?;
//!
//!     // ..
//! }
//! ```
//!
//! - Use the companion `ctenv!` macro (see `ctenv-macros` crate) for any stuff that needs compile
//! time configuration. For example:
//!
//! ```
//! // crate: foo
//! // file: src/lib.rs
//!
//! static mut BUFFER: [u8; ctenv!(BUF_SZ)] = [0; ctenv!(BUF_SZ)]
//! ```
//!
//! - Write a `.env` file containing the configuration in the *top* level crate.
//!
//! ```
//! $ cargo add foo
//!
//! $ cat .env
//! # All settings are namespaced. Syntax: $crate:$key=$value
//! foo:BUF_SZ=128
//! ```
//!
//! - `cargo build` and you are done
//!
//! # Known issues
//!
//! The dependency also needs a `.env` file or it won't build.
//!
//! There's no great to have the dependency communicate its dependents that some settings need to be
//! set in a `.env` file, other than in its crate level documentation.
//!
//! # Possible expansions
//!
//! Defaults and overrides. If a dependency contains a `.env` file those settings will be used
//! *unless* the top level crate overrides them in its own `.env` file. This should be
//! straightforward to implement but I don't know if it's actually a good idea or not.

use std::{env, error::Error, fmt, fs, path::PathBuf};

/// Call this from your build script
pub fn run() -> Result<(), Box<Error>> {
    // the name of the crate that's currently being built
    let pkg_name = env::var("CARGO_PKG_NAME")?;

    // This is a variable set by Cargo; it *usually* points to `$TOP_LEVEL_CRATE/target/..`
    // It doesn't when the user sets `build.target-dir` in .cargo/config (and maybe also when doing
    // `cargo install`?), but we are going to assume that such setting has not been set
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    // Extract `$TOP_LEVEL_CRATE` from `out_dir`
    let mut path = out_dir.clone();
    while path.file_name().and_then(|os| os.to_str()) != Some("target") {
        path.pop();
    }

    // at this point `path` should be `$TOP_LEVEL_CRATE/target`
    path.pop();
    path.push(".env");

    let dotenv = path;

    for (i, line) in fs::read_to_string(&dotenv)?.lines().enumerate() {
        if line.starts_with("#") {
            // this is a comment; ignore
            continue;
        }

        // NOTE poor man's `try { .. }` block
        let (krate, key, value) = (|| -> Option<_> {
            // Syntax: $crate:$key=$value
            let mut parts = line.splitn(2, ':');

            let krate = parts.next()?;
            let key_value = parts.next()?;

            let mut parts = key_value.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next()?;

            Some((krate, key, value))
        })()
        .ok_or(ParseError { line: i + 1 })?;

        // Is this for us?
        if krate == pkg_name {
            // XXX Maybe prefix the key with `ctenv` to avoid collisions with other build artifacts?
            fs::write(out_dir.join(key), value)?;
        }
    }

    // needed to make Cargo rebuild the dependency if the top level .env changes
    println!("cargo:rerun-if-changed={}", dotenv.display());

    Ok(())
}

#[derive(Debug)]
struct ParseError {
    line: usize,
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parse error at line {}", self.line)
    }
}
