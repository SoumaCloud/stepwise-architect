# Validation status

Build date: 26 November 2026. Version: 0.1.0 experimental alpha.

## Core and protocol checks

`npm test` covers CLI behavior, task relevance, local/sensitive-data exclusions, unknown metadata, stale-source warnings, cost arithmetic, invalid/overflowing inputs, observed-versus-forecast labels, local persistence, write contention and symlink refusal. An integration test starts the real stdio server using the official MCP SDK, initializes the protocol, lists eight tools, creates a project, receives advice, saves a decision and reads it back.

`npm run check` checks the tool catalog, workflow references, required package files and client-specific path variables. It is not a substitute for native host testing.

## Client checks

The Claude Code native validator passed, with a public-directory icon warning. The portable plugin and MCP manifests passed validation against the official Agent Plugins 1.0.0 schemas. The project-adviser skill passed the skill-creator validator. Gemini extension paths follow the official reference, but Gemini CLI was not installed for runtime testing. The Codex CLI returned no available entries for the unconfigured Stepwise source; marketplace discovery and a full session remain unverified. The full model-driven workflow still needs a fresh-session test in each client. Passing an MCP integration test does not prove the host invoked the skill, respected a recommendation or produced good architecture advice.

Local tests passed on Node 24.12.0: 17 tests, zero failures. Dependency audit after upgrading Zod reported zero known vulnerabilities. This audit is a point-in-time package check, not an independent security assessment. CI is configured for Node 22 and 24.

The initial [GitHub Actions run](https://github.com/SoumaCloud/stepwise-architect/actions/runs/45801492641) passed on both Node 22 and Node 24, including a clean dependency install, all 17 tests and package checks.

No claims of public marketplace approval, security certification, measured customer savings or validated demand are made by this alpha. The Word build report supplied with the initial delivery records the exact checks run and remaining gaps.

## Recommendation quality

Ranking is transparent: current-stage candidates are ordered by task-tag matches, with a modest preference for existing tools. Known conflicts are excluded and unknown configuration remains a check. This deterministic shortlist is an aid to the host assistant, not a benchmark proving which tool is best for every product.

## Reproduce

```sh
npm ci --ignore-scripts
npm test
npm run check
node bin/stepwise.mjs demo
node bin/stepwise.mjs compare-costs --file examples/costs.json
claude plugin validate .
```

Dependency versions are captured in package-lock.json. No native model-driven integration tests run automatically, so CI does not require assistant subscriptions or API keys.
