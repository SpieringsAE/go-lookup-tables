use go_lookup_tables::*;
#[test]
fn linear_interpolation_2d() {
    const LOOKUP_TABLE: TwoDLookup<i16,i8,f32,3,3> = create_2d_lookup!((0i16,500,1000),(0i8,3,6),(
        3f32,4.2,5.5;
        4.2f32,5.0,6.0;
        5f32,5.8,6.5));
    //interpolation
    let result1 = LOOKUP_TABLE.lookup(&750i16, &4i8, Interpolation::Linear).unwrap();
    //double extrapolation
    let result2 = LOOKUP_TABLE.lookup(&1250i16, &7i8, Interpolation::Linear).unwrap();
    let result3 = LOOKUP_TABLE.lookup(&1250i16, &-1i8, Interpolation::Linear).unwrap();
    let result4 = LOOKUP_TABLE.lookup(&-250i16, &-1i8, Interpolation::Linear).unwrap();
    let result5 = LOOKUP_TABLE.lookup(&-250i16, &7i8, Interpolation::Linear).unwrap();
    //single extrapolation
    let result6 = LOOKUP_TABLE.lookup(&750i16, &7i8, Interpolation::Linear).unwrap();
    let result7 = LOOKUP_TABLE.lookup(&750i16, &-1i8, Interpolation::Linear).unwrap();
    let result8 = LOOKUP_TABLE.lookup(&1250i16, &2i8, Interpolation::Linear).unwrap();
    let result9 = LOOKUP_TABLE.lookup(&-250i16, &2i8, Interpolation::Linear).unwrap();
    assert_eq!(result1, 5.7166667f32, "2d lookup interpolation failed");

    assert_eq!(result2, 6.5f32, "2d lookup out of bounds hold failed when both breakpoint limits where exceeded");
    assert_eq!(result3, 5.5f32, "2d lookup out of bounds hold failed when horizontal bp was above bounds and vertical below bounds");
    assert_eq!(result4, 3f32, "2d lookup out of bounds hold failed when both breakpoints where below bounds");
    assert_eq!(result5, 5f32, "2d lookup out of bounds hold failed when vertical bp was above bounds and horizontal below bounds");

    assert_eq!(result6, 6.15f32, "2d lookup out of bounds hold failed when only the vertical bp was above bounds");
    assert_eq!(result7, 4.85f32, "2d lookup out of bounds hold failed when only the vertical bp was below bounds");
    assert_eq!(result8, 5.8333335f32, "2d lookup out of bounds hold failed when only the horizontal bp was above bounds");
    assert_eq!(result9, 3.8f32, "2d lookup out of bounds hold failed when only the horizontal bp was below bounds");
}
