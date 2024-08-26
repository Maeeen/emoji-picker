# emoji-picker-hooker

The `WH_KEYBOARD` hook on Windows requires a DLL to be loaded into the target processes.
To achieve this, a strict requirement should be met: it should be a `no_std` crate: the
Rust standard library (as a `.dll`) is not available in every processes and is generally not
available in the folders where `.dll`s are located.

This introduces a few caveats, with the main one being that it requires the `/.cargo/config.toml` to
abort a panic and not unwind the stack. It works *as-of-now* but it should be investigated.

It also makes use of a `.shared` (`rws`) section in the `.dll`. The `build.rs` only prints the MSVC
linker option.

The name of the crate is to make it obvious for users that will inspect the loaded `.dll`s in their
processes.