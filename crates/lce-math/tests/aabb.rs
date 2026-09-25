//! Behavioural unit tests for [`Aabb`]. Bit-exact test vectors live in
//! `aabb_golden.rs`.

use lce_math::aabb::Aabb;
use lce_math::facing::Facing;
use lce_math::vec3::Vec3;

const UNIT: Aabb = Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);

/// A 0.6 × 1.8 × 0.6 box with its low corner at `(x, y, z)`.
fn player_at(x: f64, y: f64, z: f64) -> Aabb {
    Aabb::new(x, y, z, x + 0.6, y + 1.8, z + 0.6)
}

#[test]
fn expand_extends_only_the_leading_face() {
    let b = UNIT.expand(2.0, -3.0, 0.0);
    assert_eq!(b, Aabb::new(0.0, -3.0, 0.0, 3.0, 1.0, 1.0));
    assert_eq!(UNIT.expand(f64::NAN, 0.0, -0.0), UNIT);
}

#[test]
fn grow_shrink_and_move() {
    assert_eq!(
        UNIT.grow(0.5, 1.0, 0.0),
        Aabb::new(-0.5, -1.0, 0.0, 1.5, 2.0, 1.0)
    );
    assert_eq!(
        UNIT.shrink(0.25, 0.0, 0.5),
        Aabb::new(0.25, 0.0, 0.5, 0.75, 1.0, 0.5)
    );
    assert_eq!(
        UNIT.moved(1.0, -2.0, 3.0),
        Aabb::new(1.0, -2.0, 3.0, 2.0, -1.0, 4.0)
    );
}

#[test]
fn size_is_mean_edge_length() {
    assert_eq!(Aabb::new(0.0, 0.0, 0.0, 1.0, 2.0, 3.0).size(), 2.0);
}

#[test]
fn collide_stops_flush_against_obstacle() {
    // Mover sits 0.5 to the right of the unit box, moving left by 2.
    let mover = player_at(1.5, 0.0, 0.2);
    assert_eq!(UNIT.clip_x_collide(mover, -2.0), -0.5);
    // Moving away is unaffected.
    assert_eq!(UNIT.clip_x_collide(mover, 2.0), 2.0);
    // Short of the obstacle is unaffected.
    assert_eq!(UNIT.clip_x_collide(mover, -0.25), -0.25);
}

#[test]
fn collide_lands_on_top() {
    let mover = player_at(0.2, 1.25, 0.2);
    assert_eq!(UNIT.clip_y_collide(mover, -1.0), -0.25);
}

#[test]
fn collide_from_flush_contact_yields_zero() {
    let resting = player_at(0.2, 1.0, 0.2);
    let ya = UNIT.clip_y_collide(resting, -0.08);
    assert_eq!(ya, 0.0);
    assert!(ya.is_sign_positive());
}

#[test]
fn collide_ignores_obstacles_only_touched_on_other_axes() {
    // Directly beside the box on Z, touching its face: sliding along X passes.
    let mover = player_at(1.5, 0.0, 1.0);
    assert_eq!(UNIT.clip_x_collide(mover, -2.0), -2.0);
}

#[test]
fn collide_ignores_already_overlapping_mover() {
    let mover = player_at(0.8, 0.0, 0.2);
    assert_eq!(UNIT.clip_x_collide(mover, -2.0), -2.0);
    assert_eq!(UNIT.clip_x_collide(mover, 2.0), 2.0);
}

#[test]
fn collide_passes_nan_through() {
    let mover = player_at(1.5, 0.0, 0.2);
    assert!(UNIT.clip_x_collide(mover, f64::NAN).is_nan());
}

#[test]
fn intersects_excludes_touching_but_inner_includes_it() {
    let touching = UNIT.moved(1.0, 0.0, 0.0);
    assert!(!UNIT.intersects(touching));
    assert!(UNIT.intersects_inner(touching));

    let overlapping = UNIT.moved(0.5, 0.5, 0.5);
    assert!(UNIT.intersects(overlapping));
    assert!(UNIT.intersects_inner(overlapping));

    let apart = UNIT.moved(1.5, 0.0, 0.0);
    assert!(!UNIT.intersects(apart));
    assert!(!UNIT.intersects_inner(apart));
}

#[test]
fn nan_bound_disables_its_side_of_the_test() {
    // Overlap is `!(b1 <= a0 || b0 >= a1)`. A NaN `x0` makes the
    // first comparison false, so a box far off the low side still intersects;
    // the high side is still checked.
    let nan = Aabb::new(f64::NAN, 0.0, 0.0, 1.0, 1.0, 1.0);
    assert!(nan.intersects(UNIT.moved(-100.0, 0.0, 0.0)));
    assert!(!nan.intersects(UNIT.moved(100.0, 0.0, 0.0)));
}

#[test]
fn contains_is_open_and_lower_bound_variant_is_half_open() {
    let low_face = Vec3::new(0.0, 0.5, 0.5);
    let high_face = Vec3::new(1.0, 0.5, 0.5);
    let inside = Vec3::new(0.5, 0.5, 0.5);

    assert!(UNIT.contains(inside));
    assert!(!UNIT.contains(low_face));
    assert!(!UNIT.contains(high_face));

    assert!(UNIT.contains_including_lower_bound(inside));
    assert!(UNIT.contains_including_lower_bound(low_face));
    assert!(!UNIT.contains_including_lower_bound(high_face));
}

#[test]
fn distance_is_zero_inside_and_on_the_surface() {
    assert_eq!(UNIT.distance(Vec3::new(0.5, 0.5, 0.5)), 0.0);
    assert_eq!(UNIT.distance(Vec3::new(1.0, 0.0, 0.5)), 0.0);
    assert!(UNIT.distance(Vec3::new(-0.0, 0.5, 0.5)).is_sign_positive());
}

#[test]
fn distance_to_face_edge_and_corner() {
    // Facing a face: only one axis contributes.
    assert_eq!(UNIT.distance(Vec3::new(0.5, 3.0, 0.5)), 2.0);
    // Beyond an edge and a corner: the gaps combine.
    assert_eq!(UNIT.distance(Vec3::new(4.0, 0.5, -4.0)), 5.0);
    assert_eq!(UNIT.distance(Vec3::new(-2.0, 4.0, 7.0)), 7.0);
}

#[test]
fn distance_ignores_nan_axes() {
    let p = Vec3::new(f64::NAN, 3.0, 0.5);
    assert_eq!(UNIT.distance(p), 2.0);
}

#[test]
fn clip_reports_entry_face() {
    let cases = [
        (
            Vec3::new(-1.0, 0.5, 0.5),
            Vec3::new(2.0, 0.5, 0.5),
            Facing::West,
        ),
        (
            Vec3::new(2.0, 0.5, 0.5),
            Vec3::new(-1.0, 0.5, 0.5),
            Facing::East,
        ),
        (
            Vec3::new(0.5, -1.0, 0.5),
            Vec3::new(0.5, 2.0, 0.5),
            Facing::Down,
        ),
        (
            Vec3::new(0.5, 2.0, 0.5),
            Vec3::new(0.5, -1.0, 0.5),
            Facing::Up,
        ),
        (
            Vec3::new(0.5, 0.5, -1.0),
            Vec3::new(0.5, 0.5, 2.0),
            Facing::North,
        ),
        (
            Vec3::new(0.5, 0.5, 2.0),
            Vec3::new(0.5, 0.5, -1.0),
            Facing::South,
        ),
    ];
    for (from, to, face) in cases {
        let hit = UNIT.clip(from, to).expect("segment crosses the box");
        assert_eq!(hit.face, face, "{from} -> {to}");
    }

    let hit = UNIT
        .clip(Vec3::new(-1.0, 0.5, 0.25), Vec3::new(1.0, 0.5, 0.25))
        .expect("segment crosses the box");
    assert_eq!(hit.pos, Vec3::new(0.0, 0.5, 0.25));
}

#[test]
fn clip_misses() {
    assert_eq!(
        UNIT.clip(Vec3::new(-1.0, 1.5, 0.5), Vec3::new(2.0, 1.5, 0.5)),
        None
    );
    // Stops short of the box.
    assert_eq!(
        UNIT.clip(Vec3::new(-1.0, 0.5, 0.5), Vec3::new(-0.5, 0.5, 0.5)),
        None
    );
}

#[test]
fn clip_from_inside_reports_exit() {
    let hit = UNIT
        .clip(Vec3::new(0.5, 0.5, 0.5), Vec3::new(3.0, 0.5, 0.5))
        .expect("segment leaves the box");
    assert_eq!(hit.face, Facing::East);
    assert_eq!(hit.pos, Vec3::new(1.0, 0.5, 0.5));
}

#[test]
fn clip_tie_keeps_earliest_face() {
    // Enters exactly through the x0/y0 edge: west wins over down.
    let hit = UNIT
        .clip(Vec3::new(-1.0, -1.0, 0.5), Vec3::new(2.0, 2.0, 0.5))
        .expect("segment crosses the box");
    assert_eq!(hit.face, Facing::West);

    // Through the x0/y0/z0 corner: still west.
    let hit = UNIT
        .clip(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(2.0, 2.0, 2.0))
        .expect("segment crosses the box");
    assert_eq!(hit.face, Facing::West);

    // Through the y0/z0 edge: down wins over north.
    let hit = UNIT
        .clip(Vec3::new(0.5, -1.0, -1.0), Vec3::new(0.5, 2.0, 2.0))
        .expect("segment crosses the box");
    assert_eq!(hit.face, Facing::Down);
}

#[test]
fn display_uses_g_notation() {
    assert_eq!(UNIT.to_string(), "box[0, 0, 0 -> 1, 1, 1]");
    assert_eq!(
        Aabb::new(-0.3, 64.0, 0.1, 1234567.0, 0.00001, 1.0 / 3.0).to_string(),
        "box[-0.3, 64, 0.1 -> 1.23457e+06, 1e-05, 0.333333]"
    );
}
