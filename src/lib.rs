use std::marker::PhantomData;

use bevy_ecs::prelude::*;

pub mod prelude {
    pub use crate::{
        EnterState, EntityState, EntityStateMarker, ExitState, change_state, entity_state, trigger_change_state,
        trigger_enter_state, trigger_exit_state,
    };
}

/// Trait that marks an enum as an entity state
pub trait EntityState: Component + Clone + Copy {
    type EntityStateBundle: Bundle;

    /// Add the matching marker for the state
    fn add_marker(&self, entity: Entity, commands: &mut Commands);
}

/// Trait that marks a component as an entity state marker
pub trait EntityStateMarker<E: Component + EntityState>: Component + Clone + Copy {
    fn get_state() -> E;
}

/// Event that updates markers on entity when it changes state
#[derive(EntityEvent)]
pub struct ChangeState<E>
where
    E: Component + EntityState,
{
    pub entity: Entity,
    pub next_state: Option<E>,
}
impl<E> ChangeState<E>
where
    E: Component + EntityState,
{
    pub fn new(entity: Entity, next_state: Option<E>) -> Self {
        Self { entity, next_state }
    }
}

/// Event that is triggered when an entity enters a state
#[derive(EntityEvent)]
pub struct EnterState<E, M>
where
    E: Component + EntityState,
    M: Component + EntityStateMarker<E>,
{
    _phantom_state: PhantomData<E>,
    _phantom_marker: PhantomData<M>,

    pub entity: Entity,
}
impl<E, M> EnterState<E, M>
where
    E: Component + EntityState,
    M: Component + EntityStateMarker<E>,
{
    fn new(entity: Entity) -> Self {
        Self { _phantom_state: PhantomData, _phantom_marker: PhantomData, entity }
    }
}

/// Event that is triggered when an entity exits a state
#[derive(EntityEvent)]
pub struct ExitState<E, M>
where
    E: Component + EntityState,
    M: Component + EntityStateMarker<E>,
{
    _phantom_state: PhantomData<E>,
    _phantom_marker: PhantomData<M>,

    pub entity: Entity,
}
impl<E, M> ExitState<E, M>
where
    E: Component + EntityState,
    M: Component + EntityStateMarker<E>,
{
    fn new(entity: Entity) -> Self {
        Self { _phantom_state: PhantomData, _phantom_marker: PhantomData, entity }
    }
}

/// Event observer that modifies entities marker components
#[allow(unused)]
pub fn change_state<E>(event: On<ChangeState<E>>, mut commands: Commands)
where
    E: EntityState + Component,
{
    let entity = event.entity;
    let next_state = event.next_state;
    let entity_commands = &mut commands.entity(entity);
    entity_commands.remove::<E::EntityStateBundle>();

    let Some(next_state) = next_state else {
        return;
    };

    entity_commands.insert(next_state);
    next_state.add_marker(entity, &mut commands);
}

/// Triggers all the events needed for a state transition
/// between two states
#[allow(unused)]
pub fn trigger_change_state<
    E: EntityState,
    PreviousMarker: EntityStateMarker<E> + Default,
    NextMarker: EntityStateMarker<E> + Default,
>(
    entity: Entity,
    commands: &mut Commands,
) {
    commands.trigger(ChangeState::<E>::new(entity, Some(NextMarker::get_state())));
    commands.trigger(ExitState::<E, PreviousMarker>::new(entity));
    commands.trigger(EnterState::<E, NextMarker>::new(entity));
}

/// Triggers all the events needed for a state enter from no state
#[allow(unused)]
pub fn trigger_enter_state<E: Component + EntityState, NextMarker: Component + EntityStateMarker<E> + Default>(
    entity: Entity,
    commands: &mut Commands,
) {
    commands.trigger(ChangeState::<E>::new(entity, Some(NextMarker::get_state())));
    commands.trigger(EnterState::<E, NextMarker>::new(entity));
}

/// Triggers all the events needed for a state exit into no state
#[allow(unused)]
pub fn trigger_exit_state<E: Component + EntityState, PreviousMarker: Component + EntityStateMarker<E> + Default>(
    entity: Entity,
    commands: &mut Commands,
) {
    commands.trigger(ChangeState::<E>::new(entity, Some(PreviousMarker::get_state())));
    commands.trigger(ExitState::<E, PreviousMarker>::new(entity));
}

/// Macro for defining the states
#[macro_export]
macro_rules! entity_state {
    (
        enum_name : $enum_name:ident,
        enum_variant_names : [$($enum_variant_name:ident),* $(,)?]
    ) => {

        // Create the markers
        $(
            #[derive(Component, Debug, Clone, Copy, Default)]
            pub struct $enum_variant_name;
            impl EntityStateMarker<$enum_name> for $enum_variant_name {
                fn get_state() -> $enum_name {
                    $enum_name::$enum_variant_name
                }
            }
        )*

        // Create the enum
        #[derive(Component, Debug, Clone, Copy)]
        pub enum $enum_name {
            $($enum_variant_name),*
        }
        impl EntityState for $enum_name {
            // Create the bundle
            type EntityStateBundle = ($($enum_variant_name,)*);

            fn add_marker(&self, entity: Entity, commands: &mut Commands) {
                let mut entity_commands = commands.entity(entity);
                match self {
                    $(
                        Self::$enum_variant_name => { entity_commands.insert($enum_variant_name); }
                    )*
                }
            }
        }
    };
}
