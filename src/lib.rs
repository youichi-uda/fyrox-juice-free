//! # Fyrox Game Juice Pack
//!
//! A collection of ready-to-use game juice scripts for the Fyrox engine.
//! Drag-and-drop effects for screen shake, hit stop, tweening, visual feedback, and more.

use fyrox::script::constructor::ScriptConstructorContainer;

pub mod camera_shake;
pub mod camera_zoom_pulse;
pub mod smooth_camera_follow;
pub mod hit_stop;
pub mod slow_motion;
pub mod tween;
pub mod squash_and_stretch;
pub mod pulse;
pub mod bounce;
pub mod sprite_flash;
pub mod trail_renderer;
pub mod after_image;
pub mod damage_number;
pub mod ui_shake;
pub mod ui_bounce;
pub mod easing;

pub use camera_shake::CameraShake;
pub use camera_zoom_pulse::CameraZoomPulse;
pub use smooth_camera_follow::SmoothCameraFollow;
pub use hit_stop::HitStop;
pub use slow_motion::SlowMotion;
pub use tween::{TweenPosition, TweenScale, TweenRotation};
pub use squash_and_stretch::SquashAndStretch;
pub use pulse::Pulse;
pub use bounce::Bounce;
pub use sprite_flash::SpriteFlash;
pub use trail_renderer::TrailRenderer;
pub use after_image::AfterImage;
pub use damage_number::DamageNumber;
pub use ui_shake::UIShake;
pub use ui_bounce::UIBounce;

/// Registers all game juice scripts in the given constructor container.
///
/// ```rust,no_run
/// # use fyrox::{
/// #     core::visitor::prelude::*, core::reflect::prelude::*,
/// #     plugin::{Plugin, PluginRegistrationContext, error::GameResult},
/// # };
/// #
/// # #[derive(Visit, Reflect, Debug)]
/// # #[reflect(non_cloneable)]
/// # struct Game;
/// #
/// # impl Plugin for Game {
///   fn register(&self, context: PluginRegistrationContext) -> GameResult {
///         fyrox_juice_free::register(&context.serialization_context.script_constructors);
///         Ok(())
///   }
/// # }
/// ```
pub fn register(container: &ScriptConstructorContainer) {
    container.add::<CameraShake>("Juice: Camera Shake");
    container.add::<CameraZoomPulse>("Juice: Camera Zoom Pulse");
    container.add::<SmoothCameraFollow>("Juice: Smooth Camera Follow");
    container.add::<HitStop>("Juice: Hit Stop");
    container.add::<SlowMotion>("Juice: Slow Motion");
    container.add::<TweenPosition>("Juice: Tween Position");
    container.add::<TweenScale>("Juice: Tween Scale");
    container.add::<TweenRotation>("Juice: Tween Rotation");
    container.add::<SquashAndStretch>("Juice: Squash and Stretch");
    container.add::<Pulse>("Juice: Pulse");
    container.add::<Bounce>("Juice: Bounce");
    container.add::<SpriteFlash>("Juice: Sprite Flash");
    container.add::<TrailRenderer>("Juice: Trail Renderer");
    container.add::<AfterImage>("Juice: After Image");
    container.add::<DamageNumber>("Juice: Damage Number");
    container.add::<UIShake>("Juice: UI Shake");
    container.add::<UIBounce>("Juice: UI Bounce");
}
