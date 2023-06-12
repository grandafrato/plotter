#![no_std]

#[allow(unused)]
use micromath::F32Ext;

pub const MIN_RADIUS: f32 = 14.0;
pub const MAX_RADIUS: f32 = 31.0;
pub const MID_RADIUS: f32 = MIN_RADIUS + (MAX_RADIUS - MIN_RADIUS) / 2.0;
pub const RESOLUTION: usize = 1000;

/// Represents points in a cartesian space. `x` and `y` are in milimeters.
#[derive(Debug, PartialEq)]
pub struct PointCartesian {
    x: f32,
    y: f32,
}

impl PointCartesian {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Converts the cratesian point to an equivalent polar point.
    pub fn as_polar(&self) -> Result<PointPolar, OutOfBoundsError> {
        let radius = self.x.hypot(self.y);
        let theta = self.y.atan2(self.x);

        PointPolar::try_new(radius, theta)
    }
}

/// Returned if a coordinate is below `MIN_RADIUS` or above `MAX_RADIUS`.
#[derive(Debug, PartialEq)]
pub enum OutOfBoundsError {
    BelowMinimumRadius { radius: f32, theta: f32 },
    AboveMaximumRadius { radius: f32, theta: f32 },
    CrossesRotationMax,
    CrossesDeadZone(f32),
}

/// Represents points in a polar space. `radius` is in milimeters and `theta` is
/// in degrees.
#[derive(Debug, PartialEq, Clone)]
pub struct PointPolar {
    radius: f32,
    theta: f32,
}

impl PointPolar {
    fn try_new(radius: f32, theta: f32) -> Result<Self, OutOfBoundsError> {
        if radius > MAX_RADIUS {
            Err(OutOfBoundsError::AboveMaximumRadius { radius, theta })
        } else if radius < MIN_RADIUS {
            Err(OutOfBoundsError::BelowMinimumRadius { radius, theta })
        } else {
            Ok(Self { radius, theta })
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Shape<'a> {
    Arc {
        point: PointPolar,
        rotation: Rotation,
    },
    Polygon(&'a [PointCartesian]),
}

impl<'a> Shape<'a> {
    /// Returns an arc with a starting point of zero theta.
    pub fn circle(radius: f32) -> Result<Self, OutOfBoundsError> {
        let point = PointPolar::try_new(radius, 0.0)?;

        Ok(Self::Arc {
            point,
            rotation: Rotation::Full,
        })
    }

    pub fn arc(point: PointPolar, arc_length: f32) -> Result<Self, OutOfBoundsError> {
        let angle = (arc_length / point.radius).to_degrees() + point.theta;

        if angle > 360.0 {
            Err(OutOfBoundsError::CrossesRotationMax)
        } else {
            Ok(Self::Arc {
                point,
                rotation: Rotation::Partial(angle),
            })
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Rotation {
    Full,
    Partial(f32),
}

#[derive(Debug, PartialEq)]
pub struct Segment(PointCartesian, PointCartesian);

impl Segment {
    pub fn try_new(
        point_a: PointCartesian,
        point_b: PointCartesian,
    ) -> Result<Self, OutOfBoundsError> {
        Self(point_a, point_b)
            .check_rotation_max()?
            .check_dead_zone()
    }

    fn check_rotation_max(self) -> Result<Self, OutOfBoundsError> {
        let (point_a, point_b) = (&self.0, &self.1);

        if !point_a.x.is_sign_negative() && !point_b.x.is_sign_negative() {
            if point_a.y.is_sign_negative() ^ point_b.y.is_sign_negative() {
                return Err(OutOfBoundsError::CrossesRotationMax);
            }
        }

        Ok(self)
    }

    fn check_dead_zone(self) -> Result<Self, OutOfBoundsError> {
        let (point_a, point_b) = (&self.0, &self.1);

        let distance = ((point_a.x - point_b.x) * point_a.y + (point_b.y - point_a.y) * point_a.x)
            .abs()
            / ((point_b.x - point_a.x).powi(2) + (point_a.y - point_b.y).powi(2)).sqrt();

        if distance < MIN_RADIUS {
            return Err(OutOfBoundsError::CrossesDeadZone(distance));
        }

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::PI;

    #[test]
    fn test_new_point_cartesian() {
        let point = PointCartesian::new(2.0, 1.0);
        assert_eq!(point, PointCartesian { x: 2.0, y: 1.0 });
    }

    #[test]
    fn test_cartesian_to_polar() -> Result<(), OutOfBoundsError> {
        let points = [
            PointCartesian::new(15.0, 0.0),
            PointCartesian::new(0.0, 15.0),
            PointCartesian::new(-15.0, 0.0),
            PointCartesian::new(0.0, -15.0),
        ];

        assert_eq!(
            points[0].as_polar()?,
            PointPolar {
                radius: 15.0,
                theta: 0.0
            }
        );
        assert_eq!(
            points[1].as_polar()?,
            PointPolar {
                radius: 15.0,
                theta: 0.5 * PI,
            }
        );
        assert_eq!(
            points[2].as_polar()?,
            PointPolar {
                radius: 15.0,
                theta: PI,
            }
        );
        assert_eq!(
            points[3].as_polar()?,
            PointPolar {
                radius: 15.0,
                theta: -0.5 * PI,
            }
        );

        Ok(())
    }

    #[test]
    fn test_min_radius() {
        match PointPolar::try_new(MIN_RADIUS - 1.0, 0.0) {
            Err(OutOfBoundsError::BelowMinimumRadius { .. }) => (),
            _ => panic!("Radius isn't below minimum!"),
        }
        match PointCartesian::new(MIN_RADIUS / 2.0, MIN_RADIUS / 2.0).as_polar() {
            Err(OutOfBoundsError::BelowMinimumRadius { .. }) => (),
            _ => panic!("Radius isn't below minimum!"),
        }
    }

    #[test]
    fn test_max_radius() {
        match PointPolar::try_new(MAX_RADIUS + 1.0, 0.0) {
            Err(OutOfBoundsError::AboveMaximumRadius { .. }) => (),
            _ => panic!("Radius isn't above maximum!"),
        }
        match PointCartesian::new(MAX_RADIUS, MAX_RADIUS).as_polar() {
            Err(OutOfBoundsError::AboveMaximumRadius { .. }) => (),
            _ => panic!("Radius isn't above maximum!"),
        }
    }

    #[test]
    fn test_make_circle() -> Result<(), OutOfBoundsError> {
        let radius = MIN_RADIUS + 7.0;
        assert_eq!(
            Shape::circle(radius)?,
            Shape::Arc {
                point: PointPolar::try_new(radius, 0.0)?,
                rotation: Rotation::Full
            }
        );

        Ok(())
    }

    #[test]
    fn test_make_arc() -> Result<(), OutOfBoundsError> {
        let point = PointPolar::try_new(MIN_RADIUS, 2.0)?;
        assert_eq!(
            Shape::arc(point.clone(), PI * MIN_RADIUS)?,
            Shape::Arc {
                point,
                rotation: Rotation::Partial(182.0)
            }
        );

        Ok(())
    }

    #[test]
    fn test_new_segment() -> Result<(), OutOfBoundsError> {
        let segment = Segment::try_new(
            PointCartesian::new(MID_RADIUS, 0.0),
            PointCartesian::new(0.0, MID_RADIUS),
        )?;

        assert_eq!(
            segment,
            Segment(
                PointCartesian {
                    x: MID_RADIUS,
                    y: 0.0
                },
                PointCartesian {
                    x: 0.0,
                    y: MID_RADIUS
                }
            )
        );

        Ok(())
    }

    #[test]
    fn test_new_segment_out_of_bounds() {
        let segment_crosses_max_rot = Segment::try_new(
            PointCartesian::new(MID_RADIUS, 1.0),
            PointCartesian::new(MID_RADIUS, -1.0),
        );
        let segment_crosses_dead_zone = Segment::try_new(
            PointCartesian::new(-MID_RADIUS, MIN_RADIUS - 1.0),
            PointCartesian::new(MID_RADIUS, MIN_RADIUS - 1.0),
        );

        assert_eq!(
            segment_crosses_max_rot,
            Err(OutOfBoundsError::CrossesRotationMax)
        );
        match segment_crosses_dead_zone {
            Err(OutOfBoundsError::CrossesDeadZone(_)) => (),
            Ok(_) => panic!("Doesn't cross dead zone."),
            _ => panic!("Other failure."),
        }
    }
}
