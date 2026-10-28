use bevy::ecs::entity::Entity;
use woodpecker::{
    command::{Command, inspect},
    handle::{Error, Handle},
};

#[test]
fn handle_encoding_contract() {
    // R0: no Session, App, World, renderer, process or graphical permission.
    let values = [0, 1, 0x7fff_ffff, 0x8000_0000, 0xffff_fffe, u32::MAX];
    for index in values {
        for generation in values {
            let handle = Handle::new(index, generation);
            let json = serde_json::to_value(handle).unwrap();
            assert_eq!(
                json,
                serde_json::json!({"index":index, "generation":generation})
            );
            assert_eq!(json.as_object().unwrap().len(), 2);
            assert_eq!(json["index"].as_u64(), Some(u64::from(index)));
            assert_eq!(json["generation"].as_u64(), Some(u64::from(generation)));
            assert_eq!(serde_json::from_value::<Handle>(json).unwrap(), handle);
            let command = Command::Inspect(inspect::Command::Entities {
                entity: Some(handle),
                with: vec![],
                without: vec![],
                projection: inspect::entity::Projection::Summary {},
            });
            let encoded = serde_json::to_string(&command).unwrap();
            let decoded: Command = serde_json::from_str(&encoded).unwrap();
            assert_eq!(decoded, command);
            let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
            assert_eq!(
                value["arguments"]["entity"],
                serde_json::json!({"index":index,"generation":generation})
            );
            assert_eq!(handle.to_string(), format!("{index}:{generation}"));
            if index == u32::MAX {
                assert!(handle.entity().is_none());
                assert_eq!(
                    Entity::try_from(handle).unwrap_err(),
                    Error::InvalidBits(handle)
                );
            } else {
                let entity = handle.entity().unwrap();
                assert_eq!(entity.index_u32(), index);
                assert_eq!(entity.generation().to_bits(), generation);
                assert_eq!(Handle::from_entity(entity), handle);
                assert_eq!(Entity::try_from(handle).unwrap(), entity);
            }
        }
    }
    for invalid in [
        r#"{"index":4294967296,"generation":0}"#,
        r#"{"index":0,"generation":4294967296}"#,
        r#"{"index":-1,"generation":0}"#,
        r#"{"index":0,"generation":-1}"#,
        r#"{"index":1.5,"generation":0}"#,
        r#"{"index":0,"generation":null}"#,
        r#"{"index":0,"generation":0,"bits":9007199254740993}"#,
        "18446744073709551615",
    ] {
        assert!(
            serde_json::from_str::<Handle>(invalid).is_err(),
            "{invalid}"
        );
    }
}
