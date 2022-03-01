use serde::{Serialize, Deserialize};



/// Request from client to server
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Set the value of a string key to a string
    Set {
        /// A string key
        key: String,
        /// A string value of the key
        value: String,
    },
    /// Get value from a given key
    Get {
        /// A string key
        key: String,
    },
    /// Remove a given key
    Rm {
        /// A string key
        key: String,
    },
}

/// Request from client to server
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    /// Success status
    Success {
        /// The result of given command
        result: Option<String>,
    },
    /// Fail status
    Fail {
        /// Error message
        message: String,
    },
}