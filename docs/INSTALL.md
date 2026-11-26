# Install and test

Node.js 22+ and npm are required. This alpha is distributed from source, not published on npm. First run `npm ci --ignore-scripts` in the repository directory. To avoid global installation, invoke `node /absolute/path/to/stepwise-architect/bin/stepwise.mjs` directly. The CLI demo itself has no external dependencies; MCP requires the installed SDK.

## Claude Code

For a local test, from the repository directory:

```sh
claude plugin validate .
claude --plugin-dir .
```

Invoke the bundled skill as `/stepwise-architect:project-adviser` or describe a project-aware tool-selection task. A repository marketplace is included. Once this repository is on GitHub, users can add its owner/repository as a marketplace with `/plugin marketplace add`, then install `stepwise-architect@stepwise-architect` using `/plugin install`.

Dependency setup matters: a marketplace may copy the plugin into a new cache folder without `node_modules`. Run `npm ci --ignore-scripts` in that installed folder before enabling its MCP server. For a simpler alpha setup, use `--plugin-dir` on a prepared local clone. `.mcp.json` uses `${CLAUDE_PLUGIN_ROOT}` so the server script remains relative to the plugin's installed location.

## Codex

The repository includes a portable `plugin.json`, `mcp.json`, skills and a repo marketplace at `.agents/plugins/marketplace.json`.

```sh
codex plugin marketplace add /absolute/path/to/stepwise-architect
```

Open Plugins in the desktop app, choose Stepwise Architect as the source, install and enable the plugin, then test it in a new chat. Restart if the new local source has not appeared. Public universal-directory listing is a separate reviewed submission.

The portable server uses `${PLUGIN_ROOT}`. Prepare dependencies in the actual installed copy. If your client version cannot load the packaged local server, use the supported direct MCP connection below and use the bundled skill as instructions:

```sh
codex mcp add stepwise -- node /absolute/path/to/stepwise-architect/bin/stepwise.mjs mcp
```

Do not enable both direct and plugin MCP connections at once. No user-wide Codex settings are modified by merely downloading this repo.

## Gemini CLI

From a prepared local clone:

```sh
gemini extensions link /absolute/path/to/stepwise-architect
```

Restart Gemini CLI if necessary. Use `/stepwise` with your task or ask for the project-adviser skill. The root `gemini-extension.json` and `${extensionPath}` point to this clone's MCP server. Public GitHub distribution can use `gemini extensions install` with the repository URL, followed by dependency setup in the installed copy. The Gemini CLI is not included in this repository.

## Any stdio MCP client

Set the executable to `node` and pass two arguments: the absolute path to `bin/stepwise.mjs`, and `mcp`. Keep the executable and arguments separate. Do not use shell interpolation. Example client configuration:

```json
{
  "mcpServers": {
    "stepwise": {
      "command": "node",
      "args": ["/absolute/path/to/stepwise-architect/bin/stepwise.mjs", "mcp"]
    }
  }
}
```

The server exposes `advise`, `research_plan`, `compare_costs`, `get_project`, `init_project`, `save_decision`, `save_recipe` and `get_catalog`. Project tools require an explicit path. The server has the file permissions of its process; it is not a filesystem sandbox. Use the host's normal approval controls.

## Uninstall and local data

Remove the plugin/extension or direct MCP entry using your client's manager. Project memory remains in each project's `.stepwise/` directory until you deliberately delete it. It is plain local JSON, separate from the plugin installation.

## References

- [Claude plugin manifest reference](https://code.claude.com/docs/en/plugins/manifest-reference)
- [OpenAI plugin packaging and marketplaces](https://developers.openai.com/plugins/build/plugins)
- [Gemini extension reference](https://geminicli.com/docs/extensions/reference/)
- [Agent Plugins MCP runtime](https://agent-plugins.org/)

Instructions checked against official documentation on 26 November 2026. Host versions and directory policies can change.
