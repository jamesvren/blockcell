# Article 19: MCP Server Integration — blockcell's Standalone MCP Subsystem

> Series: *In-Depth Analysis of the Open Source Project “blockcell”* — Article 19

## What MCP is

**MCP (Model Context Protocol)** is a standard way for AI systems to discover and call external tools and data sources through a common interface.

In blockcell, MCP is a good fit for:

- platform integrations such as GitHub / GitLab
- database access such as SQLite / PostgreSQL / MySQL
- external tool packs such as Filesystem and Puppeteer
- any custom server that speaks the MCP protocol

## Current architecture

blockcell no longer stores MCP servers under `config.json5` as a `mcpServers` field. Instead, it uses a **standalone MCP configuration layer**:

- `~/.blockcell/mcp.json` — global MCP meta-config
- `~/.blockcell/mcp.d/*.json` — one file per server

The boundary between MCP and multi-agent runtime is now clearer:

- **MCP is infrastructure** — server definitions are global
- **Agents bind permission views** — each agent declares which MCP servers / tools it may see
- **Runtime is shared** — a single `McpManager` starts and reuses MCP servers within one process

## Quick start

### Option 1: add via CLI (recommended)

```bash
# Add GitHub MCP
blockcell mcp add github

# Add SQLite MCP
blockcell mcp add sqlite --db-path /tmp/test.db

# Inspect current MCP config
blockcell mcp list
```

### Option 2: edit files directly

If you use `blockcell mcp add github`, the current version writes `${env:GITHUB_PERSONAL_ACCESS_TOKEN}` into the generated file as a **literal placeholder string**. blockcell does **not** expand `${env:VAR}` syntax for MCP config today, so you should open `mcp.d/github.json` and replace it manually before use, or remove that key and rely on the parent process environment.

For example, create `~/.blockcell/mcp.d/github.json`:

> Note: `mcp.json` and `mcp.d/*.json` are currently **strict JSON**, not JSON5. Values under `env` are passed to child processes as-is.

```json
{
  "name": "github",
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-github"],
  "env": {
    "GITHUB_PERSONAL_ACCESS_TOKEN": "YOUR_GITHUB_TOKEN"
  },
  "enabled": true,
  "autoStart": true
}
```

Or create `~/.blockcell/mcp.d/sqlite.json`:

```json
{
  "name": "sqlite",
  "command": "uvx",
  "args": ["mcp-server-sqlite", "--db-path", "/tmp/test.db"],
  "enabled": true,
  "autoStart": true
}
```

After editing, restart:

```bash
blockcell agent
# or
blockcell gateway
```

## Configuration fields

Each `mcp.d/<name>.json` file supports these fields:

| Field | Type | Meaning |
|------|------|---------|
| `name` | string | Logical server name; also becomes the tool-name prefix |
| `command` | string | Executable to launch, such as `npx` or `uvx` |
| `args` | array | Startup arguments |
| `env` | object | Extra environment variables passed to the child process |
| `cwd` | string/null | Working directory |
| `enabled` | bool | Whether the server is enabled |
| `autoStart` | bool | Whether to auto-start it when blockcell starts |
| `startupTimeoutSecs` | integer | Startup / handshake timeout |
| `callTimeoutSecs` | integer | Tool call timeout |

`mcp.json` can additionally hold root-level defaults shared by all servers.

## Tool naming rule

Inside blockcell, MCP tools are exposed with the form:

```text
<serverName>__<toolName>
```

Examples:

- `github__list_issues`
- `sqlite__query`
- `filesystem__read_file`

## Relationship with multi-agent runtime

This is the key boundary after the MCP refactor:

- **MCP is not agent-owned config**
- **Agents only bind MCP visibility and permission scope**

That means:

- server definitions are global
- agents use `allowedMcpServers` / `allowedMcpTools` to control visibility
- runtime MCP processes are shared inside the same blockcell process

## CLI management commands

```bash
blockcell mcp list
blockcell mcp show github
blockcell mcp add github
blockcell mcp add sqlite --db-path /tmp/app.db
blockcell mcp add custom --raw --name custom --command uvx --arg my-mcp-server
blockcell mcp enable github
blockcell mcp disable github
blockcell mcp remove github
blockcell mcp edit github
```

Built-in templates currently supported by `blockcell mcp add <template>` are:

- `github`
- `sqlite`
- `filesystem`
- `postgres`
- `puppeteer`

## How it works internally

Internally, blockcell uses a shared `McpManager`:

1. Read `mcp.json` and `mcp.d/*.json`
2. Merge them into runtime `McpResolvedConfig`
3. Auto-start servers with `enabled && autoStart`
4. Call `tools/list`
5. Filter visible MCP tools by each agent's MCP permission view
6. Forward actual execution through `tools/call`

## Troubleshooting

### 1) `blockcell mcp list` does not show the new server

Confirm the file exists in:

- `~/.blockcell/mcp.json`
- `~/.blockcell/mcp.d/<name>.json`

Also check that the JSON is valid.

### 2) The config exists, but MCP tools do not appear

MCP config changes currently require a restart to take effect:

```bash
blockcell agent
# or
blockcell gateway
```

### 3) The server fails to start

Validate the command manually first:

```bash
uvx mcp-server-sqlite --db-path /tmp/test.db
npx -y @modelcontextprotocol/server-github
```

### 4) An agent cannot see an MCP tool

Check the agent's:

- `allowedMcpServers`
- `allowedMcpTools`

If the server or tool is not allowed, it will not be injected into that agent's visible registry.

## Recommendations

- Keep high-frequency, low-level, platform-agnostic capabilities as built-in tools
- Prefer MCP for third-party platforms, databases, and specialized external systems
- Start with `blockcell mcp add <template>` for simple cases
- Switch to direct `mcp.d/*.json` editing for advanced setups

---

*Previous: [Proxy and Provider Configuration](./18_proxy_and_provider_config.md)*

*Next: [Provider Pool](./20_provider_pool.md)*

*Index: [Series directory](./00_index.md)*
