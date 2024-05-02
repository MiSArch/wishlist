use async_graphql::{ComplexObject, Context, Error, Result, SimpleObject};
use bson::{doc, Document, Uuid};
use mongodb::{options::FindOptions, Collection, Database};
use mongodb_cursor_pagination::{error::CursorError, FindResult, PaginatedCursor};
use serde::{Deserialize, Serialize};

use crate::authorization::authorize_user;

use super::{
    connection::{
        base_connection::{BaseConnection, FindResultWrapper},
        wishlist_connection::WishlistConnection,
    },
    order_types::WishlistOrderInput,
    wishlist::Wishlist,
};

/// Type of a user owning wishlists.
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone, SimpleObject)]
#[graphql(complex)]
pub struct User {
    /// UUID of the user.
    pub _id: Uuid,
}

#[ComplexObject]
impl User {
    /// Retrieves wishlists of user.
    async fn wishlists<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "Describes that the `first` N wishlists should be retrieved.")]
        first: Option<u32>,
        #[graphql(desc = "Describes how many wishlists should be skipped at the beginning.")]
        skip: Option<u64>,
        #[graphql(desc = "Specifies the order in which wishlists are retrieved.")] order_by: Option<
            WishlistOrderInput,
        >,
    ) -> Result<WishlistConnection> {
        authorize_user(&ctx, Some(self._id))?;
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<Wishlist> = db_client.collection::<Wishlist>("wishlists");
        let wishlist_order = order_by.unwrap_or_default();
        let sorting_doc = doc! {wishlist_order.field.unwrap_or_default().as_str(): i32::from(wishlist_order.direction.unwrap_or_default())};
        let find_options = FindOptions::builder()
            .skip(skip)
            .limit(first.map(|first_u32| i64::from(first_u32)))
            .sort(sorting_doc)
            .build();
        let document_collection = collection.clone_with_type::<Document>();
        let filter = doc! {"user._id": self._id};
        let maybe_find_results: Result<FindResult<Wishlist>, CursorError> =
            PaginatedCursor::new(Some(find_options.clone()), None, None)
                .find(&document_collection, Some(&filter))
                .await;
        match maybe_find_results {
            Ok(find_results) => {
                let find_result_wrapper = FindResultWrapper(find_results);
                let connection = Into::<BaseConnection<Wishlist>>::into(find_result_wrapper);
                Ok(Into::<WishlistConnection>::into(connection))
            }
            Err(_) => return Err(Error::new("Retrieving wishlists failed in MongoDB.")),
        }
    }
}
