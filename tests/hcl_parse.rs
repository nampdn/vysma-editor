use bevy_in_app::hcl::schema::SceneDoc;

#[test]
fn parse_minimal_hcl_scene() {
    let src = r#"
        prefab = []
        entity = [
          {
            name = "Root",
            components = {
              Name = "RootEntity",
              Transform = { t = [0, 1, 2] }
            },
            children = []
          }
        ]
    "#;
    let doc: SceneDoc = hcl::from_str(src).expect("parse hcl scene");
    assert_eq!(doc.prefab.len(), 0);
    assert_eq!(doc.entity.len(), 1);
    assert_eq!(doc.entity[0].name.as_deref(), Some("Root"));
}

