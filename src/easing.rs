//! Easing functions for smooth animations.

use fyrox::core::reflect::prelude::*;
use fyrox::core::visitor::prelude::*;
use std::f32::consts::PI;

/// Easing function type used by juice scripts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Visit, Reflect, Default)]
pub enum EasingFunction {
    #[default]
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
}

impl EasingFunction {
    /// All 22 easing functions in canonical (declaration) order.
    ///
    /// Useful for iterating over every variant in tests, UI dropdowns, or
    /// preset capture/round-trip logic without manually maintaining a list.
    pub const fn all() -> [Self; 22] {
        [
            Self::Linear,
            Self::EaseInQuad,
            Self::EaseOutQuad,
            Self::EaseInOutQuad,
            Self::EaseInCubic,
            Self::EaseOutCubic,
            Self::EaseInOutCubic,
            Self::EaseInElastic,
            Self::EaseOutElastic,
            Self::EaseInOutElastic,
            Self::EaseInBack,
            Self::EaseOutBack,
            Self::EaseInOutBack,
            Self::EaseInBounce,
            Self::EaseOutBounce,
            Self::EaseInOutBounce,
            Self::EaseInSine,
            Self::EaseOutSine,
            Self::EaseInOutSine,
            Self::EaseInExpo,
            Self::EaseOutExpo,
            Self::EaseInOutExpo,
        ]
    }

    /// Parse from a PascalCase or kebab-case name.
    ///
    /// Accepts both `"EaseOutElastic"` (matching the Rust variant name, as used
    /// by `juice-pro` preset RON files) and `"ease-out-elastic"` (kebab-case,
    /// friendlier for hand-edited config). Returns `None` for unknown names so
    /// callers can surface a clear error instead of silently picking a wrong
    /// curve.
    ///
    /// This is an inherent method that returns `Option<Self>` rather than the
    /// `FromStr` trait (which returns `Result`) because the only failure mode
    /// is "unknown name" and callers consistently want an `Option`.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(name: &str) -> Option<Self> {
        Some(match name {
            "Linear" | "linear" => Self::Linear,
            "EaseInQuad" | "ease-in-quad" => Self::EaseInQuad,
            "EaseOutQuad" | "ease-out-quad" => Self::EaseOutQuad,
            "EaseInOutQuad" | "ease-in-out-quad" => Self::EaseInOutQuad,
            "EaseInCubic" | "ease-in-cubic" => Self::EaseInCubic,
            "EaseOutCubic" | "ease-out-cubic" => Self::EaseOutCubic,
            "EaseInOutCubic" | "ease-in-out-cubic" => Self::EaseInOutCubic,
            "EaseInElastic" | "ease-in-elastic" => Self::EaseInElastic,
            "EaseOutElastic" | "ease-out-elastic" => Self::EaseOutElastic,
            "EaseInOutElastic" | "ease-in-out-elastic" => Self::EaseInOutElastic,
            "EaseInBack" | "ease-in-back" => Self::EaseInBack,
            "EaseOutBack" | "ease-out-back" => Self::EaseOutBack,
            "EaseInOutBack" | "ease-in-out-back" => Self::EaseInOutBack,
            "EaseInBounce" | "ease-in-bounce" => Self::EaseInBounce,
            "EaseOutBounce" | "ease-out-bounce" => Self::EaseOutBounce,
            "EaseInOutBounce" | "ease-in-out-bounce" => Self::EaseInOutBounce,
            "EaseInSine" | "ease-in-sine" => Self::EaseInSine,
            "EaseOutSine" | "ease-out-sine" => Self::EaseOutSine,
            "EaseInOutSine" | "ease-in-out-sine" => Self::EaseInOutSine,
            "EaseInExpo" | "ease-in-expo" => Self::EaseInExpo,
            "EaseOutExpo" | "ease-out-expo" => Self::EaseOutExpo,
            "EaseInOutExpo" | "ease-in-out-expo" => Self::EaseInOutExpo,
            _ => return None,
        })
    }

    /// Evaluate the easing function at parameter `t` (0.0 to 1.0).
    pub fn evaluate(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseInQuad => t * t,
            Self::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Self::EaseInCubic => t * t * t,
            Self::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Self::EaseInElastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let c4 = (2.0 * PI) / 3.0;
                    -(2.0f32.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * c4).sin()
                }
            }
            Self::EaseOutElastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let c4 = (2.0 * PI) / 3.0;
                    2.0f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
            Self::EaseInOutElastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let c5 = (2.0 * PI) / 4.5;
                    if t < 0.5 {
                        -(2.0f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
                    } else {
                        (2.0f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
                            + 1.0
                    }
                }
            }
            Self::EaseInBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            Self::EaseOutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            Self::EaseInOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
                }
            }
            Self::EaseInBounce => 1.0 - Self::EaseOutBounce.evaluate(1.0 - t),
            Self::EaseOutBounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
            Self::EaseInOutBounce => {
                if t < 0.5 {
                    (1.0 - Self::EaseOutBounce.evaluate(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + Self::EaseOutBounce.evaluate(2.0 * t - 1.0)) / 2.0
                }
            }
            Self::EaseInSine => 1.0 - ((t * PI) / 2.0).cos(),
            Self::EaseOutSine => ((t * PI) / 2.0).sin(),
            Self::EaseInOutSine => -((PI * t).cos() - 1.0) / 2.0,
            Self::EaseInExpo => {
                if t == 0.0 {
                    0.0
                } else {
                    2.0f32.powf(10.0 * t - 10.0)
                }
            }
            Self::EaseOutExpo => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - 2.0f32.powf(-10.0 * t)
                }
            }
            Self::EaseInOutExpo => {
                if t == 0.0 || t == 1.0 {
                    t
                } else if t < 0.5 {
                    2.0f32.powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - 2.0f32.powf(-20.0 * t + 10.0)) / 2.0
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All 22 easing functions, used to assert universal boundary conditions.
    const ALL: [EasingFunction; 22] = EasingFunction::all();

    #[test]
    fn all_easings_anchor_at_endpoints() {
        // Every easing must map t=0 -> 0 and t=1 -> 1.
        for f in ALL {
            assert!(
                f.evaluate(0.0).abs() < 1e-4,
                "{f:?} should be 0 at t=0, got {}",
                f.evaluate(0.0)
            );
            assert!(
                (f.evaluate(1.0) - 1.0).abs() < 1e-4,
                "{f:?} should be 1 at t=1, got {}",
                f.evaluate(1.0)
            );
        }
    }

    #[test]
    fn evaluate_clamps_out_of_range_input() {
        for f in ALL {
            assert!((f.evaluate(-1.0) - f.evaluate(0.0)).abs() < 1e-6, "{f:?} below 0");
            assert!((f.evaluate(2.0) - f.evaluate(1.0)).abs() < 1e-6, "{f:?} above 1");
        }
    }

    #[test]
    fn linear_is_identity() {
        assert_eq!(EasingFunction::Linear.evaluate(0.0), 0.0);
        assert_eq!(EasingFunction::Linear.evaluate(0.25), 0.25);
        assert_eq!(EasingFunction::Linear.evaluate(0.5), 0.5);
        assert_eq!(EasingFunction::Linear.evaluate(1.0), 1.0);
    }

    #[test]
    fn in_quad_is_slow_then_fast() {
        let f = EasingFunction::EaseInQuad;
        // t^2 at 0.5 = 0.25, which is below the linear midpoint.
        assert!((f.evaluate(0.5) - 0.25).abs() < 1e-6);
        assert!(f.evaluate(0.5) < 0.5);
    }

    #[test]
    fn out_quad_is_fast_then_slow() {
        let f = EasingFunction::EaseOutQuad;
        // Out is the mirror of In: at 0.5 it is 0.75, above the linear midpoint.
        assert!((f.evaluate(0.5) - 0.75).abs() < 1e-6);
        assert!(f.evaluate(0.5) > 0.5);
    }

    #[test]
    fn monotonic_easings_are_non_decreasing() {
        // These curves never go backwards (excludes overshooting Back/Elastic/Bounce).
        let monotonic = [
            EasingFunction::Linear,
            EasingFunction::EaseInQuad,
            EasingFunction::EaseOutQuad,
            EasingFunction::EaseInOutQuad,
            EasingFunction::EaseInCubic,
            EasingFunction::EaseOutCubic,
            EasingFunction::EaseInOutCubic,
            EasingFunction::EaseInSine,
            EasingFunction::EaseOutSine,
            EasingFunction::EaseInOutSine,
            EasingFunction::EaseInExpo,
            EasingFunction::EaseOutExpo,
            EasingFunction::EaseInOutExpo,
        ];
        for f in monotonic {
            let mut prev = f.evaluate(0.0);
            for i in 1..=100 {
                let t = i as f32 / 100.0;
                let cur = f.evaluate(t);
                assert!(cur >= prev - 1e-4, "{f:?} decreased at t={t}: {cur} < {prev}");
                prev = cur;
            }
        }
    }

    #[test]
    fn all_returns_every_variant_exactly_once() {
        // Sanity-check the const list: every variant must appear and the size
        // must match the enum count so adding a variant without updating
        // `all()` is caught by the test suite.
        let all = EasingFunction::all();
        assert_eq!(all.len(), 22);
        // Confirm no duplicates: every variant must be distinct. (Enum is not
        // Hash, so use an O(n^2) PartialEq scan instead of a HashSet.)
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert!(a != b, "duplicate variant in EasingFunction::all(): {a:?}");
            }
        }
    }

    #[test]
    fn from_str_roundtrips_for_every_variant() {
        // Each variant must round-trip via its PascalCase Debug name, the
        // exact string preset RON files use.
        for f in EasingFunction::all() {
            let name = format!("{f:?}");
            assert_eq!(
                EasingFunction::from_str(&name),
                Some(f),
                "round-trip failed for {f:?} via PascalCase name `{name}`",
            );
        }
        // Spot-check a kebab-case form and a clearly invalid input.
        assert_eq!(
            EasingFunction::from_str("ease-out-elastic"),
            Some(EasingFunction::EaseOutElastic)
        );
        assert_eq!(EasingFunction::from_str("not-a-real-easing"), None);
    }

    #[test]
    fn elastic_overshoots_below_zero_on_ease_in() {
        // EaseInElastic dips below 0 before snapping to 1 - characteristic overshoot.
        let f = EasingFunction::EaseInElastic;
        let min = (1..50)
            .map(|i| f.evaluate(i as f32 / 50.0))
            .fold(f32::INFINITY, f32::min);
        assert!(min < 0.0, "expected EaseInElastic to dip below 0, min was {min}");
    }
}
