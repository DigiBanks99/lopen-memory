use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Draft,
    Planning,
    Building,
    Complete,
    Amending,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Draft => write!(f, "Draft"),
            State::Planning => write!(f, "Planning"),
            State::Building => write!(f, "Building"),
            State::Complete => write!(f, "Complete"),
            State::Amending => write!(f, "Amending"),
        }
    }
}

impl FromStr for State {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Draft" => Ok(State::Draft),
            "Planning" => Ok(State::Planning),
            "Building" => Ok(State::Building),
            "Complete" => Ok(State::Complete),
            "Amending" => Ok(State::Amending),
            other => Err(format!(
                "invalid state '{}'; must be one of: Draft, Planning, Building, Complete, Amending",
                other
            )),
        }
    }
}

/// Returns Ok(true) if transition is allowed, Ok(false) if it's a no-op (same state),
/// Err if the transition is invalid.
pub fn validate_transition(from: &str, to: &State) -> Result<bool, String> {
    let from_state = from.parse::<State>().map_err(|e| e)?;
    if &from_state == to {
        return Ok(false); // no-op
    }
    let allowed = match from_state {
        State::Draft => vec![State::Planning, State::Draft],
        State::Planning => vec![State::Building, State::Draft],
        State::Building => vec![State::Complete, State::Draft],
        State::Complete => vec![State::Amending, State::Draft],
        State::Amending => vec![State::Draft],
    };
    if allowed.contains(to) {
        Ok(true)
    } else {
        Err(format!("invalid transition: {} → {}", from_state, to))
    }
}
