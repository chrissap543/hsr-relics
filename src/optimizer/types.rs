use crate::relic::Stat;

pub struct Goal {
    stat: Stat,
    kind: GoalKind
}

enum GoalKind {
    MAX,
    BREAKPOINT(f32),
}