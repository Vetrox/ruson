#[derive(Debug, Clone)]
pub enum Typ {
    /// ALL
    Bot,
    /// ANY
    Top,
    /// ALL integers
    Int { constant: i64 },
}

impl Typ {
    pub fn is_simple(&self) -> bool {
        matches!(self, Typ::Bot | Typ::Top)
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, Typ::Top)
    }

    /// TODO: really think about allowed transitions.
    pub fn transition_allowed(&self, other: &Typ) -> bool {
        match self {
            Typ::Bot => true,
            Typ::Top => matches!(other, Typ::Top),
            Typ::Int { .. } => matches!(other, Typ::Int { .. } | Typ::Top),
        }
    }
}