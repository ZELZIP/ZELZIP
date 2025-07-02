use crate::Platform;
use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use thiserror::Error;

type Aes128Ctr64LE = ctr::Ctr128BE<aes::Aes128>;

// Sorry.
const THE_3DS_AES_KEY_REGION_00_AND_09: &[u8] =
    include_bytes!("v2/3ds_aes_key_region_00_and_09.bin");
const THE_3DS_AES_KEY_REGION_01: &[u8] = include_bytes!("v2/3ds_aes_key_region_01.bin");
const THE_3DS_AES_KEY_REGION_02: &[u8] = include_bytes!("v2/3ds_aes_key_region_02.bin");
const THE_3DS_AES_KEY_REGION_05: &[u8] = include_bytes!("v2/3ds_aes_key_region_05.bin");

const WII_U_AES_KEY_REGION_01: &[u8] = include_bytes!("v2/wii_u_aes_key_region_01.bin");
const WII_U_AES_KEY_REGION_02: &[u8] = include_bytes!("v2/wii_u_aes_key_region_02.bin");
const WII_U_AES_KEY_REGION_03: &[u8] = include_bytes!("v2/wii_u_aes_key_region_03.bin");

const THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_0A: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_00_version_0a.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_0B: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_00_version_0b.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_0C: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_00_version_0c.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_0D: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_00_version_0d.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_0E: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_00_version_0e.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_0F: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_00_version_0f.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_10: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_00_version_10.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_11: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_00_version_11.bin.enc");

const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_0A: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_0a.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_0B: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_0b.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_0C: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_0c.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_0D: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_0d.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_0E: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_0e.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_0F: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_0f.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_1A: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_1a.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_1B: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_1b.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_1C: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_1c.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_1D: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_1d.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_1E: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_1e.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_1F: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_1f.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_2A: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_2a.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_2B: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_2b.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_10: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_10.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_11: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_11.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_12: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_12.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_13: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_13.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_14: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_14.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_15: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_15.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_16: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_16.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_17: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_17.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_18: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_18.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_19: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_19.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_20: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_20.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_21: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_21.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_22: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_22.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_23: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_23.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_24: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_24.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_25: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_25.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_26: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_26.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_27: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_27.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_28: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_28.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_29: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_01_version_29.bin.enc");

const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_0A: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_0a.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_0B: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_0b.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_0C: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_0c.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_0D: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_0d.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_0E: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_0e.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_0F: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_0f.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_1A: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_1a.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_1B: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_1b.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_1C: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_1c.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_1D: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_1d.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_1E: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_1e.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_1F: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_1f.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_2A: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_2a.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_2B: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_2b.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_10: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_10.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_11: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_11.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_12: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_12.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_13: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_13.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_14: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_14.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_15: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_15.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_16: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_16.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_17: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_17.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_18: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_18.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_19: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_19.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_20: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_20.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_21: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_21.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_22: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_22.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_23: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_23.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_24: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_24.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_25: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_25.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_26: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_26.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_27: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_27.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_28: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_28.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_29: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_02_version_29.bin.enc");

const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_1A: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_1a.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_1B: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_1b.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_1C: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_1c.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_1D: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_1d.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_1E: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_1e.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_1F: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_1f.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_2A: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_2a.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_12: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_12.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_13: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_13.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_14: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_14.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_15: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_15.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_16: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_16.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_17: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_17.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_18: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_18.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_19: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_19.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_20: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_20.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_21: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_21.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_22: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_22.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_23: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_23.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_24: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_24.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_25: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_25.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_26: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_26.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_27: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_27.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_28: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_28.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_29: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_05_version_29.bin.enc");

const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_1A: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_1a.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_1B: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_1b.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_1C: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_1c.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_1D: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_1d.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_1E: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_1e.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_1F: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_1f.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_2A: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_2a.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_2B: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_2b.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_12: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_12.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_13: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_13.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_14: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_14.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_15: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_15.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_16: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_16.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_17: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_17.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_18: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_18.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_19: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_19.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_20: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_20.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_21: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_21.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_22: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_22.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_23: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_23.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_24: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_24.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_25: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_25.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_26: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_26.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_27: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_27.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_28: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_28.bin.enc");
const THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_29: &[u8] =
    include_bytes!("v2/3ds_hmac_key_region_09_version_29.bin.enc");

const WII_U_HMAC_KEY_ENC_REGION_01: &[u8] = include_bytes!("v2/wii_u_hmac_key_region_01.bin.enc");
const WII_U_HMAC_KEY_ENC_REGION_02: &[u8] = include_bytes!("v2/wii_u_hmac_key_region_02.bin.enc");
const WII_U_HMAC_KEY_ENC_REGION_03: &[u8] = include_bytes!("v2/wii_u_hmac_key_region_03.bin.enc");

#[derive(Error, Debug)]
pub enum V2Error {
    #[error("Unknown region encoded inside the inquiry number: {0}")]
    UnknownRegion(u64),

    #[error("Unknown region encoded inside the inquiry number: ({0}, {1})")]
    UnknownRegionOrVersion(u64, u64),
}

pub fn calculate_v2_master_key(
    platform: Platform,
    inquiry_number: u64,
    day: u8,
    month: u8,
) -> Result<u32, V2Error> {
    let region = inquiry_number / 1_000_000_000;
    let version = (inquiry_number / 10_000_000) % 100;

    let aes_key: &[u8; 16] = match platform {
        Platform::WiiU => match region {
            0x01 => WII_U_AES_KEY_REGION_01,
            0x02 => WII_U_AES_KEY_REGION_02,
            0x03 => WII_U_AES_KEY_REGION_03,

            _ => return Err(V2Error::UnknownRegion(region)),
        },

        Platform::The3ds => match region {
            0x00 | 0x09 => THE_3DS_AES_KEY_REGION_00_AND_09,
            0x01 => THE_3DS_AES_KEY_REGION_01,
            0x02 => THE_3DS_AES_KEY_REGION_02,
            0x05 => THE_3DS_AES_KEY_REGION_05,

            _ => return Err(V2Error::UnknownRegion(region)),
        },

        _ => panic!("The v2 algorithm is only available on the 3DS and the Wii U platforms"),
    }
    .try_into()
    .expect("The v2 AES key is not long enough");

    let hmac_enc = match platform {
        Platform::WiiU => match region {
            0x01 => WII_U_AES_KEY_REGION_01,
            0x02 => WII_U_AES_KEY_REGION_02,
            0x03 => WII_U_AES_KEY_REGION_03,

            _ => return Err(V2Error::UnknownRegion(region)),
        },

        Platform::The3ds => match (region, version) {
            (0x00, 0x0A) => THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_0A,
            (0x00, 0x0B) => THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_0B,
            (0x00, 0x0C) => THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_0C,
            (0x00, 0x0D) => THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_0D,
            (0x00, 0x0E) => THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_0E,
            (0x00, 0x0F) => THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_0F,
            (0x00, 0x10) => THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_10,
            (0x00, 0x11) => THE_3DS_HMAC_KEY_ENC_REGION_00_VERSION_11,

            (0x01, 0x0A) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_0A,
            (0x01, 0x0B) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_0B,
            (0x01, 0x0C) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_0C,
            (0x01, 0x0D) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_0D,
            (0x01, 0x0E) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_0E,
            (0x01, 0x0F) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_0F,
            (0x01, 0x1A) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_1A,
            (0x01, 0x1B) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_1B,
            (0x01, 0x1C) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_1C,
            (0x01, 0x1D) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_1D,
            (0x01, 0x1E) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_1E,
            (0x01, 0x1F) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_1F,
            (0x01, 0x2A) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_2A,
            (0x01, 0x2B) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_2B,
            (0x01, 0x10) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_10,
            (0x01, 0x11) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_11,
            (0x01, 0x12) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_12,
            (0x01, 0x13) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_13,
            (0x01, 0x14) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_14,
            (0x01, 0x15) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_15,
            (0x01, 0x16) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_16,
            (0x01, 0x17) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_17,
            (0x01, 0x18) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_18,
            (0x01, 0x19) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_19,
            (0x01, 0x20) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_20,
            (0x01, 0x21) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_21,
            (0x01, 0x22) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_22,
            (0x01, 0x23) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_23,
            (0x01, 0x24) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_24,
            (0x01, 0x25) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_25,
            (0x01, 0x26) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_26,
            (0x01, 0x27) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_27,
            (0x01, 0x28) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_28,
            (0x01, 0x29) => THE_3DS_HMAC_KEY_ENC_REGION_01_VERSION_29,

            (0x02, 0x0A) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_0A,
            (0x02, 0x0B) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_0B,
            (0x02, 0x0C) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_0C,
            (0x02, 0x0D) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_0D,
            (0x02, 0x0E) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_0E,
            (0x02, 0x0F) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_0F,
            (0x02, 0x1A) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_1A,
            (0x02, 0x1B) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_1B,
            (0x02, 0x1C) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_1C,
            (0x02, 0x1D) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_1D,
            (0x02, 0x1E) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_1E,
            (0x02, 0x1F) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_1F,
            (0x02, 0x2A) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_2A,
            (0x02, 0x2B) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_2B,
            (0x02, 0x10) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_10,
            (0x02, 0x11) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_11,
            (0x02, 0x12) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_12,
            (0x02, 0x13) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_13,
            (0x02, 0x14) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_14,
            (0x02, 0x15) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_15,
            (0x02, 0x16) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_16,
            (0x02, 0x17) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_17,
            (0x02, 0x18) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_18,
            (0x02, 0x19) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_19,
            (0x02, 0x20) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_20,
            (0x02, 0x21) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_21,
            (0x02, 0x22) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_22,
            (0x02, 0x23) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_23,
            (0x02, 0x24) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_24,
            (0x02, 0x25) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_25,
            (0x02, 0x26) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_26,
            (0x02, 0x27) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_27,
            (0x02, 0x28) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_28,
            (0x02, 0x29) => THE_3DS_HMAC_KEY_ENC_REGION_02_VERSION_29,

            (0x05, 0x1A) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_1A,
            (0x05, 0x1B) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_1B,
            (0x05, 0x1C) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_1C,
            (0x05, 0x1D) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_1D,
            (0x05, 0x1E) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_1E,
            (0x05, 0x1F) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_1F,
            (0x05, 0x2A) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_2A,
            (0x05, 0x12) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_12,
            (0x05, 0x13) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_13,
            (0x05, 0x14) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_14,
            (0x05, 0x15) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_15,
            (0x05, 0x16) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_16,
            (0x05, 0x17) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_17,
            (0x05, 0x18) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_18,
            (0x05, 0x19) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_19,
            (0x05, 0x20) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_20,
            (0x05, 0x21) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_21,
            (0x05, 0x22) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_22,
            (0x05, 0x23) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_23,
            (0x05, 0x24) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_24,
            (0x05, 0x25) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_25,
            (0x05, 0x26) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_26,
            (0x05, 0x27) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_27,
            (0x05, 0x28) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_28,
            (0x05, 0x29) => THE_3DS_HMAC_KEY_ENC_REGION_05_VERSION_29,

            (0x09, 0x1A) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_1A,
            (0x09, 0x1B) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_1B,
            (0x09, 0x1C) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_1C,
            (0x09, 0x1D) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_1D,
            (0x09, 0x1E) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_1E,
            (0x09, 0x1F) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_1F,
            (0x09, 0x2A) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_2A,
            (0x09, 0x2B) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_2B,
            (0x09, 0x12) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_12,
            (0x09, 0x13) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_13,
            (0x09, 0x14) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_14,
            (0x09, 0x15) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_15,
            (0x09, 0x16) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_16,
            (0x09, 0x17) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_17,
            (0x09, 0x18) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_18,
            (0x09, 0x19) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_19,
            (0x09, 0x20) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_20,
            (0x09, 0x21) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_21,
            (0x09, 0x22) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_22,
            (0x09, 0x23) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_23,
            (0x09, 0x24) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_24,
            (0x09, 0x25) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_25,
            (0x09, 0x26) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_26,
            (0x09, 0x27) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_27,
            (0x09, 0x28) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_28,
            (0x09, 0x29) => THE_3DS_HMAC_KEY_ENC_REGION_09_VERSION_29,

            _ => return Err(V2Error::UnknownRegionOrVersion(region, version)),
        },
        _ => panic!("The v2 algorithm is only available on the 3DS and the Wii U platforms"),
    };

    //println!("AES KEY: {aes_key:?}");

    #[allow(clippy::expect_used)]
    let aes_counter: &[u8; 16] = hmac_enc[16..32]
        .try_into()
        .expect("The encoded v2 HMAC file is not big enough");

    //println!("AES COUNTER: {aes_counter:?}");

    #[allow(clippy::expect_used)]
    let mut hmac_key: [u8; 32] = hmac_enc[32..64]
        .try_into()
        .expect("The encoded v2 HMAC file is not big enough");

    println!("HMAC KEY (ENC): {hmac_key:x?}");

    let mut aes = Aes128Ctr64LE::new(aes_key.into(), aes_counter.into());
    aes.apply_keystream(&mut hmac_key);

    println!("HMAC KEY (DEC): {hmac_key:x?}");

    Ok(crate::calculate_master_key_shared_v1_and_v2(
        &hmac_key,
        inquiry_number,
        day,
        month,
    ))
}

// NOTE: No tests because making all combinations would be insane
