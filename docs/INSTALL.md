# Install and test

## Claude Code with Rust

Use a current stable Rust toolchain for source builds. This version was tested with Rust 1.100.0. From the cloned repository:

```sh
cargo build --release --locked
./target/release/stepwise demo
cargo test --locked
claude plugin validate .
claude --plugin-dir .
```

Invoke `/stepwise-architect:project-adviser` with the current product task. The skill calls `scripts/stepwise.sh` relative to its installed directory. Claude substitutes `${CLAUDE_SKILL_DIR}` in the skill body. The default `.mcp.json` has no servers; Node is not needed for this path.

Source marketplace install: `/plugin install stepwise-architect --marketplace SoumaCloud/stepwise-architect`. A source install requires a Rust build in the actual installed cache copy because compiled `target/` files are not committed. The launcher reports the required build command when the helper is missing.

## Standalone native skill

From a clone with Rust installed:

```sh
./scripts/package-claude-skill.sh /absolute/output/project-adviser.tar.gz
```

This builds for the current machine and packages `project-adviser/` with its native executable, shell launcher, instructions, guide, license and target information. Extract that folder into a project's `.claude/skills/` directory and invoke `/project-adviser`. A matching archive does not require Rust or Node at runtime.

Native executables require the matching operating system and CPU shown in `BUILD.txt`. Build a separate archive for each platform. The included launcher needs a POSIX shell. Windows users need a compatible Unix environment or can build and run the Rust CLI directly; a native PowerShell launcher is not included.

## Change context or instructions

The profile is read on each task. Use `status --project /absolute/product`, preserve unchanged fields and supply a complete revised profile with `update-profile --project /absolute/product --file revised-profile.json`. Decisions and recipes remain intact. All project directory arguments must be absolute and point to an existing directory.

Edit `SKILL.md` to change Claude's workflow instructions without recompiling Rust. Claude Code detects local skill text changes; use `/reload-skills` if needed. Use `--catalog /absolute/catalog.json` for an explicitly reviewed catalog update. There is no background updater.

## Optional Node MCP bridge

The earlier bridge remains available for Codex, Gemini CLI and existing MCP clients. This separate path requires Node.js 22+ and `npm ci --ignore-scripts` in its actual installed copy. It is not needed by the Claude Rust skill. Do not configure duplicate servers.

For Codex, add the repository source with `codex plugin marketplace add /absolute/path/to/stepwise-architect`, then install from the desktop Plugins UI. The portable `mcp.json` points at the Node bridge using `${PLUGIN_ROOT}`. If the client cannot load a packaged local server, use:

```sh
codex mcp add stepwise -- node /absolute/path/to/stepwise-architect/bin/stepwise.mjs mcp
```

For Gemini CLI:

```sh
gemini extensions link /absolute/path/to/stepwise-architect
```

The Gemini manifest uses `${extensionPath}`. Restart if required, then use `/stepwise` or ask for project advice. Full native sessions in these clients still need evaluation.

Any stdio MCP client can launch executable `node` with two separate arguments: the absolute path to `bin/stepwise.mjs`, and `mcp`. The bridge exposes advice, research plans, cost comparisons and project memory. It does not provide a Rust MCP server.

## Uninstall and data

Remove the installed skill folder or use the client's plugin manager. Project memory remains in `.stepwise/` until deliberately deleted. It is plain local JSON; ignore it in product Git history and do not store credentials or customer records. Each process has its operating-system permissions; it is not a filesystem sandbox.

## Official references

- [Claude Code skills and supporting files](https://code.claude.com/docs/en/skills)
- [Claude plugin reference](https://code.claude.com/docs/en/plugins/manifest-reference)
- [OpenAI plugin packaging](https://developers.openai.com/plugins/build/plugins)
- [Gemini extension reference](https://geminicli.com/docs/extensions/reference/)

Instructions checked 26 November 2026. Client features and marketplace policies can change.
