use async_graphql::SimpleObject;

use super::{super::wishlist::Wishlist, base_connection::BaseConnection};

/// A connection of wishlists.
#[derive(SimpleObject)]
#[graphql(shareable)]
pub struct WishlistConnection {
    /// The resulting entities.
    pub nodes: Vec<Wishlist>,
    /// Whether this connection has a next page.
    pub has_next_page: bool,
    /// The total amount of items in this connection.
    pub total_count: u64,
}

/// Implementation of conversion from `BaseConnection<Wishlist>` to `WishlistConnection`.
///
/// Prevents GraphQL naming conflicts.
impl From<BaseConnection<Wishlist>> for WishlistConnection {
    fn from(value: BaseConnection<Wishlist>) -> Self {
        Self {
            nodes: value.nodes,
            has_next_page: value.has_next_page,
            total_count: value.total_count,
        }
    }
}
