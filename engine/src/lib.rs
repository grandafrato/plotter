#![no_std]

#[allow(unused)]
use micromath::F32Ext;

pub const MIN_RADIUS: f32 = 14.0;
pub const MAX_RADIUS: f32 = 31.0;

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
    pub fn to_polar(&self) -> Result<PointPolar, CoordinateOutOfBoundsError> {
        let radius = self.x.hypot(self.y);
        let mut theta = self.y.atan2(self.x).to_degrees();

        if theta.is_sign_negative() {
            theta += 360.0;
        }

        PointPolar::try_new(radius, theta)
    }
}

/// Returned if a coordinate is below `MIN_RADIUS` or above `MAX_RADIUS`.
#[derive(Debug, PartialEq)]
pub enum CoordinateOutOfBoundsError {
    BelowMinimumRadius(f32),
    AboveMaximumRadius(f32),
}

/// Represents points in a polar space. `radius` is in milimeters and `theta` is
/// in degrees.
#[derive(Debug, PartialEq)]
pub struct PointPolar {
    radius: f32,
    theta: f32,
}

impl PointPolar {
    fn try_new(radius: f32, theta: f32) -> Result<Self, CoordinateOutOfBoundsError> {
        if radius > MAX_RADIUS {
            Err(CoordinateOutOfBoundsError::AboveMaximumRadius(radius))
        } else if radius < MIN_RADIUS {
            Err(CoordinateOutOfBoundsError::BelowMinimumRadius(radius))
        } else {
            Ok(Self { radius, theta })
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Shape<'a> {
    Arc {
        point: PointPolar,
        distance: Distance,
    },
    Polygon(&'a [PointPolar]),
}

impl<'a> Shape<'a> {
    pub fn circle(radius: f32) -> Result<Self, CoordinateOutOfBoundsError> {
        let point = PointPolar::try_new(radius, 0.0)?;

        Ok(Self::Arc {
            point,
            distance: Distance::Full,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Distance {
    Full,
    Partial(f32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_point_cartesian() {
        let point = PointCartesian::new(2.0, 1.0);
        assert_eq!(point, PointCartesian { x: 2.0, y: 1.0 });
    }

    #[test]
    fn test_cartesian_to_polar() -> Result<(), CoordinateOutOfBoundsError> {
        let points = [
            PointCartesian::new(15.0, 0.0),
            PointCartesian::new(0.0, 15.0),
            PointCartesian::new(-15.0, 0.0),
            PointCartesian::new(0.0, -15.0),
        ];

        assert_eq!(
            points[0].to_polar()?,
            PointPolar {
                radius: 15.0,
                theta: 0.0
            }
        );
        assert_eq!(
            points[1].to_polar()?,
            PointPolar {
                radius: 15.0,
                theta: 90.0
            }
        );
        assert_eq!(
            points[2].to_polar()?,
            PointPolar {
                radius: 15.0,
                theta: 180.0
            }
        );
        assert_eq!(
            points[3].to_polar()?,
            PointPolar {
                radius: 15.0,
                theta: 270.0
            }
        );

        Ok(())
    }

    #[test]
    fn test_min_radius() {
        match PointPolar::try_new(MIN_RADIUS - 1.0, 0.0) {
            Err(CoordinateOutOfBoundsError::BelowMinimumRadius(_)) => (),
            _ => panic!("Radius isn't below minimum!"),
        }
        match PointCartesian::new(MIN_RADIUS / 2.0, MIN_RADIUS / 2.0).to_polar() {
            Err(CoordinateOutOfBoundsError::BelowMinimumRadius(_)) => (),
            _ => panic!("Radius isn't below minimum!"),
        }
    }

    #[test]
    fn test_max_radius() {
        match PointPolar::try_new(MAX_RADIUS + 1.0, 0.0) {
            Err(CoordinateOutOfBoundsError::AboveMaximumRadius(_)) => (),
            _ => panic!("Radius isn't above maximum!"),
        }
        match PointCartesian::new(MAX_RADIUS, MAX_RADIUS).to_polar() {
            Err(CoordinateOutOfBoundsError::AboveMaximumRadius(_)) => (),
            _ => panic!("Radius isn't above maximum!"),
        }
    }

    #[test]
    fn test_make_circle() -> Result<(), CoordinateOutOfBoundsError> {
        let radius = MIN_RADIUS + 7.0;
        assert_eq!(
            Shape::circle(radius)?,
            Shape::Arc {
                point: PointPolar::try_new(radius, 0.0)?,
                distance: Distance::Full
            }
        );

        Ok(())
    }
}
