//! Looping position bounce effect.
//!
//! Attach to any node for a bouncing/hovering animation. Great for floating
//! collectibles, bobbing NPCs, or UI indicators.

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
    script::{ScriptContext, ScriptDeinitContext, ScriptTrait},
};

/// Continuous position bounce/hover animation.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct Bounce {
    /// Offset amplitude on each axis at the peak of the bounce.
    #[visit(optional)]
    pub amplitude: InheritableVariable<Vector3<f32>>,

    /// Full cycle duration in seconds.
    #[visit(optional)]
    pub period: InheritableVariable<f32>,

    /// Easing function for the bounce shape.
    #[visit(optional)]
    pub easing: InheritableVariable<EasingFunction>,

    /// Whether the bounce is currently active.
    #[visit(optional)]
    pub active: InheritableVariable<bool>,

    #[reflect(hidden)]
    #[visit(skip)]
    timer: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    original_position: Option<Vector3<f32>>,
}

impl Default for Bounce {
    fn default() -> Self {
        Self {
            amplitude: Vector3::new(0.0, 0.3, 0.0).into(),
            period: 1.0.into(),
            easing: EasingFunction::EaseInOutSine.into(),
            active: true.into(),
            timer: 0.0,
            original_position: None,
        }
    }
}

impl_component_provider!(Bounce);
uuid_provider!(Bounce = "e1f2a3b4-bbbb-cccc-dddd-445566778899");

impl ScriptTrait for Bounce {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        let node = context.scene.graph.try_get_node_mut(context.handle)?;
        self.original_position = Some(**node.local_transform().position());
        Ok(())
    }

    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        if !*self.active {
            return Ok(());
        }

        let Some(orig) = self.original_position else {
            return Ok(());
        };

        self.timer += context.dt;
        let period = self.period.max(0.001);

        // Ping-pong.
        let raw = (self.timer % period) / period;
        let ping_pong = if raw < 0.5 {
            raw * 2.0
        } else {
            2.0 - raw * 2.0
        };

        let t = self.easing.evaluate(ping_pong);
        let offset = *self.amplitude * t;

        let node = context.scene.graph.try_get_node_mut(context.handle)?;
        node.local_transform_mut().set_position(orig + offset);

        Ok(())
    }

    fn on_deinit(&mut self, context: &mut ScriptDeinitContext) -> GameResult {
        if let Some(orig) = self.original_position {
            if let Ok(node) = context.scene.graph.try_get_node_mut(context.node_handle) {
                node.local_transform_mut().set_position(orig);
            }
        }
        Ok(())
    }
}
