use anchor_lang::prelude::*;

// ====================
// UserErrorCode: Custom Error Messages
// ====================
#[error_code]
pub enum UserErrorCode {
    #[msg("Invalid phase. Phase must be either 1 or 2.")]
    InvalidPhase,

    #[msg("User is not currently in the whitelist.")]
    NotInWhitelist,

    #[msg("User is already in the whitelist.")]
    AlreadyInWhitelist,

    #[msg("This status flag is already set for the user.")]
    FlagAlreadySet,

    #[msg("This status flag is not set for the user.")]
    FlagNotSet,


    #[msg("Unknown flag name provided.")]
    UnknownFlagName,

    #[msg("Invalid permission bit provided.")]
    InvalidPermissionBit,

    #[msg("Permission is already granted to the user.")]
    PermissionAlreadySet,

    #[msg("Permission is not granted to the user.")]
    PermissionNotSet,

    #[msg("Unknown permission name provided.")]
    UnknownPermissionName,

    #[msg("User is already an admin.")]
    AdminAlreadySet,

    #[msg("User is not an admin.")]
    AdminNotSet,
}
