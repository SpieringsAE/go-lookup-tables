use std::{fmt,
        ops::{
            Add,
            Sub,
            Mul,
            Div,
            Neg,
        },
        cmp::PartialOrd};

#[derive(Debug, Clone)]
/// Something went wrong with extrapolating, either NoneError was set or the lookuptable is not set up correctly
pub struct ExtrapolationError;

impl fmt::Display for ExtrapolationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Either index was out of bounds with the NoneError extrapolation method or the lookup table has no values")
    }
}

/// Extrapolation methods for lookup tables
pub enum Extrapolation {
    /// Error if the entered breakpoint exceeds the values in the lookup tables' breakpoints vector.
    NoneError,
    /// Hold the value at the first or last breakpoint in the lookup table if the entered breakpoint is not in the range of the breakpoints vector.
    NoneHoldExtreme,
    /// Extrapolate the result using the slope of the last or first 2 breakpoint-value pairs.
    Linear,
}

/// Interpolation methods for lookup tables
pub enum Interpolation {
    /// Interpolate the result using the slope of the 2 breakpoint-value pairs that the entered breakpoint falls between. Worst for speed but best precision.
    Linear,
    /// Don't interpolate, always rounds down to previous value. Good for speed bad for precision.
    NoneFloor,
    /// Don't interpolate, always rounds up to the next value. Good for speed bad for precision.
    NoneCeiling,
    /// Don't interpolate, rounds to the nearest value. Kind of bad for speed better for precision.
    NoneClosest,
}

/// A struct representing a 1-D lookup table, breakpoints must be an ascending vector! 1,2,3,4 and not 4,3,2,1 or 1,2,3,2
pub struct OneDLookup <
T: PartialOrd + Sub + Add + Copy + Clone,
U: Add + Sub + Copy + Clone>{
    /// The breakpoints that act as the index for the values.
    breakpoints: Vec<T>,
    /// The values that represent the result from the lookup.
    values: Vec<U>,
    /// constant value for the lookup table so it only has to be calculated at initialisation, instead of every function call.\
    /// represents the delta between the last two breakpoints
    last_diff_bp: T,
    /// constant value for the lookup table so it only has to be calculated at initialisation, instead of every function call.\
    /// represents the delta between the last two values
    last_diff_values: U,
    /// constant value for the lookup table so it only has to be calculated at initialisation, instead of every function call.\
    /// represents the delta between the first two breakpoints
    first_diff_bp: T,
    /// constant value for the lookup table so it only has to be calculated at initialisation, instead of every function call.\
    /// represents the delta between the first two values
    first_diff_values: U,
}


impl<
T: PartialOrd + Add + Copy + Clone + Sub<Output = T> + Div<Output = T>, 
U: Sub<Output = U>  + Add<Output = U> + Copy + Clone + From<T> + Mul<Output = U> + Div<Output = U> + Neg<Output = U>
>
OneDLookup<T,U>
where f64: From<T> + From<U>{
    /// Returns a (interpolated) value from the lookup table that matches the entered breakpoint.
    /// 
    /// # Arguments
    /// 
    /// * `breakpoint` - A reference to the breakpoint for which a value must be found by the lookup table
    /// * `extrapolation` - The extrapolation method to use for this lookup operation
    /// * `interpolation` - The interpolation method to use for this lookup operation
    /// 
    /// # Examples
    /// 
    /// ```
    /// # #[macro_use] extern crate go_lookup_tables; fn main() {
    /// use::go_lookup_tables::{OneDLookup, Interpolation, Extrapolation};
    /// let measured_voltage = 2000i16;
    /// let lookup_table = create_1d_lookup!((0i16,500,4500,5000), (0f32,0.0,500.0,500.0));//simple 0.5V to 4.5V pressure sensor
    /// let pressure = lookup_table.lookup(&measured_voltage, Extrapolation::NoneHoldExtreme, Interpolation::Linear).unwrap();
    /// # }
    /// ```
    pub fn lookup<Y: Copy>(&self, breakpoint: &Y, extrapolation: Extrapolation, interpolation: Interpolation) -> Result<U, ExtrapolationError>
    where T: From<Y> + From<i8>{
        let calc_breakpoint = T::from(*breakpoint);
        match self.breakpoints.iter().position(|bp| bp > &calc_breakpoint){ 
            Some(index) => {
                if index != 0 {
                    // handle interpolation
                    return match interpolation {
                        Interpolation::Linear => {
                            let interpolated_diff_bp = calc_breakpoint - *self.breakpoints.get(index -1).unwrap();
                            let diff_actual_bp = *self.breakpoints.get(index).unwrap() - *self.breakpoints.get(index-1).unwrap();
                            let diff_values = *self.values.get(index).unwrap() - *self.values.get(index-1).unwrap();
                            Ok((U::from(interpolated_diff_bp) * diff_values) / U::from(diff_actual_bp) + *self.values.get(index-1).unwrap())
                        },
                        Interpolation::NoneCeiling => {Ok(*self.values.get(index).unwrap())},
                        Interpolation::NoneFloor => {Ok(*self.values.get(index-1).unwrap())},
                        Interpolation::NoneClosest => {
                            let interpolated_diff_bp = calc_breakpoint - *self.breakpoints.get(index -1).unwrap();
                            let diff_actual_bp = *self.breakpoints.get(index).unwrap() - *self.breakpoints.get(index-1).unwrap();
                            let diff_factor = diff_actual_bp - interpolated_diff_bp;
                            let round: usize = if diff_factor > (diff_actual_bp/T::from(2i8))
                                {
                                0
                            } else {
                                1
                            };
                            Ok(*self.values.get(index-1 + round).unwrap())
                        },
                    }
                }
                // handle extrapolation at the low end
                match extrapolation {
                    Extrapolation::NoneError => Err(ExtrapolationError),
                    Extrapolation::NoneHoldExtreme => Ok(*self.values.first().unwrap()),
                    Extrapolation::Linear => {
                        let extrapolated_diff_bp = *self.breakpoints.get(1).unwrap() - calc_breakpoint;
                        Ok((U::from(extrapolated_diff_bp) * -self.first_diff_values) / U::from(self.first_diff_bp) + *self.values.get(1).unwrap())
                    }
                }
            }
            None => match extrapolation {
            // handle extrapolation at the high end
                Extrapolation::NoneError => Err(ExtrapolationError),
                Extrapolation::NoneHoldExtreme => Ok(*self.values.last().unwrap()),
                Extrapolation::Linear => {
                    let extrapolated_diff_bp: T = calc_breakpoint - *self.breakpoints.get(self.breakpoints.len()-2).unwrap();
                    Ok((U::from(extrapolated_diff_bp) * self.last_diff_values) / U::from(self.last_diff_bp) + *self.values.get(self.values.len()-2).unwrap())
                }
            }
        }
    }
    /// This method is unsafe, consider using the create_1d_lookup!() macro instead.
    /// Returns a lookup table. Only use an ascending breakpoints vector! for example  1,2,3,4 and not 4,3,2,1 or 1,2,3,2 \
    /// breakpoints and values must have the same length!
    /// 
    /// # Arguments
    /// 
    /// * `breakpoints` - The breakpoints that act as the index for the values
    /// * `values` - The values that represent the result from the lookup
    /// 
    /// # Examples
    /// 
    /// ```
    /// use::go_lookup_tables::{OneDLookup};
    /// let lookup_table = OneDLookup::new(vec![0i16,500,4500,5000], vec![0f32,0.0,500.0,500.0]); //simple 0.5V to 4.5V pressure sensor
    /// ```
    pub fn new(breakpoints: Vec<T>, values: Vec<U>) -> OneDLookup<T,U> {
        OneDLookup {
            last_diff_bp: *breakpoints.last().unwrap() - *breakpoints.get(breakpoints.len()-2).unwrap(),
            last_diff_values: *values.last().unwrap() - *values.get(values.len()-2).unwrap(),
            first_diff_bp: *breakpoints.get(1).unwrap() - *breakpoints.first().unwrap(),
            first_diff_values: *values.get(1).unwrap() - *values.first().unwrap(),
            breakpoints: breakpoints,
            values: values,
        }
    }
}

/// Returns a lookup table. Only use an ascending breakpoints vector! for example  1,2,3,4 and not 4,3,2,1 or 1,2,3,2 \
/// breakpoints and values must have the same length!
/// 
/// # Arguments
/// 
/// * `breakpoints` - The breakpoints that act as the index for the values
/// * `values` - The values that represent the result from the lookup
/// 
/// # Panics
///
/// `create_1d_lookup!` panics if breakpoints is not in ascending order or if breakpoints.len() != values.len().
/// This panic is generated at compile time.
/// 
/// # Examples
/// 
/// ```
/// # #[macro_use] extern crate go_lookup_tables; fn main() {
/// use::go_lookup_tables::*;
/// let lookup_table = create_1d_lookup!((0u16,500,4500,5000), (0f64,0.0,500.0,500.0)); //simple 0.5V to 4.5V pressure sensor
/// # }
/// ```
#[macro_export]
macro_rules! create_1d_lookup {
    (($($bps:expr),*), ($($vals:expr),*)) => {{
        const _: () = {
            if [ $($bps,)* ].len() != [ $($vals,)* ].len() {
                panic!("lengths of breakpoints and values don't match");
            }

            let mut i = 1;
            while i < [ $($bps,)* ].len() {
                if [ $($bps,)* ][i - 1] > [ $($bps,)* ][i] {
                    panic!("breakpoints aren't sorted, they should be in ascending order");
                }
                i += 1;
            }
        };
        OneDLookup::new(vec![$($bps),+], vec![$($vals),+])
    }};
}

#[cfg(test)]
mod tests {
    use super::OneDLookup;

    #[test]
    fn linear_extrapolation_signed() {
        let lookup_table = create_1d_lookup!((0i16,5000),(0f32,500.0));
        let result = lookup_table.lookup(&2500i16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
        let result1 = lookup_table.lookup(&-1000i16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
        let result2 = lookup_table.lookup(&6000i16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
        println!("sensor is an automotive 0 - 5V 0 - 500 bar pressure sensor");
        println!("linear extrapolation and interpolation enabled");
        println!("Result at 2.5V is : {}bar", result);
        assert_eq!(result, 250f32);
        println!("Result at -1V is:   {}bar", result1);
        assert_eq!(result1, -100f32);
        println!("Result at 6V is:    {}bar", result2);
        assert_eq!(result2, 600f32);
    }
    #[test]
    fn linear_extrapolation_unsigned() {
        let lookup_table = create_1d_lookup!((1000i16,5000),(0f64,500.0));
        let result = lookup_table.lookup(&3000i16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
        let result1 = lookup_table.lookup(&500i16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
        let result2 = lookup_table.lookup(&6000i16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
        println!("sensor is an automotive 1 - 5V 0 - 500 bar pressure sensor");
        println!("linear extrapolation and interpolation enabled");
        println!("Result at 3V is :    {}bar", result);
        assert_eq!(result, 250f64);
        println!("Result at 0.5V is:   {}bar", result1);
        assert_eq!(result1, -62.5f64);
        println!("Result at 6V is:     {}bar", result2);
        assert_eq!(result2, 625f64);
    }

    #[test]
    fn interpolation_closest() {
        let lookup_table = create_1d_lookup!((0i8,1,5,6),(0i8,1,6,8));
        let result = lookup_table.lookup(&3i8, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::NoneClosest).unwrap();
        let result1 = lookup_table.lookup(&4i8, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::NoneClosest).unwrap();
        let result2 = lookup_table.lookup(&2i8, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::NoneClosest).unwrap();
        assert_eq!(result, 6i8); //value is 3.5, round up to 6
        assert_eq!(result1, 6i8); //value is 4.75, round up to 6
        assert_eq!(result2, 1i8); //value is 2.25, round down to 1
    }
}
