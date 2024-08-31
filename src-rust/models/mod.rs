pub mod bookmarks;
pub mod categories;
pub mod code_delete_account;
pub mod code_reset_password;
pub mod code_validate_email;
pub mod favorites;
pub mod pages;
pub mod progresses;
pub mod session_tokens;
pub mod tags;
pub mod titles;
pub mod titles_tags;
pub mod users;

pub mod types;

pub mod prelude {
    pub use super::bookmarks::Entity as Bookmarks;
    pub use super::categories::Entity as Categories;
    pub use super::code_delete_account::Entity as CodeDeleteAccount;
    pub use super::code_reset_password::Entity as CodeResetPassword;
    pub use super::code_validate_email::Entity as CodeValidateEmail;
    pub use super::favorites::Entity as Favorites;
    pub use super::pages::Entity as Pages;
    pub use super::progresses::Entity as Progresses;
    pub use super::session_tokens::Entity as SessionTokens;
    pub use super::tags::Entity as Tags;
    pub use super::titles::Entity as Titles;
    pub use super::titles_tags::Entity as TitlesTags;
    pub use super::users::Entity as Users;

    pub use super::categories::CategoryID;
    pub use super::pages::PageID;
    pub use super::titles::TitleID;
    pub use super::users::UserID;

    pub use super::types::custom_id::CustomID;

    pub use super::*;
}
