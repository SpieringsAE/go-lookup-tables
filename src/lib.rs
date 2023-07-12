use std::{fmt,
        ops::{
            Add,
            Sub,
            Mul,
            Div,
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


pub struct OneDLookup <T: PartialOrd + Sub + Add + Copy + Clone, U: PartialOrd + Add + Sub + Mul + Copy + Clone>{
    breakpoints: Vec<T>,
    values: Vec<U>,
    last_diff_bp: T,
    last_diff_values: U,
    first_diff_bp: T,
    first_diff_values: U,
}


impl<T: PartialOrd + Sub + Add + Div<Output = f64> + Copy + Clone, U: PartialOrd + Sub + Add<Output = U> + Mul + Copy + Clone> OneDLookup<T,U> where for<'t> &'t T: Sub<&'t T, Output = T>, for<'a> &'a U: Sub<&'a U, Output = U> + Add<&'a U, Output = U>, f64: Mul<U, Output = U>{
    pub fn lookup(&self, breakpoint: &T, extrapolation: Option<Extrapolation>, interpolation: Option<Interpolation>)-> Result<U, ExtrapolationError> {
        //error handle the interpolation and extrapolation method
        let extrapolation_method = extrapolation.unwrap_or(Extrapolation::NoneError);
        let interpolation_method = interpolation.unwrap_or(Interpolation::Linear);
        //find a correct index or extrapolation at the high end
        let high_index_opt = self.breakpoints.iter().position(|bp| bp > &breakpoint);
        match high_index_opt { 
            None => match extrapolation_method {
            //  handle extrapolation at the high end
                Extrapolation::NoneError => Err(ExtrapolationError),
                Extrapolation::NoneHoldExtreme => Ok(*self.values.last().unwrap()),
                Extrapolation::Linear => {
                    let extrapolated_diff_bp: T = breakpoint - self.breakpoints.get(self.breakpoints.len()-2).unwrap();
                    let diff_factor: f64 = extrapolated_diff_bp / self.last_diff_bp;
                    Ok((diff_factor * self.last_diff_values) + *self.values.get(self.values.len()-2).unwrap())
                }
            }
            Some(index) => {
                if index == 0 { 
                    return match extrapolation_method {
                        Extrapolation::NoneError => Err(ExtrapolationError),
                        Extrapolation::NoneHoldExtreme => Ok(*self.values.first().unwrap()),
                        Extrapolation::Linear => {
                            let extrapolated_diff_bp = self.breakpoints.get(1).unwrap() - breakpoint;
                            let diff_factor:f64 = extrapolated_diff_bp / self.first_diff_bp;
                            Ok((diff_factor * self.first_diff_values) + *self.values.get(1).unwrap())
                        }
                    }
                }
                match interpolation_method {
                    Interpolation::Linear => {
                        let interpolated_diff_bp = breakpoint - self.breakpoints.get(index -1).unwrap();
                        let diff_actual_bp = self.breakpoints.get(index).unwrap() - self.breakpoints.get(index-1).unwrap();
                        let diff_factor:f64 = interpolated_diff_bp / diff_actual_bp;
                        let diff_values = self.values.get(index).unwrap() - self.values.get(index-1).unwrap();
                        Ok((diff_factor * diff_values) + *self.values.get(index-1).unwrap())
                    },
                    Interpolation::NoneCeiling => {Ok(*self.values.get(index).unwrap())},
                    Interpolation::NoneFloor => {Ok(*self.values.get(index-1).unwrap())},
                    Interpolation::NoneClosest => {
                        let interpolated_diff_bp = breakpoint - self.breakpoints.get(index -1).unwrap();
                        let diff_actual_bp = self.breakpoints.get(index).unwrap() - self.breakpoints.get(index-1).unwrap();
                        let diff_factor:f64 = interpolated_diff_bp / diff_actual_bp;
                        Ok(*self.values.get(index-1 + diff_factor.round() as usize).unwrap())
                    },
                }
            }
        }
    }

    fn new(breakpoints: Vec<T>, values: Vec<U>) -> OneDLookup<T,U> {
        OneDLookup {
            last_diff_bp: breakpoints.last().unwrap() - breakpoints.get(breakpoints.len()-2).unwrap(),
            last_diff_values: values.last().unwrap() - values.get(values.len()-2).unwrap(),
            first_diff_bp: breakpoints.get(1).unwrap() - breakpoints.first().unwrap(),
            first_diff_values: values.get(1).unwrap() - values.first().unwrap(),
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
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
