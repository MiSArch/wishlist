use async_graphql::{Context, Error, Result};
use axum::http::HeaderMap;
use bson::Uuid;
use serde::Deserialize;

// Authenticate-User HTTP header.
#[derive(Deserialize, Debug)]
pub struct AuthenticateUserHeader {
    id: Uuid,
    roles: Vec<Role>,
}

// Extraction of AuthenticateUserHeader from HeaderMap.
impl TryFrom<&HeaderMap> for AuthenticateUserHeader {
    type Error = Error;

    fn try_from(header_map: &HeaderMap) -> Result<Self, Self::Error> {
        if let Some(authenticate_user_header_value) = header_map.get("Authenticate-User") {
            dbg!(&authenticate_user_header_value);
            if let Ok(authenticate_user_header_str) = authenticate_user_header_value.to_str() {
                dbg!(&authenticate_user_header_str);
                let authenticate_user_header: AuthenticateUserHeader =
                    serde_json::from_str(authenticate_user_header_str)?;
                return Ok(authenticate_user_header);
            }
        }
        Err(Error::new("Authenticate-User header could not be parsed."))
    }
}

// Role of user.
#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum Role {
    User,
    Admin,
    Employee,
}

impl Role {
    // Defines if user has a permissive role.
    fn is_permissive(self) -> bool {
        match self {
            Self::User => false,
            Self::Admin => true,
            Self::Employee => true,
        }
    }
}

// Authenticate user of UUID for a Context.
pub fn authenticate_user(ctx: &Context, id: Uuid) -> Result<()> {
    match ctx.data::<AuthenticateUserHeader>() {
        Ok(authenticate_user_header) => check_permissions(&authenticate_user_header, id),
        Err(_) => Err(Error::new("Authenticate-User header could not be parsed.")),
    }
}

// Check if user of UUID has a valid permission according to the AuthenticateUserHeader.
pub fn check_permissions(
    authenticate_user_header: &AuthenticateUserHeader,
    id: Uuid,
) -> Result<()> {
    if authenticate_user_header
        .roles
        .iter()
        .any(|r| r.is_permissive())
        || authenticate_user_header.id == id
    {
        return Ok(());
    } else {
        let message = format!(
            "Authentication failed for user of UUID: `{}`. Operation not permitted.",
            authenticate_user_header.id
        );
        return Err(Error::new(message));
    }
}
