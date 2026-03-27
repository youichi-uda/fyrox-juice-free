//! Trail renderer effect.
//!
//! Attach to a moving node. Creates a trail of position samples that fade over time.
//! The trail is rendered by scaling/fading child "segment" nodes, or by recording
//! positions for external rendering.

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
    plugin::error::GameResult,
    graph::SceneGraph,
    scene::{dim2::rectangle::Rectangle, sprite::Sprite},
    script::{ScriptContext, ScriptTrait},
};

/// A single trail point.
#[derive(Clone, Debug, Default, Visit, Reflect)]
pub struct TrailPoint {
    pub position: Vector3<f32>,
    pub age: f32,
}

/// Trail renderer that records positions and fades them over time.
///
/// This script records the node's world position over time. It stores
/// the trail points in `points` which can be read by a custom mesh builder
/// or used with child segment nodes.
#[derive(Visit, Reflect, Clone, Debug)]
pub struct TrailRenderer {
    /// Maximum lifetime of each trail point (seconds).
    #[visit(optional)]
    pub lifetime: InheritableVariable<f32>,

    /// Minimum distance between trail points. Prevents excessive sampling when stationary.
    #[visit(optional)]
    #[reflect(min_value = 0.0)]
    pub min_distance: InheritableVariable<f32>,

    /// Maximum number of trail points.
    #[visit(optional)]
    pub max_points: InheritableVariable<usize>,

    /// Starting color of the trail (newest point).
    #[visit(optional)]
    pub start_color: InheritableVariable<Color>,

    /// Ending color of the trail (oldest point, just before fading out).
    #[visit(optional)]
    pub end_color: InheritableVariable<Color>,

    /// Starting width of the trail.
    #[visit(optional)]
    pub start_width: InheritableVariable<f32>,

    /// Ending width of the trail.
    #[visit(optional)]
    pub end_width: InheritableVariable<f32>,

    /// Whether the trail is currently emitting.
    #[visit(optional)]
    pub emitting: InheritableVariable<bool>,

    /// Current trail points (read-only from external scripts).
    #[visit(optional)]
    pub points: Vec<TrailPoint>,
}

impl Default for TrailRenderer {
    fn default() -> Self {
        Self {
            lifetime: 0.5.into(),
            min_distance: 0.05.into(),
            max_points: 50.into(),
            start_color: Color::WHITE.into(),
            end_color: Color::from_rgba(255, 255, 255, 0).into(),
            start_width: 0.2.into(),
            end_width: 0.02.into(),
            emitting: true.into(),
            points: Vec::new(),
        }
    }
}

impl_component_provider!(TrailRenderer);
uuid_provider!(TrailRenderer = "a3b4c5d6-dddd-eeee-ffff-667788990011");

impl ScriptTrait for TrailRenderer {
    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        let dt = context.dt;
        let lifetime = self.lifetime.max(0.001);

        // Age existing points and remove expired ones.
        for point in &mut self.points {
            point.age += dt;
        }
        self.points.retain(|p| p.age < lifetime);

        // Add new point if emitting.
        if *self.emitting {
            let current_pos = context
                .scene
                .graph
                .try_get(context.handle)
                .map(|n| n.global_position())?;

            let should_add = self
                .points
                .last()
                .map(|last| (current_pos - last.position).magnitude() >= *self.min_distance)
                .unwrap_or(true);

            if should_add && self.points.len() < *self.max_points {
                self.points.push(TrailPoint {
                    position: current_pos,
                    age: 0.0,
                });
            }
        }

        // Update child nodes (if any) to visualize trail segments.
        // Children are mapped 1:1 to trail points, oldest first.
        let node = &context.scene.graph[context.handle];
        let children: Vec<_> = node.children().to_vec();
        let num_children = children.len();

        if num_children > 0 {
            for (i, &child_handle) in children.iter().enumerate() {
                if i < self.points.len() {
                    let point = &self.points[i];
                    let t = point.age / lifetime;

                    // Interpolate color.
                    let color = lerp_color(*self.start_color, *self.end_color, t);
                    let width =
                        *self.start_width + (*self.end_width - *self.start_width) * t;

                    let child = &mut context.scene.graph[child_handle];
                    child
                        .local_transform_mut()
                        .set_position(point.position)
                        .set_scale(Vector3::new(width, width, width));
                    child.set_visibility(true);

                    // Set color if it's a Sprite or Rectangle.
                    if let Some(sprite) = child.cast_mut::<Sprite>() {
                        sprite.set_color(color);
                    } else if let Some(rect) = child.cast_mut::<Rectangle>() {
                        rect.set_color(color);
                    }
                } else {
                    let child = &mut context.scene.graph[child_handle];
                    child.set_visibility(false);
                }
            }
        }

        Ok(())
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
