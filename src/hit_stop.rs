//! Hit stop (frame freeze) effect.
//!
//! Attach to any node. Send [`HitStopMessage::Trigger`] to freeze the game for a short duration.
//! Works by setting a very small dt multiplier on scripts that opt in via [`HitStopMessage`].

use fyrox::{
    core::{
        impl_component_provider, reflect::prelude::*, uuid_provider,
        variable::InheritableVariable, visitor::prelude::*,
    },
    plugin::error::GameResult,
    script::{ScriptContext, ScriptMessageContext, ScriptMessagePayload, ScriptTrait},
};

/// Message to control hit stop.
#[derive(Debug, ScriptMessagePayload)]
pub enum HitStopMessage {
    /// Trigger a hit stop for the given duration (seconds).
    Trigger(f32),
    /// Cancel any active hit stop.
    Cancel,
    /// Query: is hit stop currently active? (read `is_active` field directly)
    Query,
}

/// Hit stop effect. Attach to a "game manager" node.
///
/// When triggered, `is_active` becomes true and `time_scale` drops to near zero
/// for the configured duration. Other scripts can check `is_active` and multiply
/// their dt accordingly.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct HitStop {
    /// Default freeze duration if none specified in the message.
    #[visit(optional)]
    pub default_duration: InheritableVariable<f32>,

    /// Time scale during freeze (0.0 = full freeze, 0.01 = near-freeze with slight motion).
    #[visit(optional)]
    #[reflect(min_value = 0.0, max_value = 1.0)]
    pub freeze_time_scale: InheritableVariable<f32>,

    /// Whether hit stop is currently active.
    #[visit(optional)]
    pub is_active: bool,

    /// Current remaining freeze time.
    #[reflect(hidden)]
    #[visit(skip)]
    remaining: f32,
}

impl Default for HitStop {
    fn default() -> Self {
        Self {
            default_duration: 0.07.into(),
            freeze_time_scale: 0.0.into(),
            is_active: false,
            remaining: 0.0,
        }
    }
}

impl_component_provider!(HitStop);
uuid_provider!(HitStop = "d4e5f6a7-4444-5555-6666-ddeeff001122");

impl HitStop {
    /// Returns the effective time scale (1.0 when not active, freeze_time_scale when active).
    pub fn effective_time_scale(&self) -> f32 {
        if self.is_active {
            *self.freeze_time_scale
        } else {
            1.0
        }
    }

    /// Convenience: multiply a dt by the effective time scale.
    pub fn apply_to_dt(&self, dt: f32) -> f32 {
        dt * self.effective_time_scale()
    }
}

impl ScriptTrait for HitStop {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        context
            .message_dispatcher
            .subscribe_to::<HitStopMessage>(context.handle);
        Ok(())
    }

    fn on_message(
        &mut self,
        message: &mut dyn ScriptMessagePayload,
        _ctx: &mut ScriptMessageContext,
    ) -> GameResult {
        if let Some(msg) = message.downcast_ref::<HitStopMessage>() {
            match msg {
                HitStopMessage::Trigger(duration) => {
                    let dur = if *duration <= 0.0 {
                        *self.default_duration
                    } else {
                        *duration
                    };
                    self.remaining = dur;
                    self.is_active = true;
                }
                HitStopMessage::Cancel => {
                    self.remaining = 0.0;
                    self.is_active = false;
                }
                HitStopMessage::Query => { /* Just read is_active */ }
            }
        }
        Ok(())
    }

    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        if self.is_active {
            // Use real dt (not scaled) to count down the freeze.
            self.remaining -= context.dt;
            if self.remaining <= 0.0 {
                self.remaining = 0.0;
                self.is_active = false;
            }
        }
        Ok(())
    }
}
