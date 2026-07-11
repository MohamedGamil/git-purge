---
name: Engineered Precision
colors:
  dark:
    surface: '#0d141d'
    surface-dim: '#0d141d'
    surface-bright: '#333a44'
    surface-container-lowest: '#070f18'
    surface-container-low: '#151c26'
    surface-container: '#19202a'
    surface-container-high: '#232a35'
    surface-container-highest: '#2e3540'
    on-surface: '#dce3f0'
    on-surface-variant: '#c0c7d1'
    inverse-surface: '#dce3f0'
    inverse-on-surface: '#2a313b'
    outline: '#8a919b'
    outline-variant: '#404750'
    surface-tint: '#95ccff'
    primary: '#95ccff'
    on-primary: '#003352'
    primary-container: '#61afef'
    on-primary-container: '#004167'
    inverse-primary: '#006399'
    secondary: '#a7d387'
    on-secondary: '#163800'
    secondary-container: '#2b5013'
    on-secondary-container: '#96c178'
    tertiary: '#f6bb84'
    on-tertiary: '#4b2800'
    tertiary-container: '#d59d69'
    on-tertiary-container: '#5a3408'
    error: '#ffb4ab'
    on-error: '#690005'
    error-container: '#93000a'
    on-error-container: '#ffdad6'
    primary-fixed: '#cde5ff'
    primary-fixed-dim: '#95ccff'
    on-primary-fixed: '#001d32'
    on-primary-fixed-variant: '#004a75'
    secondary-fixed: '#c3f0a1'
    secondary-fixed-dim: '#a7d387'
    on-secondary-fixed: '#0a2100'
    on-secondary-fixed-variant: '#2b5013'
    tertiary-fixed: '#ffdcbf'
    tertiary-fixed-dim: '#f6bb84'
    on-tertiary-fixed: '#2d1600'
    on-tertiary-fixed-variant: '#663e12'
    background: '#0d141d'
    on-background: '#dce3f0'
    surface-variant: '#2e3540'
  light:
    surface: '#ffffff'
    surface-dim: '#e1e2ec'
    surface-bright: '#ffffff'
    surface-container-lowest: '#ffffff'
    surface-container-low: '#f1f3fa'
    surface-container: '#ebedf5'
    surface-container-high: '#e0e2eb'
    surface-container-highest: '#d5d7e0'
    on-surface: '#171c22'
    on-surface-variant: '#404750'
    inverse-surface: '#0d141d'
    inverse-on-surface: '#fafbfe'
    outline: '#707883'
    outline-variant: '#c0c7d1'
    surface-tint: '#0061a3'
    primary: '#0061a3'
    on-primary: '#ffffff'
    primary-container: '#d1e4ff'
    on-primary-container: '#001d32'
    inverse-primary: '#95ccff'
    secondary: '#386b20'
    on-secondary: '#ffffff'
    secondary-container: '#e2f7e2'
    on-secondary-container: '#0a2100'
    tertiary: '#865300'
    on-tertiary: '#ffffff'
    tertiary-container: '#ffecdb'
    on-tertiary-container: '#2d1600'
    error: '#ba1a1a'
    on-error: '#ffffff'
    error-container: '#ffdad6'
    on-error-container: '#410002'
    primary-fixed: '#dbe6ff'
    primary-fixed-dim: '#4078f2'
    on-primary-fixed: '#002266'
    on-primary-fixed-variant: '#0044bb'
    secondary-fixed: '#e2f7e2'
    secondary-fixed-dim: '#50a14f'
    on-secondary-fixed: '#003300'
    on-secondary-fixed-variant: '#116611'
    tertiary-fixed: '#ffecdb'
    tertiary-fixed-dim: '#b76b01'
    on-tertiary-fixed: '#4d2600'
    on-tertiary-fixed-variant: '#804c00'
    background: '#fafbfe'
    on-background: '#171c22'
    surface-variant: '#c0c7d1'
typography:
  headline-lg:
    fontFamily: Inter
    fontSize: 24px
    fontWeight: '600'
    lineHeight: 32px
    letterSpacing: -0.02em
  headline-md:
    fontFamily: Inter
    fontSize: 20px
    fontWeight: '600'
    lineHeight: 28px
    letterSpacing: -0.01em
  body-md:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  body-sm:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 18px
  code-md:
    fontFamily: JetBrains Mono
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 20px
  code-sm:
    fontFamily: JetBrains Mono
    fontSize: 11px
    fontWeight: '500'
    lineHeight: 16px
  label-caps:
    fontFamily: Inter
    fontSize: 11px
    fontWeight: '700'
    lineHeight: 16px
    letterSpacing: 0.05em
rounded:
  sm: 0.25rem
  DEFAULT: 0.5rem
  md: 0.75rem
  lg: 1rem
  xl: 1.5rem
  full: 9999px
spacing:
  base: 4px
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 48px
  container-max: 1440px
  gutter: 12px
---

## Brand & Style
The brand personality is clinical, efficient, and technical. It targets senior developers and DevOps engineers who value speed, safety, and clarity when managing repository hygiene. 

The design style is **Sleek Modernism** with an **IDE-inspired** interface. It prioritizes information density without clutter, utilizing a "Utility-First" visual language. The emotional response should be one of absolute control and trust—mimicking the high-stakes environment of a terminal or a sophisticated code editor. Visual flourishes are minimized in favor of functional clarity, using subtle transitions and sharp hierarchy to guide the user through destructive actions with confidence.

## Colors & Themes (Dark / Light Variants)
The application implements two themes: a **Dark Theme** (default) based on the original design values, and a compatible **Light Theme** counterpart that flips the brightness while preserving color mapping context.

### 1. Theme Configuration
Theme settings are controlled dynamically via the `[data-theme]` attribute on the root HTML element (`document.documentElement`). If no attribute is set, the system falls back to matching the OS system color preference via CSS media queries.

- **System Mode:** Toggled dynamically using `matchMedia('(prefers-color-scheme: dark)')` to resolve the theme at runtime, with CSS media queries as a fallback safety net.
- **Color schemes:** Toggle custom property mapping at `:root[data-theme="dark"]` and `:root[data-theme="light"]`.

### 2. Palette Mappings

| Color Role | Dark Theme (Default) | Light Theme (Compatible) | Description |
| :--- | :--- | :--- | :--- |
| **Primary** | `#95ccff` | `#0061a3` | Interactive elements, focus rings, primary action states. |
| **Success** | `#a7d387` | `#386b20` | Merged branch status, success indicators, resolved states. |
| **Warning** | `#f6bb84` | `#865300` | Unmerged branch status, warning alerts. |
| **Destructive** | `#ffb4ab` | `#ba1a1a` | Critical actions (purge/delete), stale alerts, destructive buttons. |
| **Background** | `#0d141d` | `#fafbfe` | Base application viewport background. |
| **Surface** | `#19202a` | `#ffffff` | Sidebars, main grid, cards, dialog overlays. |
| **Border** | `#404750` | `#c0c7d1` | Layout gutters, low-contrast ghost borders. |

## Typography
The system uses a dual-font approach. **Inter** handles all structural UI elements, navigation, and instructional text to ensure high legibility and a modern feel. 

**JetBrains Mono** is reserved for technical data: branch names, commit hashes, file paths, and terminal outputs. This monospaced font provides the necessary alignment for comparing strings of code.

- **Scale:** Keep sizes compact to maintain high information density. 
- **Hierarchy:** Use weight (600 for headers) and color (Muted vs. White) rather than large scale shifts to differentiate information.

## Layout & Spacing
The layout follows a **Rigid Grid** philosophy typical of IDEs. 

- **Layout Model:** Use a 12-column grid for dashboard views, but primary interaction happens within a three-pane system: (1) Navigation/Filter Sidebar, (2) Branch List/Main Content, (3) Detail/Action Inspector.
- **Rhythm:** A 4px baseline grid ensures tight vertical alignment. Use `sm` (8px) for internal component padding and `md` (16px) for layout gaps.
- **Mobile Adaptation:** On mobile, the three-pane system collapses into a single-column stack. The Detail Inspector becomes a bottom sheet to preserve the context of the branch list.

## Elevation & Depth
In this design system, depth is communicated through **Tonal Layers** and **Low-Contrast Outlines** rather than traditional shadows.

- **Stacking:** The background (`#1e1e24`) is the lowest level. Sidebars and cards sit at `#25252e`. Modals and floating tooltips use `#2d2d38`.
- **Borders:** Every container must have a 1px solid border of `#3b3b4a`. This "Ghost Border" technique creates structure without the visual weight of shadows.
- **Active State:** Use a 1px border of the Primary color (`#61afef`) to indicate focus or selection.

## Shapes
A consistent 8px (`rounded-md`) corner radius is applied to all primary UI containers (cards, buttons, inputs). This balances the professional "square" look of an IDE with the approachability of modern SaaS.

- **Small Elements:** Tooltips and tags should use a 4px radius.
- **Interactive Elements:** Buttons and Input fields strictly follow the 8px rule to maintain a cohesive tactile language.

## Components
- **Buttons:** 
  - *Primary:* Solid `#61afef` with dark text. 
  - *Ghost:* Transparent background with `#3b3b4a` border; primary color text.
  - *Purge (Destructive):* Solid `#e06c75` with white text; used only for final deletion.
- **Status Badges:** 
  - Subtle: 10% opacity background of the status color with a 100% opacity text color. 
  - *Protected:* Primary color. *Merged:* Success color. *Unmerged:* Warning color. *Stale:* Muted color.
- **Input Fields:** Background `#1e1e24`, border `#3b3b4a`, font `code-md`. Focus state uses `#61afef` border.
- **Branch Cards:** Horizontal layout. Monospace branch name on the left, status badges in the center, and metadata (last commit, author) in `body-sm` muted text on the right. 
- **Lists:** Use alternating row highlights (zebra striping) at 2% opacity for high-density data tables to assist horizontal eye tracking.
