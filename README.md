# jev-cli

An unofficial, provider-neutral command-line client for [Jev](https://typesafe.ai/blog/introducing-system-one-models-and-jev), the System One model from [TypeSafe AI](https://typesafe.ai/). Written in Rust.

Supported providers:

- [TypeSafe AI](https://docs.typesafe.ai/api)
- [Cloudflare Workers AI](https://developers.cloudflare.com/ai/models/typesafe/jev/)
- [Vercel AI Gateway](https://vercel.com/ai-gateway/models/jev)

The command-line interface follows [stefafafan/jev](https://github.com/stefafafan/jev), a Go client for the same model.

## Installation

```bash
cargo install --git https://github.com/polidog/jev-cli
```

Or build from source:

```bash
cargo build --release   # binary at target/release/jev
```

## Quick start

```bash
export TYPESAFE_API_KEY=...
echo "Help! My payouts have been failing for 3 days." | jev noul "Does this convey urgency?"
```

```json
{"provider":"typesafe","model":"jev-1.13.0","answers":{"result":{"type":"noul","noul":0.95}},"usage":{"input_tokens":280,"output_tokens":20}}
```

State is always read from stdin. `jev` only reports the judgment; put thresholds downstream:

```bash
git diff | jev noul "Could this change introduce a regression?" | jq -e '.answers.result.noul < 0.2'
```

## Questions

Each subcommand asks one question. The answer is returned under the key `result`.

### Noul

Probability that the answer is yes.

```bash
jev noul "Does this require immediate action?" < incident.txt
```

### Choice

Pick one of two or more options. Repeat `--option` for each one. Commas are ordinary label characters.

```bash
git diff | jev choice --option safe --option needs-review --option unsafe \
  "How should this change be classified?"
```

### Score

Rate along ordered levels, listed from low to high. Repeat `--level` for each one.

```bash
kubectl diff -f manifest.yaml | jev score --level low --level medium --level high \
  "How risky is this deployment?"
```

### Full requests

To ask several questions about the same state in one call, or to use structured instructions and criteria, pass a complete [TypeSafe request](https://docs.typesafe.ai/api) as a file or on stdin:

```json
{
  "state": "Help! My payouts have been failing for 3 days.",
  "questions": {
    "is_urgent": {
      "type": "noul",
      "instructions": "Does this convey urgency?",
      "criteria": { "true": "Explicitly time-sensitive", "false": "No urgency expressed" }
    },
    "department": {
      "type": "choice",
      "instructions": "Which team should handle this?",
      "criteria": { "billing": "Payments, invoicing, refunds", "technical": "Bugs, outages, integrations" }
    }
  }
}
```

```bash
jev request.json
jev < request.json
```

Questions are evaluated in parallel and answered under the same keys. The request is validated before it is sent.

## Providers

Choose a provider with `--provider` / `-p`, or set `JEV_PROVIDER`. The default is `typesafe`.

| Provider | Environment variables |
| --- | --- |
| `typesafe` | `TYPESAFE_API_KEY` |
| `cloudflare` | `CLOUDFLARE_ACCOUNT_ID`, `CLOUDFLARE_API_TOKEN` |
| `vercel` | `AI_GATEWAY_API_KEY` |

```bash
jev -p cloudflare noul "Is this safe?" < input.txt
```

Requests and responses always use TypeSafe's format. `jev` translates them for each provider; for example, Vercel's `boolean` questions and answers become `noul`.

## Output

JSON is the default. Each answer keeps the Jev primitive fields:

- Noul: `type`, `noul`
- Choice: `type`, `choice`, `confidence`, `probabilities`
- Score: `type`, `score`, `confidence`, `legend`, `probabilities`

Vercel AI Gateway does not return `confidence` or `legend`.

Use `--output text` / `-o text` to print just the value. With several answers, each line is `id<TAB>value`.

```bash
$ echo "Help! payouts failing" | jev -o text choice --option billing --option technical "Which team?"
billing
```

## Exit status

| Code | Meaning |
| --- | --- |
| 0 | The evaluation succeeded |
| 1 | The request failed (missing credentials, HTTP error, invalid request) |
| 2 | Invalid command-line usage |
