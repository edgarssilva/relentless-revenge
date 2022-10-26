use bevy::{
    math::Vec3Swizzles,
    prelude::{Commands, Component, Entity, Query, Res, Transform, Vec2},
    time::Time,
};

#[allow(dead_code)]
pub enum EaseFunction {
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

#[derive(Component)]
pub struct EaseTo {
    pub position: Vec2,
    pub function: EaseFunction,
    pub duration: f32,
    pub(crate) elapsed: f32,
    pub(crate) start: Option<Vec2>,
}

impl EaseTo {
    pub fn new(position: Vec2, function: EaseFunction, duration: f32) -> Self {
        Self {
            position,
            function,
            duration,
            elapsed: 0.0,
            start: None,
        }
    }
}

pub fn ease_to_position(
    mut query: Query<(&mut Transform, &mut EaseTo, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut transform, mut ease, entity) in query.iter_mut() {
        if ease.start.is_none() {
            ease.start = Some(transform.translation.xy());
        }

        let percentage = ease.elapsed / ease.duration;

        transform.translation = ease
            .start
            .expect("EaseTo start position was not set")
            .lerp(ease.position, ease_function(percentage, &ease.function))
            .extend(transform.translation.z);

        ease.elapsed += time.delta_seconds();

        //TODO: Make the threshold value configurable
        if transform.translation.xy().distance(ease.position) < 1.5 {
            transform.translation = ease.position.extend(transform.translation.z);
            commands.entity(entity).remove::<EaseTo>();
        }
    }
}

fn ease_function(x: f32, function: &EaseFunction) -> f32 {
    match function {
        EaseFunction::EaseInSine => ease_in_sine(x),
        EaseFunction::EaseOutSine => ease_out_sine(x),
        EaseFunction::EaseInOutSine => ease_in_out_sine(x),
        EaseFunction::EaseInQuad => ease_in_quad(x),
        EaseFunction::EaseOutQuad => ease_out_quad(x),
        EaseFunction::EaseInOutQuad => ease_in_out_quad(x),
        EaseFunction::EaseInCubic => ease_in_cubic(x),
        EaseFunction::EaseOutCubic => ease_out_cubic(x),
        EaseFunction::EaseInOutCubic => ease_in_out_cubic(x),
        EaseFunction::EaseInQuart => ease_in_quart(x),
        EaseFunction::EaseOutQuart => ease_out_quart(x),
        EaseFunction::EaseInOutQuart => ease_in_out_quart(x),
        EaseFunction::EaseInQuint => ease_in_quint(x),
        EaseFunction::EaseOutQuint => ease_out_quint(x),
        EaseFunction::EaseInOutQuint => ease_in_out_quint(x),
        EaseFunction::EaseInExpo => ease_in_expo(x),
        EaseFunction::EaseOutExpo => ease_out_expo(x),
        EaseFunction::EaseInOutExpo => ease_in_out_expo(x),
        EaseFunction::EaseInCirc => ease_in_circ(x),
        EaseFunction::EaseOutCirc => ease_out_circ(x),
        EaseFunction::EaseInOutCirc => ease_in_out_circ(x),
        EaseFunction::EaseInBack => ease_in_back(x),
        EaseFunction::EaseOutBack => ease_out_back(x),
        EaseFunction::EaseInOutBack => ease_in_out_back(x),
        EaseFunction::EaseInElastic => ease_in_elastic(x),
        EaseFunction::EaseOutElastic => ease_out_elastic(x),
        EaseFunction::EaseInOutElastic => ease_in_out_elastic(x),
        EaseFunction::EaseInBounce => ease_in_bounce(x),
        EaseFunction::EaseOutBounce => ease_out_bounce(x),
        EaseFunction::EaseInOutBounce => ease_in_out_bounce(x),
    }
}

fn ease_in_sine(x: f32) -> f32 {
    1. - (1. - x).cos()
}

fn ease_out_sine(x: f32) -> f32 {
    x.sin()
}

fn ease_in_out_sine(x: f32) -> f32 {
    -(0.5 * (x * std::f32::consts::PI - 0.5).cos())
}

fn ease_in_quad(x: f32) -> f32 {
    x * x
}

fn ease_out_quad(x: f32) -> f32 {
    1. - (1. - x) * (1. - x)
}

fn ease_in_out_quad(x: f32) -> f32 {
    if x < 0.5 {
        2. * x * x
    } else {
        1. - (-2. * x + 2.).powi(2) / 2.
    }
}

fn ease_in_cubic(x: f32) -> f32 {
    x * x * x
}

fn ease_out_cubic(x: f32) -> f32 {
    1. - (1. - x).powi(3)
}

fn ease_in_out_cubic(x: f32) -> f32 {
    if x < 0.5 {
        4. * x * x * x
    } else {
        1. - (-2. * x + 2.).powi(3) / 2.
    }
}

fn ease_in_quart(x: f32) -> f32 {
    x * x * x * x
}

fn ease_out_quart(x: f32) -> f32 {
    1. - (1. - x).powi(4)
}

fn ease_in_out_quart(x: f32) -> f32 {
    if x < 0.5 {
        8. * x * x * x * x
    } else {
        1. - (-2. * x + 2.).powi(4) / 2.
    }
}

fn ease_in_quint(x: f32) -> f32 {
    x * x * x * x * x
}

fn ease_out_quint(x: f32) -> f32 {
    1. - (1. - x).powi(5)
}

fn ease_in_out_quint(x: f32) -> f32 {
    if x < 0.5 {
        16. * x * x * x * x * x
    } else {
        1. - (-2. * x + 2.).powi(5) / 2.
    }
}

fn ease_in_expo(x: f32) -> f32 {
    if x == 0. {
        0.
    } else {
        2f32.powf(10. * x - 10.)
    }
}

fn ease_out_expo(x: f32) -> f32 {
    if x == 1. {
        1.
    } else {
        1. - 2f32.powf(-10. * x)
    }
}

fn ease_in_out_expo(x: f32) -> f32 {
    if x == 0. {
        0.
    } else if x == 1. {
        1.
    } else if x < 0.5 {
        2f32.powf(20. * x - 10.) / 2.
    } else {
        (2. - 2f32.powf(-20. * x + 10.)) / 2.
    }
}

fn ease_in_circ(x: f32) -> f32 {
    1. - (1. - x * x).sqrt()
}

fn ease_out_circ(x: f32) -> f32 {
    (1. - (1. - x).powi(2)).sqrt()
}

fn ease_in_out_circ(x: f32) -> f32 {
    if x < 0.5 {
        (1. - (1. - 2. * x).powi(2)).sqrt() / 2.
    } else {
        (1. + (2. * x - 1.).powi(2)).sqrt() / 2.
    }
}

fn ease_in_back(x: f32) -> f32 {
    2.70158 * x * x * x - 1.70158 * x * x
}

fn ease_out_back(x: f32) -> f32 {
    1. - (2.70158 * (1. - x) * (1. - x) * (1. - x) - 1.70158 * (1. - x) * (1. - x))
}

fn ease_in_out_back(x: f32) -> f32 {
    if x < 0.5 {
        2. * (2.70158 * x * x * x - 1.70158 * x * x)
    } else {
        1. - 2. * (2.70158 * (1. - x) * (1. - x) * (1. - x) - 1.70158 * (1. - x) * (1. - x))
    }
}

fn ease_in_elastic(x: f32) -> f32 {
    if x == 0. {
        0.
    } else if x == 1. {
        1.
    } else {
        -2f32.powf(10. * x - 10.) * (2. * std::f32::consts::PI * (x - 0.1) / 0.4).sin()
    }
}

fn ease_out_elastic(x: f32) -> f32 {
    if x == 0. {
        0.
    } else if x == 1. {
        1.
    } else {
        2f32.powf(-10. * x) * (2. * std::f32::consts::PI * (x - 0.1) / 0.4).sin() + 1.
    }
}

fn ease_in_out_elastic(x: f32) -> f32 {
    if x == 0. {
        0.
    } else if x == 1. {
        1.
    } else if x < 0.5 {
        -2f32.powf(20. * x - 10.) * (2. * std::f32::consts::PI * (20. * x - 11.5) / 4.).sin() / 2.
    } else {
        2f32.powf(-20. * x + 10.) * (2. * std::f32::consts::PI * (20. * x - 11.5) / 4.).sin() / 2.
            + 1.
    }
}

fn ease_in_bounce(x: f32) -> f32 {
    1. - ease_out_bounce(1. - x)
}

fn ease_out_bounce(x: f32) -> f32 {
    if x < 4. / 11. {
        (121. * x * x) / 16.
    } else if x < 8. / 11. {
        (363. / 40. * x * x) - (99. / 10. * x) + 17. / 5.
    } else if x < 9. / 10. {
        (4356. / 361. * x * x) - (35442. / 1805. * x) + 16061. / 1805.
    } else {
        (54. / 5. * x * x) - (513. / 25. * x) + 268. / 25.
    }
}

fn ease_in_out_bounce(x: f32) -> f32 {
    if x < 0.5 {
        (1. - ease_out_bounce(1. - 2. * x)) / 2.
    } else {
        (1. + ease_out_bounce(2. * x - 1.)) / 2.
    }
}

// Ease in with time startvalue and change
