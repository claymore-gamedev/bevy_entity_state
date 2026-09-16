use std::marker::PhantomData;

use bevy_ecs::prelude::*;

pub mod prelude {
    
}

/// Trait that marks an enum as an entity state
pub trait EntityState<B: Bundle>: Component + Clone + Copy {
    /// Add the matching marker for the state
    fn add_marker(&self, entity: Entity, commands: &mut Commands);
}

/// Trait that marks a component as an entity state marker
pub trait EntityStateMarker<B: Bundle, E: Component + EntityState<B>>: Component + Clone + Copy {
    fn get_state() -> E;
}

/// Event that updates markers on entity when it changes state
#[derive(EntityEvent)]
pub struct ChangeState<B, E>
where
    B: Bundle,
    E: Component + EntityState<B>,
{
    _phantom_bundle: PhantomData<B>,

    entity: Entity,
    next_state: Option<E>,
}
impl<B, E> ChangeState<B, E>
where
    B: Bundle,
    E: Component + EntityState<B>,
{
    pub fn new(entity: Entity, next_state: Option<E>) -> Self {
        Self { _phantom_bundle: PhantomData, entity, next_state }
    }
}

/// Event that is triggered when an entity enters a state
#[derive(EntityEvent)]
pub struct EnterState<B, E, M>
where
    B: Bundle,
    E: Component + EntityState<B>,
    M: Component + EntityStateMarker<B, E>,
{
    _phantom_bundle: PhantomData<B>,
    _phantom_state: PhantomData<E>,
    _phantom_marker: PhantomData<M>,

    entity: Entity,
}
impl<B, E, M> EnterState<B, E, M>
where
    B: Bundle,
    E: Component + EntityState<B>,
    M: Component + EntityStateMarker<B, E>,
{
    fn new(entity: Entity) -> Self {
        Self { _phantom_bundle: PhantomData, _phantom_state: PhantomData, _phantom_marker: PhantomData, entity }
    }
}

/// Event that is triggered when an entity exits a state
#[derive(EntityEvent)]
pub struct ExitState<B, E, M>
where
    B: Bundle,
    E: Component + EntityState<B>,
    M: Component + EntityStateMarker<B, E>,
{
    _phantom_bundle: PhantomData<B>,
    _phantom_state: PhantomData<E>,
    _phantom_marker: PhantomData<M>,

    entity: Entity,
}
impl<B, E, M> ExitState<B, E, M>
where
    B: Bundle,
    E: Component + EntityState<B>,
    M: Component + EntityStateMarker<B, E>,
{
    fn new(entity: Entity) -> Self {
        Self { _phantom_bundle: PhantomData, _phantom_state: PhantomData, _phantom_marker: PhantomData, entity }
    }
}

/// Event observer that modifies entities marker components
#[allow(unused)]
pub fn change_state<B, T>(event: On<ChangeState<B, T>>, mut commands: Commands)
where
    B: Bundle,
    T: EntityState<B> + Component,
{
    let entity = event.entity;
    let next_state = event.next_state;
    let entity_commands = &mut commands.entity(entity);
    entity_commands.remove::<B>();

    let Some(next_state) = next_state else {
        return;
    };

    entity_commands.insert(next_state);
    next_state.add_marker(entity, &mut commands);
}

/// Triggers all the events needed for a state transition
#[allow(unused)]
pub fn trigger_change_state<
    B: Bundle,
    E: EntityState<B>,
    PreviousMarker: EntityStateMarker<B, E> + Default,
    NextMarker: EntityStateMarker<B, E> + Default,
>(
    entity: Entity,
    commands: &mut Commands,
) {
    commands.trigger(ChangeState::<B, E>::new(entity, Some(NextMarker::get_state())));
    commands.trigger(ExitState::<B, E, PreviousMarker>::new(entity));
    commands.trigger(EnterState::<B, E, NextMarker>::new(entity));
}

/// Triggers all the events needed for a state enter from no state
#[allow(unused)]
pub fn trigger_enter_state<
    B: Bundle,
    E: Component + EntityState<B>,
    NextMarker: Component + EntityStateMarker<B, E> + Default,
>(
    entity: Entity,
    commands: &mut Commands,
) {
    commands.trigger(ChangeState::<B, E>::new(entity, Some(NextMarker::get_state())));
    commands.trigger(EnterState::<B, E, NextMarker>::new(entity));
}

/// Triggers all the events needed for a state exit
#[allow(unused)]
pub fn trigger_exit_state<
    B: Bundle,
    E: Component + EntityState<B>,
    PreviousMarker: Component + EntityStateMarker<B, E> + Default,
>(
    entity: Entity,
    commands: &mut Commands,
) {
    commands.trigger(ChangeState::<B, E>::new(entity, Some(PreviousMarker::get_state())));
    commands.trigger(ExitState::<B, E, PreviousMarker>::new(entity));
}

/// Macro for defining the states
#[macro_export]
macro_rules! entity_state {
    (
        bundle_name : $bundle_name:ident,
        enum_name : $enum_name:ident,
        enum_variant_names : [$($enum_variant_name:ident),* $(,)?]
    ) => {
        // Create the bundle
        type $bundle_name = ($($enum_variant_name,)*);

        // Create the markers
        $(
            #[derive(Component, Debug, Clone, Copy, Default)]
            pub struct $enum_variant_name;
            impl EntityStateMarker<$bundle_name, $enum_name> for $enum_variant_name {
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
        impl EntityState<$bundle_name> for $enum_name {
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
