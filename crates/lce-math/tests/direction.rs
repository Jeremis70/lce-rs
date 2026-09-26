//! Behavioural unit tests for [`Direction`].
//! Bit-exact test vectors live in `facing_golden.rs`.

use lce_math::direction::Direction;
use lce_math::facing::Facing;

#[test]
fn opposite_is_its_own_inverse() {
    for d in Direction::ALL {
        assert_eq!(d.opposite().opposite(), d);
    }
}

#[test]
fn opposite_pairs_match_the_axis_convention() {
    assert_eq!(Direction::South.opposite(), Direction::North);
    assert_eq!(Direction::North.opposite(), Direction::South);
    assert_eq!(Direction::West.opposite(), Direction::East);
    assert_eq!(Direction::East.opposite(), Direction::West);
}

#[test]
fn rotating_four_times_clockwise_is_the_identity() {
    for d in Direction::ALL {
        let mut cur = d;
        for _ in 0..4 {
            cur = cur.clockwise();
        }
        assert_eq!(cur, d);
    }
}

#[test]
fn rotating_four_times_counter_clockwise_is_the_identity() {
    for d in Direction::ALL {
        let mut cur = d;
        for _ in 0..4 {
            cur = cur.counter_clockwise();
        }
        assert_eq!(cur, d);
    }
}

#[test]
fn clockwise_and_counter_clockwise_undo_each_other() {
    for d in Direction::ALL {
        assert_eq!(d.clockwise().counter_clockwise(), d);
        assert_eq!(d.counter_clockwise().clockwise(), d);
    }
}

#[test]
fn two_clockwise_turns_is_the_opposite() {
    for d in Direction::ALL {
        assert_eq!(d.clockwise().clockwise(), d.opposite());
    }
}

#[test]
fn each_direction_steps_along_exactly_one_horizontal_axis() {
    for d in Direction::ALL {
        let (sx, sz) = (d.step_x(), d.step_z());
        assert_eq!(sx.abs() + sz.abs(), 1, "{d:?} step was not a unit step");
    }
}

#[test]
fn direction_and_facing_round_trip() {
    for d in Direction::ALL {
        assert_eq!(Direction::from_facing(d.facing()), Some(d));
    }
}

#[test]
fn vertical_faces_have_no_horizontal_direction() {
    assert_eq!(Direction::from_facing(Facing::Down), None);
    assert_eq!(Direction::from_facing(Facing::Up), None);
}

#[test]
fn id_round_trips_through_from_id() {
    for d in Direction::ALL {
        assert_eq!(Direction::from_id(d.id()), Some(d));
        assert_eq!(Direction::try_from(d.id()), Ok(d));
    }
}

#[test]
fn out_of_range_ids_are_rejected() {
    for id in 4..=u8::MAX {
        assert_eq!(Direction::from_id(id), None);
        assert_eq!(Direction::try_from(id), Err(id));
    }
}

#[test]
fn facing_north_is_the_identity_orientation_for_horizontal_faces() {
    for f in [Facing::North, Facing::South, Facing::West, Facing::East] {
        assert_eq!(Direction::North.relative_facing(f), f);
    }
}

#[test]
fn relative_facing_swaps_down_and_up_for_every_direction() {
    for d in Direction::ALL {
        assert_eq!(d.relative_facing(Facing::Down), Facing::Up);
        assert_eq!(d.relative_facing(Facing::Up), Facing::Down);
    }
}

#[test]
fn relative_facing_is_a_permutation_of_all_six_faces() {
    use std::collections::HashSet;

    for d in Direction::ALL {
        let mapped: HashSet<Facing> = Facing::ALL
            .into_iter()
            .map(|f| d.relative_facing(f))
            .collect();
        assert_eq!(mapped.len(), 6, "{d:?} did not map onto all six faces");
    }
}
