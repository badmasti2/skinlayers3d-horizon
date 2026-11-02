fn load(document: &EntityModelDocument, key: &str) -> core::ops::ControlFlow<Option<Part>> {
    if matches!(key, "minecraft:player#main" | "minecraft:player_slim#main" | "minecraft:player_head#main") {
        core::ops::ControlFlow::Break(skinlayers3d_horizon::model(document, key))
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
