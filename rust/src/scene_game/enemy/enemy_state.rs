#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum EnemyState {
    Walking,
    Dragged,
    Falling,
    Attacking,
    StandingUp,
    Dying,
    Swallowed,
}

impl EnemyState {
    pub const fn animation_name(&self) -> &'static str {
        match self {
            EnemyState::Walking => "Walk",
            EnemyState::Dragged => "Stand",
            EnemyState::Falling => "Stand",
            EnemyState::Attacking => "Attack",
            EnemyState::StandingUp => "StandUp",
            EnemyState::Dying => "Death",
            EnemyState::Swallowed => "Stand",
        }
    }

    pub const fn is_moving_along_path(&self) -> bool {
        matches!(
            self,
            EnemyState::Walking
                | EnemyState::Attacking
                | EnemyState::StandingUp
                | EnemyState::Dying
        )
    }

    pub const fn is_falling(&self) -> bool {
        matches!(self, EnemyState::Falling | EnemyState::Swallowed)
    }

    pub const fn is_draggable(&self) -> bool {
        matches!(
            self,
            EnemyState::Walking
                | EnemyState::Attacking
                | EnemyState::StandingUp
                | EnemyState::Falling
        )
    }

    pub const fn is_physics_frozen(&self) -> bool {
        matches!(self, EnemyState::Walking)
    }
}
