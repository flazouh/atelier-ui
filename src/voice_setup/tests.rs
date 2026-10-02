use super::*;

#[test]
fn only_a_download_and_a_finished_setup_have_a_number() {
    assert_eq!(fraction(SetupPhase::Download(0.4)), Some(0.4));
    assert_eq!(fraction(SetupPhase::Ready), Some(1.));
    assert_eq!(fraction(SetupPhase::Prepare), None);
}

#[test]
fn a_bad_number_never_leaves_the_track() {
    assert_eq!(fraction(SetupPhase::Download(7.)), Some(1.));
    assert_eq!(fraction(SetupPhase::Download(-1.)), Some(0.));
    assert_eq!(fraction(SetupPhase::Download(f32::NAN)), Some(0.));
}

#[test]
fn the_words_say_how_much_has_come_down() {
    let (words, percent) = copy(SetupPhase::Download(0.42), 164.);
    assert_eq!(words, "Downloading speech model · 69 of 164 MB");
    assert_eq!(percent.as_deref(), Some("42%"));
    assert_eq!(copy(SetupPhase::Prepare, 164.).1, None);
    assert_eq!(copy(SetupPhase::Ready, 164.).0, "Ready");
}

#[test]
fn the_fill_eases_toward_its_number_without_passing_it() {
    let one = pour(0., 1., 0.016);
    let two = pour(one, 1., 0.016);
    assert!(one > 0. && two > one && two < 1.);
    assert!(pour(1., 0.5, 0.016) < 1. && pour(1., 0.5, 0.016) > 0.5);
}

#[test]
fn the_sweep_stays_on_the_cells_and_turns_around() {
    for k in 0..400 {
        let at = sweep_at(k as f32 * 0.02, CELLS);
        assert!((0. ..=(CELLS - 1) as f32 + 1e-4).contains(&at), "{at}");
    }
    assert!(sweep_at(0., CELLS) < 1e-6);
    assert!((sweep_at(SWEEP_SECONDS, CELLS) - (CELLS - 1) as f32).abs() < 1e-4);
    assert!(sweep_at(2. * SWEEP_SECONDS, CELLS) < 1e-4);
}

#[test]
fn cells_fill_from_the_left_in_step_with_the_number() {
    let lit = |fill: f32| (0..CELLS).filter(|&i| cell(i, CELLS, Some(fill), 0., false).lit > 0.7).count();
    assert_eq!(lit(0.), 0);
    assert_eq!(lit(0.5), CELLS / 2);
    assert_eq!(lit(1.), CELLS);
}

#[test]
fn the_head_cell_comes_up_part_way_between_two_whole_cells() {
    // 10.5 cells of 32.
    let head = cell(10, CELLS, Some(10.5 / CELLS as f32), 0., false);
    assert!(head.lit > 0.3 && head.lit < 0.7 && head.hot > 0.9, "{head:?}");
}

#[test]
fn the_cells_just_behind_the_head_burn_hotter_than_those_far_back() {
    let fill = 20. / CELLS as f32;
    let hot = |i| cell(i, CELLS, Some(fill), 0., false).hot;
    assert!(hot(19) > hot(18) && hot(18) > hot(17) && hot(17) > 0.);
    assert_eq!(hot(5), 0.);
}

#[test]
fn a_finished_bar_has_nothing_hot() {
    assert!((0..CELLS).all(|i| cell(i, CELLS, Some(1.), 1.3, true).hot == 0.));
}

#[test]
fn without_a_number_a_cluster_is_bright_where_it_is_and_dark_far_off() {
    let seconds = 0.5;
    let at = sweep_at(seconds, CELLS);
    let bright = cell(at.round() as usize, CELLS, None, seconds, true);
    let far = cell(((at + 14.) as usize).min(CELLS - 1), CELLS, None, seconds, true);
    assert!(bright.lit > 0.7 && far.lit == 0.);
}

#[test]
fn without_motion_nothing_depends_on_the_time() {
    for i in 0..CELLS {
        assert_eq!(cell(i, CELLS, Some(0.4), 0., false), cell(i, CELLS, Some(0.4), 9., false));
        assert_eq!(cell(i, CELLS, None, 0., false), cell(i, CELLS, None, 9., false));
    }
}

#[test]
fn a_hot_cell_is_paler_and_a_dark_one_is_fainter() {
    let base = crate::voice_waves::amber();
    let warm = cell_color(base, Cell { lit: 1., hot: 0. });
    let hot = cell_color(base, Cell { lit: 1., hot: 1. });
    let dark = cell_color(base, Cell { lit: 0., hot: 0. });
    assert!(hot.l > warm.l && hot.s < warm.s);
    assert!(dark.a < 0.2 && warm.a > 0.99);
}
