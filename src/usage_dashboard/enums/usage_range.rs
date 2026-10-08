/// How far back the dashboard looks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum UsageRange {
    Week,
    #[default]
    Fortnight,
    Month,
}
