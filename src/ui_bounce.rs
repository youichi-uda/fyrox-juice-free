//! UI element bounce effect.
//!
//! Attach to any node. Triggers a scale bounce (pop-in) animation,
//! great for buttons, notifications, achievements, etc.

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
    script::{ScriptContext, ScriptMessageContext, ScriptMessagePayload, ScriptTrait},
};

/// Message to trigger UI bounce.
#[derive(Debug, ScriptMessagePayload)]
pub enum UIBounceMessage {
    /// Trigger a bounce animation.
    Bounce,
    /// Trigger with custom overshoot intensity.
    BounceWithIntensity(f32),
}

/// UI pop/bounce effect. Scale punches up then settles back.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct UIBounce {
    /// Peak scale multiplier (e.g. 1.3 = 30% bigger at peak).
    #[visit(optional)]
    pub peak_scale: InheritableVariable<f32>,

    /// Duration of the bounce animation (seconds).
    #[visit(optional)]
    pub duration: InheritableVariable<f32>,

    /// Easing for the bounce (EaseOutElastic or EaseOutBack work great).
    #[visit(optional)]
    pub easing: InheritableVariable<EasingFunction>,

    /// Rest scale (what the node returns to after bouncing).
    #[visit(optional)]
    pub rest_scale: InheritableVariable<Vector3<f32>>,

    #[reflect(hidden)]
    #[visit(skip)]
    timer: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    active: bool,

    #[reflect(hidden)]
    #[visit(skip)]
    current_peak: f32,
}

impl Default for UIBounce {
    fn default() -> Self {
        Self {
            peak_scale: 1.3.into(),
            duration: 0.4.into(),
            easing: EasingFunction::EaseOutElastic.into(),
            rest_scale: Vector3::new(1.0, 1.0, 1.0).into(),
            timer: 0.0,
            active: false,
            current_peak: 1.3,
        }
    }
}

impl_component_provider!(UIBounce);
uuid_provider!(UIBounce = "e7f8a9b0-1111-2222-3333-001122334455");

impl ScriptTrait for UIBounce {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        context
            .message_dispatcher
            .subscribe_to::<UIBounceMessage>(context.handle);
        Ok(())
    }

    fn on_message(
        &mut self,
        message: &mut dyn ScriptMessagePayload,
        _ctx: &mut ScriptMessageContext,
    ) -> GameResult {
        if let Some(msg) = message.downcast_ref::<UIBounceMessage>() {
            match msg {
                UIBounceMessage::Bounce => {
                    self.current_peak = *self.peak_scale;
                    self.timer = 0.0;
                    self.active = true;
                }
                UIBounceMessage::BounceWithIntensity(intensity) => {
                    self.current_peak = 1.0 + (*self.peak_scale - 1.0) * intensity;
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

        // Easing goes from 0 to 1, we map it:
        // At t=0: scale = peak_scale (start big)
        // At t=1: scale = rest_scale (settle)
        let eased = self.easing.evaluate(t);
        let scale_factor = self.current_peak + (1.0 - self.current_peak) * eased;

        let scale = *self.rest_scale * scale_factor;
        let node = context.scene.graph.try_get_node_mut(context.handle)?;
        node.local_transform_mut().set_scale(scale);

        if t >= 1.0 {
            self.active = false;
            node.local_transform_mut().set_scale(*self.rest_scale);
        }

        Ok(())
    }
}
