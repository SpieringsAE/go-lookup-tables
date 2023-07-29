use go_lookup_tables::*;

#[test]
fn extrapolation_linear_1d() {
    const LOOKUP_TABLE: OneDLookup<i16, f32, 2> = create_1d_lookup!((0i16,5000),(0f32,500.0));
    let result1 = LOOKUP_TABLE.lookup(&-1000i16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
    let result2 = LOOKUP_TABLE.lookup(&6000i16, crate::Extrapolation::Linear, crate::Interpolation::Linear).unwrap();
    assert_eq!(result1, -100f32);
    assert_eq!(result2, 600f32);
}

#[test]
fn extrapolation_none_hold_1d() {
    const LOOKUP_TABLE: OneDLookup<i16, f32, 2> = create_1d_lookup!((0i16,5000),(0f32,500.0));
    let result1 = LOOKUP_TABLE.lookup(&-1000i16, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::Linear).unwrap();
    let result2 = LOOKUP_TABLE.lookup(&6000i16, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::Linear).unwrap();
    assert_eq!(result1, 0f32);
    assert_eq!(result2, 500f32);
}

#[test]
fn extrapolation_none_error_1d() {
    const LOOKUP_TABLE: OneDLookup<i16, f32, 2> = create_1d_lookup!((0i16,5000),(0f32,500.0));
    let result1 = LOOKUP_TABLE.lookup(&-1000i16, crate::Extrapolation::NoneError, crate::Interpolation::Linear);
    let result2 = LOOKUP_TABLE.lookup(&6000i16, crate::Extrapolation::NoneError, crate::Interpolation::Linear);
    let result3 = LOOKUP_TABLE.lookup(&2500i16, crate::Extrapolation::NoneError, crate::Interpolation::Linear);
    assert!(result1.is_err());
    assert!(result2.is_err());
    assert!(result3.is_ok());
}

#[test]
fn interpolation_linear_1d() {
    const LOOKUP_TABLE: OneDLookup<i16, i32, 4> = create_1d_lookup!((0i16,500,4500,5000), (0i32,0,500,500));
    let result = LOOKUP_TABLE.lookup(&500i16, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::Linear).unwrap();
    let result1 = LOOKUP_TABLE.lookup(&505i16, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::Linear).unwrap();
    let result2 = LOOKUP_TABLE.lookup(&4495i16, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::Linear).unwrap();
    let result3 = LOOKUP_TABLE.lookup(&2500i16, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::Linear).unwrap();
    assert_eq!(result, 0i32);
    assert_eq!(result1, 0i32);
    assert_eq!(result2, 499i32);
    assert_eq!(result3, 250i32);
}

#[test]
fn interpolation_closest_1d() {
    const LOOKUP_TABLE: OneDLookup<i8, i8, 4> = create_1d_lookup!((0i8,1,5,6),(0i8,1,6,8));
    let result = LOOKUP_TABLE.lookup(&3i8, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::NoneClosest).unwrap();
    let result1 = LOOKUP_TABLE.lookup(&4i8, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::NoneClosest).unwrap();
    let result2 = LOOKUP_TABLE.lookup(&2i8, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::NoneClosest).unwrap();
    assert_eq!(result, 6i8); //value is 3.5, round up to 6
    assert_eq!(result1, 6i8); //value is 4.75, round up to 6
    assert_eq!(result2, 1i8); //value is 2.25, round down to 1
}

#[test]
fn interpolation_floor_1d() {
    const LOOKUP_TABLE: OneDLookup<i8, i8, 4> = create_1d_lookup!((0i8,2,5,6),(0i8,1,6,8));
    let result = LOOKUP_TABLE.lookup(&0i8, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::NoneFloor).unwrap();
    let result1 = LOOKUP_TABLE.lookup(&1i8, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::NoneFloor).unwrap();
    assert_eq!(result, 0i8);
    assert_eq!(result, result1);
}

#[test]
fn interpolation_ceiling_1d() {
    const LOOKUP_TABLE: OneDLookup<i8, i8, 4> = create_1d_lookup!((0i8,2,5,6),(0i8,1,6,8));
    let result = LOOKUP_TABLE.lookup(&3i8, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::NoneCeiling).unwrap();
    let result1 = LOOKUP_TABLE.lookup(&4i8, crate::Extrapolation::NoneHoldExtreme, crate::Interpolation::NoneCeiling).unwrap();
    assert_eq!(result, 6i8);
    assert_eq!(result, result1);
}