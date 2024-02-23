use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum TokenClaimsPurpose {
    VerifyRegister,
    ResetPassword,
    DeleteAccount,
    None,
}

impl Default for TokenClaimsPurpose {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<TokenClaimsPurpose>,
}
