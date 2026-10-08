use crate::status_bar::SystemLoad;

const GIB: u64 = 1024 * 1024 * 1024;

fn load(used_gib: f64, total_gib: u64) -> SystemLoad {
    SystemLoad {
        cpu: 0.234,
        memory_used: (used_gib * GIB as f64) as u64,
        memory_total: total_gib * GIB,
        ..SystemLoad::default()
    }
}

#[test]
fn memory_is_told_in_gigabytes_with_a_decimal_below_ten() {
    assert_eq!(load(12.4, 32).memory_words(), "12 / 32 GB");
    assert_eq!(load(6.25, 16).memory_words(), "6.2 / 16 GB");
}

#[test]
fn the_memory_fraction_is_used_over_all_and_never_divides_by_nothing() {
    assert_eq!(load(8., 32).memory_fraction(), 0.25);
    assert_eq!(SystemLoad::default().memory_fraction(), 0.);
}

#[test]
fn the_processor_is_a_whole_percent() {
    assert_eq!(load(1., 8).cpu_words(), "23%");
    assert_eq!(
        SystemLoad {
            cpu: 7.,
            ..SystemLoad::default()
        }
        .cpu_words(),
        "100%",
        "held at all of it"
    );
}

#[test]
fn the_memory_hover_names_what_the_app_holds_when_it_is_known() {
    let mut system = load(8., 32);
    assert_eq!(system.memory_tooltip().lines().count(), 1);
    system.app_memory = Some(GIB * 3 / 2);
    assert_eq!(
        system.memory_tooltip().lines().last(),
        Some("atelier holds 1.5 GB")
    );
    system.app_memory = Some(300 * 1024 * 1024);
    assert_eq!(
        system.memory_tooltip().lines().last(),
        Some("atelier holds 300 MB")
    );
}
