# Title IDs

The unique 64 bit value of a title.

## Formatting

Title IDs are used all around the library, they are two 32 bit values, an upper value, usually related with the issuer of the title (the game publisher, an internal team, etc) and a lower value, usually acting as the unique identifier of the title.

By default on printing they are formated with their two parts as hex with a dash as a divisor, but on lots of retail titles the lower part is a valid ASCII string. Also some titles IDs are well-known values on specific platforms.

The library allows to access all this different meaning with "display wrappers":

```rust
use zelzip_niiebla::TitleId;

let id = TitleId::new(5350613616540337985)

// By default the safe formatter is used
assert_eq!("4a4132bc-48414741", format!("{title_id}"))

// Using the alternative mode `#` uppercase hex can be enabled.
assert_eq!("4A4132BC-48414741", format!("{title_id:#}"))

// Display the lower half of the ID as text, if the lower half is not valid UTF-8 then the safe formatter will be used as a fallback.
assert_eq!("4a4132bc-HAGA", format!("{}", title_id.display_ascii()))

// The alternative mode `#` is still available
assert_eq!("4A4132BC-HAGA", format!("{:#}", title_id.display_ascii()))

// The output of this function will depend if the title ID is the one associated with a well-known "System title" (BOOT2, IOSXX, BC, etc)
title_id.display_wii_platform()
```
