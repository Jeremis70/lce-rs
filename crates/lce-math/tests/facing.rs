//! Behavioural unit tests for [`Facing`].
//! Bit-exact test vectors live in `facing_golden.rs`.

use lce_math::facing::Facing;

#[test]
fn opposite_is_its_own_inverse() {
    for f in Facing::ALL {
        assert_eq!(f.opposite().opposite(), f);
    }
}

#[test]
fn opposite_pairs_match_the_axis_convention() {
    assert_eq!(Facing::Down.opposite(), Facing::Up);
    assert_eq!(Facing::Up.opposite(), Facing::Down);
    assert_eq!(Facing::North.opposite(), Facing::South);
    assert_eq!(Facing::South.opposite(), Facing::North);
    assert_eq!(Facing::West.opposite(), Facing::East);
    assert_eq!(Facing::East.opposite(), Facing::West);
}

#[test]
fn each_face_steps_along_exactly_one_axis() {
    for f in Facing::ALL {
        let steps = (f.step_x(), f.step_y(), f.step_z());
        let nonzero = [steps.0, steps.1, steps.2]
            .iter()
            .filter(|&&s| s != 0)
            .count();
        assert_eq!(nonzero, 1, "{f:?} stepped on more than one axis: {steps:?}");
        let magnitude = steps.0.abs() + steps.1.abs() + steps.2.abs();
        assert_eq!(magnitude, 1, "{f:?} step was not a unit step: {steps:?}");
    }
}

#[test]
fn steps_point_toward_the_face_named() {
    assert_eq!(Facing::Down.step_y(), -1);
    assert_eq!(Facing::Up.step_y(), 1);
    assert_eq!(Facing::North.step_z(), -1);
    assert_eq!(Facing::South.step_z(), 1);
    assert_eq!(Facing::West.step_x(), -1);
    assert_eq!(Facing::East.step_x(), 1);
}

#[test]
fn a_face_and_its_opposite_step_in_opposite_directions() {
    for f in Facing::ALL {
        let o = f.opposite();
        assert_eq!(f.step_x(), -o.step_x());
        assert_eq!(f.step_y(), -o.step_y());
        assert_eq!(f.step_z(), -o.step_z());
    }
}

#[test]
fn id_round_trips_through_from_id() {
    for f in Facing::ALL {
        assert_eq!(Facing::from_id(f.id()), Some(f));
        assert_eq!(Facing::try_from(f.id()), Ok(f));
    }
}

#[test]
fn out_of_range_ids_are_rejected() {
    for id in 6..=u8::MAX {
        assert_eq!(Facing::from_id(id), None);
        assert_eq!(Facing::try_from(id), Err(id));
    }
}
