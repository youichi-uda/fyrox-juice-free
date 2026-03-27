//! Floating damage number popup.
//!
//! Attach to a node. When triggered, the node floats upward and fades out,
//! then disables itself. Ideal for UI overlays or 3D text nodes showing
//! damage, score, or other numeric feedback.

use crate::easing::EasingFunction;
use fyrox::{
    core::{
        algebra::Vector3,
        color::Color,
        impl_component_provider,
        reflect::prelude::*,
        uuid_provider,
        variable::InheritableVariable,
        visitor::prelude::*,
    },
    graph::SceneGraph,
    plugin::error::GameResult,
    scene::{dim2::rectangle::Rectangle, sprite::Sprite},
    script::{ScriptContext, ScriptMessageContext, ScriptMessagePayload, ScriptTrait},
};

/// Message to trigger a damage number popup.
#[derive(Debug, ScriptMessagePayload)]
pub enum DamageNumberMessage {
    /// Show the damage number. The node becomes visible and starts animating.
    Show,
    /// Reset and hide.
    Hide,
}

/// Floating damage/score number effect.
///
/// Attach to a text or sprite node. When triggered, the node:
/// 1. Becomes visible
/// 2. Floats upward (or in `float_direction`)
/// 3. Optionally scales up then down
/// 4. Fades out via color alpha
/// 5. Hides itself when done
#[derive(Visit, Reflect, Clone, Debug)]
pub struct DamageNumber {
    /// Direction and speed of float (units per second).
    #[visit(optional)]
    pub float_velocity: InheritableVariable<Vector3<f32>>,

    /// Total animation duration (seconds).
    #[visit(optional)]
    pub duration: InheritableVariable<f32>,

    /// Initial scale punch (how big it gets at peak).
    #[visit(optional)]
    pub scale_punch: InheritableVariable<f32>,

    /// Easing for the float movement.
    #[visit(optional)]
    pub move_easing: InheritableVariable<EasingFunction>,

    /// Easing for the fade-out (alpha).
    #[visit(optional)]
    pub fade_easing: InheritableVariable<EasingFunction>,

    /// Color of the number at start.
    #[visit(optional)]
    pub color: InheritableVariable<Color>,

    /// If true, hides the node on init (waits for Show message).
    #[visit(optional)]
    pub start_hidden: InheritableVariable<bool>,

    #[reflect(hidden)]
    #[visit(skip)]
    timer: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    active: bool,

    #[reflect(hidden)]
    #[visit(skip)]
    start_position: Vector3<f32>,
}

impl Default for DamageNumber {
    fn default() -> Self {
        Self {
            float_velocity: Vector3::new(0.0, 2.0, 0.0).into(),
            duration: 0.8.into(),
            scale_punch: 1.5.into(),
            move_easing: EasingFunction::EaseOutQuad.into(),
            fade_easing: EasingFunction::EaseInQuad.into(),
            color: Color::from_rgba(255, 50, 50, 255).into(),
            start_hidden: true.into(),
            timer: 0.0,
            active: false,
            start_position: Vector3::zeros(),
        }
    }
}

impl_component_provider!(DamageNumber);
uuid_provider!(DamageNumber = "c5d6e7f8-ffff-0000-1111-889900112233");

impl ScriptTrait for DamageNumber {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        context
            .message_dispatcher
            .subscribe_to::<DamageNumberMessage>(context.handle);
        if *self.start_hidden {
            let node = context.scene.graph.try_get_node_mut(context.handle)?;
            node.set_visibility(false);
        }
        Ok(())
    }

    fn on_message(
        &mut self,
        message: &mut dyn ScriptMessagePayload,
        context: &mut ScriptMessageContext,
    ) -> GameResult {
        if let Some(msg) = message.downcast_ref::<DamageNumberMessage>() {
            match msg {
                DamageNumberMessage::Show => {
                    let node = context.scene.graph.try_get_node_mut(context.handle)?;
                    self.start_position = **node.local_transform().position();
                    node.set_visibility(true);
                    self.timer = 0.0;
                    self.active = true;
                }
                DamageNumberMessage::Hide => {
                    let node = context.scene.graph.try_get_node_mut(context.handle)?;
                    node.set_visibility(false);
                    self.active = false;
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

        // Movement.
        let move_t = self.move_easing.evaluate(t);
        let offset = *self.float_velocity * duration * move_t;
        let pos = self.start_position + offset;

        // Scale: punch then return. Peak at t=0.2.
        let scale_t = if t < 0.2 {
            let phase = t / 0.2;
            1.0 + (*self.scale_punch - 1.0) * phase
        } else {
            let phase = (t - 0.2) / 0.8;
            *self.scale_punch + (1.0 - *self.scale_punch) * phase
        };

        // Fade (start fading at 50% through).
        let fade_t = if t < 0.5 {
            0.0
        } else {
            self.fade_easing.evaluate((t - 0.5) / 0.5)
        };
        let alpha = ((1.0 - fade_t) * 255.0) as u8;

        let node = context.scene.graph.try_get_node_mut(context.handle)?;
        node.local_transform_mut()
            .set_position(pos)
            .set_scale(Vector3::new(scale_t, scale_t, scale_t));

        let color = Color::from_rgba(self.color.r, self.color.g, self.color.b, alpha);
        if let Some(sprite) = node.cast_mut::<Sprite>() {
            sprite.set_color(color);
        } else if let Some(rect) = node.cast_mut::<Rectangle>() {
            rect.set_color(color);
        }

        if t >= 1.0 {
            self.active = false;
            node.set_visibility(false);
            // Reset position for re-use.
            node.local_transform_mut().set_position(self.start_position);
            node.local_transform_mut()
                .set_scale(Vector3::new(1.0, 1.0, 1.0));
        }

        Ok(())
    }
}
