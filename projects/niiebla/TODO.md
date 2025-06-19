# TODO (NiiEBLA)

- Make Wii WAD Filesystem Representation (W2FSR)?
  - NOT Implement with custom Serialize, ONLY ALLOW STRICT TYPES OF FORMAT
  - Do it in another crate.
- When all is finished make a README.md
- Add logging support (also tracing?)

## TODO comments firsts
- Ticket V1
- footer
- Backup wads
- Trucha bug
- U8

Content editing supoort on non-Wii consoles:
- Missing handeling off SHA-256
- Edit content grups in TMD

Lax limitations (that can be fixed if there is interest):
- Wii Development titles are not supported.

Limitations (will not be implemented unless a lot of interest is arised and documentation is improved):
- Content viewing and editing only available on Wii titles.
- CRL data on WAD files is not preserved.
- Arbitrary content types is not supported (understading and documenting the meaning of its bitflags would be required).
