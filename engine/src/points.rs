use crate::{OutOfBoundsError, MAX_RADIUS, MIN_RADIUS};
#[allow(unused)]
use micromath::F32Ext;

/// Represents points in a cartesian space. `x` and `y` are in milimeters.
#[derive(Debug, PartialEq)]
pub struct PointCartesian {
    pub x: f32,
    pub y: f32,
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

/// Represents points in a polar space. `radius` is in milimeters and `theta` is
/// in degrees.
#[derive(Debug, PartialEq, Clone)]
pub struct PointPolar {
    pub radius: f32,
    pub theta: f32,
}

impl PointPolar {
    pub fn try_new(radius: f32, theta: f32) -> Result<Self, OutOfBoundsError> {
        if radius > MAX_RADIUS {
            Err(OutOfBoundsError::AboveMaximumRadius { radius, theta })
        } else if radius < MIN_RADIUS {
            Err(OutOfBoundsError::BelowMinimumRadius { radius, theta })
        } else {
            Ok(Self { radius, theta })
        }
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
}
