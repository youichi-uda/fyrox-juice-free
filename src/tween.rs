//! Tween scripts for position, scale, and rotation.
//!
//! Attach to a node and configure start/end values, duration, and easing.
//! The tween plays automatically on init or can be triggered via message.

use crate::easing::EasingFunction;
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        color::Color,
        impl_component_provider,
        pool::Handle,
        reflect::prelude::*,
        uuid_provider,
        variable::InheritableVariable,
        visitor::prelude::*,
    },
    plugin::error::GameResult,
    graph::SceneGraph,
    scene::{dim2::rectangle::Rectangle, graph::Graph, node::Node, sprite::Sprite},
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

// ─── TweenColor ───

/// Per-channel linear interpolation in 8-bit RGBA. `t` is clamped to [0, 1].
///
/// This mirrors `sprite_flash::lerp_color` and is kept inline so `tween` does
/// not need a public re-export from `sprite_flash`.
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::from_rgba(
        (a.r as f32 + (b.r as f32 - a.r as f32) * t) as u8,
        (a.g as f32 + (b.g as f32 - a.g as f32) * t) as u8,
        (a.b as f32 + (b.b as f32 - a.b as f32) * t) as u8,
        (a.a as f32 + (b.a as f32 - a.a as f32) * t) as u8,
    )
}

fn set_color(graph: &mut Graph, handle: Handle<Node>, color: Color) {
    let node = &mut graph[handle];
    if let Some(sprite) = node.cast_mut::<Sprite>() {
        sprite.set_color(color);
    } else if let Some(rect) = node.cast_mut::<Rectangle>() {
        rect.set_color(color);
    }
}

/// Message to control [`TweenColor`]. Reuses [`TweenMessage`] since the
/// playback controls (Play/Pause/Resume/Stop/Reverse) are identical to the
/// other tween scripts; the dedicated alias documents the binding.
pub type TweenColorMessage = TweenMessage;

/// Tweens the color of a `Sprite` or `Rectangle` node from `start_color` to
/// `end_color`.
///
/// Mirrors [`TweenPosition`] / [`TweenScale`] / [`TweenRotation`]: same
/// playback/loop/easing semantics, same auto-play behavior, same message API.
/// Works with the same node types as [`crate::sprite_flash::SpriteFlash`]
/// (Sprite and Rectangle).
#[derive(Visit, Reflect, Clone, Debug)]
pub struct TweenColor {
    /// Color at t=0.
    #[visit(optional)]
    pub start_color: InheritableVariable<Color>,
    /// Color at t=1.
    #[visit(optional)]
    pub end_color: InheritableVariable<Color>,

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

impl Default for TweenColor {
    fn default() -> Self {
        Self {
            start_color: Color::WHITE.into(),
            end_color: Color::TRANSPARENT.into(),
            duration: 0.5.into(),
            easing: EasingFunction::EaseOutQuad.into(),
            loop_mode: TweenLoop::Once.into(),
            auto_play: true.into(),
            delay: 0.0.into(),
            state: TweenState::default(),
        }
    }
}

impl_component_provider!(TweenColor);
uuid_provider!(TweenColor = "c9d0e1f2-9999-aaaa-bbbb-223344556677");

impl ScriptTrait for TweenColor {
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
            let color = lerp_color(*self.start_color, *self.end_color, t);
            set_color(&mut context.scene.graph, context.handle, color);
        }
        Ok(())
    }
}

impl TweenColor {
    fn sync_state(&mut self) {
        *self.state.duration = *self.duration;
        *self.state.easing = *self.easing;
        *self.state.loop_mode = *self.loop_mode;
        *self.state.auto_play = *self.auto_play;
        *self.state.delay = *self.delay;
    }
}

#[cfg(test)]
mod tween_color_tests {
    use super::*;

    #[test]
    fn lerp_color_interpolates_each_channel_and_clamps_t() {
        // Endpoints: t=0 -> a, t=1 -> b. Channels interpolate independently
        // and t is clamped to [0, 1].
        let a = Color::from_rgba(0, 0, 0, 255);
        let b = Color::from_rgba(200, 100, 50, 0);

        let start = lerp_color(a, b, 0.0);
        assert_eq!((start.r, start.g, start.b, start.a), (0, 0, 0, 255));

        let end = lerp_color(a, b, 1.0);
        assert_eq!((end.r, end.g, end.b, end.a), (200, 100, 50, 0));

        // Midpoint: each channel sits halfway.
        let mid = lerp_color(a, b, 0.5);
        assert_eq!(mid.r, 100);
        assert_eq!(mid.g, 50);
        assert_eq!(mid.b, 25);
        // Alpha goes 255 -> 0 so midpoint is 127 (floor of 127.5 via as u8).
        assert_eq!(mid.a, 127);

        // Out-of-range t is clamped, not extrapolated.
        let below = lerp_color(a, b, -1.0);
        assert_eq!((below.r, below.g, below.b, below.a), (0, 0, 0, 255));
        let above = lerp_color(a, b, 2.0);
        assert_eq!((above.r, above.g, above.b, above.a), (200, 100, 50, 0));
    }
}
