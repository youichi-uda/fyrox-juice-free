//! Camera FOV zoom pulse effect.
//!
//! Attach to a Camera node. Send a [`ZoomPulseMessage::Trigger`] to play a quick
//! FOV punch-in and release.

use crate::easing::EasingFunction;
use fyrox::{
    core::{
        impl_component_provider, reflect::prelude::*, uuid_provider,
        variable::InheritableVariable, visitor::prelude::*,
    },
    plugin::error::GameResult,
    scene::camera::{Camera, Projection},
    script::{ScriptContext, ScriptMessageContext, ScriptMessagePayload, ScriptTrait},
};

/// Message to trigger the zoom pulse.
#[derive(Debug, ScriptMessagePayload)]
pub enum ZoomPulseMessage {
    /// Trigger a zoom pulse with optional override intensity (None = use default).
    Trigger(Option<f32>),
}

/// Quick FOV punch effect for impact feedback. Attach to a Camera node.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct CameraZoomPulse {
    /// How much FOV changes (in radians) at peak. Positive = zoom out, negative = zoom in.
    #[visit(optional)]
    pub fov_delta: InheritableVariable<f32>,

    /// Total duration of the pulse in seconds.
    #[visit(optional)]
    pub duration: InheritableVariable<f32>,

    /// Easing for the attack phase (0 -> peak).
    #[visit(optional)]
    pub attack_easing: InheritableVariable<EasingFunction>,

    /// Easing for the release phase (peak -> 0).
    #[visit(optional)]
    pub release_easing: InheritableVariable<EasingFunction>,

    /// Fraction of duration spent on attack (0.0..1.0). Rest is release.
    #[visit(optional)]
    #[reflect(min_value = 0.0, max_value = 1.0)]
    pub attack_ratio: InheritableVariable<f32>,

    #[reflect(hidden)]
    #[visit(skip)]
    timer: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    active: bool,

    #[reflect(hidden)]
    #[visit(skip)]
    original_fov: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    current_delta: f32,
}

impl Default for CameraZoomPulse {
    fn default() -> Self {
        Self {
            fov_delta: (-0.15).into(),
            duration: 0.2.into(),
            attack_easing: EasingFunction::EaseOutQuad.into(),
            release_easing: EasingFunction::EaseInQuad.into(),
            attack_ratio: 0.3.into(),
            timer: 0.0,
            active: false,
            original_fov: 0.0,
            current_delta: 0.0,
        }
    }
}

impl_component_provider!(CameraZoomPulse);
uuid_provider!(CameraZoomPulse = "b2c3d4e5-2222-3333-4444-bbccddeeff00");

impl ScriptTrait for CameraZoomPulse {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        context
            .message_dispatcher
            .subscribe_to::<ZoomPulseMessage>(context.handle);
        Ok(())
    }

    fn on_message(
        &mut self,
        message: &mut dyn ScriptMessagePayload,
        context: &mut ScriptMessageContext,
    ) -> GameResult {
        if let Some(msg) = message.downcast_ref::<ZoomPulseMessage>() {
            match msg {
                ZoomPulseMessage::Trigger(override_delta) => {
                    if !self.active {
                        let node = &context.scene.graph[context.handle];
                        if let Some(camera) = node.cast::<Camera>() {
                            if let Projection::Perspective(ref persp) = *camera.projection() {
                                self.original_fov = persp.fov;
                            }
                        }
                    }
                    self.current_delta = override_delta.unwrap_or(*self.fov_delta);
                    self.timer = 0.0;
                    self.active = true;
                }
            }
        }
        Ok(())
    }

    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        if !self.active {
            return Ok(());
        }

        self.timer += context.dt;
        let duration = self.duration.max(0.001);
        let t = (self.timer / duration).min(1.0);

        let attack_end = self.attack_ratio.clamp(0.01, 0.99);

        let fov_offset = if t < attack_end {
            // Attack phase.
            let phase_t = t / attack_end;
            self.current_delta * self.attack_easing.evaluate(phase_t)
        } else {
            // Release phase.
            let phase_t = (t - attack_end) / (1.0 - attack_end);
            self.current_delta * (1.0 - self.release_easing.evaluate(phase_t))
        };

        let node = &mut context.scene.graph[context.handle];
        if let Some(camera) = node.cast_mut::<Camera>() {
            if let Projection::Perspective(ref mut persp) = *camera.projection_mut() {
                persp.fov = self.original_fov + fov_offset;
            }
        }

        if t >= 1.0 {
            self.active = false;
            // Restore exact original FOV.
            let node = &mut context.scene.graph[context.handle];
            if let Some(camera) = node.cast_mut::<Camera>() {
                if let Projection::Perspective(ref mut persp) = *camera.projection_mut() {
                    persp.fov = self.original_fov;
                }
            }
        }

        Ok(())
    }
}
