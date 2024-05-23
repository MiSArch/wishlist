use async_graphql::{InputObject, SimpleObject};
use bson::Uuid;
use std::collections::HashSet;

#[derive(SimpleObject, InputObject)]
pub struct CreateWishlistInput {
    /// UUID of user owning the wishlist.
    pub user_id: Uuid,
    /// UUIDs of product variants in wishlist.
    pub product_variant_ids: HashSet<Uuid>,
    /// Wishlist name.
    pub name: String,
}

#[derive(SimpleObject, InputObject)]
pub struct UpdateWishlistInput {
    /// UUID of wishlist to update.
    pub id: Uuid,
    /// product variant UUIDs of wishlist to update
    pub product_variant_ids: Option<HashSet<Uuid>>,
    /// Wishlist name to update
    pub name: Option<String>,
}
