# TODO (NiiEBLA)

## Roadmap
- Finish TicketV1 dumping and check if it parses back
- Change `BigEndian` and `LittleEndian` to `BE` and `LE`
- Avoid magic numbers on `size()`
- Trucha bug
    - Pararell option (?)
- Dev titles support (`is_dev()`, use dev decrypt keys)
- Port to Python the tooling
- E2E testing
- Documentation
- Logging

- DSi support
    - `wad.is_tad()`

- U8
- Backup WADs
    - Save data
- LZ77
- WAD footer
- ASH

- README.md

Content editing supoort on non-Wii and non-DSi platforms:
- Missing handeling off SHA-256
- Edit content entry groups in TMDv1

Lax limitations (that can be fixed if there is interest):
- 1:1 byte match on the V1 section of a `PreSwitchTicket` is not ensured, although the resulting data will always be valid, also in most cases a 100% byte match will usually happen.

Limitations (will not be implemented unless a lot of interest is arised and documentation is improved):
- Content viewing and editing only available on Wii titles.
- CRL data on WAD files is not preserved.
- Arbitrary content types is not supported (understading and documenting the meaning of its bitflags would be required).
