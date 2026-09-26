use super::Provider;
use crate::http;
use crate::model::{Answer, NoulCriteria, Question, Request, Response, Text, Usage};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct Vercel;

impl Provider for Vercel {
    fn evaluate(&self, req: &Request) -> anyhow::Result<Response> {
        let body = VercelRequest {
            state: &req.state,
            questions: req.questions.iter().map(|(k, q)| (k.as_str(), VercelQuestion(q))).collect(),
        };
        let headers = [
            ("ai-model-id", "typesafe-ai/jev"),
            ("ai-evaluation-model-specification-version", "4"),
            ("ai-gateway-protocol-version", "0.0.1"),
            ("ai-gateway-auth-method", "api-key"),
        ];
        let url = "https://ai-gateway.vercel.sh/v4/ai/evaluation-model";
        let res: VercelResponse = http::post(url, &http::env_key("AI_GATEWAY_API_KEY")?, &headers, &body)?;
        Ok(res.into())
    }
}

// --- request: AI SDK names TypeSafe's `noul` as `boolean` ---

#[derive(Serialize)]
struct VercelRequest<'a> {
    state: &'a Value,
    questions: IndexMap<&'a str, VercelQuestion<'a>>,
}

struct VercelQuestion<'a>(&'a Question);

impl Serialize for VercelQuestion<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(tag = "type", rename = "boolean")]
        struct Boolean<'a> {
            instructions: &'a Text,
            #[serde(skip_serializing_if = "Option::is_none")]
            criteria: &'a Option<NoulCriteria>,
        }
        match self.0 {
            Question::Noul { instructions, criteria } => Boolean { instructions, criteria }.serialize(s),
            q => q.serialize(s),
        }
    }
}

// --- response ---

#[derive(Deserialize)]
struct VercelResponse {
    answers: IndexMap<String, VercelAnswer>,
    usage: Option<VercelUsage>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum VercelAnswer {
    Boolean { probability: f64 },
    Choice { choice: String, probabilities: Option<IndexMap<String, f64>> },
    Score { score: f64, probabilities: Option<IndexMap<String, f64>> },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VercelUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
}

impl From<VercelResponse> for Response {
    fn from(r: VercelResponse) -> Self {
        let answers = r
            .answers
            .into_iter()
            .map(|(k, a)| {
                let a = match a {
                    VercelAnswer::Boolean { probability } => Answer::Noul { noul: probability },
                    VercelAnswer::Choice { choice, probabilities } => {
                        Answer::Choice { choice, confidence: None, probabilities }
                    }
                    VercelAnswer::Score { score, probabilities } => {
                        Answer::Score { score, confidence: None, legend: None, probabilities }
                    }
                };
                (k, a)
            })
            .collect();
        let usage = r.usage.map(|u| Usage { input_tokens: u.input_tokens, output_tokens: u.output_tokens });
        Response { model: None, answers, usage }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn noul_is_sent_as_boolean() {
        let req: Request = serde_json::from_value(json!({"state": "s", "questions": {
            "a": {"type": "noul", "instructions": "x"},
            "b": {"type": "choice", "instructions": "y", "criteria": {"p": null, "q": "desc"}}
        }}))
        .unwrap();
        let body = VercelRequest {
            state: &req.state,
            questions: req.questions.iter().map(|(k, q)| (k.as_str(), VercelQuestion(q))).collect(),
        };
        let v = serde_json::to_value(&body).unwrap();
        assert_eq!(v["questions"]["a"], json!({"type": "boolean", "instructions": "x"}));
        assert_eq!(v["questions"]["b"]["type"], "choice");
        assert_eq!(v["questions"]["b"]["criteria"], json!({"p": null, "q": "desc"}));
    }

    #[test]
    fn boolean_answer_becomes_noul() {
        let r: VercelResponse = serde_json::from_value(json!({
            "answers": {"a": {"type": "boolean", "probability": 0.9}},
            "usage": {"inputTokens": 3, "outputTokens": 1}
        }))
        .unwrap();
        let r = Response::from(r);
        assert_eq!(r.answers["a"], Answer::Noul { noul: 0.9 });
        assert_eq!(r.usage.unwrap().input_tokens, Some(3));
    }
}
