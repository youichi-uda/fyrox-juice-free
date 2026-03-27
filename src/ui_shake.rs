//! UI element shake effect.
//!
//! Attach to any node used for UI or HUD. Triggers a brief positional shake
//! for feedback (e.g. health bar shake on damage).

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
    script::{ScriptContext, ScriptDeinitContext, ScriptMessageContext, ScriptMessagePayload, ScriptTrait},
};

/// Message to trigger UI shake.
#[derive(Debug, ScriptMessagePayload)]
pub enum UIShakeMessage {
    /// Trigger shake with given intensity (0.0..1.0).
    Shake(f32),
    /// Stop shaking immediately.
    Stop,
}

/// UI element shake. Attach to a UI-space node for feedback on events.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct UIShake {
    /// Maximum pixel/unit offset at full intensity.
    #[visit(optional)]
    pub max_offset: InheritableVariable<f32>,

    /// Duration of the shake in seconds.
    #[visit(optional)]
    pub duration: InheritableVariable<f32>,

    /// Oscillation frequency (shakes per second).
    #[visit(optional)]
    pub frequency: InheritableVariable<f32>,

    /// Decay rate (how fast it calms down).
    #[visit(optional)]
    pub decay: InheritableVariable<f32>,

    #[reflect(hidden)]
    #[visit(skip)]
    timer: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    intensity: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    active: bool,

    #[reflect(hidden)]
    #[visit(skip)]
    original_position: Option<Vector3<f32>>,

    #[reflect(hidden)]
    #[visit(skip)]
    time_accum: f32,
}

impl Default for UIShake {
    fn default() -> Self {
        Self {
            max_offset: 5.0.into(),
            duration: 0.3.into(),
            frequency: 20.0.into(),
            decay: 3.0.into(),
            timer: 0.0,
            intensity: 0.0,
            active: false,
            original_position: None,
            time_accum: 0.0,
        }
    }
}

impl_component_provider!(UIShake);
uuid_provider!(UIShake = "d6e7f8a9-0000-1111-2222-990011223344");

fn ui_noise(seed: u32, x: f32) -> f32 {
    let n = seed.wrapping_add((x * 1000.0) as u32);
    let n = n.wrapping_mul(0x5bd1e995);
    let n = n ^ (n >> 15);
    let n = n.wrapping_mul(0x27d4eb2d);
    (n as f32 / u32::MAX as f32) * 2.0 - 1.0
}

impl ScriptTrait for UIShake {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        context
            .message_dispatcher
            .subscribe_to::<UIShakeMessage>(context.handle);
        let node = context.scene.graph.try_get_node_mut(context.handle)?;
        self.original_position = Some(**node.local_transform().position());
        Ok(())
    }

    fn on_message(
        &mut self,
        message: &mut dyn ScriptMessagePayload,
        _ctx: &mut ScriptMessageContext,
    ) -> GameResult {
        if let Some(msg) = message.downcast_ref::<UIShakeMessage>() {
            match msg {
                UIShakeMessage::Shake(intensity) => {
                    self.intensity = intensity.clamp(0.0, 1.0);
                    self.timer = 0.0;
                    self.time_accum = 0.0;
                    self.active = true;
                }
                UIShakeMessage::Stop => {
                    self.active = false;
                    self.intensity = 0.0;
                }
            }
        }
        Ok(())
    }

    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        if !self.active {
            if let Some(orig) = self.original_position {
                let node = context.scene.graph.try_get_node_mut(context.handle)?;
                node.local_transform_mut().set_position(orig);
            }
            return Ok(());
        }

        self.timer += context.dt;
        self.time_accum += context.dt * *self.frequency;

        let duration = self.duration.max(0.001);
        let t = (self.timer / duration).min(1.0);

        // Envelope: decays over time.
        let envelope = self.intensity * (1.0 - t).powf(*self.decay);

        let offset_x = ui_noise(42, self.time_accum) * *self.max_offset * envelope;
        let offset_y = ui_noise(137, self.time_accum) * *self.max_offset * envelope;

        if let Some(orig) = self.original_position {
            let node = context.scene.graph.try_get_node_mut(context.handle)?;
            node.local_transform_mut()
                .set_position(orig + Vector3::new(offset_x, offset_y, 0.0));
        }

        if t >= 1.0 {
            self.active = false;
            if let Some(orig) = self.original_position {
                let node = context.scene.graph.try_get_node_mut(context.handle)?;
                node.local_transform_mut().set_position(orig);
            }
        }

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
