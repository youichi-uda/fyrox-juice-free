//! Slow motion effect with smooth transitions.
//!
//! Attach to a manager node. Send [`SlowMotionMessage`] to activate/deactivate.

use crate::easing::EasingFunction;
use fyrox::{
    core::{
        impl_component_provider, reflect::prelude::*, uuid_provider,
        variable::InheritableVariable, visitor::prelude::*,
    },
    plugin::error::GameResult,
    script::{ScriptContext, ScriptMessageContext, ScriptMessagePayload, ScriptTrait},
};

/// Message to control slow motion.
#[derive(Debug, ScriptMessagePayload)]
pub enum SlowMotionMessage {
    /// Enter slow motion with optional custom time scale (None = use default).
    Enter(Option<f32>),
    /// Exit slow motion.
    Exit,
    /// Toggle slow motion on/off.
    Toggle,
}

/// Slow motion controller. Provides a `current_time_scale` that other scripts
/// can read and apply to their dt.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct SlowMotion {
    /// Target time scale during slow motion (e.g. 0.3 = 30% speed).
    #[visit(optional)]
    #[reflect(min_value = 0.01, max_value = 1.0)]
    pub target_time_scale: InheritableVariable<f32>,

    /// How long the transition in takes (seconds, in real time).
    #[visit(optional)]
    pub transition_in_duration: InheritableVariable<f32>,

    /// How long the transition out takes (seconds, in real time).
    #[visit(optional)]
    pub transition_out_duration: InheritableVariable<f32>,

    /// Easing for entering slow motion.
    #[visit(optional)]
    pub ease_in: InheritableVariable<EasingFunction>,

    /// Easing for exiting slow motion.
    #[visit(optional)]
    pub ease_out: InheritableVariable<EasingFunction>,

    /// Maximum duration of slow motion (0 = unlimited). After this, automatically exits.
    #[visit(optional)]
    #[reflect(min_value = 0.0)]
    pub max_duration: InheritableVariable<f32>,

    /// Current effective time scale (read this from other scripts).
    #[visit(optional)]
    pub current_time_scale: f32,

    /// Whether slow motion is active (or transitioning).
    #[visit(optional)]
    pub is_active: bool,

    #[reflect(hidden)]
    #[visit(skip)]
    state: SlowMotionState,

    #[reflect(hidden)]
    #[visit(skip)]
    timer: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    active_timer: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    override_scale: f32,
}

#[derive(Clone, Debug, Default)]
enum SlowMotionState {
    #[default]
    Idle,
    TransitionIn,
    Active,
    TransitionOut,
}

impl Default for SlowMotion {
    fn default() -> Self {
        Self {
            target_time_scale: 0.3.into(),
            transition_in_duration: 0.15.into(),
            transition_out_duration: 0.3.into(),
            ease_in: EasingFunction::EaseOutQuad.into(),
            ease_out: EasingFunction::EaseInQuad.into(),
            max_duration: 0.0.into(),
            current_time_scale: 1.0,
            is_active: false,
            state: SlowMotionState::Idle,
            timer: 0.0,
            active_timer: 0.0,
            override_scale: 0.0,
        }
    }
}

impl_component_provider!(SlowMotion);
uuid_provider!(SlowMotion = "e5f6a7b8-5555-6666-7777-eeff00112233");

impl ScriptTrait for SlowMotion {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        context
            .message_dispatcher
            .subscribe_to::<SlowMotionMessage>(context.handle);
        Ok(())
    }

    fn on_message(
        &mut self,
        message: &mut dyn ScriptMessagePayload,
        _ctx: &mut ScriptMessageContext,
    ) -> GameResult {
        if let Some(msg) = message.downcast_ref::<SlowMotionMessage>() {
            match msg {
                SlowMotionMessage::Enter(scale) => {
                    self.override_scale = scale.unwrap_or(*self.target_time_scale);
                    self.state = SlowMotionState::TransitionIn;
                    self.timer = 0.0;
                    self.active_timer = 0.0;
                    self.is_active = true;
                }
                SlowMotionMessage::Exit => {
                    self.state = SlowMotionState::TransitionOut;
                    self.timer = 0.0;
                }
                SlowMotionMessage::Toggle => {
                    if self.is_active {
                        self.state = SlowMotionState::TransitionOut;
                        self.timer = 0.0;
                    } else {
                        self.override_scale = *self.target_time_scale;
                        self.state = SlowMotionState::TransitionIn;
                        self.timer = 0.0;
                        self.active_timer = 0.0;
                        self.is_active = true;
                    }
                }
            }
        }
        Ok(())
    }

    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        self.timer += context.dt;

        match self.state {
            SlowMotionState::Idle => {
                self.current_time_scale = 1.0;
            }
            SlowMotionState::TransitionIn => {
                let dur = self.transition_in_duration.max(0.001);
                let t = (self.timer / dur).min(1.0);
                let eased = self.ease_in.evaluate(t);
                self.current_time_scale = 1.0 + (self.override_scale - 1.0) * eased;
                if t >= 1.0 {
                    self.state = SlowMotionState::Active;
                    self.timer = 0.0;
                }
            }
            SlowMotionState::Active => {
                self.current_time_scale = self.override_scale;
                self.active_timer += context.dt;
                if *self.max_duration > 0.0 && self.active_timer >= *self.max_duration {
                    self.state = SlowMotionState::TransitionOut;
                    self.timer = 0.0;
                }
            }
            SlowMotionState::TransitionOut => {
                let dur = self.transition_out_duration.max(0.001);
                let t = (self.timer / dur).min(1.0);
                let eased = self.ease_out.evaluate(t);
                self.current_time_scale = self.override_scale + (1.0 - self.override_scale) * eased;
                if t >= 1.0 {
                    self.state = SlowMotionState::Idle;
                    self.current_time_scale = 1.0;
                    self.is_active = false;
                }
            }
        }

        Ok(())
    }
}
