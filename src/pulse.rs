//! Looping pulse (scale oscillation) effect.
//!
//! Attach to any node for a breathing/pulsing animation. Useful for collectibles,
//! highlighted UI elements, etc.

use crate::easing::EasingFunction;
use fyrox::{
    core::{
        algebra::Vector3,
        impl_component_provider,
        reflect::prelude::*,
        uuid_provider,
        variable::InheritableVariable,
        visitor::prelude::*,
    },
    graph::SceneGraph,
    plugin::error::GameResult,
    script::{ScriptContext, ScriptTrait},
};

/// Continuous scale pulse. Oscillates between `min_scale` and `max_scale`.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct Pulse {
    /// Minimum scale (at the "inhale" trough).
    #[visit(optional)]
    pub min_scale: InheritableVariable<Vector3<f32>>,

    /// Maximum scale (at the "exhale" peak).
    #[visit(optional)]
    pub max_scale: InheritableVariable<Vector3<f32>>,

    /// Full cycle duration in seconds (min -> max -> min).
    #[visit(optional)]
    pub period: InheritableVariable<f32>,

    /// Easing function for the oscillation shape.
    #[visit(optional)]
    pub easing: InheritableVariable<EasingFunction>,

    /// Whether the pulse is currently active.
    #[visit(optional)]
    pub active: InheritableVariable<bool>,

    #[reflect(hidden)]
    #[visit(skip)]
    timer: f32,
}

impl Default for Pulse {
    fn default() -> Self {
        Self {
            min_scale: Vector3::new(0.9, 0.9, 0.9).into(),
            max_scale: Vector3::new(1.1, 1.1, 1.1).into(),
            period: 1.0.into(),
            easing: EasingFunction::EaseInOutSine.into(),
            active: true.into(),
            timer: 0.0,
        }
    }
}

impl_component_provider!(Pulse);
uuid_provider!(Pulse = "d0e1f2a3-aaaa-bbbb-cccc-334455667788");

impl ScriptTrait for Pulse {
    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        if !*self.active {
            return Ok(());
        }

        self.timer += context.dt;
        let period = self.period.max(0.001);

        // Ping-pong: 0->1->0 over one period.
        let raw = (self.timer % period) / period;
        let ping_pong = if raw < 0.5 {
            raw * 2.0
        } else {
            2.0 - raw * 2.0
        };

        let t = self.easing.evaluate(ping_pong);
        let scale = self.min_scale.lerp(&self.max_scale, t);

        let node = context.scene.graph.try_get_node_mut(context.handle)?;
        node.local_transform_mut().set_scale(scale);

        Ok(())
    }
}
