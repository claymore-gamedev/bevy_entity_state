# bevy_entity_state

A Bevy library to add minimal easy to read and reason about entity finite state machines.

## Why

I come from many years of developing in Godot and other object oriented languages.  I like my state machines the way that I like them.  That being:
an enter hook, an exit hook, an update loop when it's active, and transitions defined inside the state.  While the existing crates for bevy are all functional and well made ([seldom_state](https://github.com/Seldom-SE/seldom_state) and [bevy_fsm](https://github.com/MolecularSadism/bevy_fsm)), they don't work how I want and/or don't mesh with my brain well.

The bevy [```State```](https://docs.rs/bevy/latest/bevy/state/index.html) struct is perfect and works exactly how I want, but only exists for global application state.  This library is my attempt at making an entity state that functions in a similar way with easy to hook into enter, exit, and update methods without adding any additional functionality.

I would still consider myself an amatuer at coding in Rust, especially Bevy, but I am moderately proud of this code and it's general enough to be useful to other people, so here it is.

## Example

```rust
use bevy::prelude::*;
use bevy_entity_state::prelude::*;

/// This macro creates an enum, markers for each variant of the enum, 
/// a bundle containing all the markers, and implements the EntityState\EntityStateMarker
/// traits for the enum and markers respectively.
entity_state!(
    enum_name:States,
    enum_variant_names: [
        Idle,
        Wander,
    ]
);

/// This is a generic component that serves as a proxy for something useful.  
/// Picture it's your own more useful component.
#[derive(Component, Default)]
struct EntityData {
    state_timer: Timer,
}

/// This is a generic plugin that represents an entity.  Do what you will with this
pub struct TestPlugin;
impl Plugin for TestPlugin {
    fn build(&self, app: &mut App) {
        // WARN: This observer is required.  It handles updating the markers 
        // On an entity when its state is changed.
        app.add_observer(change_state::<States>);

        // Setup of idle state
        app.add_observer(on_enter_idle);
        app.add_observer(on_exit_idle);
        app.add_systems(Update, update_idle_state);

        // Setup of wander state
        app.add_observer(on_enter_wander);
        app.add_observer(on_exit_wander);
        app.add_systems(Update, update_wander_state);

        // Startup that spawns a test entity
        app.add_systems(Startup, test);
    }
}

fn test(mut commands: Commands) {

    let entity = commands.spawn(EntityData::default()).id();

    // WARN: The initial state of the entity is configured by emitting an enter event
    // for the initial state.  This also means the first update won't be called until the next time commands
    // are executed, but what can you do?  Life's unfair and I'm willing to make that sacrifice.
    trigger_enter_state::<States, Idle>(entity, &mut commands);
}

// IDLE STATE SYSTEMS.  You could put these in a different plugin or move the functions to a different file

fn on_enter_idle(event: On<EnterState<States, Idle>>, mut query: Query<&mut EntityData>) {
    warn!("{:?} entered idle state", event.entity);
    let Ok(mut entity_data) = query.get_mut(event.entity) else {
        return;
    };
    entity_data.state_timer = Timer::new(Duration::from_secs_f32(5.0), TimerMode::Once);
}

fn on_exit_idle(event: On<ExitState<States, Idle>>) {
    warn!("{:?} exited idle state", event.entity);
}

fn update_idle_state(time: Res<Time>, query: Query<(Entity, &mut EntityData), With<Idle>>, mut commands: Commands) {
    for (entity, mut entity_data) in query {
        entity_data.state_timer.tick(time.delta());
        if !entity_data.state_timer.just_finished() {
            continue;
        }

        // WARN: This is how you trigger a state transition.  You call this function with 4 generic arguments,
        // The marker bundle, the state enum, the current state marker, and the next state marker.
        trigger_change_state::<States, Idle, Wander>(entity, &mut commands);
    }
}

// WANDER STATE SYSTEMS.  You could put these in a different plugin or move the functions to a different file

fn on_enter_wander(event: On<EnterState<States, Wander>>, mut query: Query<&mut EntityData>) {
    warn!("{:?} entered wander state", event.entity);
    let Ok(mut entity_data) = query.get_mut(event.entity) else {
        return;
    };
    entity_data.state_timer = Timer::new(Duration::from_secs_f32(5.0), TimerMode::Once);
}

fn on_exit_wander(event: On<ExitState<States, Wander>>) {
    warn!("{:?} exited wander state", event.entity);
}

fn update_wander_state(time: Res<Time>, query: Query<(Entity, &mut EntityData), With<Wander>>, mut commands: Commands) {
    for (entity, mut entity_data) in query {
        entity_data.state_timer.tick(time.delta());
        if !entity_data.state_timer.just_finished() {
            continue;
        }

        // See previous warning.
        trigger_change_state::<States, Wander, Idle>(entity, &mut commands);
    }
}
```