//! Damage flash / color flash effect.
//!
//! Attach to a Sprite or Rectangle node. Triggers a brief color flash (e.g. white on hit)
//! then returns to the original color.

use crate::easing::EasingFunction;
use fyrox::{
    core::{
        color::Color,
        impl_component_provider,
        pool::Handle,
        reflect::prelude::*,
        uuid_provider,
        variable::InheritableVariable,
        visitor::prelude::*,
    },
    plugin::error::GameResult,
    scene::{dim2::rectangle::Rectangle, graph::Graph, node::Node, sprite::Sprite},
    script::{ScriptContext, ScriptMessageContext, ScriptMessagePayload, ScriptTrait},
};

/// Message to trigger a sprite flash.
#[derive(Debug, ScriptMessagePayload)]
pub enum SpriteFlashMessage {
    /// Flash with the default color.
    Flash,
    /// Flash with a custom color.
    FlashColor(Color),
}

/// Brief color flash effect for damage/hit feedback.
/// Works with Sprite and Rectangle nodes.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct SpriteFlash {
    /// The flash color (default: white).
    #[visit(optional)]
    pub flash_color: InheritableVariable<Color>,

    /// Duration of the flash in seconds.
    #[visit(optional)]
    pub duration: InheritableVariable<f32>,

    /// Easing for the fade-out from flash color back to original.
    #[visit(optional)]
    pub easing: InheritableVariable<EasingFunction>,

    #[reflect(hidden)]
    #[visit(skip)]
    timer: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    active: bool,

    #[reflect(hidden)]
    #[visit(skip)]
    original_color: Color,

    #[reflect(hidden)]
    #[visit(skip)]
    current_flash_color: Color,
}

impl Default for SpriteFlash {
    fn default() -> Self {
        Self {
            flash_color: Color::WHITE.into(),
            duration: 0.15.into(),
            easing: EasingFunction::EaseOutQuad.into(),
            timer: 0.0,
            active: false,
            original_color: Color::WHITE,
            current_flash_color: Color::WHITE,
        }
    }
}

impl_component_provider!(SpriteFlash);
uuid_provider!(SpriteFlash = "f2a3b4c5-cccc-dddd-eeee-556677889900");

fn get_color(graph: &Graph, handle: Handle<Node>) -> Option<Color> {
    let node = &graph[handle];
    if let Some(sprite) = node.cast::<Sprite>() {
        return Some(sprite.color());
    }
    if let Some(rect) = node.cast::<Rectangle>() {
        return Some(rect.color());
    }
    None
}

fn set_color(graph: &mut Graph, handle: Handle<Node>, color: Color) {
    let node = &mut graph[handle];
    if let Some(sprite) = node.cast_mut::<Sprite>() {
        sprite.set_color(color);
    } else if let Some(rect) = node.cast_mut::<Rectangle>() {
        rect.set_color(color);
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::from_rgba(
        (a.r as f32 + (b.r as f32 - a.r as f32) * t) as u8,
        (a.g as f32 + (b.g as f32 - a.g as f32) * t) as u8,
        (a.b as f32 + (b.b as f32 - a.b as f32) * t) as u8,
        (a.a as f32 + (b.a as f32 - a.a as f32) * t) as u8,
    )
}

impl ScriptTrait for SpriteFlash {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        context
            .message_dispatcher
            .subscribe_to::<SpriteFlashMessage>(context.handle);
        if let Some(color) = get_color(&context.scene.graph, context.handle) {
            self.original_color = color;
        }
        Ok(())
    }

    fn on_message(
        &mut self,
        message: &mut dyn ScriptMessagePayload,
        context: &mut ScriptMessageContext,
    ) -> GameResult {
        if let Some(msg) = message.downcast_ref::<SpriteFlashMessage>() {
            if !self.active {
                if let Some(color) = get_color(&context.scene.graph, context.handle) {
                    self.original_color = color;
                }
            }
            self.current_flash_color = match msg {
                SpriteFlashMessage::Flash => *self.flash_color,
                SpriteFlashMessage::FlashColor(c) => *c,
            };
            self.timer = 0.0;
            self.active = true;

            set_color(&mut context.scene.graph, context.handle, self.current_flash_color);
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
        let eased = self.easing.evaluate(t);

        let color = lerp_color(self.current_flash_color, self.original_color, eased);
        set_color(&mut context.scene.graph, context.handle, color);

        if t >= 1.0 {
            self.active = false;
            set_color(&mut context.scene.graph, context.handle, self.original_color);
        }

        Ok(())
    }
}
