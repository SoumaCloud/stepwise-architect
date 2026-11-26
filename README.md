# Stepwise Architect

A free **Claude Code skill with a Rust core** for indie builders. It reads your project context, helps choose a tool for the current task, compares costs and saves decisions or reusable prompts.

Instructions live in `skills/project-adviser/SKILL.md`. Rust handles the local commands. Claude supplies reasoning, code inspection and live research through the tools available in its session. The Claude skill needs **no Node, MCP server or extra model API key**.

**Version 0.2.0 is an experimental alpha.** The earlier optional Node MCP bridge remains available for Codex, Gemini CLI and existing integrations. It is separate from the Rust skill.

## Try Rust

Install a current stable Rust toolchain, clone the repository, and run from its directory:

```sh
cargo build --release --locked
./target/release/stepwise demo
cargo test --locked
```

The executable embeds the initial catalog and workflows and runs from any directory. Use `--catalog /absolute/catalog.json` to select an explicitly reviewed local update.

## Use the Claude Code skill

After building, launch the source plugin:

```sh
claude --plugin-dir /absolute/path/to/stepwise-architect
```

Then ask:

```text
/stepwise-architect:project-adviser Choose a cost-effective review workflow for this product
```

The skill reads the saved context, calls Rust and verifies current evidence. It does not auto-start the Node MCP server.

To package a standalone skill for the same operating system and CPU as your build machine:

```sh
./scripts/package-claude-skill.sh /absolute/output/project-adviser.tar.gz
```

Extract the archive's `project-adviser/` folder into a project's `.claude/skills/` directory, then invoke `/project-adviser`. It contains the Rust binary, launcher, instructions and guide. Recipients do not need Rust or Node to run a matching native executable. Build separate archives for other platforms; macOS ARM64 binaries do not run on Linux or Windows. The launcher needs a POSIX shell.

Source marketplace install: `/plugin install stepwise-architect --marketplace SoumaCloud/stepwise-architect`. Build Rust in the actual installed plugin copy. Public directory approval is separate from hosting this repo. See [installation details](docs/INSTALL.md).

## Adapt to project changes

Copy `examples/seo-project.json` and replace it with your actual product, tools and requirements. Initialize an explicit existing project directory:

```sh
./target/release/stepwise init --project /absolute/product --file examples/seo-project.json
./target/release/stepwise advise --project /absolute/product --stage review --task "Review code within my budget"
./target/release/stepwise status --project /absolute/product
```

When requirements change, read the saved profile and preserve unchanged fields in a complete revised object:

```sh
./target/release/stepwise update-profile --project /absolute/product --file /absolute/revised-profile.json
./target/release/stepwise advise --project /absolute/product --stage database --task "Revisit the database under my new data restrictions"
```

The next recommendation uses the revised context. Earlier decisions and recipes remain available. Skill instructions can be edited independently of the executable. There is no automatic self-modification or background monitoring.

Other commands:

```sh
./target/release/stepwise research --project /absolute/product --task "Validate demand for this idea"
./target/release/stepwise compare-costs --file examples/costs.json
./target/release/stepwise remember --project /absolute/product --file examples/decision.json
./target/release/stepwise recipe --project /absolute/product --file examples/recipe.json
```

The cost example uses invented USD inputs: **160 current, 115 proposed and 45 estimated monthly saving**, with 60 switching cost and about 1.33 months payback. Replace every rate and quantity with comparable actual workload inputs. These are not vendor prices or verified savings.

## Coverage and limitations

There are **20 curated candidates and 15 workflows**, covering research, validation, planning, architecture, UI, coding, review, agents, database, testing, automation, deployment, marketing, operations and costs. Ranking uses task tags, existing tools and explicit constraints. Unknown metadata remains a check. Claude must verify exact project fit, current capabilities and pricing.

- Only `init`, `update-profile`, `remember` and `recipe` write project state. The helper makes no network calls, launches no child commands and emits no telemetry.
- State is plain local JSON in `.stepwise/`, compatible with the older Node bridge. Ignore it in the product's Git history. Do not store secrets or customer records there.
- Claude can see command results and may process them with its model provider. A sensitive-data flag does not change Claude's own settings. This is not a filesystem sandbox or a compliance certification.
- Research remains a plan until Claude collects evidence. The bundled catalog is not a live feed. Daily updates, billing imports and an architecture dashboard are later milestones.
- Passing CLI checks does not prove that every model session follows the skill or produces good architecture advice. Native model-driven evaluations are still needed.

## Development

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

For the optional legacy bridge, use Node.js 22+, `npm ci --ignore-scripts`, `npm test` and `npm run check`.

See [validation](docs/VALIDATION.md), [the day-wise plan](docs/ROADMAP.md), [competition notes](docs/MARKET_NOTES.md), [contributing](CONTRIBUTING.md) and [security](SECURITY.md).

MIT licensed. No signup is required for the local helper. Claude and external services follow their own plans and terms.
