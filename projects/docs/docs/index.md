# ZEL.ZIP Official Documentation

[ZEL.ZIP](https://zel.zip) is a group of developers that build the set of utilitary software and libraries known as the "ZEL.ZIP tools". Currently there are the following tools available:

- [NiiEBLA](./niiebla): Rust parsing library for various Nintendo file formats. With accurate support for multiple niche entries and extensions (TikV1, Wii Savegame data, etc).
- [ReNUS](./renus): Rust web API wrapper that makes easy to make requests and download content from the [Nintendo Update Server (NUS)](https://wiibrew.org/wiki/NUS).
- [ViiENTO](./viiento): CLI wrapper around the [NiiEBLA](./niiebla) and [ReNUS](./renus) libraries. Allows packing, unpacking, editing and fakesigning of files and download content from the [NUS](https://wiibrew.org/wiki/NUS) with ease.
- IceBrk: Library that implements the different algorithms used on the Nintendo consoles to generate the parental control master key. Available both on [Rust](https://docs.rs/zelzip_icebrk) and on [JavaScript and TypeScript](https://wasm.icebrk.docs.zel.zip).
