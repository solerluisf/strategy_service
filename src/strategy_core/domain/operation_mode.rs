use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationMode {
    Live,
    Paper,
    Readonly,
    Offline,
}

impl std::fmt::Display for OperationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationMode::Live => write!(f, "live"),
            OperationMode::Paper => write!(f, "paper"),
            OperationMode::Readonly => write!(f, "readonly"),
            OperationMode::Offline => write!(f, "offline"),
        }
    }
}

impl std::str::FromStr for OperationMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "live" => Ok(OperationMode::Live),
            "paper" => Ok(OperationMode::Paper),
            "readonly" => Ok(OperationMode::Readonly),
            "offline" => Ok(OperationMode::Offline),
            _ => Err(format!("Unknown operation mode: {}", s)),
        }
    }
}
