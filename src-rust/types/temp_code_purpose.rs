use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "temp_codes_purpose", rename_all = "snake_case")]
pub enum TempCodePurpose {
    DeleteAccount,
    ResetPassword,
    ValidateEmail,
}
