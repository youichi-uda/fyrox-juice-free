//! Volume-preserving squash and stretch effect.
//!
//! Attach to a node. Trigger via message on landing, jumping, hitting, etc.
//! The deformation preserves approximate volume (stretch on one axis compresses the others).

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

/// Message to trigger squash/stretch.
#[derive(Debug, ScriptMessagePayload)]
pub enum SquashStretchMessage {
    /// Trigger a squash (e.g. on landing). Intensity 0.0..1.0.
    Squash(f32),
    /// Trigger a stretch (e.g. on jump). Intensity 0.0..1.0.
    Stretch(f32),
}

/// Volume-preserving squash and stretch. Attach to a sprite or mesh node.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct SquashAndStretch {
    /// Which local axis to squash/stretch along (default: Y up).
    #[visit(optional)]
    pub axis: InheritableVariable<SquashAxis>,

    /// Maximum squash amount (how much the main axis shrinks, e.g. 0.5 = halved).
    #[visit(optional)]
    #[reflect(min_value = 0.0, max_value = 1.0)]
    pub max_squash: InheritableVariable<f32>,

    /// Maximum stretch amount (how much the main axis extends, e.g. 0.5 = 1.5x).
    #[visit(optional)]
    #[reflect(min_value = 0.0, max_value = 2.0)]
    pub max_stretch: InheritableVariable<f32>,

    /// Duration of the effect (seconds).
    #[visit(optional)]
    pub duration: InheritableVariable<f32>,

    /// Easing for the snap-to-peak phase.
    #[visit(optional)]
    pub attack_easing: InheritableVariable<EasingFunction>,

    /// Easing for the return-to-normal phase.
    #[visit(optional)]
    pub release_easing: InheritableVariable<EasingFunction>,

    /// Fraction of duration for the attack phase.
    #[visit(optional)]
    #[reflect(min_value = 0.05, max_value = 0.95)]
    pub attack_ratio: InheritableVariable<f32>,

    #[reflect(hidden)]
    #[visit(skip)]
    timer: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    active: bool,

    #[reflect(hidden)]
    #[visit(skip)]
    current_intensity: f32,

    /// Positive = stretch, negative = squash.
    #[reflect(hidden)]
    #[visit(skip)]
    direction: f32,
}

/// Which axis the squash/stretch operates on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Visit, Reflect, Default)]
pub enum SquashAxis {
    X,
    #[default]
    Y,
    Z,
}

impl Default for SquashAndStretch {
    fn default() -> Self {
        Self {
            axis: SquashAxis::Y.into(),
            max_squash: 0.4.into(),
            max_stretch: 0.3.into(),
            duration: 0.2.into(),
            attack_easing: EasingFunction::EaseOutQuad.into(),
            release_easing: EasingFunction::EaseOutElastic.into(),
            attack_ratio: 0.2.into(),
            timer: 0.0,
            active: false,
            current_intensity: 0.0,
            direction: 0.0,
        }
    }
}

impl_component_provider!(SquashAndStretch);
uuid_provider!(SquashAndStretch = "c9d0e1f2-9999-aaaa-bbbb-223344556677");

impl ScriptTrait for SquashAndStretch {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        context
            .message_dispatcher
            .subscribe_to::<SquashStretchMessage>(context.handle);
        Ok(())
    }

    fn on_message(
        &mut self,
        message: &mut dyn ScriptMessagePayload,
        _ctx: &mut ScriptMessageContext,
    ) -> GameResult {
        if let Some(msg) = message.downcast_ref::<SquashStretchMessage>() {
            match msg {
                SquashStretchMessage::Squash(intensity) => {
                    self.direction = -1.0;
                    self.current_intensity = intensity.clamp(0.0, 1.0);
                    self.timer = 0.0;
                    self.active = true;
                }
                SquashStretchMessage::Stretch(intensity) => {
                    self.direction = 1.0;
                    self.current_intensity = intensity.clamp(0.0, 1.0);
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

        let attack_end = self.attack_ratio.clamp(0.05, 0.95);

        let envelope = if t < attack_end {
            let phase_t = t / attack_end;
            self.attack_easing.evaluate(phase_t)
        } else {
            let phase_t = (t - attack_end) / (1.0 - attack_end);
            1.0 - self.release_easing.evaluate(phase_t)
        };

        // Calculate the deformation amount.
        let max_amount = if self.direction > 0.0 {
            *self.max_stretch
        } else {
            *self.max_squash
        };
        let deform = self.direction * max_amount * self.current_intensity * envelope;

        // Volume preservation: if main axis scales by (1+d), others scale by 1/sqrt(1+d).
        let main_scale = 1.0 + deform;
        let cross_scale = if main_scale > 0.01 {
            1.0 / main_scale.sqrt()
        } else {
            1.0
        };

        let scale = match *self.axis {
            SquashAxis::X => Vector3::new(main_scale, cross_scale, cross_scale),
            SquashAxis::Y => Vector3::new(cross_scale, main_scale, cross_scale),
            SquashAxis::Z => Vector3::new(cross_scale, cross_scale, main_scale),
        };

        let node = context.scene.graph.try_get_node_mut(context.handle)?;
        node.local_transform_mut().set_scale(scale);

        if t >= 1.0 {
            self.active = false;
            node.local_transform_mut()
                .set_scale(Vector3::new(1.0, 1.0, 1.0));
        }

        Ok(())
    }
}
