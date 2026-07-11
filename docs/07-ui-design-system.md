# 07 — UI Design System (One Dark Pro × Material)

`Status: Draft` · `Owner: Design` · `Last-updated: 2026-07-11` ·
`Related: [CONVENTIONS](../delivery/CONVENTIONS.md), [00-vision-and-scope.md](00-vision-and-scope.md), [01-tech-stack.md](01-tech-stack.md), [06-ui-spec.md](06-ui-spec.md), [11-safety-model.md](11-safety-model.md)`

> The visual language for the **Git Purge** desktop app ([06-ui-spec.md](06-ui-spec.md)):
> **Material design** (elevation, motion, states) restrained to a **minimalist,
> content-first** feel, colored by the **One Dark Pro** palette with a derived light
> theme and a `system` mode. This is **R12**. Everything here is CSS custom properties
> ("design tokens") — **no heavy component or UI library** ([01-tech-stack.md](01-tech-stack.md)).
> Component *behavior* and screens live in [06](06-ui-spec.md); this doc owns *looks*.

---

## 1. Design principles

1. **Minimalist & content-first (R12).** The branch table, diff, and plan are the
   product; chrome recedes. No decorative gradients, no feature we do not need.
2. **Calm, not loud.** Color is used *semantically* — to classify and to warn — not for
   decoration. A screen at rest is mostly neutral surface + text.
3. **Material, restrained.** We borrow Material's elevation, state layers, focus rings,
   and motion easing, but flatten it: at most three elevation steps, subtle shadows.
4. **Safety is legible.** Destructive affordances look distinct (danger tokens, typed
   confirmation); safe defaults look calm. The visual system reinforces
   [11-safety-model.md](11-safety-model.md).
5. **Accessible by construction.** WCAG AA contrast in both themes; **never color-only**
   (§8); visible focus; respects `prefers-reduced-motion` and `prefers-color-scheme`.
6. **Two themes, one system.** Light, dark, and system share identical *semantic*
   tokens; only the palette mapping changes, so components are written once.

---

## 2. Color — the One Dark Pro palette

### 2.1 Palette tokens (raw hues)

The canonical **One Dark Pro** colors, exposed as palette-level tokens. These are the
*only* place raw hex appears; semantic tokens (§2.2) reference them so themes stay
consistent.

| Token | Hex | One Dark Pro role |
| :--- | :--- | :--- |
| `--odp-bg` | `#282c34` | editor background |
| `--odp-bg-deep` | `#21252b` | panel / sidebar (recessed) |
| `--odp-bg-raised` | `#2c313a` | raised card |
| `--odp-fg` | `#abb2bf` | foreground text |
| `--odp-fg-strong` | `#d7dae0` | bright foreground |
| `--odp-black` | `#3f4451` | bright black / borders |
| `--odp-black-deep` | `#282c34` | black |
| `--odp-red` | `#e06c75` | red |
| `--odp-green` | `#98c379` | green |
| `--odp-yellow` | `#e5c07b` | yellow |
| `--odp-orange` | `#d19a66` | orange (dark yellow) |
| `--odp-blue` | `#61afef` | blue (accent) |
| `--odp-purple` | `#c678dd` | magenta / purple |
| `--odp-cyan` | `#56b6c2` | cyan |
| `--odp-gray` | `#5c6370` | comment / gray |
| `--odp-selection` | `#3e4451` | selection |
| `--odp-accent` | `#61afef` | UI accent (= blue) |

The **light** theme derives from the same hues via One Dark Pro's natural light sibling
("One Light"), keeping identical semantics at accessible contrast on light surfaces:

| Token | Hex (light) | Role |
| :--- | :--- | :--- |
| `--olt-bg` | `#fafafa` | app background |
| `--olt-surface` | `#ffffff` | surface |
| `--olt-surface-variant` | `#f0f0f1` | recessed surface |
| `--olt-fg` | `#383a42` | foreground text |
| `--olt-fg-strong` | `#202227` | strong foreground |
| `--olt-border` | `#d4d4d6` | border |
| `--olt-red` | `#e45649` | red |
| `--olt-green` | `#50a14f` | green |
| `--olt-amber` | `#c18401` | amber (yellow) |
| `--olt-orange` | `#b76b01` | orange |
| `--olt-blue` | `#4078f2` | blue (accent) |
| `--olt-purple` | `#a626a4` | purple |
| `--olt-cyan` | `#0184bc` | cyan |
| `--olt-gray` | `#a0a1a7` | comment / gray |
| `--olt-selection` | `#dbe6ff` | selection |

### 2.2 Semantic tokens

Components reference **only** these; they never touch raw palette tokens. Same names in
both themes → write components once.

| Semantic token | Meaning | Dark → palette | Light → palette |
| :--- | :--- | :--- | :--- |
| `--surface` | base surface | `#282c34` | `#ffffff` |
| `--surface-variant` | recessed panels (nav, headers) | `#21252b` | `#f0f0f1` |
| `--surface-raised` | cards, dialogs, menus | `#2c313a` | `#ffffff` |
| `--bg` | app backdrop | `#21252b` | `#fafafa` |
| `--on-surface` | primary text | `#abb2bf` | `#383a42` |
| `--on-surface-strong` | headings / emphasis | `#d7dae0` | `#202227` |
| `--muted` | secondary text, hints | `#5c6370` | `#a0a1a7` |
| `--border` | dividers, table lines, inputs | `#3f4451` | `#d4d4d6` |
| `--primary` | primary actions, accent | `#61afef` | `#4078f2` |
| `--on-primary` | text/icon on primary | `#1b1d23` | `#ffffff` |
| `--success` | merged / safe / ok | `#98c379` | `#50a14f` |
| `--warning` | stale / caution | `#e5c07b` | `#c18401` |
| `--danger` | delete / unmerged / destructive | `#e06c75` | `#e45649` |
| `--info` | active / informational | `#56b6c2` | `#0184bc` |
| `--accent-purple` | non-standard naming | `#c678dd` | `#a626a4` |
| `--selection` | selected rows / text | `#3e4451` | `#dbe6ff` |
| `--focus-ring` | keyboard focus outline | `#61afef` | `#4078f2` |
| `--overlay` | dialog scrim | `rgba(0,0,0,.55)` | `rgba(30,32,38,.35)` |

### 2.3 Theme CSS blocks

`:root` holds palette + shared (non-color) tokens; `[data-theme]` maps semantics. The
`system` mode sets **no** `data-theme` attribute and lets `@media (prefers-color-scheme)`
choose (§7).

```css
/* tokens.css — palette + shared tokens */
:root {
  /* One Dark Pro palette (dark source) */
  --odp-bg:#282c34; --odp-bg-deep:#21252b; --odp-bg-raised:#2c313a;
  --odp-fg:#abb2bf; --odp-fg-strong:#d7dae0; --odp-black:#3f4451;
  --odp-red:#e06c75; --odp-green:#98c379; --odp-yellow:#e5c07b; --odp-orange:#d19a66;
  --odp-blue:#61afef; --odp-purple:#c678dd; --odp-cyan:#56b6c2; --odp-gray:#5c6370;
  --odp-selection:#3e4451; --odp-accent:#61afef;
  /* One Light palette (light source) */
  --olt-bg:#fafafa; --olt-surface:#ffffff; --olt-surface-variant:#f0f0f1;
  --olt-fg:#383a42; --olt-fg-strong:#202227; --olt-border:#d4d4d6;
  --olt-red:#e45649; --olt-green:#50a14f; --olt-amber:#c18401; --olt-orange:#b76b01;
  --olt-blue:#4078f2; --olt-purple:#a626a4; --olt-cyan:#0184bc; --olt-gray:#a0a1a7;
  --olt-selection:#dbe6ff;
}

/* Dark theme mapping */
:root[data-theme="dark"], .theme-dark {
  --bg:var(--odp-bg-deep); --surface:var(--odp-bg); --surface-variant:var(--odp-bg-deep);
  --surface-raised:var(--odp-bg-raised);
  --on-surface:var(--odp-fg); --on-surface-strong:var(--odp-fg-strong); --muted:var(--odp-gray);
  --border:var(--odp-black);
  --primary:var(--odp-blue); --on-primary:#1b1d23;
  --success:var(--odp-green); --warning:var(--odp-yellow); --danger:var(--odp-red);
  --info:var(--odp-cyan); --accent-purple:var(--odp-purple);
  --selection:var(--odp-selection); --focus-ring:var(--odp-blue);
  --overlay:rgba(0,0,0,.55);
  color-scheme: dark;
}

/* Light theme mapping */
:root[data-theme="light"], .theme-light {
  --bg:var(--olt-bg); --surface:var(--olt-surface); --surface-variant:var(--olt-surface-variant);
  --surface-raised:var(--olt-surface);
  --on-surface:var(--olt-fg); --on-surface-strong:var(--olt-fg-strong); --muted:var(--olt-gray);
  --border:var(--olt-border);
  --primary:var(--olt-blue); --on-primary:#ffffff;
  --success:var(--olt-green); --warning:var(--olt-amber); --danger:var(--olt-red);
  --info:var(--olt-cyan); --accent-purple:var(--olt-purple);
  --selection:var(--olt-selection); --focus-ring:var(--olt-blue);
  --overlay:rgba(30,32,38,.35);
  color-scheme: light;
}

/* System mode: no data-theme set → follow the OS */
@media (prefers-color-scheme: dark)  { :root:not([data-theme]) { /* re-apply .theme-dark map */ } }
@media (prefers-color-scheme: light) { :root:not([data-theme]) { /* re-apply .theme-light map */ } }
```

> Implementation note: to avoid duplicating the maps, `useTheme` (§7) resolves `system`
> to a concrete `data-theme` at runtime by reading `matchMedia('(prefers-color-scheme: dark)')`,
> so exactly one of the two mapping blocks is ever active. The `@media` fallback above is
> a no-JS safety net.

---

## 3. Typography

System font stack (zero external font downloads → fast, offline, native feel). A mono
stack for SHAs, refs, code, and diffs.

```css
:root {
  --font-sans: system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  --font-mono: ui-monospace, "JetBrains Mono", "Fira Code", "SF Mono", Menlo, Consolas, monospace;

  --text-xs:  0.75rem;  --lh-xs:  1rem;      /* 12/16 — badges, meta */
  --text-sm:  0.8125rem;--lh-sm:  1.125rem;  /* 13/18 — table cells, labels */
  --text-base:0.875rem; --lh-base:1.375rem;  /* 14/22 — body (dense desktop) */
  --text-md:  1rem;     --lh-md:  1.5rem;    /* 16/24 — dialog body */
  --text-lg:  1.25rem;  --lh-lg:  1.75rem;   /* 20/28 — section titles */
  --text-xl:  1.5rem;   --lh-xl:  2rem;      /* 24/32 — screen titles */
  --text-2xl: 2rem;     --lh-2xl: 2.5rem;    /* 32/40 — stat-tile numbers */

  --fw-regular:400; --fw-medium:500; --fw-semibold:600;
}
```

Rules: body copy is `--text-base`/`--fw-regular`; titles use `--fw-semibold` +
`--on-surface-strong`. All refs/SHAs/paths use `--font-mono`. Never go below
`--text-xs`. Line length in prose (reports) capped at ~72ch.

---

## 4. Spacing, radius, elevation, motion

### 4.1 Spacing — 8px grid (4px half-step)

```css
:root {
  --space-0:0; --space-1:4px; --space-2:8px; --space-3:12px; --space-4:16px;
  --space-5:20px; --space-6:24px; --space-8:32px; --space-10:40px; --space-12:48px;
  --density-row:32px;   /* dense virtualized table row height */
  --nav-rail:220px;     /* left nav width */
}
```

Component padding is always a grid multiple (buttons `--space-2 --space-4`, cards
`--space-4`, dialogs `--space-6`). Table rows are `--density-row` (fixed, for
virtualization — [06 §4.2](06-ui-spec.md)).

### 4.2 Radius

```css
:root { --radius-sm:4px; --radius-md:8px; --radius-lg:12px; --radius-pill:999px; }
```

Inputs/buttons `--radius-md`; cards/dialogs `--radius-lg`; chips/badges `--radius-pill`.

### 4.3 Elevation (Material, restrained — max 3 steps)

```css
:root[data-theme="dark"], .theme-dark {
  --elevation-0:none;
  --elevation-1:0 1px 2px rgba(0,0,0,.40);
  --elevation-2:0 4px 12px rgba(0,0,0,.45);          /* menus, popovers */
  --elevation-3:0 12px 32px rgba(0,0,0,.55);         /* dialogs */
}
:root[data-theme="light"], .theme-light {
  --elevation-0:none;
  --elevation-1:0 1px 2px rgba(16,24,40,.06), 0 1px 3px rgba(16,24,40,.10);
  --elevation-2:0 4px 12px rgba(16,24,40,.10);
  --elevation-3:0 12px 32px rgba(16,24,40,.16);
}
```

Surfaces flat by default (`--elevation-0`); raise only on overlay layers. State layers:
hover `rgba(255,255,255,.06)` dark / `rgba(0,0,0,.04)` light; active a touch stronger.

### 4.4 Motion

```css
:root {
  --dur-fast:120ms; --dur-base:200ms; --dur-slow:320ms;
  --ease-standard:cubic-bezier(.2,0,0,1);
  --ease-emphasized:cubic-bezier(.2,0,0,1.2);
  --ease-exit:cubic-bezier(.4,0,1,1);
}
@media (prefers-reduced-motion: reduce) {
  * { animation-duration:1ms !important; transition-duration:1ms !important; }
}
```

Use `--dur-fast` for hover/press, `--dur-base` for enter/expand, `--dur-slow` only for
dialog scrim. No looping/decorative animation (R12) except the indeterminate progress bar.

---

## 5. Iconography

A **lightweight inline-SVG icon set** (Lucide-style: 1.5px stroke, 24×24 grid,
`currentColor`, `fill="none"`), shipped as Vue components — **no icon font, no CDN**
(offline + CSP-safe, [01-tech-stack.md](01-tech-stack.md)). Icons inherit `color`, so
they take semantic tokens automatically.

Canonical set: `check-circle` (merged/ok), `alert-triangle` (unmerged/destructive),
`clock` (stale), `activity` (active), `lock` (protected), `tag` (non-standard/tag),
`git-branch`, `git-commit`, `arrow-up`/`arrow-down` (ahead/behind), `database` (backup),
`rotate-ccw` (restore), `trash` (delete), `search`, `filter`, `x`, `loader`.

```vue
<!-- components/icon/IconCheck.vue -->
<template>
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none"
       stroke="currentColor" stroke-width="1.75" stroke-linecap="round"
       stroke-linejoin="round" aria-hidden="true" focusable="false">
    <path d="M20 6 9 17l-5-5" />
  </svg>
</template>
```

---

## 6. Core component inventory

Minimal markup + the load-bearing styles. Full behavior is in [06](06-ui-spec.md).

### 6.1 Buttons — primary / secondary / danger

```css
.btn { display:inline-flex; align-items:center; gap:var(--space-2);
  height:32px; padding:0 var(--space-4); border-radius:var(--radius-md);
  font:var(--fw-medium) var(--text-sm)/1 var(--font-sans); cursor:pointer;
  border:1px solid transparent; transition:background var(--dur-fast) var(--ease-standard); }
.btn:focus-visible { outline:2px solid var(--focus-ring); outline-offset:2px; }
.btn--primary   { background:var(--primary); color:var(--on-primary); }
.btn--secondary { background:transparent; color:var(--on-surface); border-color:var(--border); }
.btn--danger    { background:var(--danger); color:#fff; }
.btn:disabled   { opacity:.5; cursor:not-allowed; }
```

Rules: exactly **one** primary button per view/dialog. `--danger` only for destructive
actions (delete/execute-destructive). Secondary is the default for everything else.

### 6.2 Badges / chips — classification

Classification (CONVENTIONS §8) → **color + icon + text**, never color alone (§8).

| Classification | Token | Icon | Label |
| :--- | :--- | :--- | :--- |
| `merged` | `--success` | `check-circle` | Merged |
| `unmerged` | `--danger` | `alert-triangle` | Unmerged |
| `stale` | `--warning` | `clock` | Stale |
| `active` | `--info` | `activity` | Active |
| `protected` | `--primary` | `lock` | Protected |
| `nonStandard` | `--accent-purple` | `tag` | Non-standard |
| `ahead`/`behind` | `--muted` | `arrow-up`/`down` | `+n` / `−n` |

```css
.chip { display:inline-flex; align-items:center; gap:var(--space-1);
  height:20px; padding:0 var(--space-2); border-radius:var(--radius-pill);
  font:var(--fw-medium) var(--text-xs)/1 var(--font-sans);
  color:var(--chip-color); background:color-mix(in srgb, var(--chip-color) 16%, transparent);
  border:1px solid color-mix(in srgb, var(--chip-color) 36%, transparent); }
```
```html
<span class="chip" style="--chip-color:var(--success)"><IconCheck/>Merged</span>
<span class="chip" style="--chip-color:var(--danger)"><IconAlert/>Unmerged</span>
```

### 6.3 Tables — dense, virtualized

Dense rows (`--density-row`), zebra via `--surface-variant`, sticky header on
`--surface-variant`, 1px `--border` dividers. The table is **windowed** — only visible
rows render — and exposes `role="grid"` with full-dataset `aria-rowcount`
([06 §4.2](06-ui-spec.md)). Selected rows use `--selection`.

```css
.table__header { position:sticky; top:0; background:var(--surface-variant);
  color:var(--muted); font:var(--fw-semibold) var(--text-xs)/1 var(--font-sans);
  text-transform:uppercase; letter-spacing:.04em; }
.row { height:var(--density-row); display:grid; align-items:center;
  border-bottom:1px solid var(--border); }
.row:hover     { background:color-mix(in srgb, var(--on-surface) 6%, transparent); }
.row[aria-selected="true"] { background:var(--selection); }
.cell--mono    { font-family:var(--font-mono); color:var(--muted); }
```

### 6.4 Tabs

Underline-style, minimal. Active tab: `--primary` text + 2px underline; others
`--muted`. `role="tablist"`; arrow-key navigation.

```css
.tab { padding:var(--space-2) var(--space-3); color:var(--muted); font:var(--fw-medium) var(--text-sm); border-bottom:2px solid transparent; }
.tab[aria-selected="true"] { color:var(--primary); border-bottom-color:var(--primary); }
```

### 6.5 Dialogs & confirmations (incl. destructive typed-confirm)

Centered on an `--overlay` scrim, `--surface-raised`, `--radius-lg`, `--elevation-3`;
focus trapped, `Esc` cancels. **Two tiers** ([06 §4.6](06-ui-spec.md), [11](11-safety-model.md)):

- **Standard confirm:** title + body + `[Cancel][Confirm]` (Confirm is `--primary`).
- **Destructive typed-confirm:** danger-accented header, an impact summary, and a text
  input; the danger button stays **disabled** until the typed token matches exactly.

```vue
<!-- components/DestructiveDialog.vue (essence) -->
<script setup lang="ts">
const props = defineProps<{ token: string; summary: string }>();
const typed = ref('');
const armed = computed(() => typed.value === props.token);
</script>
<template>
  <div class="dialog dialog--danger" role="alertdialog" aria-modal="true">
    <header><IconAlert/> Destructive action</header>
    <p>{{ summary }}</p>
    <label>Type <code>{{ token }}</code> to confirm</label>
    <input v-model="typed" class="input" autocomplete="off" />
    <footer>
      <button class="btn btn--secondary">Cancel</button>
      <button class="btn btn--danger" :disabled="!armed">Execute</button>
    </footer>
  </div>
</template>
```

### 6.6 Toasts

Bottom-right stack, `--surface-raised` + `--elevation-2`, a left accent bar in the
semantic token (`--danger`/`--warning`/`--success`/`--info`), auto-dismiss (except
errors), `aria-live="polite"` (`assertive` for errors). Max one line + optional action.

### 6.7 Progress bar

Determinate (fed by `current/total` from `gitpurge://progress`) or indeterminate
(`total==0`). Track `--surface-variant`, fill `--primary`, `role="progressbar"` with
`aria-valuenow`. The only permitted looping animation (R12).

```css
.progress { height:4px; background:var(--surface-variant); border-radius:var(--radius-pill); overflow:hidden; }
.progress__fill { height:100%; background:var(--primary); transition:width var(--dur-base) var(--ease-standard); }
```

### 6.8 Empty states

Centered, single muted icon (`--muted`), one line of `--on-surface`, one line of
`--muted` hint, and exactly one primary action. No illustrations (R12). Content per
screen in [06 §4](06-ui-spec.md).

### 6.9 Form controls

Inputs/selects: 32px tall, `--surface`, 1px `--border`, `--radius-md`, focus →
`--focus-ring`. Invalid → `--danger` border + inline message (used by `settings_save`
regex/glob validation, `repo_add`). Toggles for booleans (the pre-op-backup toggle uses
`--success` when on). Labels always visible (no placeholder-as-label).

```css
.input { height:32px; padding:0 var(--space-3); background:var(--surface);
  border:1px solid var(--border); border-radius:var(--radius-md);
  color:var(--on-surface); font:var(--text-sm) var(--font-sans); }
.input:focus-visible { outline:2px solid var(--focus-ring); outline-offset:1px; border-color:var(--primary); }
.input[aria-invalid="true"] { border-color:var(--danger); }
```

### 6.10 Tree view

Indented rows (`--space-4` per depth), folder/file icons (`--muted` folders,
`--on-surface` files), monospace names, breadcrumb header. Selected file row uses
`--selection`. Used by the contents-@-commit viewer ([06 §4.3](06-ui-spec.md)).

### 6.11 Diff view styling

Unified or side-by-side. Added lines: `--success` text on
`color-mix(in srgb, var(--success) 12%, transparent)`; removed: `--danger` on a red
tint; gutter line numbers `--muted`; hunk headers `--info`. Monospace throughout.
Never rely on the tint alone — prefix `+`/`−` gutters carry the meaning (§8).

```css
.diff__line--add { background:color-mix(in srgb, var(--success) 12%, transparent); }
.diff__line--del { background:color-mix(in srgb, var(--danger) 12%, transparent); }
.diff__gutter { color:var(--muted); font-family:var(--font-mono); user-select:none; }
```

---

## 7. Theming implementation (Vue)

Tokens live in **`apps/desktop/src/styles/tokens.css`** (imported once in `main.ts`).
A **`useTheme`** composable in **`apps/desktop/src/composables/useTheme.ts`** owns the
`light | dark | system` choice, resolves `system` against the OS, writes `data-theme`
on `<html>`, and persists via the `settings` store (`settings_save`, [06 §4.9](06-ui-spec.md)).

```ts
// apps/desktop/src/composables/useTheme.ts
import { ref, watchEffect } from 'vue';
export type ThemeMode = 'light' | 'dark' | 'system';
const mode = ref<ThemeMode>('system');
const media = window.matchMedia('(prefers-color-scheme: dark)');

function resolve(m: ThemeMode): 'light' | 'dark' {
  return m === 'system' ? (media.matches ? 'dark' : 'light') : m;
}
export function useTheme() {
  function apply() { document.documentElement.setAttribute('data-theme', resolve(mode.value)); }
  media.addEventListener('change', () => { if (mode.value === 'system') apply(); });
  watchEffect(apply);
  return {
    mode,
    setMode(m: ThemeMode) { mode.value = m; /* settings store persists via settings_save */ },
  };
}
```

Boot order in `main.ts`: import `tokens.css` → hydrate `settings` store
(`settings_get`) → `useTheme().setMode(settings.theme)` before first paint to avoid a
flash. Because `system` is resolved to a concrete `data-theme`, exactly one mapping in
§2.3 is active at any time.

---

## 8. Accessibility: status is never color-only

Every classification/status conveys meaning through **color + icon + text** together
(WCAG 1.4.1). The chip table in §6.2 is the single source of truth; the diff view (§6.11)
pairs tint with `+`/`−` gutters; toasts (§6.6) pair the accent bar with an icon and
text. Contrast: body text and semantic tokens meet **AA** on their surfaces in both
themes; focus uses a 2px `--focus-ring` at `outline-offset` (never removed). This
realizes the accessibility notes in [06 §6](06-ui-spec.md).

---

## 9. Do / Don't (keep it minimalist — R12)

**Do**
- Use semantic tokens (`--primary`, `--danger`…), never raw `--odp-*`/`--olt-*` or hex.
- Keep one primary action per view; let neutral surfaces dominate.
- Pair every color signal with an icon and a text label.
- Use the 8px grid and the three-step elevation scale; keep motion short and purposeful.
- Reserve `--danger` for genuinely destructive actions; keep safe defaults calm.

**Don't**
- Don't add a component/CSS framework or an icon font (offline + CSP + R12).
- Don't introduce new hues outside the palette or a fourth elevation level.
- Don't use color as the only differentiator, or remove focus outlines.
- Don't animate decoratively or loop anything but the progress bar.
- Don't hardcode light/dark values in components — theme only through tokens.

---

## 10. Traceability

| Req | Where satisfied |
| :--- | :--- |
| **R12** minimalist, intuitive; light/dark/system; Material on One Dark Pro | §1 principles, §2 palette + themes, §4 Material tokens, §7 theming, §9 do/don't. |
| **R3** legible exploration (badges, tables, diff) | §6.2 chips, §6.3 tables, §6.11 diff. |
| Safety model surfacing ([11](11-safety-model.md)) | §6.1 danger button, §6.5 destructive typed-confirm, §6.9 backup toggle. |
| Accessibility ([06 §6](06-ui-spec.md)) | §8 color-plus-icon-plus-text, focus, contrast. |
