#![allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Polygon { pub normal: [f32;3], pub vertices: [[f32;5];4] }
#[derive(Clone, Debug)]
pub struct Part { pub name: String, pub pose: [f32;9], pub polygons: Vec<Polygon>, pub children: Vec<Part> }
pub struct EntityModelDocument(std::collections::HashMap<String,Part>);
impl Part { fn parse(_: &str, value: &Part) -> Self { value.clone() } }
#[path = "../mixins/skinlayers3d_horizon/model.rs"]
mod skinlayers3d_horizon;

#[test]
fn only_overlay_parts_change_and_poses_are_preserved() {
    let mut layers=Vec::new();
    for name in ["head","hat","body","jacket","left_sleeve","right_sleeve","left_pants","right_pants"] {
        layers.push(Part { name: name.into(), pose: [1.0;9], children: vec![], polygons: vec![Polygon {
            normal: [0.0,0.0,1.0], vertices: [[0.0,0.0,0.0,0.0,0.0],[2.0,0.0,0.0,0.03125,0.0],[2.0,2.0,0.0,0.03125,0.03125],[0.0,2.0,0.0,0.0,0.03125]]
        }] });
    }
    let root=Part { name: "root".into(), pose: [0.0;9], children: layers, polygons: vec![] };
    for key in ["minecraft:player#main","minecraft:player_slim#main","minecraft:player_head#main","minecraft:player#helmet","minecraft:zombie#main"] {
        let document=EntityModelDocument([(key.into(),root.clone())].into());
        let result=skinlayers3d_horizon::model(&document,key).unwrap();
        for part in &result.children {
            assert_eq!(part.pose,[1.0;9]);
            let expected=if key.ends_with("#main") && key != "minecraft:zombie#main" && part.name!="head" && part.name!="body" { 24 } else { 1 };
            assert_eq!(part.polygons.len(),expected,"{key} {}",part.name);
        }
    }
}

