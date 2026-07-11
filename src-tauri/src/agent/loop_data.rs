//! Loop data for the Orchestrator Agent.
//! Tracks iteration count, last response, and offset during a single message loop.
//! Reset each time the agent starts processing a new user message.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoopData {
    /// Current iteration number within the message loop
    pub iteration: u64,
    /// The last model response as raw String (used for [H2] repeat detection)
    pub last_response: Option<String>,
    /// Offset within current response (used for streaming tokens alone)
    pub offset: u64,
}

impl LoopData {
    pub fn reset_iteration(&mut self) {
        self.last_response = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_data_default() {
        let ld = LoopData::default();
        assert_eq!(ld.iteration, 0);
        assert!(ld.last_response.is_none());
        assert_eq!(ld.offset, 0);
    }

    #[test]
    fn test_loop_data_reset_iteration() {
        let mut ld = LoopData {
            iteration: 5,
            last_response: Some("previous output".into()),
            offset: 10,
        };
        ld.reset_iteration();
        assert!(ld.last_response.is_none());
        // iteration and offset remain unchanged
        assert_eq!(ld.iteration, 5);
        assert_eq!(ld.offset, 10);
    }

    #[test]
    fn test_loop_data_iteration_increment() {
        let mut ld = LoopData::default();
        ld.iteration += 1;
        assert_eq!(ld.iteration, 1);
    }
}