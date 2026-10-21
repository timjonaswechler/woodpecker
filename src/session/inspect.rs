use super::protocol::Diagnostic;
use crate::command::inspect::*;
use bevy::{
    ecs::reflect::{ReflectComponent, ReflectResource},
    prelude::{AppTypeRegistry, World},
};
mod entities;
mod value;

pub(super) fn query(world: &World, query: Command) -> std::result::Result<Output, Diagnostic> {
    match query {
        Command::Entities {
            entity,
            with,
            without,
            projection,
        } => entities::query(world, entity, &with, &without, projection),
        Command::Resources {
            selector,
            projection,
        } => resources(world, selector, projection),
    }
}

fn resources(
    world: &World,
    selector: Selector,
    projection: Projection,
) -> std::result::Result<Output, Diagnostic> {
    let registry = world.resource::<AppTypeRegistry>().read();
    let mut registrations: Vec<_> = match &selector {
        Selector::All {} => registry
            .iter()
            .filter(|r| r.data::<ReflectResource>().is_some())
            .collect(),
        Selector::Type { type_path } => vec![
            registry
                .get_with_type_path(type_path)
                .ok_or_else(|| Diagnostic::new("unknown_type_path", type_path))?,
        ],
    };
    registrations.sort_by_key(|registration| registration.type_info().type_path());
    let mut items = Vec::new();
    for registration in registrations {
        let type_path = registration.type_info().type_path().to_string();
        let entity = world
            .components()
            .get_id(registration.type_id())
            .and_then(|id| world.resource_entities().get(id))
            .and_then(|entity| world.get_entity(entity).ok());
        if entity.is_none()
            && (matches!(projection, Projection::Metadata {})
                || matches!(selector, Selector::All {}))
        {
            continue;
        }
        let result = match projection {
            Projection::Metadata {} => Result::Metadata { type_path },
            Projection::Value {} => {
                let value = match (entity, registration.data::<ReflectComponent>()) {
                    (None, _) => Value::Unavailable {
                        reason: Status::Missing,
                    },
                    (_, None) => Value::Unavailable {
                        reason: Status::NotReflectable,
                    },
                    (Some(entity), Some(component)) => {
                        match component.reflect(entity.into_filtered()) {
                            None => Value::Unavailable {
                                reason: Status::NotReflectable,
                            },
                            Some(reflected) => value::serialize(reflected, &registry),
                        }
                    }
                };
                Result::Value { type_path, value }
            }
        };
        items.push(Item::Resource { result });
    }
    Ok(Output { items })
}
