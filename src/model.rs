//! TypeSafe System One wire types. Every provider converts to and from these.
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Instructions / criteria text: a string, or structured JSON referencing `state` fields.
pub type Text = Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub state: Value,
    pub questions: IndexMap<String, Question>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
    Noul {
        instructions: Text,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        criteria: Option<NoulCriteria>,
    },
    Choice {
        instructions: Text,
        /// option -> description (None when the label says it all)
        criteria: IndexMap<String, Option<Text>>,
    },
    Score {
        instructions: Text,
        /// ordered levels, low to high
        criteria: Vec<Text>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoulCriteria {
    #[serde(rename = "true", default, skip_serializing_if = "Option::is_none")]
    pub yes: Option<Text>,
    #[serde(rename = "false", default, skip_serializing_if = "Option::is_none")]
    pub no: Option<Text>,
}

/// `confidence`, `probabilities`, `legend` are optional: Vercel omits some of them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
    Noul {
        noul: f64,
    },
    Choice {
        choice: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        confidence: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        probabilities: Option<IndexMap<String, f64>>,
    },
    Score {
        score: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        confidence: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        legend: Option<IndexMap<String, String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        probabilities: Option<IndexMap<String, f64>>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub answers: IndexMap<String, Answer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}
