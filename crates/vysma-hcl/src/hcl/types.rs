use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default)]
pub struct HclTags(pub Vec<String>);

#[derive(Component, Debug, Clone)]
pub struct HclPersistent(pub String);

#[derive(Resource, Default)]
pub struct HclPersistStore(pub std::collections::HashMap<String, Transform>); 