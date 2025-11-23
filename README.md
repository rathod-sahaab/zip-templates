# ZipTemplates

ZipTemplates is a tiny, fast templating approach for rendering runtime-specified template strings.

The core idea is simple:

- Parse a template string into two parallel arrays: `statics` (the literal parts) and `placeholders` (the variable slots).
- When rendering, map placeholders to their substituted values, "zip" the placeholders with the statics array, and join the interleaved pieces to produce the final string.

This yields a predictable, low-allocation rendering approach suitable for high-throughput or runtime-driven templating scenarios.

## Why ZipTemplates

- Minimal runtime overhead: rendering is a single pass that zips two arrays and joins the pieces.
- Low allocations: avoids building many intermediate strings while concatenating multiple parts.
- Simple to implement and reason about.
- Works well when the template is provided at runtime and templates are simple (interpolation only).

## Concept / Contract

- Input: a template string containing placeholder markers (e.g. `{{0}}`, `$0`, or any chosen syntax), and a mapping/array of substitution values to fill placeholders.
- Output: a single rendered string with placeholders replaced by their corresponding values (coerced to strings).
- Errors: if a placeholder index is missing, behavior is either (configurable) to insert an empty string or raise/return an error. The library should document and provide an option for strict mode.
- Complexity: parse O(n) over template length; render O(p) where p is number of placeholders (with a final join cost dependent on output length).

## Example (conceptual)

This is a language-agnostic example showing the algorithm.

1. Parse a template into `statics` and `placeholders` arrays.

Template: `"Hello, {{0}}! You have {{1}} new messages."`

- statics => ["Hello, ", "! You have ", " new messages."]
- placeholders => ["0", "1"]

2. Render by mapping placeholders to values and zipping:

Values array: `["Alice", 5]`

Zipped pieces: ["Hello, ", "Alice", "! You have ", "5", " new messages."]

Join => `"Hello, Alice! You have 5 new messages."`

Named / nested placeholders example:

Template: `"Hi, {{user.name.first}} — balance: {{account.balance}} USD"`

- statics => ["Hi, ", " — balance: ", " USD"]
- placeholders => ["user.name.first", "account.balance"]

Values object: `{ user: { name: { first: 'Sam' } }, account: { balance: 12.34 } }`

Resolution: look up each placeholder as a dot-path on the values object:

Zipped pieces: ["Hi, ", "Sam", " — balance: ", "12.34", " USD"]

Join => `"Hi, Sam — balance: 12.34 USD"`

## Minimal API (example signatures)

- parse(template: string) -> { statics: string[], placeholders: string[] }
- render(parsed, values: (string | number | null | undefined)[], {strict?: boolean} = {}) -> string

Notes:

- `placeholders` can be numeric indices or named keys depending on parsing syntax.
- For named placeholders you may pass an object map instead of an array.

## Usage (pseudo-JavaScript)

```js
const template = ZipTemplate.parse("Hi, {{name}} — balance: {{balance}} USD");
const out = template.render({ name: "Sam", balance: 12.34 });
console.log(out); // "Hi, Sam — balance: 12.34 USD"
```

## How this compares to common templating solutions

- Handlebars / Mustache / Nunjucks / EJS

  - These are full-featured templating engines with logic (conditionals, loops), helpers, partials, escaping, and more.
  - ZipTemplates intentionally focuses only on interpolation. It does not provide logic, control flow, or template helpers.
  - Pros vs heavy engines: much smaller runtime, fewer allocations, simpler mental model, faster for plain interpolation.
  - Cons vs heavy engines: lacks features like HTML escaping, conditionals, partials, and custom helpers.

- Native template literals (JS backticks) / string interpolation in other languages

  - Native templates are compiled into code at build time or used inline when source code contains the literal templates.
  - ZipTemplates is designed for templates that are specified at runtime (e.g., user-provided templates, templates from a database, or dynamically constructed templates).
  - Native templates are more ergonomic when templates are static and known at coding time; ZipTemplates shines when templates arrive or change at runtime.

- Simple concatenation or join

  - Manual concatenation is straightforward but can become error-prone and allocate intermediate strings when building larger outputs.
  - ZipTemplates reduces allocations by preparing arrays and performing a single join at the end.

- Performance and memory
  - ZipTemplates reduces the number of intermediate string concatenations, which can reduce GC pressure and improve throughput in hot paths that do many renders.
  - It’s best to benchmark in your environment. For simple interpolation-only templates, ZipTemplates will usually outperform heavier template engines because it does less work and allocates less.

## Theoretical performance calculations

This section gives a compact, language-agnostic view of the costs involved when parsing and rendering with ZipTemplates. It focuses on asymptotic behavior, memory (allocations), and a small worked numeric example you can use to estimate cost for your templates.

- $T$ — template length (characters)
- $p$ — number of placeholders
- $s$ — number of static segments ($s = p + 1$)
- $S_{\mathrm{avg}}$ — average static segment length
- $L_{\mathrm{avg}}$ — average length of resolved placeholder values
- $O$ — total output length

Display formulas (LaTeX):

$$
O = s \cdot S_{\mathrm{avg}} + p \cdot L_{\mathrm{avg}}
$$

$$
\mathrm{Time_{parse}} = O(T)
$$

$$
\mathrm{Time_{render}} = O(p + O)
$$

$$
\mathrm{Space_{aux}} = O(s + p) \quad\text{(plus final output } O(O)\text{)}
$$

Parsing

- Time: O(T). Parsing scans the template once to split it into `statics` and `placeholders`.
- Memory: O(s + p) for the two arrays (number of entries). Each static string is usually a slice/substring of the template (language-dependent); if slicing copies, account for those allocations.

Rendering

- Time: O(p + O). Resolving p placeholders (object lookups or index lookups) and then joining the pieces to produce O characters.
  - The cost to convert placeholder values to strings is proportional to the total length of those stringified values (roughly p \* L_avg).
- Memory / allocations:
  - One array of pieces of size s + p (or equivalent internal buffers) is created.
  - The final output string of length O must be allocated once by the runtime (join/concatenate step).
  - Per-placeholder temporary string allocations occur if values need coercion to string (depends on runtime/language).

Comparison to common alternatives (rough)

- Repeated concatenation (e.g., building a string by incremental += or via many small concatenations): may cause many intermediate allocations depending on language and runtime. In the worst case, concatenating k parts naively can create O(k) intermediate buffers and lead to extra copying; cost can approach O(k \* O) work in badly optimized implementations.
- Heavy template engines: these typically parse into an AST (similar parse cost O(T)) but then execute node-by-node doing more work: condition evaluation, helper calls, escaping, and iteration. That extra work increases CPU cost and allocations proportional to feature usage.

Worked numeric example

- Suppose a template with p = 10 placeholders, s = 11 statics. Let S_avg = 20 chars and L_avg = 8 chars.
  - Output length O = 11*20 + 10*8 = 220 + 80 = 300 characters.
  - Parsing cost: O(T) (T might be around 300–400 chars depending on placeholder syntax).
  - Render cost: resolving 10 placeholders (small), constructing array of 21 pieces, and allocating final string of 300 chars.
  - Compared to naive repeated concatenation of 21 pieces, ZipTemplates performs a single final allocation for the joined result and a small array allocation; the naive approach may do several intermediate allocations depending on the runtime.

Practical benchmarking guidance

- Benchmark with realistic templates and value shapes (short vs long substitutions, many vs few placeholders).
- Measure both time and memory/GC allocations. In many languages you can track allocated bytes and number of GC cycles.
- If templates are reused, measure the effect of caching parsed templates (avoid paying parse cost on each render).

Rules of thumb

- For interpolation-only templates: rendering cost is dominated by final output size O and number of placeholders p; ZipTemplates minimizes intermediate allocations by using a single join/concatenate step.
- If you need logic (loops/conditionals) or safe HTML escaping, compare the extra CPU/allocations of a full engine against the development and maintenance costs of implementing those features yourself.

Summary

- Asymptotically, ZipTemplates is optimal for the interpolation-only use case: parse O(T), render O(p + O) time, and a small O(s+p) extra memory for arrays plus one O(O) allocation for the output string.

Fallback (plain text):

- Output length: O = s _ S_avg + p _ L_avg
- Parse time: Time_parse = O(T)
- Render time: Time_render = O(p + O)
- Aux space: Space_aux = O(s + p) + O(O) (final output)

## Benchmarks

For small strings on `Ryzen 8700GE`

| Benchmark                                 | Time ns (avg) |
| :---------------------------------------- | ------------: |
| flatten json then `zip_templates::render` |        360.13 |
| already flat json `zip_templates::render` |    **36.787** |
| `zip_templates::render_from_vec_smart`    |    **21.823** |
| `tera::render`                            |        1090.5 |
| `mystical_runic::render`                  |        747.51 |
| `simple_replace`                          |        503.04 |
| `simple_replace_flat`                     |        181.57 |

```
     Running benches/zip_templates_bench.rs (target/release/deps/zip_templates_bench-4b3e190c814a6892)
Gnuplot not found, using plotters backend
zip_templates::render   time:   [359.53 ns 360.13 ns 360.85 ns]
                        change: [−4.1652% −2.7557% −1.0608%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) high mild
  11 (11.00%) high severe

zip_templates::render_flat
                        time:   [36.707 ns 36.787 ns 36.880 ns]
                        change: [−4.7696% −4.3873% −4.0016%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe

zip_templates::render_from_vec_smart
                        time:   [21.784 ns 21.823 ns 21.863 ns]
                        change: [−8.7911% −8.4338% −8.0677%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe

tera::render            time:   [1.0891 µs 1.0905 µs 1.0921 µs]
                        change: [−4.8755% −4.4775% −4.0901%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe

mystical_runic::render  time:   [744.88 ns 747.51 ns 750.97 ns]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe

simple_replace          time:   [502.19 ns 503.04 ns 504.08 ns]
                        change: [−1.6621% −1.2387% −0.8054%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe

simple_replace_flat     time:   [181.32 ns 181.57 ns 181.86 ns]
                        change: [+12.015% +12.434% +12.882%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
```

## Trade-offs and when to use

Use ZipTemplates when:

- You only need interpolation (no loops/conditionals/helpers).
- Templates are provided or changed at runtime.
- You care about minimizing allocations and maximizing rendering throughput.

Avoid ZipTemplates when:

- You need escaping, conditionals, loops, or template composition across files.
- You need a mature ecosystem of helpers and tooling (internationalization, partials, etc.).

## Edge cases and considerations

- Missing placeholders: decide between empty string substitution vs. throwing an error (provide strict mode).
- Escaping: ZipTemplates does not escape HTML or other contexts by default — callers must escape values where needed.
- Large templates: parsing and storing arrays uses memory proportional to template structure — still usually less allocation-heavy than repeated concatenation.
- Placeholder collision / ambiguous syntax: pick a clear placeholder syntax and document it.

## Extensibility

Possible small additions that remain lightweight:

- Strict mode: throw on missing values.
- Named placeholders: support object maps for rendering.
- Caching parsed templates: if templates are reused, store parsed results to avoid reparsing.

## Quick checklist for implementers

- [ ] Choose placeholder syntax and document it.
- [ ] Implement `parse` and `render` with clear semantics for missing values.
- [ ] Add an optional `escape` hook for common contexts (HTML, URI).
- [ ] Add tests: happy path and missing placeholder behavior.

## Files changed / created

- `README.md` — this file: describes the project, usage, and comparisons.

## Final notes

ZipTemplates is intentionally small and focused. If you need richer templating features, pair ZipTemplates with a small set of utilities (escaping, caching, and a strict mode) or switch to a full template engine when complexity demands it.
