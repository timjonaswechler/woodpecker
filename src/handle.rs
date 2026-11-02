//! Session-local entity handles and validation against a live Bevy World.
use bevy_ecs::{
    entity::{Entity, EntityGeneration, EntityIndex},
    world::World,
};

include!("handle/value.rs");

impl Handle {
    /// Losslessly encodes a Bevy entity for the current session.
    pub const fn from_entity(entity: Entity) -> Self {
        Self {
            index: entity.index_u32(),
            generation: entity.generation().to_bits(),
        }
    }

    /// Reconstructs an [`Entity`] when the raw index bits are representable.
    ///
    /// This does not check whether the entity is live. Use [`Self::resolve`] for validation against
    /// a particular [`World`].
    pub fn entity(self) -> Option<Entity> {
        let index = EntityIndex::from_raw_u32(self.index)?;
        Some(Entity::from_index_and_generation(
            index,
            EntityGeneration::from_bits(self.generation),
        ))
    }

    /// Resolves and validates this handle against the current [`World`].
    ///
    /// Despawned entities and reused indices with another generation return
    /// [`Error::NotLive`].
    pub fn resolve(self, world: &World) -> Result<Entity, Error> {
        let entity = self.entity().ok_or(Error::InvalidBits(self))?;
        world
            .get_entity(entity)
            .map(|_| entity)
            .map_err(|_| Error::NotLive(self))
    }
}

impl From<Entity> for Handle {
    fn from(value: Entity) -> Self {
        Self::from_entity(value)
    }
}

impl TryFrom<Handle> for Entity {
    type Error = Error;

    fn try_from(value: Handle) -> Result<Self, Self::Error> {
        value.entity().ok_or(Error::InvalidBits(value))
    }
}
