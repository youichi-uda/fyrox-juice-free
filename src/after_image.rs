//! After-image (ghosting) effect for fast-moving objects.
//!
//! Attach to a node. Periodically spawns "ghost" copies at the node's position
//! that fade out and get removed. Works by toggling visibility of pre-placed
//! child "ghost" nodes in a round-robin fashion.

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

/// Message to control the after-image effect.
#[derive(Debug, ScriptMessagePayload)]
pub enum AfterImageMessage {
    /// Enable after-image generation.
    Enable,
    /// Disable after-image generation (existing ghosts continue fading).
    Disable,
}

/// Ghost instance state.
#[derive(Clone, Debug, Default)]
struct GhostState {
    age: f32,
    active: bool,
}

/// After-image effect. Pre-place child nodes as "ghost slots" and this script
/// will cycle through them, placing them at the parent's position and fading them out.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct AfterImage {
    /// Interval between ghost spawns (seconds).
    #[visit(optional)]
    pub spawn_interval: InheritableVariable<f32>,

    /// How long each ghost persists before fully fading.
    #[visit(optional)]
    pub ghost_lifetime: InheritableVariable<f32>,

    /// Starting opacity (0..255).
    #[visit(optional)]
    pub start_opacity: InheritableVariable<u8>,

    /// Ghost tint color (applied with fading opacity).
    #[visit(optional)]
    pub ghost_color: InheritableVariable<Color>,

    /// Whether currently generating ghosts.
    #[visit(optional)]
    pub active: InheritableVariable<bool>,

    #[reflect(hidden)]
    #[visit(skip)]
    spawn_timer: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    ghost_states: Vec<GhostState>,

    #[reflect(hidden)]
    #[visit(skip)]
    next_ghost_index: usize,
}

impl Default for AfterImage {
    fn default() -> Self {
        Self {
            spawn_interval: 0.05.into(),
            ghost_lifetime: 0.3.into(),
            start_opacity: 180.into(),
            ghost_color: Color::from_rgba(200, 200, 255, 255).into(),
            active: true.into(),
            spawn_timer: 0.0,
            ghost_states: Vec::new(),
            next_ghost_index: usize::MAX,
        }
    }
}

impl_component_provider!(AfterImage);
uuid_provider!(AfterImage = "b4c5d6e7-eeee-ffff-0000-778899001122");

impl ScriptTrait for AfterImage {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        context
            .message_dispatcher
            .subscribe_to::<AfterImageMessage>(context.handle);

        // Initialize ghost states for all children.
        let node = &context.scene.graph[context.handle];
        let child_count = node.children().len();
        self.ghost_states = vec![GhostState::default(); child_count];
        self.next_ghost_index = 0;

        // Hide all children initially.
        let children: Vec<_> = node.children().to_vec();
        for &child_handle in &children {
            let child = &mut context.scene.graph[child_handle];
            child.set_visibility(false);
        }

        Ok(())
    }

    fn on_message(
        &mut self,
        message: &mut dyn ScriptMessagePayload,
        _ctx: &mut ScriptMessageContext,
    ) -> GameResult {
        if let Some(msg) = message.downcast_ref::<AfterImageMessage>() {
            match msg {
                AfterImageMessage::Enable => *self.active = true,
                AfterImageMessage::Disable => *self.active = false,
            }
        }
        Ok(())
    }

    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        let dt = context.dt;
        let lifetime = self.ghost_lifetime.max(0.001);

        let node = &context.scene.graph[context.handle];
        let children: Vec<_> = node.children().to_vec();
        let parent_pos = context
            .scene
            .graph
            .try_get(context.handle)
            .map(|n| n.global_position())?;

        if children.is_empty() {
            return Ok(());
        }

        // Ensure ghost_states matches children count.
        if self.ghost_states.len() != children.len() {
            self.ghost_states
                .resize(children.len(), GhostState::default());
        }

        // Spawn new ghosts.
        if *self.active {
            self.spawn_timer += dt;
            if self.spawn_timer >= *self.spawn_interval {
                self.spawn_timer -= *self.spawn_interval;

                let idx = self.next_ghost_index % children.len();
                self.next_ghost_index = idx + 1;

                self.ghost_states[idx] = GhostState {
                    age: 0.0,
                    active: true,
                };

                let child = &mut context.scene.graph[children[idx]];
                child.set_visibility(true);
                child.local_transform_mut().set_position(parent_pos);
            }
        }

        // Update and fade all ghosts.
        for (i, state) in self.ghost_states.iter_mut().enumerate() {
            if !state.active {
                continue;
            }

            state.age += dt;
            let t = (state.age / lifetime).min(1.0);

            if t >= 1.0 {
                state.active = false;
                let child = &mut context.scene.graph[children[i]];
                child.set_visibility(false);
                continue;
            }

            // Fade opacity.
            let opacity = (*self.start_opacity as f32 * (1.0 - t)) as u8;
            let color = Color::from_rgba(
                self.ghost_color.r,
                self.ghost_color.g,
                self.ghost_color.b,
                opacity,
            );

            let child = &mut context.scene.graph[children[i]];
            // Apply scale shrink for visual fade.
            let scale = 1.0 - t * 0.3; // Slight shrink as it fades.
            child
                .local_transform_mut()
                .set_scale(Vector3::new(scale, scale, scale));

            if let Some(sprite) = child.cast_mut::<Sprite>() {
                sprite.set_color(color);
            } else if let Some(rect) = child.cast_mut::<Rectangle>() {
                rect.set_color(color);
            }
        }

        Ok(())
    }
}
