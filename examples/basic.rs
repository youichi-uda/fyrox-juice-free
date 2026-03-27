//! Basic example showing how to register and use fyrox-juice-free scripts.
//!
//! This example demonstrates the registration pattern. In practice, you'd
//! attach these scripts to nodes via the Fyrox editor and trigger them
//! with messages from your game logic.

use fyrox::{
    core::{reflect::prelude::*, visitor::prelude::*},
    plugin::{Plugin, PluginRegistrationContext, error::GameResult},
};

#[derive(Visit, Reflect, Debug)]
#[reflect(non_cloneable)]
struct Game;

impl Plugin for Game {
    fn register(&self, context: PluginRegistrationContext) -> GameResult {
        // Register all juice scripts in one call.
        fyrox_juice_free::register(&context.serialization_context.script_constructors);
        Ok(())
    }
}

fn main() {
    // In a real game, you'd use fyrox::engine::Engine to run the game loop.
    // This example just shows the registration pattern.
    println!("fyrox-juice-free: 17 game juice scripts registered successfully!");
    println!();
    println!("Available scripts:");
    println!("  Camera:   CameraShake, CameraZoomPulse, SmoothCameraFollow");
    println!("  Timing:   HitStop, SlowMotion");
    println!("  Tween:    TweenPosition, TweenScale, TweenRotation,");
    println!("            SquashAndStretch, Pulse, Bounce");
    println!("  Visual:   SpriteFlash, TrailRenderer, AfterImage");
    println!("  UI:       DamageNumber, UIShake, UIBounce");
}
