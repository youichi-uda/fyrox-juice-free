//! Trauma-based camera/node shake effect.
//!
//! Attach this script to any node. Call [`CameraShakeMessage::AddTrauma`] to trigger shake.
//! The shake intensity is proportional to `trauma^2` for a more natural feel.

use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
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

/// Message to control the camera shake.
#[derive(Debug, ScriptMessagePayload)]
pub enum CameraShakeMessage {
    /// Add trauma (0.0..1.0). Trauma is clamped to 1.0 max.
    AddTrauma(f32),
    /// Immediately stop all shake.
    Reset,
}

/// Trauma-based shake effect. Attach to any node (typically a camera parent pivot).
///
/// The node's local position is offset each frame by a random amount proportional to `trauma^2`.
/// Trauma decays over time at `decay_rate` per second.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct CameraShake {
    /// How fast trauma decays per second (e.g. 1.0 = fully decays in 1 second).
    #[visit(optional)]
    pub decay_rate: InheritableVariable<f32>,

    /// Maximum positional offset on each axis when trauma = 1.0.
    #[visit(optional)]
    pub max_offset: InheritableVariable<Vector3<f32>>,

    /// Maximum rotational offset on each axis (Euler radians: roll, pitch, yaw)
    /// when trauma = 1.0. Default is zero, so existing scenes get no rotation
    /// shake unless they opt in.
    #[visit(optional)]
    pub max_rotation: InheritableVariable<Vector3<f32>>,

    /// Shake frequency (oscillations per second). Higher = more jittery.
    #[visit(optional)]
    pub frequency: InheritableVariable<f32>,

    /// Current trauma level (0.0 to 1.0). Exposed for debugging; usually controlled via messages.
    #[visit(optional)]
    pub trauma: f32,

    /// The original position before shake was applied, used to restore the node.
    #[reflect(hidden)]
    #[visit(skip)]
    original_position: Option<Vector3<f32>>,

    /// The original rotation before shake was applied, used to restore the node.
    #[reflect(hidden)]
    #[visit(skip)]
    original_rotation: Option<UnitQuaternion<f32>>,

    /// Internal time accumulator for noise sampling.
    #[reflect(hidden)]
    #[visit(skip)]
    time: f32,

    /// Simple seed for deterministic pseudo-random values.
    #[visit(optional)]
    pub seed: InheritableVariable<u32>,
}

impl Default for CameraShake {
    fn default() -> Self {
        Self {
            decay_rate: 1.5.into(),
            max_offset: Vector3::new(0.3, 0.3, 0.0).into(),
            max_rotation: Vector3::default().into(),
            frequency: 15.0.into(),
            trauma: 0.0,
            original_position: None,
            original_rotation: None,
            time: 0.0,
            seed: 0.into(),
        }
    }
}

impl_component_provider!(CameraShake);
uuid_provider!(CameraShake = "a1b2c3d4-1111-2222-3333-aabbccddeeff");

/// Simple hash-based noise in [-1, 1].
fn noise(seed: u32, x: f32) -> f32 {
    let n = seed.wrapping_add((x * 1000.0) as u32);
    let n = n.wrapping_mul(0x5bd1e995);
    let n = n ^ (n >> 15);
    let n = n.wrapping_mul(0x27d4eb2d);
    (n as f32 / u32::MAX as f32) * 2.0 - 1.0
}

impl CameraShake {
    /// Compute the current frame's translational offset and rotational shake
    /// for the configured trauma / time / seed. Pure (no engine state), so it
    /// can be unit-tested without spinning up a `ScriptContext`.
    ///
    /// Returned rotation is the *delta* from the rest pose: at `trauma = 0` or
    /// `max_rotation = 0` this is the identity quaternion.
    fn shake_offsets(&self) -> (Vector3<f32>, UnitQuaternion<f32>) {
        let shake_amount = self.trauma * self.trauma;
        let seed = *self.seed;

        let offset_x = noise(seed, self.time) * self.max_offset.x * shake_amount;
        let offset_y = noise(seed.wrapping_add(100), self.time) * self.max_offset.y * shake_amount;
        let offset_z =
            noise(seed.wrapping_add(200), self.time) * self.max_offset.z * shake_amount;

        let rot_x = noise(seed.wrapping_add(300), self.time) * self.max_rotation.x * shake_amount;
        let rot_y = noise(seed.wrapping_add(400), self.time) * self.max_rotation.y * shake_amount;
        let rot_z = noise(seed.wrapping_add(500), self.time) * self.max_rotation.z * shake_amount;

        (
            Vector3::new(offset_x, offset_y, offset_z),
            UnitQuaternion::from_euler_angles(rot_x, rot_y, rot_z),
        )
    }
}

impl ScriptTrait for CameraShake {
    fn on_init(&mut self, context: &mut ScriptContext) -> GameResult {
        context
            .message_dispatcher
            .subscribe_to::<CameraShakeMessage>(context.handle);
        let node = context.scene.graph.try_get_node_mut(context.handle)?;
        self.original_position = Some(**node.local_transform().position());
        self.original_rotation = Some(**node.local_transform().rotation());
        Ok(())
    }

    fn on_message(
        &mut self,
        message: &mut dyn ScriptMessagePayload,
        _ctx: &mut ScriptMessageContext,
    ) -> GameResult {
        if let Some(msg) = message.downcast_ref::<CameraShakeMessage>() {
            match msg {
                CameraShakeMessage::AddTrauma(amount) => {
                    self.trauma = (self.trauma + amount).min(1.0);
                }
                CameraShakeMessage::Reset => {
                    self.trauma = 0.0;
                }
            }
        }
        Ok(())
    }

    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        if self.trauma <= 0.0 {
            let node = context.scene.graph.try_get_node_mut(context.handle)?;
            if let Some(orig) = self.original_position {
                node.local_transform_mut().set_position(orig);
            }
            if let Some(orig) = self.original_rotation {
                node.local_transform_mut().set_rotation(orig);
            }
            return Ok(());
        }

        self.time += context.dt * *self.frequency;

        let (translation, shake_rot) = self.shake_offsets();

        let node = context.scene.graph.try_get_node_mut(context.handle)?;
        if let Some(orig) = self.original_position {
            node.local_transform_mut().set_position(orig + translation);
        }
        if let Some(orig) = self.original_rotation {
            node.local_transform_mut().set_rotation(orig * shake_rot);
        }

        self.trauma = (self.trauma - *self.decay_rate * context.dt).max(0.0);

        Ok(())
    }

    fn on_deinit(&mut self, context: &mut ScriptDeinitContext) -> GameResult {
        if let Ok(node) = context.scene.graph.try_get_node_mut(context.node_handle) {
            if let Some(orig) = self.original_position {
                node.local_transform_mut().set_position(orig);
            }
            if let Some(orig) = self.original_rotation {
                node.local_transform_mut().set_rotation(orig);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_is_zero_by_default_and_applied_when_max_rotation_is_set() {
        // Default config: no rotational shake, so the rotation delta is
        // identity even at full trauma. This guards backward compatibility
        // for scenes that only used positional shake.
        let mut s = CameraShake {
            trauma: 1.0,
            time: 0.5,
            ..CameraShake::default()
        };
        let (_, rot) = s.shake_offsets();
        let (rx, ry, rz) = rot.euler_angles();
        assert!(rx.abs() < 1e-6, "default max_rotation should yield zero rx, got {rx}");
        assert!(ry.abs() < 1e-6, "default max_rotation should yield zero ry, got {ry}");
        assert!(rz.abs() < 1e-6, "default max_rotation should yield zero rz, got {rz}");

        // Opt in to rotational shake: with trauma > 0 and a non-zero
        // max_rotation, at least one axis should be non-zero. (The noise
        // function is deterministic, so this is stable.)
        s.max_rotation = Vector3::new(0.2, 0.2, 0.2).into();
        let (_, rot) = s.shake_offsets();
        let (rx, ry, rz) = rot.euler_angles();
        assert!(
            rx.abs() + ry.abs() + rz.abs() > 1e-4,
            "non-zero max_rotation + full trauma should produce some rotation; got ({rx}, {ry}, {rz})"
        );

        // Trauma at zero: no rotation regardless of max_rotation.
        s.trauma = 0.0;
        let (_, rot) = s.shake_offsets();
        let (rx, ry, rz) = rot.euler_angles();
        assert!(rx.abs() < 1e-6 && ry.abs() < 1e-6 && rz.abs() < 1e-6);
    }
}
