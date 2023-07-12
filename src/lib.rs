use std::{fmt,
        ops::{
            Add,
            Sub,
        },
        cmp::PartialOrd};

#[derive(Debug, Clone)]
pub struct ExtrapolationError;

impl fmt::Display for ExtrapolationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Either index was out of bounds with the NoneError extrapolation method or the lookup table has no values")
    }
}

pub enum Extrapolation {
    NoneError,
    NoneHoldExtreme,
    Linear,
}

pub enum Interpolation {
    Linear,
    NoneFloor,
    NoneCeiling,
    NoneClosest,
}


pub struct OneDLookup <T: PartialOrd + Sub + Add + Copy + Clone, U: Add + Sub + Copy + Clone>{
    breakpoints: Vec<T>,
    values: Vec<U>,
    last_diff_bp: f64,
    last_diff_values: f64,
    first_diff_bp: f64,
    first_diff_values: f64,
}


impl<T: PartialOrd + Sub + Add + Copy + Clone, U: Sub + Add<Output = U> + Copy + Clone> OneDLookup<T,U> where T: Sub<T, Output = T>, U: Sub<U, Output = U> + Add<U, Output = U> + From<f64>, f64: From<T>, f64: From<U>{
    pub fn lookup<Y: Copy>(&self, breakpoint: &Y, extrapolation: Extrapolation, interpolation: Interpolation) -> Result<U, ExtrapolationError>  where T: From<Y> {
        let calc_breakpoint = T::from(*breakpoint);
        let high_index_opt = self.breakpoints.iter().position(|bp| bp > &calc_breakpoint);
        match high_index_opt { 
            None => match extrapolation {
            //  handle extrapolation at the high end
                Extrapolation::NoneError => Err(ExtrapolationError),
                Extrapolation::NoneHoldExtreme => Ok(*self.values.last().unwrap()),
                Extrapolation::Linear => {
                    let extrapolated_diff_bp: T = calc_breakpoint - *self.breakpoints.get(self.breakpoints.len()-2).unwrap();
                    let diff_factor: f64 = f64::from(extrapolated_diff_bp) / self.last_diff_bp;
                    Ok(U::from(diff_factor * self.last_diff_values) + *self.values.get(self.values.len()-2).unwrap())
                }
            }
            Some(index) => {
                if index == 0 { 
                    return match extrapolation {
                        Extrapolation::NoneError => Err(ExtrapolationError),
                        Extrapolation::NoneHoldExtreme => Ok(*self.values.first().unwrap()),
                        Extrapolation::Linear => {
                            let extrapolated_diff_bp = *self.breakpoints.get(1).unwrap() - calc_breakpoint;
                            let diff_factor = f64::from(extrapolated_diff_bp) / self.first_diff_bp;
                            Ok(U::from(diff_factor * self.first_diff_values) + *self.values.get(1).unwrap())
                        }
                    }
                }
                match interpolation {
                    Interpolation::Linear => {
                        let interpolated_diff_bp = calc_breakpoint - *self.breakpoints.get(index -1).unwrap();
                        let diff_actual_bp = *self.breakpoints.get(index).unwrap() - *self.breakpoints.get(index-1).unwrap();
                        let diff_factor:f64 = f64::from(interpolated_diff_bp) / f64::from(diff_actual_bp);
                        let diff_values = *self.values.get(index).unwrap() - *self.values.get(index-1).unwrap();
                        Ok(U::from(diff_factor * f64::from(diff_values)) + *self.values.get(index-1).unwrap())
                    },
                    Interpolation::NoneCeiling => {Ok(*self.values.get(index).unwrap())},
                    Interpolation::NoneFloor => {Ok(*self.values.get(index-1).unwrap())},
                    Interpolation::NoneClosest => {
                        let interpolated_diff_bp = calc_breakpoint - *self.breakpoints.get(index -1).unwrap();
                        let diff_actual_bp = *self.breakpoints.get(index).unwrap() - *self.breakpoints.get(index-1).unwrap();
                        let diff_factor:f64 = f64::from(interpolated_diff_bp) / f64::from(diff_actual_bp);
                        Ok(*self.values.get(index-1 + diff_factor.round() as usize).unwrap())
                    },
                }
            }
        }
    }

    fn new(breakpoints: Vec<T>, values: Vec<U>) -> OneDLookup<T,U> {
        OneDLookup {
            last_diff_bp: f64::from(*breakpoints.last().unwrap() - *breakpoints.get(breakpoints.len()-2).unwrap()),
            last_diff_values: f64::from(*values.last().unwrap() - *values.get(values.len()-2).unwrap()),
            first_diff_bp: f64::from(*breakpoints.get(1).unwrap() - *breakpoints.first().unwrap()),
            first_diff_values: f64::from(*values.first().unwrap() - *values.get(1).unwrap()),
            breakpoints: breakpoints,
            values: values,
        }
    }
}



pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::OneDLookup;

    #[test]
    fn linear_extrapolation_signed() {
        let lookup_table = OneDLookup::new(vec![0i16,5000], vec![0f64,500.0]);
        let result = lookup_table.lookup(&2500i16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
        let result1 = lookup_table.lookup(&-1000i16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
        let result2 = lookup_table.lookup(&6000i16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
        println!("sensor is an automotive 0 - 5V 0 - 500 bar pressure sensor");
        println!("linear extrapolation and interpolation enabled");
        println!("Result at 2.5V is : {}bar", result);
        assert_eq!(result, 250f64);
        println!("Result at -1V is:   {}bar", result1);
        assert_eq!(result1, -100f64);
        println!("Result at 6V is:    {}bar", result2);
        assert_eq!(result2, 600f64);
    }
    #[test]
    fn linear_extrapolation_unsigned() {
        let lookup_table = OneDLookup::new(vec![1000u16,5000], vec![0f64,500.0]);
        let result = lookup_table.lookup(&3000u16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
        let result1 = lookup_table.lookup(&500u16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
        let result2 = lookup_table.lookup(&6000u16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
        println!("sensor is an automotive 1 - 5V 0 - 500 bar pressure sensor");
        println!("linear extrapolation and interpolation enabled");
        println!("Result at 3V is :    {}bar", result);
        assert_eq!(result, 250f64);
        println!("Result at 0.5V is:   {}bar", result1);
        assert_eq!(result1, -62.5f64);
        println!("Result at 6V is:     {}bar", result2);
        assert_eq!(result2, 625f64);
    }
}
