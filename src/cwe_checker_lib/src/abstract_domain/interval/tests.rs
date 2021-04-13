use super::*;

impl IntervalDomain {
    /// Return a new interval domain of 8-byte integers.
    pub fn mock(start: i64, end: i64) -> IntervalDomain {
        IntervalDomain::new(Bitvector::from_i64(start), Bitvector::from_i64(end))
    }

    /// Return a new interval domain of 1-byte integers.
    pub fn mock_i8(start: i8, end: i8) -> IntervalDomain {
        IntervalDomain::new(Bitvector::from_i8(start), Bitvector::from_i8(end))
    }

    pub fn mock_with_bounds(
        lower_bound: Option<i64>,
        start: i64,
        end: i64,
        upper_bound: Option<i64>,
    ) -> IntervalDomain {
        let mut domain = IntervalDomain::mock(start, end);
        domain.update_widening_lower_bound(&lower_bound.map(|b| Bitvector::from_i64(b)));
        domain.update_widening_upper_bound(&upper_bound.map(|b| Bitvector::from_i64(b)));
        domain
    }

    pub fn mock_i8_with_bounds(
        lower_bound: Option<i8>,
        start: i8,
        end: i8,
        upper_bound: Option<i8>,
    ) -> IntervalDomain {
        let mut domain = IntervalDomain::mock_i8(start, end);
        domain.update_widening_lower_bound(&lower_bound.map(|b| Bitvector::from_i8(b)));
        domain.update_widening_upper_bound(&upper_bound.map(|b| Bitvector::from_i8(b)));
        domain
    }
}

#[test]
fn signed_merge() {
    // simple widening examples
    let a = IntervalDomain::mock_with_bounds(None, 0, 3, Some(10));
    let b = IntervalDomain::mock_with_bounds(None, 2, 5, None);
    assert_eq!(
        a.merge(&b),
        IntervalDomain::mock_with_bounds(None, 0, 10, None)
    );
    let a = IntervalDomain::mock_with_bounds(Some(-3), 1, 1, None);
    let b = IntervalDomain::mock_with_bounds(None, 2, 2, Some(5));
    assert_eq!(
        a.merge(&b),
        IntervalDomain::mock_with_bounds(Some(-3), 1, 2, Some(5))
    );
    let a = IntervalDomain::mock_with_bounds(Some(-3), 1, 1, None);
    let b = IntervalDomain::mock_with_bounds(None, 3, 3, Some(5));
    assert_eq!(
        a.merge(&b),
        IntervalDomain::mock_with_bounds(None, -3, 5, None)
    );
    let a = IntervalDomain::mock_with_bounds(None, 1, 5, None);
    let b = IntervalDomain::mock_with_bounds(None, -1, -1, Some(5));
    assert_eq!(a.merge(&b), IntervalDomain::new_top(ByteSize::new(8)));
    let a = IntervalDomain::mock_with_bounds(None, 1, 5, None);
    let b = IntervalDomain::mock_with_bounds(None, 3, 3, Some(10));
    assert_eq!(
        a.merge(&b),
        IntervalDomain::mock_with_bounds(None, 1, 5, Some(10))
    );
    let a = IntervalDomain::mock_with_bounds(None, 20, -5, None);
    let b = IntervalDomain::mock_with_bounds(None, 0, 0, Some(50));
    assert_eq!(a.merge(&a), IntervalDomain::new_top(ByteSize::new(8))); // Interval wraps and is thus merged to `Top`, even though a = a
    assert_eq!(a.merge(&b), IntervalDomain::new_top(ByteSize::new(8)));

    // Widening process corresponding to a very simple loop counter variable
    let mut var = IntervalDomain::mock(0, 0);
    let update = IntervalDomain::mock_with_bounds(None, 1, 1, Some(99));
    var = var.merge(&update);
    assert_eq!(var, IntervalDomain::mock_with_bounds(None, 0, 1, Some(99)));
    let update = IntervalDomain::mock_with_bounds(None, 1, 2, Some(99));
    var = var.merge(&update);
    assert_eq!(var, IntervalDomain::mock_with_bounds(None, 0, 99, None));
    let update = IntervalDomain::mock_with_bounds(None, 1, 99, None);
    var = var.merge(&update);
    assert_eq!(var, IntervalDomain::mock_with_bounds(None, 0, 99, None));

    // Widening process corresponding to a loop counter variable with bound in the wrong direction
    let mut var = IntervalDomain::mock(0, 0);
    let update = IntervalDomain::mock_with_bounds(Some(-3), 1, 1, None);
    var = var.merge(&update);
    assert_eq!(var, IntervalDomain::mock_with_bounds(Some(-3), 0, 1, None));
    let update = IntervalDomain::mock_with_bounds(Some(-3), 1, 2, None);
    var = var.merge(&update);
    assert_eq!(var, IntervalDomain::mock_with_bounds(None, -3, 2, None));
    let update = IntervalDomain::mock_with_bounds(Some(-3), -2, 3, None);
    var = var.merge(&update);
    assert_eq!(var, IntervalDomain::new_top(ByteSize::new(8)));
}

#[test]
fn cast_zero_and_signed_extend() {
    // Zero extend
    let val = IntervalDomain::mock_i8_with_bounds(Some(1), 3, 5, Some(30));
    let extended_val = val.cast(CastOpType::IntZExt, ByteSize::new(8));
    assert_eq!(
        extended_val,
        IntervalDomain::mock_with_bounds(Some(1), 3, 5, Some(30))
    );
    let val = IntervalDomain::mock_i8_with_bounds(Some(-10), 0, 5, Some(30));
    let extended_val = val.cast(CastOpType::IntZExt, ByteSize::new(8));
    assert_eq!(
        extended_val,
        IntervalDomain::mock_with_bounds(None, 0, 5, Some(30))
    );
    let val = IntervalDomain::mock_i8_with_bounds(Some(-15), -10, 5, None);
    let extended_val = val.cast(CastOpType::IntZExt, ByteSize::new(8));
    assert_eq!(
        extended_val,
        IntervalDomain::mock_with_bounds(None, 0, 255, None)
    );
    let val = IntervalDomain::mock_i8_with_bounds(Some(-14), -9, -5, Some(-2));
    let extended_val = val.cast(CastOpType::IntZExt, ByteSize::new(8));
    assert_eq!(
        extended_val,
        IntervalDomain::mock_with_bounds(Some(242), 247, 251, Some(254))
    );
    let val = IntervalDomain::mock_i8_with_bounds(Some(-20), -10, -5, Some(3));
    let extended_val = val.cast(CastOpType::IntZExt, ByteSize::new(8));
    assert_eq!(
        extended_val,
        IntervalDomain::mock_with_bounds(Some(236), 246, 251, Some(255))
    );

    // Sign extend
    let val = IntervalDomain::mock_i8_with_bounds(Some(1), 3, 5, Some(30));
    let extended_val = val.cast(CastOpType::IntSExt, ByteSize::new(8));
    assert_eq!(
        extended_val,
        IntervalDomain::mock_with_bounds(Some(1), 3, 5, Some(30))
    );
    let val = IntervalDomain::mock_i8_with_bounds(Some(-10), 0, 5, Some(30));
    let extended_val = val.cast(CastOpType::IntSExt, ByteSize::new(8));
    assert_eq!(
        extended_val,
        IntervalDomain::mock_with_bounds(Some(-10), 0, 5, Some(30))
    );
    let val = IntervalDomain::mock_i8_with_bounds(Some(-15), -10, 127, None);
    let extended_val = val.cast(CastOpType::IntSExt, ByteSize::new(8));
    assert_eq!(
        extended_val,
        IntervalDomain::mock_with_bounds(Some(-15), -10, 127, None)
    );
    let val = IntervalDomain::mock_i8_with_bounds(None, -10, -5, None);
    let extended_val = val.cast(CastOpType::IntSExt, ByteSize::new(8));
    assert_eq!(
        extended_val,
        IntervalDomain::mock_with_bounds(Some(-128), -10, -5, Some(127))
    );
    let val = IntervalDomain::mock_i8_with_bounds(Some(-20), -10, -5, Some(3));
    let extended_val = val.cast(CastOpType::IntSExt, ByteSize::new(8));
    assert_eq!(
        extended_val,
        IntervalDomain::mock_with_bounds(Some(-20), -10, -5, Some(3))
    );
}

#[test]
fn subpiece() {
    let val = IntervalDomain::mock_with_bounds(None, -3, 5, Some(10));
    let subpieced_val = val.subpiece(ByteSize::new(0), ByteSize::new(1));
    assert_eq!(
        subpieced_val,
        IntervalDomain::mock_i8_with_bounds(None, -3, 5, None)
    );
    let val = IntervalDomain::mock_with_bounds(Some(-30), 2, 5, Some(10));
    let subpieced_val = val.subpiece(ByteSize::new(0), ByteSize::new(1));
    assert_eq!(
        subpieced_val,
        IntervalDomain::mock_i8_with_bounds(Some(-30), 2, 5, Some(10))
    );
    let val = IntervalDomain::mock_with_bounds(Some(-500), 2, 5, Some(10));
    let subpieced_val = val.subpiece(ByteSize::new(0), ByteSize::new(1));
    assert_eq!(
        subpieced_val,
        IntervalDomain::mock_i8_with_bounds(None, 2, 5, None)
    );
    let val = IntervalDomain::mock_with_bounds(Some(-30), 2, 567, Some(777));
    let subpieced_val = val.subpiece(ByteSize::new(0), ByteSize::new(1));
    assert_eq!(subpieced_val, IntervalDomain::new_top(ByteSize::new(1)));
    let val = IntervalDomain::mock_with_bounds(Some(-30), 2, 3, Some(777));
    let subpieced_val = val.subpiece(ByteSize::new(1), ByteSize::new(1));
    assert_eq!(subpieced_val, IntervalDomain::new_top(ByteSize::new(1)));
    let val = IntervalDomain::mock_with_bounds(Some(-30), 512, 512, Some(777));
    let subpieced_val = val.subpiece(ByteSize::new(1), ByteSize::new(1));
    assert_eq!(
        subpieced_val,
        IntervalDomain::mock_i8_with_bounds(None, 2, 2, None)
    );
}

#[test]
fn un_op() {
    // Int2Comp
    let mut val = IntervalDomain::mock_with_bounds(None, -3, 5, Some(10));
    val = val.un_op(UnOpType::Int2Comp);
    assert_eq!(
        val,
        IntervalDomain::mock_with_bounds(Some(-10), -5, 3, None)
    );
    let mut val = IntervalDomain::mock_i8_with_bounds(Some(-128), -3, 5, Some(127));
    val = val.un_op(UnOpType::Int2Comp);
    assert_eq!(
        val,
        IntervalDomain::mock_i8_with_bounds(Some(-127), -5, 3, None)
    );
    // IntNegate
    let mut val = IntervalDomain::mock_with_bounds(None, -3, 5, Some(10));
    val = val.un_op(UnOpType::IntNegate);
    assert_eq!(val, IntervalDomain::new_top(ByteSize::new(8)));
    let mut val = IntervalDomain::mock_with_bounds(None, -4, -4, Some(10));
    val = val.un_op(UnOpType::IntNegate);
    assert_eq!(val, IntervalDomain::mock(3, 3));
}

#[test]
fn add() {
    let lhs = IntervalDomain::mock_with_bounds(None, 3, 7, Some(10));
    let rhs = IntervalDomain::mock_with_bounds(Some(-20), -3, 0, Some(10));
    let result = lhs.bin_op(BinOpType::IntAdd, &rhs);
    assert_eq!(
        result,
        IntervalDomain::mock_with_bounds(Some(-17), 0, 7, Some(10))
    );
    let lhs = IntervalDomain::mock_i8_with_bounds(Some(-121), -120, -120, Some(10));
    let rhs = IntervalDomain::mock_i8_with_bounds(Some(-10), -9, 0, Some(10));
    let result = lhs.bin_op(BinOpType::IntAdd, &rhs);
    assert_eq!(result, IntervalDomain::new_top(ByteSize::new(1)));
    let lhs = IntervalDomain::mock_i8_with_bounds(Some(-100), -30, 40, Some(100));
    let rhs = IntervalDomain::mock_i8_with_bounds(Some(-100), -30, 20, Some(50));
    let result = lhs.bin_op(BinOpType::IntAdd, &rhs);
    assert_eq!(
        result,
        IntervalDomain::mock_i8_with_bounds(None, -60, 60, Some(90))
    );
}

#[test]
fn sub() {
    let lhs = IntervalDomain::mock_with_bounds(None, 3, 7, Some(10));
    let rhs = IntervalDomain::mock_with_bounds(Some(-20), -3, 0, Some(10));
    let result = lhs.bin_op(BinOpType::IntSub, &rhs);
    assert_eq!(
        result,
        IntervalDomain::mock_with_bounds(Some(-7), 3, 10, Some(13))
    );
    let lhs = IntervalDomain::mock_i8_with_bounds(Some(-121), -120, -120, Some(10));
    let rhs = IntervalDomain::mock_i8_with_bounds(Some(-10), -9, 9, Some(10));
    let result = lhs.bin_op(BinOpType::IntSub, &rhs);
    assert_eq!(result, IntervalDomain::new_top(ByteSize::new(1)));
    let lhs = IntervalDomain::mock_i8_with_bounds(Some(-100), 2, 40, Some(100));
    let rhs = IntervalDomain::mock_i8_with_bounds(Some(-50), -30, 3, Some(100));
    let result = lhs.bin_op(BinOpType::IntSub, &rhs);
    assert_eq!(
        result,
        IntervalDomain::mock_i8_with_bounds(Some(-98), -1, 70, Some(90))
    );
}

#[test]
fn multiplication() {
    let lhs = IntervalDomain::mock_with_bounds(None, 3, 7, Some(10));
    let rhs = IntervalDomain::mock_with_bounds(Some(-20), -3, 0, Some(10));
    let result = lhs.bin_op(BinOpType::IntMult, &rhs);
    assert_eq!(
        result,
        IntervalDomain::mock_with_bounds(Some(-200), -21, 0, Some(100))
    );
    let lhs = IntervalDomain::mock_with_bounds(Some(-4), -3, 1, Some(2));
    let rhs = IntervalDomain::mock_with_bounds(Some(-6), -5, 7, Some(8));
    let result = lhs.bin_op(BinOpType::IntMult, &rhs);
    assert_eq!(
        result,
        IntervalDomain::mock_with_bounds(Some(-32), -21, 15, Some(16))
    );
    let lhs = IntervalDomain::mock_i8_with_bounds(None, 3, 7, Some(50));
    let rhs = IntervalDomain::mock_i8_with_bounds(Some(-30), -3, 0, Some(50));
    let result = lhs.bin_op(BinOpType::IntMult, &rhs);
    assert_eq!(
        result,
        IntervalDomain::mock_i8_with_bounds(None, -21, 0, None)
    );
}

#[test]
fn shift_left() {
    let lhs = IntervalDomain::mock_i8_with_bounds(None, 3, 3, Some(50));
    let rhs = IntervalDomain::mock_i8_with_bounds(Some(1), 2, 3, Some(4));
    let result = lhs.bin_op(BinOpType::IntLeft, &rhs);
    assert_eq!(result, IntervalDomain::new_top(ByteSize::new(1)));
    let lhs = IntervalDomain::mock_i8_with_bounds(None, 3, 4, Some(5));
    let rhs = IntervalDomain::mock_i8_with_bounds(Some(1), 2, 2, Some(4));
    let result = lhs.bin_op(BinOpType::IntLeft, &rhs);
    assert_eq!(
        result,
        IntervalDomain::mock_i8_with_bounds(None, 12, 16, None)
    );
    let lhs = IntervalDomain::mock_i8_with_bounds(Some(2), 3, 4, Some(64));
    let rhs = IntervalDomain::mock_i8_with_bounds(Some(0), 1, 1, Some(4));
    let result = lhs.bin_op(BinOpType::IntLeft, &rhs);
    assert_eq!(
        result,
        IntervalDomain::mock_i8_with_bounds(None, 6, 8, None)
    );
    let lhs = IntervalDomain::mock_with_bounds(Some(2), 3, 4, Some(64));
    let rhs = IntervalDomain::mock_i8_with_bounds(None, 127, 127, None);
    let result = lhs.bin_op(BinOpType::IntLeft, &rhs);
    assert_eq!(result, IntervalDomain::mock(0, 0));
}

#[test]
fn simple_interval_contains() {
    let domain = IntervalDomain::mock(-10, 5);
    assert!(!domain.interval.contains(&Bitvector::from_i64(-11)));
    assert!(domain.interval.contains(&Bitvector::from_i64(-10)));
    assert!(domain.interval.contains(&Bitvector::from_i64(-4)));
    assert!(domain.interval.contains(&Bitvector::from_i64(5)));
    assert!(!domain.interval.contains(&Bitvector::from_i64(6)));
}

#[test]
fn add_signed_bounds() {
    let interval = IntervalDomain::mock_with_bounds(Some(-100), -10, 10, Some(100));

    // signed_less_equal
    let x = interval
        .clone()
        .add_signed_less_equal_bound(&Bitvector::from_i64(20));
    assert_eq!(
        x.unwrap(),
        IntervalDomain::mock_with_bounds(Some(-100), -10, 10, Some(20))
    );
    let x = interval
        .clone()
        .add_signed_less_equal_bound(&Bitvector::from_i64(-5));
    assert_eq!(
        x.unwrap(),
        IntervalDomain::mock_with_bounds(Some(-100), -10, -5, None)
    );
    let x = interval
        .clone()
        .add_signed_less_equal_bound(&Bitvector::from_i64(-20));
    assert!(x.is_err());

    //signed_greater_equal
    let x = interval
        .clone()
        .add_signed_greater_equal_bound(&Bitvector::from_i64(20));
    assert!(x.is_err());
    let x = interval
        .clone()
        .add_signed_greater_equal_bound(&Bitvector::from_i64(-5));
    assert_eq!(
        x.unwrap(),
        IntervalDomain::mock_with_bounds(None, -5, 10, Some(100))
    );
    let x = interval
        .clone()
        .add_signed_greater_equal_bound(&Bitvector::from_i64(-20));
    assert_eq!(
        x.unwrap(),
        IntervalDomain::mock_with_bounds(Some(-20), -10, 10, Some(100))
    );
}

#[test]
fn add_unsigned_bounds() {
    let positive_interval = IntervalDomain::mock_with_bounds(Some(10), 20, 30, Some(40));
    let wrapped_interval = IntervalDomain::mock_with_bounds(Some(-100), -10, 10, Some(100));
    let negative_interval = IntervalDomain::mock_with_bounds(Some(-40), -30, -20, Some(-10));

    // unsigned_less_equal
    let x = positive_interval
        .clone()
        .add_unsigned_less_equal_bound(&Bitvector::from_i64(35));
    assert_eq!(
        x.unwrap(),
        IntervalDomain::mock_with_bounds(Some(10), 20, 30, Some(35))
    );
    let x = positive_interval
        .clone()
        .add_unsigned_less_equal_bound(&Bitvector::from_i64(15));
    assert!(x.is_err());

    let x = wrapped_interval
        .clone()
        .add_unsigned_less_equal_bound(&Bitvector::from_i64(35));
    assert_eq!(
        x.unwrap(),
        IntervalDomain::mock_with_bounds(None, 0, 10, Some(35))
    );
    let x = wrapped_interval
        .clone()
        .add_unsigned_less_equal_bound(&Bitvector::from_i64(-5));
    assert_eq!(x.unwrap(), wrapped_interval); // Cannot remove a subinterval from the domain

    let x = negative_interval
        .clone()
        .add_unsigned_less_equal_bound(&Bitvector::from_i64(-25));
    assert_eq!(
        x.unwrap(),
        IntervalDomain::mock_with_bounds(Some(-40), -30, -25, None)
    );
    let x = negative_interval
        .clone()
        .add_unsigned_less_equal_bound(&Bitvector::from_i64(-35));
    assert!(x.is_err());

    // unsigned_greater_equal
    let x = positive_interval
        .clone()
        .add_unsigned_greater_equal_bound(&Bitvector::from_i64(25));
    assert_eq!(
        x.unwrap(),
        IntervalDomain::mock_with_bounds(None, 25, 30, Some(40))
    );
    let x = positive_interval
        .clone()
        .add_unsigned_greater_equal_bound(&Bitvector::from_i64(35));
    assert!(x.is_err());

    let x = wrapped_interval
        .clone()
        .add_unsigned_greater_equal_bound(&Bitvector::from_i64(5));
    assert_eq!(x.unwrap(), wrapped_interval);
    let x = wrapped_interval
        .clone()
        .add_unsigned_greater_equal_bound(&Bitvector::from_i64(35));
    assert_eq!(
        x.unwrap(),
        IntervalDomain::mock_with_bounds(Some(-100), -10, -1, None)
    );
    let x = wrapped_interval
        .clone()
        .add_unsigned_greater_equal_bound(&Bitvector::from_i64(-50));
    assert_eq!(
        x.unwrap(),
        IntervalDomain::mock_with_bounds(Some(-50), -10, -1, None)
    );

    let x = negative_interval
        .clone()
        .add_unsigned_greater_equal_bound(&Bitvector::from_i64(25));
    assert_eq!(x.unwrap(), negative_interval);
    let x = negative_interval
        .clone()
        .add_unsigned_greater_equal_bound(&Bitvector::from_i64(-25));
    assert_eq!(
        x.unwrap(),
        IntervalDomain::mock_with_bounds(None, -25, -20, Some(-10))
    );
}

#[test]
fn add_not_equal_bounds() {
    let interval = IntervalDomain::mock_with_bounds(None, -10, 10, None);

    let x = interval
        .clone()
        .add_not_equal_bound(&Bitvector::from_i64(-20));
    assert_eq!(
        x.unwrap(),
        IntervalDomain::mock_with_bounds(Some(-19), -10, 10, None)
    );
    let x = interval
        .clone()
        .add_not_equal_bound(&Bitvector::from_i64(-0));
    assert_eq!(x.unwrap(), interval);
    let x = interval
        .clone()
        .add_not_equal_bound(&Bitvector::from_i64(20));
    assert_eq!(
        x.unwrap(),
        IntervalDomain::mock_with_bounds(None, -10, 10, Some(19))
    );

    let interval = IntervalDomain::mock(5, 5);
    let x = interval
        .clone()
        .add_not_equal_bound(&Bitvector::from_i64(5));
    assert!(x.is_err());
    let interval = IntervalDomain::mock(5, 6);
    let x = interval.add_not_equal_bound(&Bitvector::from_i64(5));
    assert_eq!(x.unwrap(), IntervalDomain::mock(6, 6));
}

#[test]
fn intersection() {
    let interval1 = IntervalDomain::mock_with_bounds(Some(-100), -10, 10, Some(100));
    let interval2 = IntervalDomain::mock_with_bounds(Some(-20), 2, 30, None);
    let intersection = interval1.intersect(&interval2).unwrap();
    assert_eq!(
        intersection,
        IntervalDomain::mock_with_bounds(Some(-20), 2, 10, Some(100))
    );
    assert!(interval1.intersect(&IntervalDomain::mock(50, 55)).is_err());
}