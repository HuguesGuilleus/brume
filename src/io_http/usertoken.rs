//! A user token send to authentificate itself.
//! The token created by the server, and send to the client.
//!
//! Format:
//! - token: `"U0." + base64(creatation_time:u64 right right* hmac)`
//! - right: `id_len:u4 level:u4 id:(id_len)u8`
//! Always in big endian.
//! Hmac is the signature of decoded
//!
//! ```txt
//! token = U0.AAAAAGmkdDgSOBMqJBI0YvIvXJ0Zy9vqDWaolQ71F5Qi38N4U7mgnWe0fH06lQM
//! key = b"Very Secret /// Very Secret /// "
//! now = 1772385336 (seconds since Epoch)
//! UserToken {
//!     level: UserLevel::EditData,
//!     id: 56,
//!     groups: [
//!         (UserLevel::Admin, 42),
//!         (UserLevel::SuperAdmin, 0x1234),
//!         (UserLevel::None, 0),
//!         (UserLevel::None, 0),
//!         (UserLevel::None, 0),
//!         (UserLevel::None, 0),
//!         (UserLevel::None, 0),
//!         (UserLevel::None, 0),
//!         (UserLevel::None, 0),
//!         (UserLevel::None, 0),
//!         (UserLevel::None, 0),
//!         (UserLevel::None, 0),
//!         (UserLevel::None, 0),
//!         (UserLevel::None, 0),
//!         (UserLevel::None, 0),
//!     ],
//! }
//! ```

use crate::*;
use axum::http::StatusCode;
use crypto::mac::Mac;

/// The base64 decoded token length
pub const MAX_TOKEN_LEN: usize = 8 + 8 + UserToken::GROUP_MAX * 4 + 32;

const EXPIRED_DURATION: u64 = 7 * 12 * 3600;

pub fn encode_user_token(user: &UserToken, key: &[u8], now: u64) -> String {
    let mut buff = [0u8; MAX_TOKEN_LEN];

    // Add creation date
    buff[..8].copy_from_slice(&(now.to_be_bytes()));

    // Encode level and id.
    let mut w = 8;
    for (level, id) in user.iter() {
        let len = match id {
            _ if id <= 0xFF => {
                buff[w + 1] = id as u8;
                1u8
            }
            _ if id <= 0xFFFF => {
                buff[w + 1] = (id >> 8) as u8;
                buff[w + 2] = id as u8;
                2u8
            }
            _ if id <= 0xFF_FFFF => {
                buff[w + 1] = (id >> 16) as u8;
                buff[w + 2] = (id >> 8) as u8;
                buff[w + 3] = id as u8;
                3u8
            }
            _ => {
                buff[w + 1] = (id >> 24) as u8;
                buff[w + 2] = (id >> 16) as u8;
                buff[w + 3] = (id >> 8) as u8;
                buff[w + 4] = id as u8;
                4u8
            }
        };
        buff[w] = (len << 4) | (level as u8);
        w += 1 + len as usize;
    }

    // Sign token
    let mut hasher = crypto::hmac::Hmac::new(crypto::sha2::Sha256::new(), key);
    hasher.input(&buff[..w]);
    hasher.raw_result(&mut buff[w..w + 32]);

    // Prefix and encode token body
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let mut out = String::from("U0.");
    URL_SAFE_NO_PAD.encode_string(&buff[..w + 32], &mut out);

    out
}

pub fn decode(token: &str, key: &[u8], now: u64) -> super::Result<UserToken> {
    use base64::Engine;
    // Check and remove prefix
    if !token.starts_with("U0.") {
        return Err(WrapError::http(
            StatusCode::BAD_REQUEST,
            "Invalid token prefix, expected prefix 'U0.'",
        ));
    }
    let token = &token[3..];

    // Decode base64
    let mut data = [0u8; MAX_TOKEN_LEN];
    let len = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode_slice(token, &mut data)
        .map_err(|err| {
            WrapError::http(StatusCode::BAD_REQUEST, "base64 token decoding fail").add_err(err)
        })?;
    let data = &data[..len];
    if data.len() < 8 + 2 + 32 {
        return Err(WrapError::http(
            StatusCode::BAD_REQUEST,
            "The token is too short",
        ));
    }

    // Check expiration
    let creation = u64::from_be_bytes(data[0..8].try_into().unwrap());
    if EXPIRED_DURATION < now - creation {
        return Err(WrapError::http(
            StatusCode::BAD_REQUEST,
            "The token is expired",
        ));
    }

    // Check signature
    let signature_begin = data.len() - 32;
    let mut hasher = crypto::hmac::Hmac::new(crypto::sha2::Sha256::new(), key);
    hasher.input(&data[..signature_begin]);
    let mut processed_signature: [u8; 32] = [0u8; 32];
    hasher.raw_result(&mut processed_signature);
    let token_signature: &[u8] = &data[signature_begin..];
    if token_signature != processed_signature {
        return Err(WrapError::http(
            StatusCode::BAD_REQUEST,
            "The token signature is invalid",
        ));
    }

    // Decode user data
    let data = &data[8..signature_begin];
    let (user_level, user_id, mut data) = decode_one(data)?;

    let mut user = UserToken {
        level: user_level,
        id: user_id,
        groups: [(UserLevel::None, 0); UserToken::GROUP_MAX],
    };

    // Decode group data
    let mut i = 0;
    while data.len() > 0 && i < UserToken::GROUP_MAX {
        let (level, id, rest) = decode_one(data)?;
        user.groups[i] = (level, id);
        data = rest;
        i += 1;
    }

    Ok(user)
}

/// Parse on tuple of level and id.
fn decode_one(data: &[u8]) -> Result<(UserLevel, u32, &[u8])> {
    let first = data[0];
    let level = match first & 0xF {
        0 => UserLevel::None,
        1 => UserLevel::SeeData,
        2 => UserLevel::EditData,
        3 => UserLevel::Admin,
        4 => UserLevel::SuperAdmin,
        _ => {
            return Err(WrapError::http(
                StatusCode::BAD_REQUEST,
                "The token contain value unknown or wrong syntax",
            ));
        }
    };

    let len = first as usize >> 4;
    if 4 < len {
        return Err(WrapError::http(
            StatusCode::BAD_REQUEST,
            "The token contain value unknown or wrong syntax",
        ));
    } else if data.len() < len + 1 {
        return Err(WrapError::http(
            StatusCode::BAD_REQUEST,
            "The token is too short",
        ));
    }

    let data = &data[1..];
    let mut id = 0u32;
    for i in 0..len {
        id <<= 8;
        id += data[i] as u32;
    }

    Ok((level, id, &data[len..]))
}

#[test]
fn test_encoding() {
    let token = "U0.AAAAAGmkdDgSOBMqJBI0YvIvXJ0Zy9vqDWaolQ71F5Qi38N4U7mgnWe0fH06lQM";
    let key = b"Very Secret /// Very Secret /// ";
    let user = UserToken::DEV_EDITOR;

    assert_eq!(user, decode(token, key, 1772385340).unwrap());
    assert_eq!(token, encode_user_token(&user, key, 1772385336));
}
