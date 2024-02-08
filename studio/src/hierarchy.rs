use bevy::{
    prelude::*,
    utils::{EntityHashMap, EntityHashSet},
};

/// Find the parent entity of `target`. Returns `None` if `target` has no
/// parents.
///
/// - Time: Constant
/// - Space: Constant
pub(crate) fn find_parent(target: Entity, q_parents: &Query<Option<&Parent>>) -> Option<Entity> {
    q_parents
        .get(target)
        .ok()
        .and_then(|parent| parent.map(|p| **p))
}

/// Find the nearest common ancestor entity of a set of `entities`. Returns
/// `None` if the provided `entities` do not have a common ancestor.
///
/// - Time: O(DN)
/// - Space: O(N)
///
/// where
/// - D = depth of the most deeply-nested member of `entites`
/// - N = the number of `entities` provided
pub(crate) fn nearest_common_ancestor(
    entities: &[Entity],
    q_parents: &Query<Option<&Parent>>,
) -> Option<Entity> {
    // Find the depth of each entity in the tree
    let mut depths: EntityHashMap<Entity, usize> = EntityHashMap::default();
    for &ent in entities {
        depths.insert(ent, find_depth(ent, q_parents));
    }

    // Find the lowest depth of all the entities
    let Some(lowest_depth) = depths.values().copied().reduce(usize::min) else {
        return None;
    };

    // Traverse up the tree from each entity until we find each one's ancestor
    // at the level of lowest_depth
    let mut equalized: EntityHashSet<Entity> = EntityHashSet::default();
    for ent in entities {
        let mut depth = depths[ent];
        let mut result = *ent;

        while depth > lowest_depth {
            result = find_parent(result, q_parents).unwrap();
            depth -= 1;
        }

        equalized.insert(result);
    }

    // NOTE: If a member of `entities` should be considered a valid result (i.e.
    //       if it should count as its own "ancestor"), then we should check the
    //       length of `equalized` here and return if it only has one member.
    //       This isn't a use case we need currently, but it could be supported
    //       by adding a some kind of mode switch to this function's parameters.

    // Walk up the tree from each of those entities until we find the common ancestor
    let mut parents: EntityHashSet<Entity> = EntityHashSet::default();
    loop {
        for &ent in equalized.iter() {
            if let Some(parent) = find_parent(ent, q_parents) {
                parents.insert(parent);
            } else {
                return None;
            }
        }

        if parents.len() == 1 {
            break parents.iter().copied().next();
        }

        // Reset the loop:
        //   equalized = parents
        //   parents = {}
        equalized.clear();
        equalized.extend(parents.iter());

        parents.clear();
    }
}

fn find_depth(target: Entity, q_parents: &Query<Option<&Parent>>) -> usize {
    let mut result = 0;
    let mut parent = find_parent(target, q_parents);

    while let Some(p) = parent {
        result += 1;
        parent = find_parent(p, q_parents);
    }

    result
}
