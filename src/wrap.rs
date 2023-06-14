use crate::identifier::{TreeIdentifier, TreeIdentifierVec};
use crate::{Flattened, TreeItem};

#[must_use]
pub fn wrap<'a>(items: &'a [Flattened<'a>], width: u16) -> Vec<Flattened<'a>> {
    items
        .iter()
        .map(|flattened| Flattened {
            identifier: flattened.identifier.clone(),
            item: flattened.item.clone(),
        })
        .collect::<Vec<_>>()
}
