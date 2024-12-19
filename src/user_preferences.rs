#[derive(Debug, Clone)]
pub struct UserPreference {
    pub user_id: i32,
    pub dark_mode: bool,
    pub notifications: bool,
}

pub const USER_PREFERENCES_LIST: [UserPreference; 3] = [
    UserPreference {
        user_id: 1,
        dark_mode: true,
        notifications: true,
    },
    UserPreference {
        user_id: 2,
        dark_mode: false,
        notifications: true,
    },
    UserPreference {
        user_id: 3,
        dark_mode: true,
        notifications: false,
    },
];
