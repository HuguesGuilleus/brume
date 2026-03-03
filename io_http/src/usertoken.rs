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

    pub fn allow(&self, target_id: u32, target_level: UserLevel) -> bool {
        for (level, id) in self.iter() {
            if id == target_id {
                return target_level <= level;
            }
        }
        false
    }

    fn iter(&self) -> impl Iterator<Item = (UserLevel, u32)> {
        std::iter::once((self.level, self.id))
            .chain(self.groups.into_iter().take_while(|&(_, id)| id != 0))
    }
}

#[test]
fn user_token_allow() {
    let mut user = UserToken::default();
    user.groups[0] = (UserLevel::EditData, 36);
    user.groups[1] = (UserLevel::Admin, 42);

    assert!(user.allow(36, UserLevel::EditData));
    assert!(!user.allow(36, UserLevel::Admin));
    assert!(user.allow(42, UserLevel::EditData));
}
