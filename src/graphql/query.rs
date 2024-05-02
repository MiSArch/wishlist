use std::any::type_name;

use async_graphql::{Context, Error, Object, Result};

use bson::Uuid;
use mongodb::{bson::doc, Collection, Database};
use serde::Deserialize;

use super::model::{user::User, wishlist::Wishlist};
use crate::authorization::authorize_user;

/// Describes GraphQL wishlist queries.
pub struct Query;

#[Object]
impl Query {
    /// Entity resolver for user of specific id.
    #[graphql(entity)]
    async fn user_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of user to retrieve.")] id: Uuid,
    ) -> Result<User> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<User> = db_client.collection::<User>("users");
        query_object(&collection, id).await
    }

    /// Retrieves wishlist of specific id.
    async fn wishlist<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of wishlist to retrieve.")] id: Uuid,
    ) -> Result<Wishlist> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<Wishlist> = db_client.collection::<Wishlist>("wishlists");
        let wishlist = query_object(&collection, id).await?;
        authorize_user(&ctx, Some(wishlist.user._id))?;
        Ok(wishlist)
    }

    /// Entity resolver for wishlist of specific id.
    #[graphql(entity)]
    async fn wishlist_entity_resolver<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(key, desc = "UUID of wishlist to retrieve.")] id: Uuid,
    ) -> Result<Wishlist> {
        let db_client = ctx.data::<Database>()?;
        let collection: Collection<Wishlist> = db_client.collection::<Wishlist>("wishlists");
        let wishlist = query_object(&collection, id).await?;
        authorize_user(&ctx, Some(wishlist.user._id))?;
        Ok(wishlist)
    }
}

/// Shared function to query an object: T from a MongoDB collection of object: T.
///
/// * `connection` - MongoDB database connection.
/// * `id` - UUID of object.
pub async fn query_object<T: for<'a> Deserialize<'a> + Unpin + Send + Sync>(
    collection: &Collection<T>,
    id: Uuid,
) -> Result<T> {
    match collection.find_one(doc! {"_id": id }, None).await {
        Ok(maybe_object) => match maybe_object {
            Some(object) => Ok(object),
            None => {
                let message = format!("{} with UUID: `{}` not found.", type_name::<T>(), id);
                Err(Error::new(message))
            }
        },
        Err(_) => {
            let message = format!("{} with UUID: `{}` not found.", type_name::<T>(), id);
            Err(Error::new(message))
        }
    }
}
