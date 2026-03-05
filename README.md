# tophub-cli

一个基于 Rust 的 Tophub API 命令行工具，支持异步请求、并发抓取、重试、进度条和多格式导出。

## 功能
- 异步 HTTP 请求（`reqwest` + `tokio`）
- 从 `.env` 或 `--apikey` 读取 API Key
- 多个 `hashid` 并发查询（`node` 命令）
- 并发查询进度条（`node` 多 hashid）
- 请求失败自动重试（`node` 命令每个 hashid 最多 3 次）
- 失败自动丢弃（重试 3 次后跳过，不中断整个任务）
- 导出格式支持：`csv` / `json` / `jsonl`
- 导出文件名自动时间戳：`YYYY-MM-DD-HH-MM-SS.<fmt>`

## 环境要求
- Rust stable（建议最新稳定版）
- 有效的 Tophub API Key

## 安装与运行
在项目根目录执行：

```bash
cargo build --release
```

开发调试直接运行：

```bash
cargo run -- --help
```

## API Key 配置
优先级：`--apikey` > `.env`

### 方式 1：.env
创建 `.env`：

```env
TOPHUB_APIKEY="YOUR_ACCESS_KEY"
```

### 方式 2：命令行参数

```bash
cargo run -- --apikey YOUR_ACCESS_KEY nodes -p 1
```

## 命令总览

```bash
tophub-cli [OPTIONS] <COMMAND>
```

- `nodes` 获取全部榜单列表
- `node` 获取单个或多个榜单最新详细内容
- `node-historys` 获取单个榜单历史数据集
- `search` 全网热点内容搜索
- `hot` 获取今日热榜榜中榜
- `snapshots` 获取某榜单快照列表或详情集合
- `snapshot` 获取单个榜单指定快照详情
- `calendar-events` 获取热点日历事件
- `batch` 并发请求多个接口（`nodes/node/node-historys/search`）

## 常用示例

```bash
# 全部榜单（第 1 页）
cargo run -- nodes -p 1

# 导出全部榜单（p=1..100）到 jsonl
cargo run -- nodes --dumpall

# 单个榜单详情
cargo run -- node mproPpoq6O

# 多个榜单详情（逗号分隔，按输入顺序输出）
cargo run -- node mproPpoq6O,KqndgxeLl9

# node 结果导出为 csv/json/jsonl
cargo run -- node mproPpoq6O --dump csv
cargo run -- node mproPpoq6O,KqndgxeLl9 --dump json
cargo run -- node mproPpoq6O,KqndgxeLl9 --dump jsonl

# 历史数据
cargo run -- node-historys mproPpoq6O 2023-01-01

# 搜索
cargo run -- search 苹果 -p 1 --hashid mproPpoq6O

# 榜中榜
cargo run -- hot --date 2023-11-04

# 快照列表 / 详情集合
cargo run -- snapshots mproPpoq6O --date 2025-11-11 --details 0
cargo run -- snapshots mproPpoq6O --date 2025-11-11 --details 1

# 单个快照详情
cargo run -- snapshot mproPpoq6O 12345

# 日历事件
cargo run -- calendar-events --mode week --date 2023-11-04 --categories 1,2,3
```

## 导出说明

### `nodes --dumpall`
- 行为：固定拉取 `p=1..100`
- 输出：`<timestamp>.jsonl`
- 内容：每行一个节点（JSON Lines）

### `node --dump <fmt>`
支持 `csv` / `json` / `jsonl`。

#### CSV（已扁平化热点）
`node --dump csv` 会把 `data.items[]` 中每个热点提取为一行，字段如下：

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

#### JSON
- 输出结构：`{"items": [...]}`

#### JSONL
- 每行一个 `node` 结果对象（包含 `hashid` 与响应数据）

## 重试与容错策略（node 命令）
- `node` 请求失败（包括非 200）会自动重试
- 每个 `hashid` 最多 3 次
- 连续失败 3 次后自动丢弃该 `hashid`
- 多 hashid 模式下，其他 `hashid` 正常继续，不会整体失败

## 参数约束
- `snapshots --details` 仅支持 `0` 或 `1`
- `calendar-events --mode` 仅支持 `day|week|month`
- `node` 的 `hashid` 不能为空

## 进度条
- `node` 多 hashid 查询时显示进度条
- 单 hashid 不显示进度条

## 目录补充
- `skills/SKILL.md`：为 LLM 准备的技能文档（命令映射、参数规则、导出约定）

## 许可证
如需开源发布，请补充你的 License 文件（例如 MIT）。
