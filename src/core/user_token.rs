#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum UserLevel {
    #[default]
    None = 0,
    SeeData = 1,
    EditData = 2,
    SeeUser = 3,
    EditUser = 4,
    Admin = 5,
    SuperAdmin = 6,
}

/// A user token send to authentificate itself.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserToken {
    /// Global user level in this server.
    pub level: UserLevel,
    /// The user identifier.
    pub id: u32,
    /// Identifier of the groups.level-id.
    pub groups: [(UserLevel, u32); UserToken::GROUP_MAX],
}

impl UserToken {
    pub const GROUP_MAX: usize = 15;

    pub fn allowed(&self, _id: u32, _l: UserLevel) -> bool {
        true
    }
}
