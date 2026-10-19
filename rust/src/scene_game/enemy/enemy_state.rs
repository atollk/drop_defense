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
            EnemyState::Walking => "Enemy/Walk",
            EnemyState::Dragged => "Enemy/Stand",
            EnemyState::Falling => "Enemy/Stand",
            EnemyState::Attacking => "Enemy/Attack",
            EnemyState::StandingUp => "Enemy/StandUp",
            EnemyState::Dying => "Enemy/Death",
            EnemyState::Swallowed => "Enemy/Stand",
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

    pub const fn is_portal_key(&self) -> bool {
        matches!(self, EnemyState::Dragged)
    }
}
