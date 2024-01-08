use std::collections::HashSet;

use async_graphql::{Context, Error, FieldResult, Object};
use bson::Bson;
use mongodb::{
    bson::{doc, DateTime},
    Collection,
};
use uuid::Uuid;

use crate::{
    mutation_input_structs::{AddWishlistInput, UpdateWishlistInput},
    query_root::query_wishlist,
    wishlist::Wishlist,
};

/// Describes GraphQL wishlist mutations.
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Adds a wishlist with a user_id, a list of product_variant_ids and a name.
    ///
    /// * `ctx` - GraphQL context containing DB connection.
    /// * `input` - `AddWishlistInput`.
    ///
    /// Formats UUIDs as hyphenated lowercase Strings.
    async fn add_wishlist<'a>(
        &self,
        ctx: &Context<'a>,
        input: AddWishlistInput,
    ) -> FieldResult<Wishlist> {
        let collection: &Collection<Wishlist> = ctx.data_unchecked::<Collection<Wishlist>>();
        let normalized_product_variant_ids: HashSet<String> = input
            .product_variant_ids
            .iter()
            .map(|id| id.as_hyphenated().to_string())
            .collect();
        let current_timestamp = DateTime::now();
        let stringified_user_id = input.user_id.as_hyphenated().to_string();
        let wishlist = Wishlist {
            _id: Uuid::new_v4().as_hyphenated().to_string(),
            user_id: stringified_user_id,
            product_variant_ids: normalized_product_variant_ids,
            name: input.name,
            created_at: current_timestamp,
            last_updated_at: current_timestamp,
        };
        match collection.insert_one(wishlist, None).await {
            Ok(result) => {
                let stringified_uuid = string_uuid_from_bson(result.inserted_id)?;
                query_wishlist(&collection, &stringified_uuid).await
            }
            Err(_) => Err(Error::new("Adding wishlist failed in MongoDB.")),
        }
    }

    /// Updates name and/or product_variant_ids of a specific wishlist referenced with an id.
    ///
    /// * `ctx` - GraphQL context containing DB connection.
    /// * `input` - `UpdateWishlistInput`.
    ///
    /// Formats UUIDs as hyphenated lowercase Strings.
    async fn update_wishlist<'a>(
        &self,
        ctx: &Context<'a>,
        input: UpdateWishlistInput,
    ) -> FieldResult<Wishlist> {
        let collection: &Collection<Wishlist> = ctx.data_unchecked::<Collection<Wishlist>>();
        let stringified_uuid = input.id.as_hyphenated().to_string();
        let current_timestamp = DateTime::now();
        update_product_variant_ids(&collection, &stringified_uuid, &input, &current_timestamp)
            .await?;
        update_name(&collection, &stringified_uuid, &input, &current_timestamp).await?;
        let wishlist = query_wishlist(&collection, &stringified_uuid).await?;
        Ok(wishlist)
    }

    /// Deletes wishlist of id.
    ///
    /// * `ctx` - GraphQL context containing DB connection.
    /// * `id` - UUID of wishlist to delete.
    async fn delete_wishlist<'a>(
        &self,
        ctx: &Context<'a>,
        #[graphql(desc = "UUID of wishlist to delete.")] id: Uuid,
    ) -> FieldResult<bool> {
        let collection: &Collection<Wishlist> = ctx.data_unchecked::<Collection<Wishlist>>();
        let stringified_uuid = id.as_hyphenated().to_string();
        if let Err(_) = collection
            .delete_one(doc! {"_id": &stringified_uuid }, None)
            .await
        {
            let message = format!(
                "Deleting wishlist of id: `{}` failed in MongoDB.",
                stringified_uuid
            );
            return Err(Error::new(message));
        }
        Ok(true)
    }
}

/// Extracts UUID String from Bson.
///
/// Adding a wishlist returns a String formated UUID in a Bson document. This function helps to extract the UUID.
fn string_uuid_from_bson(bson: Bson) -> FieldResult<String> {
    match bson {
        Bson::String(id) => Ok(id),
        _ => {
            let message = format!(
                "Returned id: `{}` needs to be a String in order to be parsed as a Uuid",
                bson
            );
            Err(Error::new(message))
        }
    }
}

/// Updates product variant ids of a wishlist.
///
/// * `collection` - MongoDB collection to update.
/// * `stringified_uuid` - UUID of wishlist to update.
/// * `input` - `UpdateWishlistInput`.
async fn update_product_variant_ids(
    collection: &Collection<Wishlist>,
    stringified_uuid: &String,
    input: &UpdateWishlistInput,
    current_timestamp: &DateTime,
) -> FieldResult<()> {
    if let Some(definitely_product_variant_ids) = &input.product_variant_ids {
        let normalized_product_variant_ids: Vec<String> = definitely_product_variant_ids
            .iter()
            .map(|id| id.as_hyphenated().to_string())
            .collect();
        if let Err(_) = collection.update_one(doc!{"_id": &stringified_uuid }, doc!{"$set": {"product_variant_ids": normalized_product_variant_ids, "last_updated_at": current_timestamp}}, None).await {
            let message = format!("Updating product_variant_ids of wishlist of id: `{}` failed in MongoDB.", &stringified_uuid);
            return Err(Error::new(message))
        }
    }
    Ok(())
}

/// Updates name of a wishlist.
///
/// * `collection` - MongoDB collection to update.
/// * `stringified_uuid` - UUID of wishlist to update.
/// * `input` - `UpdateWishlistInput`.
async fn update_name(
    collection: &Collection<Wishlist>,
    stringified_uuid: &String,
    input: &UpdateWishlistInput,
    current_timestamp: &DateTime,
) -> FieldResult<()> {
    if let Some(definitely_name) = &input.name {
        let result = collection
            .update_one(
                doc! {"_id": &stringified_uuid },
                doc! {"$set": {"name": definitely_name, "last_updated_at": current_timestamp}},
                None,
            )
            .await;
        if let Err(_) = result {
            let message = format!(
                "Updating name of wishlist of id: `{}` failed in MongoDB.",
                &stringified_uuid
            );
            return Err(Error::new(message));
        }
    }
    Ok(())
}