use super::*;
use crypto::mac::Mac;
use std::iter;

#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
pub enum UserLevel {
    /// No right.
    #[default]
    None = 0,
    /// The user can read data but not write it.
    SeeData = 1,
    /// Can read and write data and view index of users.
    EditData = 2,
    /// The user can read, write data.
    /// The user can add some user.
    /// The user can promote some user until admin.
    Admin = 3,
    /// The user can read, write data.
    /// Can add or remove user, and remove the group.
    /// Can degrade some user.
    SuperAdmin = 4,
}

/// A user token send to authentificate itself.
/// The token created by the server, and send to the client.
///
/// Format:
/// - token: `"T0." + base64(creatation_time:u64 right right* hmac)`
/// - right: `id_len:u4 level:u4 id:(id_len)u8`
/// Always in big endian.
/// Hmac is the signature of decoded
///
/// ```
/// use brume::{UserLevel, UserToken};
///
/// let token = "T0.AAAAAGmkdDgSOBMqJBI0YvIvXJ0Zy9vqDWaolQ71F5Qi38N4U7mgnWe0fH06lQM";
/// let key = b"Very Secret /// Very Secret /// ";
/// let user = UserToken {
///     level: UserLevel::EditData,
///     id: 56,
///     groups: [
///         (UserLevel::Admin, 42),
///         (UserLevel::SuperAdmin, 0x1234),
///         (UserLevel::None, 0),
///         (UserLevel::None, 0),
///         (UserLevel::None, 0),
///         (UserLevel::None, 0),
///         (UserLevel::None, 0),
///         (UserLevel::None, 0),
///         (UserLevel::None, 0),
///         (UserLevel::None, 0),
///         (UserLevel::None, 0),
///         (UserLevel::None, 0),
///         (UserLevel::None, 0),
///         (UserLevel::None, 0),
///         (UserLevel::None, 0),
///     ],
/// };
///
/// assert_eq!(user, UserToken::decode(token, key, 1772385340).unwrap());
/// assert_eq!(token, &user.encode(key, 1772385336));
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserToken {
    /// Global user level in this server.
    pub level: UserLevel,
    /// The user identifier.
    pub id: u32,
    /// Identifier of the groups and associate level.
    /// Use only started item, stop when the group id is `0`.
    pub groups: [(UserLevel, u32); UserToken::GROUP_MAX],
}

impl UserToken {
    pub const GROUP_MAX: usize = 15;
    /// The base64 decoded token length
    pub const MAX_TOKEN_LEN: usize = 8 + 8 + Self::GROUP_MAX * 4 + 32;

    const EXPIRED_DURATION: u64 = 7 * 12 * 3600;

    pub fn allow(&self, target_id: u32, target_level: UserLevel) -> bool {
        for (level, id) in self.iter() {
            if id == target_id {
                return target_level <= level;
            }
        }
        false
    }

    pub fn iter(&self) -> impl Iterator<Item = (UserLevel, u32)> {
        iter::once((self.level, self.id))
            .chain(self.groups.into_iter().take_while(|&(_, id)| id != 0))
    }

    pub fn encode(&self, key: &[u8], now: u64) -> String {
        let mut buff = [0u8; Self::MAX_TOKEN_LEN];

        // Add creation date
        buff[..8].copy_from_slice(&(now.to_be_bytes()));

        // Encode level and id.
        let mut w = 8;
        for (level, id) in self.iter() {
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
        let mut out = String::from("T0.");
        URL_SAFE_NO_PAD.encode_string(&buff[..w + 32], &mut out);

        out
    }

    pub fn decode(token: &str, key: &[u8], now: u64) -> super::Result<Self> {
        use base64::Engine;
        // Check and remove prefix
        if !token.starts_with("T0.") {
            return Err(errw::TOKEN_PREFIX);
        }
        let token = &token[3..];

        // Decode base64
        let mut data = [0u8; Self::MAX_TOKEN_LEN];
        let len = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode_slice(token, &mut data)
            .map_err(|_| errw::TOKEN_BASE64)?;
        let data = &data[..len];
        if data.len() < 8 + 2 + 32 {
            return Err(errw::TOKEN_TO_SHORT);
        }

        // Check expiration
        let creation = u64::from_be_bytes(data[0..8].try_into().unwrap());
        if Self::EXPIRED_DURATION < now - creation {
            return Err(errw::TOKEN_EXPIRED);
        }

        // Check signature
        let signature_begin = data.len() - 32;
        let mut hasher = crypto::hmac::Hmac::new(crypto::sha2::Sha256::new(), key);
        hasher.input(&data[..signature_begin]);
        let mut processed_signature: [u8; 32] = [0u8; 32];
        hasher.raw_result(&mut processed_signature);
        let token_signature: &[u8] = &data[signature_begin..];
        if token_signature != processed_signature {
            return Err(errw::TOKEN_WRONG_SIGNATURE);
        }

        // Decode user data
        let data = &data[8..signature_begin];
        let (user_level, user_id, mut data) = Self::parse_one(data)?;

        let mut user = Self {
            level: user_level,
            id: user_id,
            groups: [(UserLevel::None, 0); Self::GROUP_MAX],
        };

        // Decode group data
        let mut i = 0;
        while data.len() > 0 && i < Self::GROUP_MAX {
            let (level, id, rest) = Self::parse_one(data)?;
            user.groups[i] = (level, id);
            data = rest;
            i += 1;
        }

        Ok(user)
    }

    /// Parse on tuple of level and id.
    fn parse_one(data: &[u8]) -> Result<(UserLevel, u32, &[u8])> {
        let first = data[0];
        let level = match first & 0xF {
            0 => UserLevel::None,
            1 => UserLevel::SeeData,
            2 => UserLevel::EditData,
            3 => UserLevel::Admin,
            4 => UserLevel::SuperAdmin,
            _ => return Err(errw::TOKEN_WRONG_VALUE),
        };

        let len = first as usize >> 4;
        if 4 < len {
            return Err(errw::TOKEN_WRONG_VALUE);
        } else if data.len() < len + 1 {
            return Err(errw::TOKEN_TO_SHORT);
        }

        let data = &data[1..];
        let mut id = 0u32;
        for i in 0..len {
            id <<= 8;
            id += data[i] as u32;
        }

        Ok((level, id, &data[len..]))
    }
}

#[test]
fn user_token_allow() {
    let mut user = UserToken::default();
    user.groups[0] = (UserLevel::EditData, 36);
    user.groups[1] = (UserLevel::Admin, 42);

    let user = user;
    assert!(user.allow(36, UserLevel::EditData));
    assert!(!user.allow(36, UserLevel::Admin));
    assert!(user.allow(42, UserLevel::EditData));
}

#[test]
fn user_token_iter() {
    assert_eq!(
        vec![
            (UserLevel::EditData, 56),
            (UserLevel::Admin, 42),
            (UserLevel::SuperAdmin, 0x1234),
        ],
        UserToken {
            level: UserLevel::EditData,
            id: 56,
            groups: [
                (UserLevel::Admin, 42),
                (UserLevel::SuperAdmin, 0x1234),
                (UserLevel::None, 0),
                (UserLevel::None, 0),
                (UserLevel::None, 0),
                (UserLevel::None, 0),
                (UserLevel::None, 0),
                (UserLevel::None, 0),
                (UserLevel::None, 0),
                (UserLevel::None, 0),
                (UserLevel::None, 0),
                (UserLevel::None, 0),
                (UserLevel::None, 0),
                (UserLevel::None, 0),
                (UserLevel::None, 0),
            ],
        }
        .iter()
        .collect::<Vec<_>>()
    )
}
