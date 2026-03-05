# Tophub CLI Skill

## Purpose
Use `tophub-cli` to fetch Tophub API data, support concurrent node queries, and export results to timestamped files.

## Preconditions
- Run commands in project root: `tophub-cli`
- API key must be provided by either:
  - global CLI arg: `--apikey <KEY>`
  - `.env` variable: `TOPHUB_APIKEY=...`

## Command Routing
Map user intent to command:

- List leaderboard nodes:
  - `tophub-cli nodes -p <page>`
- Dump all node lists (page 1..100) to JSONL:
  - `tophub-cli nodes --dumpall`
- Query one or multiple node details:
  - `tophub-cli node <hashid>`
  - `tophub-cli node <hashid1,hashid2,...>`
- Query node history by date:
  - `tophub-cli node-historys <hashid> <YYYY-MM-DD>`
- Search hot content:
  - `tophub-cli search <keyword> -p <page> --hashid <optional_hashid>`
- Daily hot top list:
  - `tophub-cli hot --date <YYYY-MM-DD>`
- Node snapshots list/details:
  - `tophub-cli snapshots <hashid> [--date <YYYY-MM-DD>] [--details 0|1]`
- Single snapshot detail:
  - `tophub-cli snapshot <hashid> <ssid>`
- Calendar events:
  - `tophub-cli calendar-events [--mode day|week|month] [--date <YYYY-MM-DD>] [--categories <ids|all>]`
- Concurrent mixed fetch:
  - `tophub-cli batch --p <page> --hashid <hashid> --date <YYYY-MM-DD> --q <keyword>`

## Node Command Special Behavior
- `node` supports comma-separated hashids.
- For multiple hashids, requests run concurrently.
- Output order follows input order.
- A progress bar is shown during multi-hashid requests.

## Dump Behavior
`node` supports export:
- `--dump csv`
- `--dump json`
- `--dump jsonl`

Generated filename pattern:
- `YYYY-MM-DD-HH-MM-SS.<fmt>`

Examples:
- `tophub-cli node mproPpoq6O --dump csv`
- `tophub-cli node mproPpoq6O,KqndgxeLl9 --dump jsonl`

CSV export is flattened by hotspot item (one hotspot per row), with columns:
- `hashid`
- `name`
- `display`
- `domain`
- `logo`
- `latest_update_timestamp`
- `rank`
- `title`
- `description`
- `url`
- `extra`
- `thumbnail`
- `time`

## Output Contract
- Without dump options, commands print pretty JSON to stdout.
- With dump options, command prints completion summary including file path.

## Validation Rules
- `snapshots --details` only accepts `0` or `1`.
- `calendar-events --mode` only accepts `day`, `week`, `month`.
- Empty hashid input for `node` is invalid.

## Suggested LLM Workflow
1. Infer user intent and select command from Command Routing.
2. Build parameters with strict date and enum formats.
3. Prefer `--apikey` if user provides key in prompt; otherwise rely on `.env`.
4. If user asks for files, prefer `--dump` options.
5. Return concise execution summary and output file location.

## Quick Examples
```bash
tophub-cli --apikey YOUR_KEY nodes -p 1
tophub-cli node mproPpoq6O,KqndgxeLl9
tophub-cli node mproPpoq6O --dump csv
tophub-cli nodes --dumpall
tophub-cli calendar-events --mode week --date 2026-03-05 --categories 1,2,3
```
