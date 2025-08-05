# Parental Control Master Key Algorithms

On all Nintendo consoles with a parental control system (Wii, DSi, Wii U, 3DS, Switch, etc) in case the main key is forgotten there is a function called "master key" in which if the "inquiry number" displayed at the screen is sent to the Nintendo Support they will send you back a unique value to disable the restrictions of the system.

This "master key" follows a set of algorithms depending of the device and version of it, multiple third party implementations exists at this time: `IceBrk` ([Rust](https://docs.rs/zelzip_icebrk) and [WASM](https://www.npmjs.com/package/@zelzip.dev/icebrk) library by the [ZELZIP team](https://zelzip.dev/credits)) and [`mkey`](https://github.com/dazjo/mkey) (Python and C implementations by [Dazzozo](https://github.com/Dazzozo), [dazjo](https://github.com/dazjo), [neimod](https://github.com/neimod), WulfyStylez, Shiny Quagsire and [marcan](https://marcan.st/)).

The master key always have a fixed length, in case the result value has fewer digits than it should be the value must be prefixed with the proper number of leading zeroes when displayed to the user.

## Version 0

Available on:

- Wii (all versions)
- DSi (all versions)
- 3DS (from 1.0.0 to 6.3.0)
- Wii U (from 1.0.0 to 4.1.0)

### Invariants

- Master key length: 5 digits.
- Inquiry number length: 10 digits.

### Constants

| Platform    | `Polynomial` | `Addout` |
| ----------- | ------------ | -------- |
| Wii & DSi   | `0x04C11DB7` | `0x14C1` |
| Wii U & 3DS | `0x04C65DB7` | `0x1657` |

### Explanation

1. The `Input` value is calculated by creating an ASCII buffer with the follow format `<MONTH><DAY><INQUIRY>` where `<MONTH>` and `<DAY>` are the console system date with two digits (with a leading zero if less than 10) and `<INQUIRY>` is the inquiry number trimmed over its first four digits (and padded with zeroes if required).

2. A CRC-32 (with an initial value of `0xFFFFFFFF`, `XOROUT` equal to `0xAAAA`, and Normal (non-reversed) `Polynomial` following the constants table above and both the input and output bit-reflected) value is calculated over the "Input" value.

3. Then the `Addout` value is added to the checksum choosing the correct value out of the constants table.

4. The result value of the addition is trimmed over its first five digits, getting the master key.

### Reference implementations

- [ZELZIP IceBrk (Rust)](https://github.com/ZELZIP/ZELZIP/blob/main/projects/icebrk/src/v0.rs).
- mkey:
  - [Python](https://github.com/dazjo/mkey/blob/master/mkey.py#L212-L235).
  - [C](https://github.com/dazjo/mkey/blob/master/source/mkey.c#L211-L241).

## Version 1

Available on:

- 3DS (from 7.0.0 to 7.1.0)

### Invariants

- Master key length: 5 digits.
- Inquiry number length: 10 digits.

### Explanation

1. The `Input` value is calculated by creating an ASCII buffer with the follow format `<MONTH><DAY><INQUIRY>` where `<MONTH>` and `<DAY>` are the console system date with two digits (with a leading zero if less than 10) and `<INQUIRY>` is the inquiry number (padded with zeroes if required).

2. The `Region` is decoded as the most significant digit of the inquiry number.

3. The correct `HMAC Key` is loaded given the `Region` value (Originally stored at the `.rodata` section of the `mset` (System Settings) title main binary at the 3DS NAND).

4. An HMAC-SHA256 (with the previously loaded `HMAC Key`) hash is calculated over the `Input` value, the first four bytes of this hash are interpreted as a little endian 32-bit integer, this integer is then trimmed over its first five digits, getting the master key.

### Reference implementations

- ZELZIP IceBrk (Rust):
  - [Version 1 only code section](https://github.com/ZELZIP/ZELZIP/blob/main/projects/icebrk/src/v1.rs).
  - [Version 1 and 2 shared code section](https://github.com/ZELZIP/ZELZIP/blob/main/projects/icebrk/src/icebrk.rs#L45-L76).
  - [Blobs](https://github.com/ZELZIP/ZELZIP/tree/main/projects/icebrk/src/v1).
- mkey (Nintendo Homebrew Server fork):
  - [Python](https://github.com/nh-server/mkey/blob/master/mkey.py#L245-L369).
  - [C](https://github.com/nh-server/mkey/blob/master/source/mkey.c#L247-L392).
  - [Blobs](https://github.com/nh-server/mkey/tree/master/data).

## Version 2

Available on:

- 3DS (from 7.2.0 to 11.15.0)
- Wii U (from 5.0.0 to 5.5.5)

### Invariants

- Master key length: 5 digits.
- Inquiry number length: 10 digits.

### `masterkey.bin` format

| Offset | Size | Description      |
| ------ | ---- | ---------------- |
| 0      | 1    | `Region`         |
| 1      | 1    | `Version`        |
| 2      | 14   | All zeroes       |
| 16     | 16   | `AES Counter`    |
| 32     | 32   | `HMAC Key (enc)` |

### Explanation

1. The `Input` value is calculated by creating an ASCII buffer with the follow format `<MONTH><DAY><INQUIRY>` where `<MONTH>` and `<DAY>` are the console system date with two digits (with a leading zero if less than 10) and `<INQUIRY>` is the inquiry number (padded with zeroes if required).

2. The `Region` is decoded as the most significant digit of the inquiry number.

3. The `Version` is decoded as the second and third most significant digits of the inquiry number concatenated together (Only on 3DS).

4. The correct `AES Key` and encoded `HMAC data` (aka [masterkey.bin](https://www.3dbrew.org/wiki/CVer#masterkey.bin) file) are loaded given the `Region` (and `Version` if it's on the 3DS).

5. The `AES Counter` and the encrypted `HMAC Key (enc)` are extracted from the `HMAC data` following the previous presented table.

6. The `HMAC Key (enc)` is decrypted with AES-128-CTR (Little endian 64-bit counter) using the `AES Key` and `AES Counter` (padded with 48 zeroes), resulting in the `HMAC Key (dec)`.

7. An HMAC-SHA256 (with the previously decoded `HMAC Key (dec)`) hash is calculated over the `Input` value, the first four bytes of this hash are interpreted as a 32-bit integer (little endian if it's on the 3DS, big endian if it's on the Wii U), this integer is then trimmed over its first five digits, getting the master key.

### Reference implementations

- ZELZIP IceBrk (Rust):
  - [Version 2 only code section](https://github.com/ZELZIP/ZELZIP/blob/main/projects/icebrk/src/v2.rs).
  - [Version 1 and 2 shared code section](https://github.com/ZELZIP/ZELZIP/blob/main/projects/icebrk/src/icebrk.rs#L45-L76).
  - [Blobs](https://github.com/ZELZIP/ZELZIP/tree/main/projects/icebrk/src/v2).
- mkey (Nintendo Homebrew Server fork):
  - [Python](https://github.com/nh-server/mkey/blob/master/mkey.py#L245-L369).
  - [C](https://github.com/nh-server/mkey/blob/master/source/mkey.c#L247-L392).
  - [Blobs](https://github.com/nh-server/mkey/tree/master/data).

## Version 3

Available on:

- Switch (from 1.0.0 to 7.0.1)

### Invariants

- Master key length: 8 digits.
- Inquiry number length: 10 digits.

### Explanation

1. The `Input` value is calculated as the ASCII representation of the inquiry number.

2. The `Version` is decoded as the second and third most significant digits of the inquiry number concatenated together (Only on 3DS).

3. The correct `HMAC Key` is loaded given the `Version` value.

4. An HMAC-SHA256 (with the previously loaded `HMAC Key`) hash is calculated over the `Input` value, the first eight bytes of this hash are interpreted as a 64-bit little endian integer, this integer is then ANDed with the value `0x0000FFFFFFFFFFFF` and trimmed over its first eight digits, getting the master key.

### Reference implementations

- ZELZIP IceBrk (Rust):
  - [Code](https://github.com/ZELZIP/ZELZIP/blob/main/projects/icebrk/src/v3.rs).
  - [Blobs](https://github.com/ZELZIP/ZELZIP/tree/main/projects/icebrk/src/v3).
- mkey (Nintendo Homebrew Server fork):
  - [Python](https://github.com/dazjo/mkey/blob/master/mkey.py#L354-L427).
  - [C](https://github.com/dazjo/mkey/blob/master/source/mkey.c#L394-L503).
  - [Blobs](https://github.com/nh-server/mkey/tree/master/data).

## Version 4

Available on:

- Switch (from 8.0.0 to 14.1.2)

### Invariants

- Master key length: 8 digits.
- Inquiry number length: 10 digits.

### Explanation

This algorithm (and probably all that will come later on) requires the "Device ID" of the console, this value is only accessible by the Nintendo Support or by using homebrew tools.

Access to homebrew applications remove all usefulness of knowing this algorithm as [these same tools can be used to remove the parental control lock](https://gbatemp.net/threads/reset-parental-control-nx-an-easy-to-reset-the-pin-for-controls.556891/).

### Reference implementations

- mkey (Nintendo Homebrew Server fork):
  - [Python](https://github.com/dazjo/mkey/blob/master/mkey.py#L354-L427).
  - [C](https://github.com/dazjo/mkey/blob/master/source/mkey.c#L394-L503).
  - [Blobs](https://github.com/nh-server/mkey/tree/master/data).
