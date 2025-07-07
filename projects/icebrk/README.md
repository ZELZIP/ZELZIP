# ZEL.ZIP Icebrk library

WASM implementation of the different algorithms used on Nintendo consoles to generate the parental control master key.

## Getting Started

If you are using:

- Rust, check the [crate documentation](https://docs.rs/zelzip_icebrk).
- JavaScript or TypeScript, check the [typed NPM library documentation](https://wasm.icebrk.docs.zel.zip).

## Limitations

- No support for the Nintendo Switch v4 algorithm as it requires a Device ID value only obtainable using homebrew tools, [these same tools also allows for disabling any sort of parental control](https://gbatemp.net/threads/reset-parental-control-nx-an-easy-to-reset-the-pin-for-controls.556891/) making the support of this version redundant.
