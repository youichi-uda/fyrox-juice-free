//! Tween scripts for position, scale, and rotation.
//!
//! Attach to a node and configure start/end values, duration, and easing.
//! The tween plays automatically on init or can be triggered via message.

use crate::easing::EasingFunction;
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        impl_component_provider,
        reflect::prelude::*,
        uuid_provider,
        variable::InheritableVariable,
        visitor::prelude::*,
    },
    plugin::error::GameResult,
    graph::SceneGraph,
    script::{ScriptContext, ScriptMessageContext, ScriptMessagePayload, ScriptTrait},
};

/// Message to control any tween script.
#[derive(Debug, ScriptMessagePayload)]
pub enum TweenMessage {
    /// Start/restart the tween from the beginning.
    Play,
    /// Pause the tween at its current position.
    Pause,
    /// Resume a paused tween.
    Resume,
    /// Stop and reset to start value.
    Stop,
    /// Play in reverse (end -> start).
    Reverse,
}

/// How the tween behaves when it completes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Visit, Reflect, Default)]
pub enum TweenLoop {
    /// Play once and stop.
    #[default]
    Once,
    /// Loop from start to end repeatedly.
    Loop,
    /// Ping-pong between start and end.
    PingPong,
}

/// Common tween state shared by all tween types.
#[derive(Visit, Reflect, Clone, Debug)]
struct TweenState {
    #[visit(optional)]
    duration: InheritableVariable<f32>,
    #[visit(optional)]
    easing: InheritableVariable<EasingFunction>,
    #[visit(optional)]
    loop_mode: InheritableVariable<TweenLoop>,
    #[visit(optional)]
    auto_play: InheritableVariable<bool>,
    #[visit(optional)]
    delay: InheritableVariable<f32>,

    #[visit(skip)]
    timer: f32,
    #[visit(skip)]
    playing: bool,
    #[visit(skip)]
    reversed: bool,
    #[visit(skip)]
    delay_timer: f32,
}

impl Default for TweenState {
    fn default() -> Self {
        Self {
            duration: 0.5.into(),
            easing: EasingFunction::EaseOutQuad.into(),
            loop_mode: TweenLoop::Once.into(),
            auto_play: true.into(),
            delay: 0.0.into(),
            timer: 0.0,
            playing: false,
            reversed: false,
            delay_timer: 0.0,
        }
    }
}

impl TweenState {
    fn init(&mut self) {
        if *self.auto_play {
            self.playing = true;
            self.timer = 0.0;
            self.delay_timer = 0.0;
        }
    }

    fn handle_message(&mut self, message: &dyn ScriptMessagePayload) {
        if let Some(msg) = message.downcast_ref::<TweenMessage>() {
            match msg {
                TweenMessage::Play => {
                    self.timer = 0.0;
                    self.playing = true;
                    self.reversed = false;
                    self.delay_timer = 0.0;
                }
                TweenMessage::Pause => {
                    self.playing = false;
                }
                TweenMessage::Resume => {
                    self.playing = true;
                }
                TweenMessage::Stop => {
                    self.timer = 0.0;
                    self.playing = false;
                    self.reversed = false;
                }
                TweenMessage::Reverse => {
                    self.timer = 0.0;
                    self.playing = true;
                    self.reversed = true;
                    self.delay_timer = 0.0;
                }
            }
        }
    }

    /// Returns the eased t value (0..1), or None if not playing / still in delay.
    fn tick(&mut self, dt: f32) -> Option<f32> {
        if !self.playing {
            return None;
        }

        // Handle delay.
        if self.delay_timer < *self.delay {
            self.delay_timer += dt;
            return None;
        }

        self.timer += dt;
        let duration = self.duration.max(0.001);
        let mut raw_t = self.timer / duration;

        match *self.loop_mode {
            TweenLoop::Once => {
                if raw_t >= 1.0 {
                    raw_t = 1.0;
                    self.playing = false;
                }
            }
            TweenLoop::Loop => {
                raw_t %= 1.0;
            }
            TweenLoop::PingPong => {
                let cycle = raw_t % 2.0;
                raw_t = if cycle > 1.0 { 2.0 - cycle } else { cycle };
            }
        }

        if self.reversed {
            raw_t = 1.0 - raw_t;
        }

        Some(self.easing.evaluate(raw_t))
    }
}

// ─── TweenPosition ───

/// Tweens a node's local position from `start_position` to `end_position`.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct TweenPosition {
    #[visit(optional)]
    pub start_position: InheritableVariable<Vector3<f32>>,
    #[visit(optional)]
    pub end_position: InheritableVariable<Vector3<f32>>,

    /// Duration in seconds.
    #[visit(optional)]
    pub duration: InheritableVariable<f32>,
    #[visit(optional)]
    pub easing: InheritableVariable<EasingFunction>,
    #[visit(optional)]
    pub loop_mode: InheritableVariable<TweenLoop>,
    /// If true, plays automatically on init.
    #[visit(optional)]
    pub auto_play: InheritableVariable<bool>,
    /// Delay before starting (seconds).
    #[visit(optional)]
    pub delay: InheritableVariable<f32>,

    #[reflect(hidden)]
    #[visit(skip)]
    state: TweenState,
}

impl Default for TweenPosition {
    fn default() -> Self {
        Self {
            start_position: Vector3::zeros().into(),
            end_position: Vector3::new(0.0, 1.0, 0.0).into(),
            duration: 0.5.into(),
            easing: EasingFunction::EaseOutQuad.into(),
            loop_mode: TweenLoop::Once.into(),
            auto_play: true.into(),
            delay: 0.0.into(),
            state: TweenState::default(),
        }
    }
}

impl_component_provider!(TweenPosition);
uuid_provider!(TweenPosition = "f6a7b8c9-6666-7777-8888-ff0011223344");

impl ScriptTrait for TweenPosition {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        self.sync_state();
        self.state.init();
        context
            .message_dispatcher
            .subscribe_to::<TweenMessage>(context.handle);
        Ok(())
    }

    fn on_message(
        &mut self,
        message: &mut dyn ScriptMessagePayload,
        _ctx: &mut ScriptMessageContext,
    ) -> GameResult {
        self.sync_state();
        self.state.handle_message(message);
        Ok(())
    }

    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        if let Some(t) = self.state.tick(context.dt) {
            let pos = self.start_position.lerp(&self.end_position, t);
            let node = context.scene.graph.try_get_node_mut(context.handle)?;
            node.local_transform_mut().set_position(pos);
        }
        Ok(())
    }
}

impl TweenPosition {
    fn sync_state(&mut self) {
        *self.state.duration = *self.duration;
        *self.state.easing = *self.easing;
        *self.state.loop_mode = *self.loop_mode;
        *self.state.auto_play = *self.auto_play;
        *self.state.delay = *self.delay;
    }
}

// ─── TweenScale ───

/// Tweens a node's local scale from `start_scale` to `end_scale`.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct TweenScale {
    #[visit(optional)]
    pub start_scale: InheritableVariable<Vector3<f32>>,
    #[visit(optional)]
    pub end_scale: InheritableVariable<Vector3<f32>>,
    #[visit(optional)]
    pub duration: InheritableVariable<f32>,
    #[visit(optional)]
    pub easing: InheritableVariable<EasingFunction>,
    #[visit(optional)]
    pub loop_mode: InheritableVariable<TweenLoop>,
    #[visit(optional)]
    pub auto_play: InheritableVariable<bool>,
    #[visit(optional)]
    pub delay: InheritableVariable<f32>,

    #[reflect(hidden)]
    #[visit(skip)]
    state: TweenState,
}

impl Default for TweenScale {
    fn default() -> Self {
        Self {
            start_scale: Vector3::new(1.0, 1.0, 1.0).into(),
            end_scale: Vector3::new(1.5, 1.5, 1.5).into(),
            duration: 0.5.into(),
            easing: EasingFunction::EaseOutQuad.into(),
            loop_mode: TweenLoop::Once.into(),
            auto_play: true.into(),
            delay: 0.0.into(),
            state: TweenState::default(),
        }
    }
}

impl_component_provider!(TweenScale);
uuid_provider!(TweenScale = "a7b8c9d0-7777-8888-9999-001122334455");

impl ScriptTrait for TweenScale {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        self.sync_state();
        self.state.init();
        context
            .message_dispatcher
            .subscribe_to::<TweenMessage>(context.handle);
        Ok(())
    }

    fn on_message(
        &mut self,
        message: &mut dyn ScriptMessagePayload,
        _ctx: &mut ScriptMessageContext,
    ) -> GameResult {
        self.sync_state();
        self.state.handle_message(message);
        Ok(())
    }

    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        if let Some(t) = self.state.tick(context.dt) {
            let scale = self.start_scale.lerp(&self.end_scale, t);
            let node = context.scene.graph.try_get_node_mut(context.handle)?;
            node.local_transform_mut().set_scale(scale);
        }
        Ok(())
    }
}

impl TweenScale {
    fn sync_state(&mut self) {
        *self.state.duration = *self.duration;
        *self.state.easing = *self.easing;
        *self.state.loop_mode = *self.loop_mode;
        *self.state.auto_play = *self.auto_play;
        *self.state.delay = *self.delay;
    }
}

// ─── TweenRotation ───

/// Tweens a node's local rotation from `start_angles` to `end_angles` (Euler angles in radians).
#[derive(Visit, Reflect, Clone, Debug)]
pub struct TweenRotation {
    /// Start rotation as Euler angles (roll, pitch, yaw) in radians.
    #[visit(optional)]
    pub start_angles: InheritableVariable<Vector3<f32>>,
    /// End rotation as Euler angles (roll, pitch, yaw) in radians.
    #[visit(optional)]
    pub end_angles: InheritableVariable<Vector3<f32>>,
    #[visit(optional)]
    pub duration: InheritableVariable<f32>,
    #[visit(optional)]
    pub easing: InheritableVariable<EasingFunction>,
    #[visit(optional)]
    pub loop_mode: InheritableVariable<TweenLoop>,
    #[visit(optional)]
    pub auto_play: InheritableVariable<bool>,
    #[visit(optional)]
    pub delay: InheritableVariable<f32>,

    #[reflect(hidden)]
    #[visit(skip)]
    state: TweenState,
}

impl Default for TweenRotation {
    fn default() -> Self {
        Self {
            start_angles: Vector3::zeros().into(),
            end_angles: Vector3::new(0.0, std::f32::consts::TAU, 0.0).into(),
            duration: 1.0.into(),
            easing: EasingFunction::Linear.into(),
            loop_mode: TweenLoop::Once.into(),
            auto_play: true.into(),
            delay: 0.0.into(),
            state: TweenState::default(),
        }
    }
}

impl_component_provider!(TweenRotation);
uuid_provider!(TweenRotation = "b8c9d0e1-8888-9999-aaaa-112233445566");

impl ScriptTrait for TweenRotation {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        self.sync_state();
        self.state.init();
        context
            .message_dispatcher
            .subscribe_to::<TweenMessage>(context.handle);
        Ok(())
    }

    fn on_message(
        &mut self,
        message: &mut dyn ScriptMessagePayload,
        _ctx: &mut ScriptMessageContext,
    ) -> GameResult {
        self.sync_state();
        self.state.handle_message(message);
        Ok(())
    }

    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        if let Some(t) = self.state.tick(context.dt) {
            let start = UnitQuaternion::from_euler_angles(
                self.start_angles.x,
                self.start_angles.y,
                self.start_angles.z,
            );
            let end = UnitQuaternion::from_euler_angles(
                self.end_angles.x,
                self.end_angles.y,
                self.end_angles.z,
            );
            let rotation = start.slerp(&end, t);
            let node = context.scene.graph.try_get_node_mut(context.handle)?;
            node.local_transform_mut().set_rotation(rotation);
        }
        Ok(())
    }
}

impl TweenRotation {
    fn sync_state(&mut self) {
        *self.state.duration = *self.duration;
        *self.state.easing = *self.easing;
        *self.state.loop_mode = *self.loop_mode;
        *self.state.auto_play = *self.auto_play;
        *self.state.delay = *self.delay;
    }
}
