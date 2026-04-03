# MCP Tool Design Rules (Mandatory)

These rules apply whenever creating, modifying, or reviewing MCP tool definitions.
Violations must be flagged during code review (Step 7).

## 1. Language: English Only

All tool `name`, `description`, and parameter `description` fields MUST be in English.

- User queries arrive in Korean, but tool interfaces are for the LLM, not the user.
- Research evidence: non-English tool descriptions cause parameter value language mismatch
  and degrade selection accuracy (arxiv: "Lost in Execution", 2601.05366).
- Korean labels belong in UI display layers (tool_definitions.yaml `description.ko`), never in MCP schema.

**Bad:**
```
"웹에서 엔티티 관련 정보를 검색합니다."
```

**Good:**
```
"Search the web for information using keyword search, URL scraping, or deep research.
 Use when the user asks to look up facts, news, or external information.
 Do NOT use for internal KB lookups -- use kb_entity_search instead."
```

## 2. Description Structure (Minimum 3 Sentences)

Every tool description MUST include:

1. **Purpose** -- what the tool does (1 sentence)
2. **When to use** -- specific scenarios (1-2 sentences)
3. **When NOT to use** -- disambiguation from similar tools (1 sentence)
4. **Parameter guidance** -- non-obvious parameter semantics (as needed)

Template:
```
"{Purpose}. Use when {scenario}. Do NOT use for {alternative scenario} -- use {other_tool} instead.
 {Parameter notes if needed}."
```

## 3. Naming Convention

- Format: `verb_noun` (e.g., `search_web`, `extract_entities`, `execute_code`)
- NO duplicate verbs across tools unless clearly namespaced
- Avoid generic names like `search` or `analyze` alone
- If two tools share a verb, add a distinguishing noun:
  - Bad: `search_terms` vs `search_entities` (too similar)
  - Good: `lookup_translation_terms` vs `search_kb_entities`

## 4. Tool Count

- Target: **5-10 tools** per MCP server (optimal range)
- Maximum: **15 tools** (requires strong disambiguation in descriptions)
- Above 20: selection accuracy cliff -- must use deferred/dynamic tool registry

## 5. No Hidden Routing

Do NOT hide mode-based branching inside a single tool with `type="auto"`.

**Bad:**
```
web_search(query, search_type="auto")  # internally classifies as search/scrape/research
```

**Acceptable:**
```
web_search(query, search_type="search"|"scrape"|"research")  # explicit enum, LLM chooses
```

**Best:**
```
search_web(query, max_results)      # keyword search
scrape_url(url)                     # URL content extraction
research_topic(question)            # deep research with citations
```

Split into separate tools WHEN:
- Input schemas differ significantly (URL vs keyword vs question)
- Output formats differ
- Failure modes differ

Keep as one tool with enum WHEN:
- Same input schema, same output schema
- Operations are truly variants of the same action

## 6. Parameter Design

- Use `enum` for constrained values (not free-text with documentation)
- Use descriptive names: `source_language` not `lang`, `max_results` not `n`
- Required parameters: keep minimal (1-3)
- Optional parameters: provide sensible defaults
- Add `description` to every parameter in JSON Schema

## 7. Response Design

- Return structured, agent-friendly data (names, statuses, counts)
- Include actionable error messages: what failed, what was expected, how to fix
- Do NOT return raw database rows, UUIDs without context, or full HTML
- Keep response size reasonable -- summarize or paginate large results

## 8. Anti-Patterns (Must Avoid)

| Anti-Pattern | Why It Fails |
|---|---|
| Wrapping every API endpoint as a tool | Semantic noise, context bloat |
| Similar tool names with subtle distinctions | LLMs select by keyword similarity |
| Long negative instruction lists in description | LLMs weight positive matching more |
| Returning all data indiscriminately | Wastes tokens, confuses agent |
| Korean-only descriptions | Degrades tool selection accuracy |
| `auto` classification inside tools | Unpredictable behavior, double failure point |

## 9. Review Checklist

When reviewing MCP tool definitions (Step 7), verify:

- [ ] All descriptions in English, 3+ sentences
- [ ] No two tools with the same verb in their name
- [ ] No hidden `auto` routing without explicit enum
- [ ] Total tool count <= 15
- [ ] Each description includes "when to use" AND "when NOT to use"
- [ ] Parameters use enums where applicable
- [ ] Error responses are actionable
- [ ] No Korean in `name`, `description`, or parameter `description`
