# smart-commit website

Adds to [../AGENTS.md](../AGENTS.md).

Rust + Topcoat `=0.11.0`: server-rendered HTML, no JavaScript, one dark theme
with one violet accent. Run: `cargo run -p smart-commit-website` (or
`topcoat dev`; `HOST=0.0.0.0` for a phone). Release build:
`cargo build --profile website -p smart-commit-website`; image:
`docker compose up -d --build` here.

## Content

- **Facts** (version, repo, installer, models, devices, benchmark, roadmap)
  are read by `build.rs` into `src/facts.rs`. Never type one into a page.
- **Measurements** are copied from `../docs/measurements.md` into
  `src/measures.rs`; `tests/routes.rs` fails until they match. Charts are
  HTML/CSS: the bar decorates, the number is text.
- **Other copy** is handwritten: check it against `../docs/` and the CLI;
  never invent a number or command.
- Only `/` may sell; other pages are developer docs: plain titles, no
  slogans, no calls to action. Linux only, no macOS conventions. Writing a
  message is local; installing, updating and downloading models use the
  network.

## Code

- A page is `src/app/<page>.rs` with only its `#[page]` (a module without one
  adds no route); its sections are components in `src/app/<page>/`, one per
  file. Split by sections, not size.
- New page: the module, a row in `site::PAGES` (identified by its handler,
  never its path), a row in `tests/routes.rs::PAGES`.
- `src/ui/` holds site components; `src/components/` is Topcoat UI, copied
  with `topcoat ui add`. Never add one that needs the JS runtime (`sheet`,
  `sidebar`, `navigation_menu`, `dialog`, `tabs`, `alert_dialog`), nor
  `suspense`, `live!`, `emit!`.
- Topcoat 0.11.0 guides: `~/.cargo/registry/src/*/topcoat-0.11.0/docs` and
  `topcoat-view-macro-0.11.0/docs`. Not in 0.11.0: `with(...)`, `Metadata`,
  `link`.
- Reusable markup is a `#[component]` (called `card(item: x)`) with typed
  props: enums for variants, `&str` for text, `#[default]`, `child: Child`.
- Classes: `class="..."`, `class!(...)` to compose, `StaticClass` for
  constants; never `format!` or a `&str` constant. Wrappers take
  `attrs: Attributes` and merge with `class!(BASE, attrs.remove("class"))`;
  callers cannot override a class the component sets.
- Interpolate numbers directly, `(count)`; a non-text value gets its own
  type (`ui::chart::Share`). Compute `let` values before the `view!`.
- Links: `href!(handler)`, `.fragment("id")`, `.is_current(cx)`; only
  external URLs are strings. Icons: `IconData` in `ui/icons.rs` via
  `icon(...)`.
- `cargo fmt` always; `topcoat fmt` only if the installed CLI handles 0.11.

## Styles

- `styles.css` is the Tailwind entry (theme + palette: near-black, violet,
  Bricolage Grotesque); `build.rs` builds it from the utilities in
  `src/**/*.rs` and `assets/style.css`. If the served CSS is stale, touch
  `build.rs`.
- Layout and spacing are Tailwind utilities in Rust. `assets/style.css` has
  only the font and plain-element defaults; never a class Tailwind also
  defines (`grid`, `hidden`).
- No animations or effects. No fixed lengths (`rem`, `em`, `%`, `fr`,
  `clamp()`); breakpoints in `rem`. Fonts are local (`assets/fonts`), never
  a font host.
- Use the `frontend-design` skill for visual changes, checked against the
  existing page.

## Done

Tests check facts, commands, settings, errors and links against the product,
never wording: do not test a sentence or heading.

Any change to markup, CSS, assets or layout-affecting copy is also checked
visually at 1280, 820 and 390 px:

1. Start the site; `scripts/visual-check.sh /path ...` for every page it
   reaches (layout, header, stylesheet: all).
2. Look at every screenshot: overflow, clipped or badly wrapped text,
   orphaned grid items, spacing, alignment, small tap targets, contrast,
   altered states (menu, scrolled table, focus).
3. Fix and recheck. Report which pages and sizes you checked and which not.
