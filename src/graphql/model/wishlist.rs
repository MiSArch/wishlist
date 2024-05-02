use std::{cmp::Ordering, collections::HashSet};

use async_graphql::{ComplexObject, Result, SimpleObject};
use bson::datetime::DateTime;
use bson::Uuid;
use serde::{Deserialize, Serialize};

use super::{
    connection::product_variant_connection::ProductVariantConnection,
    foreign_types::ProductVariant,
    order_datatypes::{CommonOrderInput, OrderDirection},
    user::User,
};

/// The Wishlist of a user.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, SimpleObject)]
#[graphql(complex)]
pub struct Wishlist {
    /// Wishlist UUID.
    pub _id: Uuid,
    /// User.
    pub user: User,
    /// Name of Wishlist.
    pub name: String,
    /// Timestamp when Wishlist was created.
    pub created_at: DateTime,
    /// Timestamp when Wishlist was last updated.
    pub last_updated_at: DateTime,
    #[graphql(skip)]
    pub internal_product_variants: HashSet<ProductVariant>,
}

#[ComplexObject]
impl Wishlist {
    /// Retrieves product variants.
    async fn product_variants(
        &self,
        #[graphql(desc = "Describes that the `first` N product variants should be retrieved.")]
        first: Option<usize>,
        #[graphql(
            desc = "Describes how many product variants should be skipped at the beginning."
        )]
        skip: Option<usize>,
        #[graphql(desc = "Specifies the order in which product variants are retrieved.")] order_by: Option<
            CommonOrderInput,
        >,
    ) -> Result<ProductVariantConnection> {
        let mut product_variants: Vec<ProductVariant> =
            self.internal_product_variants.clone().into_iter().collect();
        sort_product_variants(&mut product_variants, order_by);
        let total_count = product_variants.len();
        let definitely_skip = skip.unwrap_or(0);
        let definitely_first = first.unwrap_or(usize::MAX);
        let product_variants_part: Vec<ProductVariant> = product_variants
            .into_iter()
            .skip(definitely_skip)
            .take(definitely_first)
            .collect();
        let has_next_page = total_count > product_variants_part.len() + definitely_skip;
        Ok(ProductVariantConnection {
            nodes: product_variants_part,
            has_next_page,
            total_count: total_count as u64,
        })
    }
}

/// Sorts product variants according to BaseOrder.
///
/// * `product_variants` - Product variants to sort.
/// * `order_by` - Specifies order of sorted result.
fn sort_product_variants(
    product_variants: &mut Vec<ProductVariant>,
    order_by: Option<CommonOrderInput>,
) {
    let comparator: fn(&ProductVariant, &ProductVariant) -> bool =
        match order_by.unwrap_or_default().direction.unwrap_or_default() {
            OrderDirection::Asc => {
                |first_product_variant, second_product_variant: &ProductVariant| {
                    first_product_variant < second_product_variant
                }
            }
            OrderDirection::Desc => |first_product_variant, second_product_variant| {
                first_product_variant > second_product_variant
            },
        };
    product_variants.sort_by(|first_product_variant, second_product_variant| {
        match comparator(first_product_variant, second_product_variant) {
            true => Ordering::Less,
            false => Ordering::Greater,
        }
    });
}

impl From<Wishlist> for Uuid {
    fn from(value: Wishlist) -> Self {
        value._id
    }
}
