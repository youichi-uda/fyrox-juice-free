//! Smooth camera follow with dead zone and look-ahead.
//!
//! Attach to a node (typically a camera or camera pivot). Set `target` to the node to follow.

use fyrox::{
    core::{
        algebra::Vector3,
        impl_component_provider,
        pool::Handle,
        reflect::prelude::*,
        uuid_provider,
        variable::InheritableVariable,
        visitor::prelude::*,
    },
    plugin::error::GameResult,
    graph::SceneGraph,
    scene::node::Node,
    script::{ScriptContext, ScriptTrait},
};

/// Smooth follow script. Attach to a camera pivot and assign a target node.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct SmoothCameraFollow {
    /// The node to follow.
    #[visit(optional)]
    pub target: InheritableVariable<Handle<Node>>,

    /// Smoothing speed. Higher = snappier. Range: 1.0 (very smooth) to 20.0+ (nearly instant).
    #[visit(optional)]
    #[reflect(min_value = 0.01)]
    pub smooth_speed: InheritableVariable<f32>,

    /// Offset from target position (e.g. Vector3(0, 5, -10) for a behind-and-above view).
    #[visit(optional)]
    pub offset: InheritableVariable<Vector3<f32>>,

    /// Dead zone radius. Camera won't move if target is within this distance of current aim.
    #[visit(optional)]
    #[reflect(min_value = 0.0)]
    pub dead_zone: InheritableVariable<f32>,

    /// Look-ahead multiplier. Camera shifts in the direction the target is moving.
    #[visit(optional)]
    #[reflect(min_value = 0.0)]
    pub look_ahead: InheritableVariable<f32>,

    #[reflect(hidden)]
    #[visit(skip)]
    prev_target_pos: Option<Vector3<f32>>,
}

impl Default for SmoothCameraFollow {
    fn default() -> Self {
        Self {
            target: Handle::NONE.into(),
            smooth_speed: 5.0.into(),
            offset: Vector3::new(0.0, 5.0, -10.0).into(),
            dead_zone: 0.0.into(),
            look_ahead: 0.0.into(),
            prev_target_pos: None,
        }
    }
}

impl_component_provider!(SmoothCameraFollow);
uuid_provider!(SmoothCameraFollow = "c3d4e5f6-3333-4444-5555-ccddeeff0011");

impl ScriptTrait for SmoothCameraFollow {
    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        let target_handle = *self.target;
        if target_handle.is_none() {
            return Ok(());
        }

        let target_pos = context
            .scene
            .graph
            .try_get(target_handle)
            .map(|n| n.global_position())?;

        // Calculate velocity for look-ahead.
        let velocity = if let Some(prev) = self.prev_target_pos {
            target_pos - prev
        } else {
            Vector3::zeros()
        };
        self.prev_target_pos = Some(target_pos);

        let desired = target_pos + *self.offset + velocity * *self.look_ahead;

        let current_pos = context
            .scene
            .graph
            .try_get(context.handle)
            .map(|n| **n.local_transform().position())?;

        let diff = desired - current_pos;
        let dist = diff.magnitude();

        // Dead zone check.
        if dist < *self.dead_zone {
            return Ok(());
        }

        // Exponential smoothing (frame-rate independent).
        let alpha = 1.0 - (-*self.smooth_speed * context.dt).exp();
        let new_pos = current_pos + diff * alpha;

        let node = context.scene.graph.try_get_node_mut(context.handle)?;
        node.local_transform_mut().set_position(new_pos);

        Ok(())
    }
}
