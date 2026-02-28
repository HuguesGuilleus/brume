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
    /// Add some user and promote to admin.
    /// Can add some admin user.
    Admin = 4,
    /// The user can read, write data.
    /// Can add or remove user, and remove the group.
    /// Can degrade some user.
    SuperAdmin = 5,
}

/// A user token send to authentificate itself.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserToken {
    /// Global user level in this server.
    pub level: UserLevel,
    /// The user identifier.
    pub id: u32,
    /// Identifier of the groups and associate level.
    pub groups: [(UserLevel, u32); UserToken::GROUP_MAX],
}

impl UserToken {
    pub const GROUP_MAX: usize = 15;

    pub fn allow_group(&self, group_id: u32, target_level: UserLevel) -> bool {
        for (level, gid) in self.groups {
            if gid == group_id {
                return target_level <= level;
            }
        }
        false
    }
}

#[test]
fn user_token_allow() {
    let mut user = UserToken::default();
    user.groups[1] = (UserLevel::EditData, 36);
    user.groups[2] = (UserLevel::Admin, 42);

    let user = user;
    assert!(user.allow_group(36, UserLevel::EditData));
    assert!(!user.allow_group(36, UserLevel::Admin));
    assert!(user.allow_group(42, UserLevel::EditData));
}
