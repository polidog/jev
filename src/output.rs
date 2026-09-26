use crate::cli::{Output, ProviderKind};
use crate::model::{Answer, Response};
use clap::ValueEnum;
use serde::Serialize;

pub fn render(format: Output, provider: ProviderKind, res: &Response) -> String {
    match format {
        Output::Json => {
            let name = provider.to_possible_value().expect("no skipped variants");
            format!("{}\n", json(name.get_name(), res))
        }
        Output::Text => text(res),
    }
}

fn json(provider: &str, res: &Response) -> String {
    #[derive(Serialize)]
    struct Out<'a> {
        provider: &'a str,
        #[serde(flatten)]
        res: &'a Response,
    }
    serde_json::to_string(&Out { provider, res }).unwrap()
}

/// One answer: just its value. Several: `id<TAB>value` per line.
fn text(res: &Response) -> String {
    let single = res.answers.len() == 1;
    res.answers
        .iter()
        .map(|(id, a)| {
            let v = match a {
                Answer::Noul { noul } => noul.to_string(),
                Answer::Choice { choice, .. } => choice.clone(),
                Answer::Score { score, .. } => score.to_string(),
            };
            if single { format!("{v}\n") } else { format!("{id}\t{v}\n") }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn res(v: serde_json::Value) -> Response {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn text_output() {
        let one = res(json!({"answers": {"result": {"type": "choice", "choice": "billing"}}}));
        assert_eq!(text(&one), "billing\n");
        let two = res(json!({"answers": {"a": {"type": "noul", "noul": 0.9}, "b": {"type": "score", "score": 1.5}}}));
        assert_eq!(text(&two), "a\t0.9\nb\t1.5\n");
    }

    #[test]
    fn json_output_keeps_api_shape() {
        let r = res(json!({"model": "jev-1.13.0", "answers": {"result": {"type": "noul", "noul": 0.95}}}));
        assert_eq!(
            json("typesafe", &r),
            r#"{"provider":"typesafe","model":"jev-1.13.0","answers":{"result":{"type":"noul","noul":0.95}}}"#
        );
    }
}
