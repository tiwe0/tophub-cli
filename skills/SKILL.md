# Tophub CLI 技能说明

## 作用
使用 `tophub-cli` 获取 Tophub API 榜单数据，支持并发节点查询，并将结果导出为带时间戳的文件。

## 前置条件
- 在该技能目录下运行命令：`tophub-cli`，检验是否正确安装并可用。
- 对于以下命令，无需 API key：
  - `query-db`
- 对于以下命令，需要提供 API key，必须通过以下方式之一提供：
  - 命令:
    - `node`
  - 提供 API key 的方式（优先级从高到低）：
    - `.env` 文件变量：`TOPHUB_APIKEY=...`
    - 全局 CLI 参数：`--apikey <KEY>`

## 命令路由
根据用户意图选择命令，有时用户的意图可能需要多条命令：

- 查询节点数据库：
  - `tophub-cli query-db [--category <类别>] [--name <名称>] [--id <id>]`
- 列出榜单节点：
  - `tophub-cli nodes -p <页码>`
- 导出全部榜单（1~100页）到 JSONL：
  - `tophub-cli nodes --dumpall`
- 查询一个或多个节点详情：
  - `tophub-cli node <hashid>`
  - `tophub-cli node <hashid1,hashid2,...>`
- 查询节点历史：
  - `tophub-cli node-historys <hashid> <YYYY-MM-DD>`
- 全网热搜内容：
  - `tophub-cli search <关键词> -p <页码> --hashid <可选hashid>`
- 今日热榜榜中榜：
  - `tophub-cli hot --date <YYYY-MM-DD>`
- 节点快照列表/详情：
  - `tophub-cli snapshots <hashid> [--date <YYYY-MM-DD>] [--details 0|1]`
- 单个快照详情：
  - `tophub-cli snapshot <hashid> <ssid>`
- 日历事件：
  - `tophub-cli calendar-events [--mode day|week|month] [--date <YYYY-MM-DD>] [--categories <ids|all>]`
- 并发混合请求：
  - `tophub-cli batch --p <页码> --hashid <hashid> --date <YYYY-MM-DD> --q <关键词>`

## node 命令特殊说明
- `node` 支持逗号分隔多个 hashid。
- 多 hashid 时并发请求，输出顺序与输入一致。
- 多节点请求时会显示进度条。

## 导出行为
`node` 支持导出，查询时，优先加上 --dump 参数，方便后续用户溯源和数据分析。支持以下导出格式：
- `--dump csv`
- `--dump json`
- `--dump jsonl`
- `--name <filename>`（可带或不带扩展名）

生成文件名格式：
- `YYYY-MM-DD-HH-MM-SS.<fmt>`
或
- `--name` 指定的文件名（若未包含对应扩展名会自动补齐）

示例：
- `tophub-cli node mproPpoq6O --dump csv`
- `tophub-cli node mproPpoq6O,KqndgxeLl9 --dump jsonl`
- `tophub-cli node mproPpoq6O --dump csv --name weibo.csv`

CSV 导出按热点条目展开，每行一个热点，字段包括：
- `hashid`、`name`、`display`、`domain`、`logo`、`latest_update_timestamp`、`rank`、`title`、`description`、`url`、`extra`、`thumbnail`、`time`

## 输出约定
- 无 dump 选项时，命令输出格式化 JSON 到 stdout。
- 有 dump 选项时，输出导出完成提示及文件路径。

## 校验规则
- `snapshots --details` 仅支持 `0` 或 `1`。
- `calendar-events --mode` 仅支持 `day`、`week`、`month`。
- `node` 的 hashid 不能为空。

## LLM 推荐工作流
1. 推断用户意图，选择合适命令；当用户需求模糊时，需要确认用户需求；使用 API 且查询量比较大时，需要询问用户确认需求，防止开销过大。
2. 严格格式化日期和枚举参数。
3. 用户有 key 优先用 `--apikey`，否则用 `.env`。
4. 查询时，优先启用导出选项 `--dump` 保存到 csv 文件
5. 使用 wc 命令查看导出文件的规模，如果数据过大，则只整理输出前 10 行，并提示用户数据已导出到文件中，建议用户查看文件获取完整数据。
6. 返回热点整理，输出文件路径。整理需要包括 节点名称、标题、描述、URL、时间等关键信息，方便用户快速浏览。

## 快速示例
```bash
tophub-cli --apikey YOUR_KEY nodes -p 1
tophub-cli node mproPpoq6O,KqndgxeLl9
tophub-cli node mproPpoq6O --dump csv
tophub-cli calendar-events --mode week --date 2026-03-05 --categories 1,2,3
```

## 常见情景
1. 用户: 我想看看财经类的榜单有哪些
   - LLM: 使用 `query-db` 命令查询财经类榜单，输出简要列表。
2. 用户: 看一下微博的热搜
   - LLM: 使用 `query-db` 查找微博相关榜单，如果只找到一个节点，那么获取 hashid 后使用 `node` 命令查询详情; 如果找到多个节点，则需要用户确认具体查询哪个节点。
3. 用户: 我想看财经新闻
   - LLM: 使用 `query-db` 命令查询财经新闻相关的榜单，输出简要列表；询问用户需要查看哪些节点的详情；根据用户需求使用 `node` 命令查询详情。

## 所有命令参数说明

### 全局参数
- `--apikey <KEY>`：Tophub API key，优先于环境变量 `TOPHUB_APIKEY`。

### 子命令与参数

#### nodes
- `-p, --p <u32>`：页码，默认 1，每页 100 条。
- `--dumpall`：从 p=1 拉取到 p=100，并将所有榜单逐行写入 nodes.jsonl。
- `--name <String>`：导出文件名（不含或包含扩展名）。

#### query-db
- `--category <String>`：可选，类别，如 财经、报刊。
- `--name <String>`：可选，名称，支持模糊匹配。
- `--id <String>`：可选，id，支持精确或模糊。

#### node
- `<hashid>`：榜单 hashid，支持逗号分隔多个值。
- `--dump <csv|json|jsonl>`：导出结果格式。
- `--name <String>`：导出文件名（不含或包含扩展名）。

#### node-historys
- `<hashid>`：榜单 hashid。
- `<date>`：日期，格式 YYYY-MM-DD。

#### search
- `<q>`：搜索关键词。
- `-p, --p <u32>`：页码，默认 1。
- `--hashid <String>`：可选，限定某个榜单 hashid。

#### hot
- `--date <YYYY-MM-DD>`：日期，格式 YYYY-MM-DD。

#### snapshots
- `<hashid>`：榜单 hashid。
- `--date <YYYY-MM-DD>`：可选，日期，默认当天。
- `--details <0|1>`：可选，0=仅快照列表，1=包含详细内容。

#### snapshot
- `<hashid>`：榜单 hashid。
- `<ssid>`：快照 ID。

#### calendar-events
- `--mode <day|week|month>`：模式，默认 day。
- `--date <YYYY-MM-DD>`：可选，日期，默认当天。
- `--categories <String>`：可选，分类 ID，多个用逗号分隔，或 all。

#### batch
- `--p <u32>`：页码，默认 1。
- `--hashid <String>`：榜单 hashid。
- `--date <YYYY-MM-DD>`：日期。
- `--q <String>`：搜索关键词。

---
每个参数均有类型和用途说明，详见上方命令示例。
