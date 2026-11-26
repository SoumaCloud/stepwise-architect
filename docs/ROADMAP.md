# Day-wise prototype plan

Planning estimate: one developer, 15 focused working days after the initial alpha, plus time for tester responses. This is a delivery plan, not a guaranteed market outcome. The initial CLI/MCP alpha is already included in this repository. Reuse it rather than starting over.

| Day | Work | Concrete completion test |
| --- | --- | --- |
| 1 | Recruit five indie builders and record their current stage, stack, bills and recent tool-choice problem | Five consented profiles and a ranked problem list; no invented demand |
| 2 | Test the current install and demo with one builder per client | First successful session in Claude Code, Codex and Gemini CLI; record exact versions |
| 3 | Improve project context and mandatory requirement handling | SEO and sensitive-data examples produce different justified decisions; unknowns remain explicit |
| 4 | Verify a focused catalog of 20 to 30 tools | Each entry has official sources, access date, deployment details, pricing units and limitations |
| 5 | Run the research workflow on three actual product ideas | Source-linked alternatives, deduplicated pain evidence and a validation experiment for each |
| 6 | Benchmark review choices on five representative PRs | Useful findings, false positives, time and total cost recorded for each workflow |
| 7 | Add UI inspection recipes and test two real pages | Evidence-backed changes, desktop/mobile checks and a before/after record |
| 8 | Compare one single-agent and one multi-agent workflow | Same task and quality bar; calls, retries, latency and cost recorded |
| 9 | Extend the cost model with workload templates and provenance | Traceable inputs for review, AI calls, hosting, database and maintenance; no fake live prices |
| 10 | Add explicit recommendation outcomes and review triggers | Accepted and rejected choices persist; later advice respects the saved reason |
| 11 | Add on-demand official-source update checks | Dated diff with expected impact; no automatic migrations or unrequested background tasks |
| 12 | Prototype a read-only architecture view | Nodes show chosen tools, links, costs, assumptions and prompts; unknowns are visible |
| 13 | Make installation reliable across supported client versions | Fresh-clone and packaged-install smoke tests pass, dependencies resolve, uninstall documented |
| 14 | Run the public beta with the five recruited builders | Each completes a real task and reports friction, usefulness and measured outcomes |
| 15 | Fix the largest failures and publish a versioned release | Tagged source, test report, short demo and next milestone based on observed use |

## Release gate

Proposed beta gate, to be tested rather than treated as traction: all three clients complete the same project workflow; at least four of five testers reach first useful advice without developer intervention; at least three return for a second task within seven days; every financial claim has traceable inputs; no known severe data-handling failure. If these fail, narrow the scope and repair the workflow before adding more tools.

## Launch sequence

Start with a 60-second demo showing one actual project and one justified tool choice. Publish the GitHub repo, reproducible setup and three realistic examples. Invite feedback where self-promotion is allowed; manually approve public posts and outreach. Track install completion, first useful advice, repeat use, accepted recommendations and observed cost/quality outcomes. Stars are secondary.

## Later milestones

After the beta, consider opt-in scheduled update checks, a decision graph, billing imports, outcome-based ranking and team policy controls. Add a paid hosted offering only after repeat use establishes which ongoing service people value. Preserve the free local core. Do not promise compliance certification or universal autonomous architecture.
