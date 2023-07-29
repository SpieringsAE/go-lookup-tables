use std::{fmt,
        ops::{
            Add,
            Sub,
            Mul,
            Div,
            Neg,
        },
        cmp::PartialOrd,
    convert::Infallible};

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
T: PartialOrd + Sub + Add + Div + Copy + Clone,
U: Add + Sub + Div + Mul + Copy + Clone,
const C: usize>{
    /// The breakpoints that act as the index for the values.
    breakpoints: [T;C],
    /// The values that represent the result from the lookup.
    values: [U;C],
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
U: Sub<Output = U>  + Add<Output = U> + Copy + Clone + From<T> + Mul<Output = U> + Div<Output = U> + Neg<Output = U>,
const C: usize
>
OneDLookup<T,U,C>{
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
    /// const LOOKUP_TABLE: OneDLookup<i16,f32,4> = create_1d_lookup!((0,500,4500,5000), (0.0,0.0,500.0,500.0));//simple 0.5V to 4.5V pressure sensor
    /// let pressure = LOOKUP_TABLE.lookup(&measured_voltage, Extrapolation::NoneHoldExtreme, Interpolation::Linear).unwrap();
    /// assert_eq!(pressure, 187.5f32)
    /// # }
    /// ```
    pub fn lookup<Y: Copy>(&self, breakpoint: &Y, extrapolation: Extrapolation, interpolation: Interpolation) -> Result<U, ExtrapolationError>
    where T: From<Y> + From<i8>{
        let calc_breakpoint = T::from(*breakpoint);
        match self.breakpoints.iter().position(|bp| bp >= &calc_breakpoint){ 
            Some(index) => {
                if self.breakpoints[index] == calc_breakpoint {
                    return Ok(self.values[index]) 
                }
                else if index != 0 {
                    // handle interpolation
                    return match interpolation {
                        Interpolation::Linear => {
                            let interpolated_diff_bp = calc_breakpoint - self.breakpoints[index -1];
                            let diff_actual_bp = self.breakpoints[index] - self.breakpoints[index-1];
                            let diff_values = self.values[index] - self.values[index-1];
                            Ok((U::from(interpolated_diff_bp) * diff_values) / U::from(diff_actual_bp) + self.values[index-1])
                        },
                        Interpolation::NoneCeiling => {Ok(*self.values.get(index).unwrap())},
                        Interpolation::NoneFloor => {Ok(*self.values.get(index-1).unwrap())},
                        Interpolation::NoneClosest => {
                            let interpolated_diff_bp = calc_breakpoint - self.breakpoints[index -1];
                            let diff_actual_bp = self.breakpoints[index] - self.breakpoints[index-1];
                            let diff_factor = diff_actual_bp - interpolated_diff_bp;
                            let round: usize = if diff_factor > (diff_actual_bp/T::from(2))
                                {
                                0
                            } else {
                                1
                            };
                            Ok(self.values[index-1 + round])
                        },
                    }
                }
                // handle extrapolation at the low end
                match extrapolation {
                    Extrapolation::NoneError => Err(ExtrapolationError),
                    Extrapolation::NoneHoldExtreme => Ok(self.values[0]),
                    Extrapolation::Linear => {
                        let extrapolated_diff_bp = self.breakpoints[1] - calc_breakpoint;
                        Ok((U::from(extrapolated_diff_bp) * -self.first_diff_values) / U::from(self.first_diff_bp) + self.values[1])
                    }
                }
            }
            None => match extrapolation {
            // handle extrapolation at the high end
                Extrapolation::NoneError => Err(ExtrapolationError),
                Extrapolation::NoneHoldExtreme => Ok(self.values[self.values.len()-1]),
                Extrapolation::Linear => {
                    let extrapolated_diff_bp: T = calc_breakpoint - self.breakpoints[self.breakpoints.len()-2];
                    Ok((U::from(extrapolated_diff_bp) * self.last_diff_values) / U::from(self.last_diff_bp) + self.values[self.values.len()-2])
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
    /// const LOOKUP_TABLE: OneDLookup<i16,f32,4> = OneDLookup::new([0,500,4500,5000], [0.0,0.0,500.0,500.0], 500, 0.0, -500, 0.0); //simple 0.5V to 4.5V pressure sensor
    /// ```
    pub const fn new(breakpoints: [T;C], values: [U;C], last_diff_bp: T, last_diff_values: U, first_diff_bp: T, first_diff_values: U) -> OneDLookup<T,U,C> where [T;C]: Sized, [U;C]: Sized {
        OneDLookup {
            last_diff_bp,
            last_diff_values,
            first_diff_bp,
            first_diff_values,
            breakpoints,
            values,
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
/// const LOOKUP_TABLE: OneDLookup<i16,f32,4> = create_1d_lookup!((0,500,4500,5000), (0.0,0.0,500.0,500.0)); //simple 0.5V to 4.5V pressure sensor
/// # }
/// ```
#[macro_export]
macro_rules! create_1d_lookup {
    (($($bps:expr),*), ($($vals:expr),*)) => {{
        const _: () = {
            let breakpoints = [ $($bps,)* ];
            let values = [ $($vals,)* ];
            if breakpoints.len() != values.len() {
                panic!("lengths of breakpoints and values don't match");
            }

            let mut i = 1;
            while i < breakpoints.len() {
                if breakpoints[i - 1] > breakpoints[i] {
                    panic!("breakpoints aren't sorted, they should be in ascending order");
                }
                // let bp_diff = breakpoints[i] - breakpoints[i - 1];
                // let val_diff = values[i] - values[i - 1];
                // let val = test.to_value(bp_diff) * val_diff;
                i += 1;
            }
        };
        let lookup = OneDLookup::new(
            [$($bps),+],
            [$($vals),+],
            [ $($bps,)* ][[ $($bps,)* ].len()-1] - [ $($bps,)* ][[ $($bps,)* ].len()-2],
            [ $($vals,)* ][[ $($vals,)* ].len()-1] - [ $($vals,)* ][[ $($vals,)* ].len()-2],
            [ $($bps,)* ][1] - [ $($bps,)* ][0],
            [ $($vals,)* ][1] - [ $($vals,)* ][0],
        );

        // let bp_diff = lookup.breakpoints[1] - lookup.breakpoints[0];
        // let val_diff = lookup.values[1] - lookup.values[0];
        // bp_diff.into() * val_diff;

        lookup
    }};
}

/// A struct representing a 2-D lookup table, breakpoints must be an ascending vectors! 1,2,3,4 and not 4,3,2,1 or 1,2,3,2
/// 
/// example:
/// /*
///     x   0   500 1000
///     0   3.0 4.2 5.5
///     3   4.2 5.0 6.0
///     6   5.0 5.8 6.5
/// */
pub struct TwoDLookup<
S: PartialOrd + Sub + Add + Div + Copy + Clone,
T: PartialOrd + Sub + Add + Div + Copy + Clone,
U: Add + Sub + Div + Mul + Copy + Clone,
const N: usize,
const M: usize>{
    ///The horizontal breakpoints
    breakpoints_h: [S;N],
    ///The vertical breakpoints
    breakpoints_v: [T;M],
    ///The values matrix
    values:        [[U;N];M],
}

impl<
S: PartialOrd + Add + Copy + Clone + Sub<Output = S> + Div<Output =S>, 
T: PartialOrd + Add + Copy + Clone + Sub<Output = T> + Div<Output = T>, 
U: Sub<Output = U>  + Add<Output = U> + Copy + Clone + From<T> + From<S> + Mul<Output = U> + Div<Output = U> + Neg<Output = U>,
const N: usize,
const M: usize,
>TwoDLookup<S,T,U,N,M> {
    /// Returns a (interpolated) value from the lookup table that matches the entered breakpoints.
    /// 
    /// # Arguments
    /// 
    /// * `breakpoint_h` - A reference to the horizontal breakpoint for which a value must be found by the lookup table
    /// * `breakpoint_v` - A reference to the vertical breakpoint for which a value must be found by the lookup table
    /// * `interpolation` - The interpolation method to use for this lookup operation
    /// 
    /// # Examples
    /// 
    /// ```
    /// # #[macro_use] extern crate go_lookup_tables; fn main() {
    /// use::go_lookup_tables::{TwoDLookup, Interpolation};
    /// let rpm = 750i16;
    /// let throttle_pos = 4;
    /// const LOOKUP_TABLE: TwoDLookup<i16,i8,f32,3,3> = create_2d_lookup!((0,500,1000),(0,3,6),(
    /// 3.0,4.2,5.5;
    /// 4.2,5.0,6.0;
    /// 5.0,5.8,6.5)); //only a small part of an actual injector table
    /// /* full table:
    ///     x   0   500 1000
    ///     0   3.0 4.2 5.5
    ///     3   4.2 5.0 6.0
    ///     6   5.0 5.8 6.5
    ///  */
    /// let injector_time = LOOKUP_TABLE.lookup(&rpm, &throttle_pos, Interpolation::Linear).unwrap();
    /// assert_eq!(injector_time, 5.7166667f32)
    /// # }
    /// ```
    pub fn lookup<Y: Copy, Z: Copy>(&self, breakpoint_h: &Y, breakpoint_v: &Z, interpolation: Interpolation) -> Result<U, Infallible>
    where S: From<Y> + From<i8>, T: From<Z> + From<i8>, U: From<i8>{
        let calc_breakpoint_h = S::from(*breakpoint_h);
        let calc_breakpoint_v = T::from(*breakpoint_v);
        //get horizontal index which will be used to generate an intermediary array of values
        let indexes_h = match self.breakpoints_h.iter().position(|bp| bp >= &calc_breakpoint_h) {
            Some(index) => {
                //easy exit if bp matches existing bp
                if self.breakpoints_h[index] == calc_breakpoint_h {
                    (index,None)
                //horizontal interpolation zone
                } else if index != 0 {
                    match interpolation {
                        Interpolation::Linear => {
                            (index, Some(index -1))
                        },
                        Interpolation::NoneCeiling => (index,None),
                        Interpolation::NoneFloor => (index-1,None),
                        Interpolation::NoneClosest => {
                            let interpolated_diff_bp_h = calc_breakpoint_h - self.breakpoints_h[index -1];
                            let diff_actual_bp_h = self.breakpoints_h[index] - self.breakpoints_h[index-1];
                            let diff_factor_h = diff_actual_bp_h - interpolated_diff_bp_h;
                            let round: usize = if diff_factor_h > (diff_actual_bp_h/S::from(2))
                                {
                                0
                            } else {
                                1
                            };
                            (index-1+round,None)
                        }
                    }
                } else {
                    //low end out of bounds horizontal
                    (0,None)
                }
            },
            //high end out of bounds horizontal
            None => (self.breakpoints_h.len()-1,None)
        };
        //get the vertical index and calculate the value
        match self.breakpoints_v.iter().position(|bp| bp >= &calc_breakpoint_v) {
            Some(index) => {
                //easy exit if bp matches existing bp
                if self.breakpoints_v[index] == calc_breakpoint_v {
                    match interpolation {
                        Interpolation::Linear => {
                            Ok(self.interpolate(indexes_h, (index,None), calc_breakpoint_h, calc_breakpoint_v))
                        },
                        Interpolation::NoneCeiling | Interpolation::NoneFloor | Interpolation::NoneClosest => Ok(self.values[index][indexes_h.0]),
                    }
                //vertical interpolation zone
                } else if index != 0 {
                    match interpolation {
                        Interpolation::Linear => {
                            Ok(self.interpolate(indexes_h, (index,Some(index-1)), calc_breakpoint_h, calc_breakpoint_v))
                        },
                        Interpolation::NoneCeiling => Ok(self.values[index][indexes_h.0]),
                        Interpolation::NoneFloor => Ok(self.values[index-1][indexes_h.0]),
                        Interpolation::NoneClosest => {
                            let interpolated_diff_bp_v = calc_breakpoint_v - self.breakpoints_v[index -1];
                            let diff_actual_bp_v = self.breakpoints_v[index] - self.breakpoints_v[index-1];
                            let diff_factor_v = diff_actual_bp_v - interpolated_diff_bp_v;
                            let round: usize = if diff_factor_v > (diff_actual_bp_v/T::from(2))
                                {
                                0
                            } else {
                                1
                            };
                            Ok(self.values[index-1 + round][indexes_h.0])
                        }
                    }
                } else {
                    //low end out of bounds vertical
                    match interpolation {                
                        Interpolation::Linear => {
                            Ok(self.interpolate(indexes_h, (0,None), calc_breakpoint_h, calc_breakpoint_v))
                        },
                        Interpolation::NoneCeiling | Interpolation::NoneFloor | Interpolation::NoneClosest => Ok(self.values[0][indexes_h.0]),
                    }
                }
            },
            //high end out of bounds vertical
            None => {
                match interpolation {                
                    Interpolation::Linear => {
                        Ok(self.interpolate(indexes_h, (self.breakpoints_v.len()-1,None), calc_breakpoint_h, calc_breakpoint_v))
                    },
                    Interpolation::NoneCeiling | Interpolation::NoneFloor | Interpolation::NoneClosest => Ok(self.values[self.values.len()-1][indexes_h.0]),
                }
            }
        }
    }

    fn interpolate(&self, indexes_h: (usize,Option<usize>), indexes_v: (usize,Option<usize>), breakpoint_h: S, breakpoint_v: T) -> U {
        let intermediary_values = if indexes_h.1.is_some() && indexes_v.1.is_some() {
            let interpolated_diff_bp_h = breakpoint_h - self.breakpoints_h[indexes_h.1.unwrap()];
            let diff_actual_bp_h = self.breakpoints_h[indexes_h.0] - self.breakpoints_h[indexes_h.1.unwrap()];
            let diff_values_l = self.values[indexes_v.1.unwrap()][indexes_h.0] - self.values[indexes_v.1.unwrap()][indexes_h.1.unwrap()];
            let diff_values_h = self.values[indexes_v.0][indexes_h.0] - self.values[indexes_v.0][indexes_h.1.unwrap()];

            [
                (U::from(interpolated_diff_bp_h) * diff_values_l) / U::from(diff_actual_bp_h) + self.values[indexes_v.1.unwrap()][indexes_h.1.unwrap()],
                (U::from(interpolated_diff_bp_h) * diff_values_h) / U::from(diff_actual_bp_h) + self.values[indexes_v.0][indexes_h.1.unwrap()]
            ]
        } else if indexes_h.1.is_none() && indexes_v.1.is_none() {
            return self.values[indexes_v.0][indexes_h.0]
        } else if indexes_h.1.is_none() {
             [
                self.values[indexes_v.1.unwrap()][indexes_h.0],
                self.values[indexes_v.0][indexes_h.0]
            ]
        } else {
            let interpolated_diff_bp_h = breakpoint_h - self.breakpoints_h[indexes_h.1.unwrap()];
            let diff_actual_bp_h = self.breakpoints_h[indexes_h.0] - self.breakpoints_h[indexes_h.1.unwrap()];
            let diff_values_h = self.values[indexes_v.0][indexes_h.0] - self.values[indexes_v.0][indexes_h.1.unwrap()];
            return (U::from(interpolated_diff_bp_h) * diff_values_h) / U::from(diff_actual_bp_h) + self.values[indexes_v.0][indexes_h.1.unwrap()]
        }; 
        
        let interpolated_diff_bp_v = breakpoint_v - self.breakpoints_v[indexes_v.1.unwrap()];
        let diff_actual_bp_v = self.breakpoints_v[indexes_v.0] - self.breakpoints_v[indexes_v.1.unwrap()];
        (U::from(interpolated_diff_bp_v) * (intermediary_values[1]-intermediary_values[0]))/ U::from(diff_actual_bp_v)+intermediary_values[0]
    }

    /// This method is unsafe, consider using the create_2d_lookup!() macro instead.
    /// Returns a lookup table. Only use an ascending breakpoints vectors! for example  1,2,3,4 and not 4,3,2,1 or 1,2,3,2 \
    /// breakpoints and values must have the same length in the horizontal and vertical direction!
    /// 
    /// # Arguments
    /// 
    /// * `breakpoints_h` - The breakpoints that act as the horizontal index for the values
    /// * `breakpoints_v` - The breakpoints that act as the vertical index for the values
    /// * `values` - The values that represent the result from the lookup
    /// 
    /// # Examples
    /// 
    /// ```
    /// use::go_lookup_tables::{TwoDLookup};
    /// const LOOKUP_TABLE: TwoDLookup<i16,i8,f32,3,3> = TwoDLookup::new([0,500,1000],[0,3,6],[
    /// [3.0,4.2,5.5],
    /// [4.2,5.0,6.0],
    /// [5.0,5.8,6.5]]); //simple 0.5V to 4.5V pressure sensor
    /// /* full table:
    ///     x   0   500 1000
    ///     0   3.0 4.2 5.5
    ///     3   4.2 5.0 6.0
    ///     6   5.0 5.8 6.5
    ///  */
    /// ```
    pub const fn new(breakpoints_h: [S;N], breakpoints_v: [T;M], values: [[U;N];M])-> TwoDLookup<S,T,U,N,M> {
        TwoDLookup { breakpoints_h, breakpoints_v, values }
    }
}

/// Returns a lookup table. Only use an ascending breakpoints vectors! for example  1,2,3,4 and not 4,3,2,1 or 1,2,3,2 \
/// breakpoints and values must have the same length in the horizontal and vertical direction!
/// 
/// # Arguments
/// 
/// * `breakpoints_horizontal` - The breakpoints that act as the horizontal index for the values
/// * `breakpoints_vertical` - The breakpoints that act as the vertical index for the values
/// * `values` - The values that represent the result from the lookup
/// 
/// # Panics
///
/// `create_2d_lookup!` panics if breakpoints is not in ascending order or if breakpoints.len() != values.len().
/// This panic is generated at compile time.
/// 
/// # Examples
/// 
/// ```
/// # #[macro_use] extern crate go_lookup_tables; fn main() {
/// use::go_lookup_tables::*;
/// const LOOKUP_TABLE: TwoDLookup<i16,i8,f32,3,3> = create_2d_lookup!((0,500,1000),(0,3,6),(
/// 3.0,4.2,5.5;
/// 4.2,5.0,6.0;
/// 5.0,5.8,6.5));
/// /* full table:
///     x   0   500 1000
///     0   3   4.2 5.5
///     3   4.2 5.0 6.0
///     6   5   5.8 6.5
///  */
/// # }
/// ```
#[macro_export]
macro_rules! create_2d_lookup {
    (($($bps_h:expr),*), ($($bps_v:expr),*), ($($($vals:expr),*);*)) => {{

        let breakpoints_h = [ $($bps_h,)* ];
        let breakpoints_v = [ $($bps_v,)* ];
        let values = [ $( [ $($vals),* ] ),* ];
        if breakpoints_v.len() != values.len() {
            panic!("the vertical lengths of breakpoints and values don't match");
        }

        if breakpoints_h.len() != values[0].len() {
            panic!("the horizontal lengths of breakpoints and values don't match.");
        }

        let mut i = 0;

        i = 1;
        while i < breakpoints_h.len() {
            if breakpoints_h[i - 1] > breakpoints_h[i] {
                panic!("horizontal breakpoints aren't sorted, they should be in ascending order");
            }
            i += 1;
        }
        i = 1;
        while i < breakpoints_v.len() {
            if breakpoints_v[i - 1] > breakpoints_v[i] {
                panic!("vertical breakpoints aren't sorted, they should be in ascending order");
            }
            i += 1;
        }

        TwoDLookup::new(
            [$($bps_h),+],
            [$($bps_v),+],
            [ $( [ $($vals),+ ] ),+ ],
        )
    }};
}