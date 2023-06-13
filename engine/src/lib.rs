#![no_std]

use crate::points::{PointCartesian, PointPolar};
use core::f32::consts::PI;

#[allow(unused)]
use micromath::F32Ext;

pub const MIN_RADIUS: f32 = 14.0;
pub const MAX_RADIUS: f32 = 31.0;
pub const MID_RADIUS: f32 = MIN_RADIUS + (MAX_RADIUS - MIN_RADIUS) / 2.0;

pub mod points;

/// Returned if a coordinate is out of bounds.
#[derive(Debug, PartialEq)]
pub enum OutOfBoundsError {
    BelowMinimumRadius { radius: f32, theta: f32 },
    AboveMaximumRadius { radius: f32, theta: f32 },
    CrossesRotationMax,
    CrossesDeadZone(f32),
}

#[derive(Debug, PartialEq)]
pub enum Shape<'a> {
    CenterArc {
        point: PointPolar,
        rotation: Rotation,
    },
    Polygon(&'a [PointCartesian]),
}

impl<'a> Shape<'a> {
    /// Returns an arc with a starting point of zero theta.
    pub fn circle(radius: f32) -> Result<Self, OutOfBoundsError> {
        let point = PointPolar::try_new(radius, 0.0)?;

        Ok(Self::CenterArc {
            point,
            rotation: Rotation::Full,
        })
    }

    pub fn center_arc(point: PointPolar, arc_length: f32) -> Result<Self, OutOfBoundsError> {
        let angle = arc_length + point.theta;

        if angle > 2.0 * PI {
            Err(OutOfBoundsError::CrossesRotationMax)
        } else {
            Ok(Self::CenterArc {
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

    pub fn step(&self, distance: f32) -> Option<PointPolar> {
        let step_distance = self.distance();
        if !(0.0..=step_distance).contains(&distance) {
            return None;
        }
        let (point_a, point_b) = (&self.0, &self.1);

        let x = point_a.x - distance * (point_a.x - point_b.x) / step_distance;
        let y = point_a.y - distance * (point_a.y - point_b.y) / step_distance;

        // Points within segment are assumed to have been checked for being within bounds.
        Some(PointCartesian::new(x, y).as_polar().unwrap())
    }

    fn distance(&self) -> f32 {
        let (point_a, point_b) = (&self.0, &self.1);

        ((point_b.x - point_a.x).powi(2) + (point_b.y - point_a.y).powi(2)).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::points::{PointCartesian, PointPolar};

    #[test]
    fn test_make_circle() -> Result<(), OutOfBoundsError> {
        let radius = MIN_RADIUS + 7.0;
        assert_eq!(
            Shape::circle(radius)?,
            Shape::CenterArc {
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
            Shape::center_arc(point.clone(), PI)?,
            Shape::CenterArc {
                point,
                rotation: Rotation::Partial(PI + 2.0)
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

    #[test]
    fn test_segment_distance() -> Result<(), OutOfBoundsError> {
        let segment = Segment::try_new(
            PointCartesian::new(MIN_RADIUS, MIN_RADIUS),
            PointCartesian::new(-MIN_RADIUS, MIN_RADIUS),
        )?;

        assert_eq!(MIN_RADIUS * 2.0, segment.distance());

        Ok(())
    }

    #[test]
    fn test_segment_step() -> Result<(), OutOfBoundsError> {
        let (point_a, point_b) = (
            PointCartesian::new(MIN_RADIUS, MIN_RADIUS),
            PointCartesian::new(-MIN_RADIUS, MIN_RADIUS),
        );
        let (polar_a, polar_b) = (point_a.as_polar()?, point_b.as_polar()?);
        let polar_middle = PointPolar::try_new(MIN_RADIUS, 0.5 * PI)?;
        let segment = Segment::try_new(point_a, point_b)?;

        assert_eq!(Some(polar_a), segment.step(0.0));
        assert_eq!(Some(polar_b), segment.step(MIN_RADIUS * 2.0));
        assert_eq!(Some(polar_middle), segment.step(MIN_RADIUS));
        assert_eq!(None, segment.step(-1.0));
        assert_eq!(None, segment.step(MIN_RADIUS * 2.0 + 1.0));

        Ok(())
    }
}
