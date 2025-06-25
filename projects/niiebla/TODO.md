# TODO (NiiEBLA)

## Roadmap

- Port to Python the tooling
- E2E testing

- Wii dev titles support (`is_dev()`, use dev decrypt keys)
- Trucha bug
  - Pararell option (?)
- DSi cryptographic method
- U8
- Backup WADs
  - Save data
- LZ77
- WAD footer
- ASH
- Fix TMD content kind as bitflags
- Footer

- README.md

Content editing upport on non-Wii and non-DSi platforms:

Limitations (will not be implemented unless a lot of interest is arised and documentation is improved):

- Content viewing and editing only available on Wii and DSi titles.
- CRL data on WAD files is not preserved.
- Arbitrary content types is not supported (understading and documenting the meaning of its bitflags would be required).
- Modifying contents on titles with TMD version 1 will not edit its content entry groups hashes.
- "Section total size" and "size of each region" are not checked for correctness when parsing.

Hard limitations (Cannot or are to complex to be fixed):

- 1:1 byte match on the V1 section of a `PreSwitchTicket` is not ensured.
