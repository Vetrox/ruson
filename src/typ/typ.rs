use std::cmp::PartialEq;
use Typ::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Typ {
    /// ALL: We have to honor what the programmer wrote and emit the code.
    Bot,
    /// ANY: We can choose any value, as convenient.
    Top,
    /// Concrete value integers
    Int { constant: i64 },
    IntTop,
    IntBot,
    /// Tuples; finite collections of unrelated Types, kept in parallel
    Tuple { typs: Vec<Typ> },
    TupleTop,
    TupleBot,
    Ctrl,
}

impl Typ {
    /// Simple types are implemented fully here.  "Simple" means: the code and
    /// type hierarchy are simple, not that the Type is conceptually simple.
    pub fn is_simple(&self) -> bool {
        matches!(self, Bot | Top | Ctrl)
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, Top | Int { .. })
    }

    pub fn transition_allowed(&self, other: &Typ) -> bool {
        self.meet(&other) == *self
    }

    pub fn join(&self, other: &Typ) -> Typ {
        self.dual().meet(&other.dual()).dual()
    }

    pub fn dual(&self) -> Typ {
        match self {
            Bot => Top,
            Top => Bot,
            Int { .. } => self.clone(),
            IntTop => IntBot,
            IntBot => IntTop,
            Tuple { .. } => self.clone(),
            TupleTop => TupleBot,
            TupleBot => TupleTop,
            Ctrl => Ctrl
        }
    }

    pub fn meet(&self, other: &Typ) -> Typ {
        match self {
            Bot => Bot,
            Top => other.clone(),
            Int { constant } => match other {
                Int { constant: o_constant } => if constant == o_constant {
                    self.clone()
                } else {
                    IntBot
                },
                IntTop | Top => self.clone(),
                IntBot => IntBot,
                _ => Bot,
            },
            IntTop => match other {
                Top => self.clone(),
                Int { .. } | IntTop | IntBot => other.clone(),
                _ => Bot
            }
            IntBot => match other {
                Top => self.clone(),
                Int { .. } | IntTop | IntBot => IntBot,
                _ => Bot
            }
            Tuple { .. } | TupleTop | TupleBot => {
                if self == other {
                    return self.clone();
                }
                panic!("not implemented yet")
            },
            Ctrl => match other {
                Top => Ctrl,
                _ => Bot
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::typ::typ::Typ::{Bot, Ctrl, Int, IntTop, Top, TupleTop};

    #[test]
    fn should_meet_top_and_bot() {
        // Arrange & Act
        let result = Top.meet(&Bot);

        // Assert
        assert!(matches!(result, Bot));
    }

    #[test]
    fn should_join_top_and_bot() {
        // Arrange & Act
        let result = Top.join(&Bot);

        // Assert
        assert!(matches!(result, Top));
    }

    #[test]
    fn should_meet_top_and_int_top() {
        // Arrange & Act
        let result = Top.meet(&IntTop);

        // Assert
        assert!(matches!(result, IntTop));
    }

    #[test]
    fn should_meet_ctrl_and_tuple_top() {
        // Arrange & Act
        let result = Ctrl.meet(&TupleTop);

        // Assert
        assert!(matches!(result, Bot));
    }

    #[test]
    fn should_join_int_and_tuple_top() {
        // Arrange & Act
        let result = Int { constant: 84 }.join(&TupleTop);

        // Assert
        assert!(matches!(result, Top));
    }

    #[test]
    fn should_allow_transition_from_bot_to_int() {
        // Arrange & Act
        let result = Bot.transition_allowed(&Int { constant: 84 });

        // Assert
        assert!(result);
    }

    #[test]
    fn should_not_allow_transition_from_int_to_bot() {
        // Arrange & Act
        let result = Int { constant: 84 }.transition_allowed(&Bot);

        // Assert
        assert!(!result);
    }
}

