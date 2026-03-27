# fyrox-juice-free

Game juice effects pack for the [Fyrox](https://fyrox.rs) engine. Drop-in scripts for screen shake, hit stop, tweening, visual feedback, and more.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
fyrox-juice-free = "0.1"
```

## Quick Start

Register all scripts in your plugin:

```rust
impl Plugin for Game {
    fn register(&self, context: PluginRegistrationContext) -> GameResult {
        fyrox_juice_free::register(&context.serialization_context.script_constructors);
        Ok(())
    }
}
```

Then attach scripts to nodes in the Fyrox editor, or trigger them via messages:

```rust
// Add screen shake
context.message_sender.send_to_target(
    shake_node,
    CameraShakeMessage::AddTrauma(0.6),
);

// Trigger hit stop
context.message_sender.send_to_target(
    hitstop_node,
    HitStopMessage::Trigger(0.07),
);
```

## Included Scripts (17)

### Camera

| Script | Description |
|--------|-------------|
| **CameraShake** | Trauma-based screen shake with quadratic falloff |
| **CameraZoomPulse** | FOV punch in/out on impact |
| **SmoothCameraFollow** | Smooth follow with dead zone and look-ahead |

### Timing

| Script | Description |
|--------|-------------|
| **HitStop** | Frame freeze on hit with configurable duration |
| **SlowMotion** | Slow motion with eased transitions |

### Tweening

| Script | Description |
|--------|-------------|
| **TweenPosition** | Position tween with 22 easing functions |
| **TweenScale** | Scale tween |
| **TweenRotation** | Rotation tween (Slerp) |
| **SquashAndStretch** | Volume-preserving squash/stretch deformation |
| **Pulse** | Looping scale oscillation |
| **Bounce** | Looping position bounce |

### Visual

| Script | Description |
|--------|-------------|
| **SpriteFlash** | Damage flash (white/color flash on hit) |
| **TrailRenderer** | Motion trail with fade |
| **AfterImage** | Ghosting/after-image for fast movement |

### UI / Feedback

| Script | Description |
|--------|-------------|
| **DamageNumber** | Floating damage number popup |
| **UIShake** | UI element shake |
| **UIBounce** | UI element pop/bounce |

### Easing Functions (22)

Linear, EaseIn/Out/InOut for: Quad, Cubic, Elastic, Back, Bounce, Sine, Expo

## License

MIT

---

## Want more?

**Fyrox Juice Pro** includes:

| Feature | Free | Pro |
|---------|------|-----|
| All 17 core scripts | :white_check_mark: | :white_check_mark: |
| 22 easing functions | :white_check_mark: | :white_check_mark: |
| Editor UI integration | :x: | :white_check_mark: |
| Effect preset library | :x: | :white_check_mark: |
| Advanced configuration wizard | :x: | :white_check_mark: |
| Priority support & updates | :x: | :white_check_mark: |

:point_right: **[Get Fyrox Juice Pro on Gumroad](https://gumroad.com/)** (coming soon)
