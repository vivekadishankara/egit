# Clone Button — Implementation Notes

## Goal

A "Clone" button next to the branch dropdown that opens a popover with HTTP and SSH tabs, each showing the respective clone URL and a copy button.

## Current implementation

- **File**: `src/components/clone_button.rs`
- **Mechanism**: `<details>`/`<summary>` for native popover toggle (works without JS). CSS-only tab switching via hidden `<input type="radio">` + `<label>` elements. No Leptos event handlers.
- **URLs**: Computed eagerly from `window.location.origin` / `window.location.hostname` via `web_sys`, falling back to localhost during SSR.
- **No copy button** — couldn't get Leptos event handlers to work, and inline `onclick` was not attempted.

## Approaches that didn't work

### 1. `<details>`/`<summary>` + `on:click` on tab buttons (Leptos view macro)

The `<details>` opened natively (native HTML behavior works). But:
- URL signals stayed empty — lazy-load via `on:click` on `<summary>` didn't fire.
- Tab button `on:click` handlers didn't fire — clicking HTTP/SSH did nothing.

### 2. `<button>` + conditional popover (`Show` or `move \|\|` closure)

Button rendered and was clickable (cursor changed), but `on:click` handler never fired. Popover never appeared.

### 3. `<button>` + `style:display` (always rendered, CSS-hidden)

Same result — `on:click` didn't fire.

### 4. `<div role="button">` instead of `<button>`

Same result — `on:click` didn't fire.

### 5. Explicit `leptos::ev::MouseEvent` type annotation on `on:click` closures

Same result — didn't help.

### 6. `NodeRef` + imperative `.on(ev::click, ...)` via `Effect::new`

Couldn't get past type inference errors for `HtmlElement::on()` — `cannot infer type` even with explicit `MouseEvent` annotation on the handler closure.

## Observations

- **`on:change` on `<select>` works** — the theme switcher uses it successfully, suggesting the Leptos event system is functional.
- **`on:click` on `<button>` / `<div>` / `<a>` does NOT work** — at least not in this component context. All approaches with `on:click` in the `view!` macro failed silently.
- **Native HTML works** — `<details>` toggles, `<label>` clicks for radio buttons, `<style>` CSS selectors all work because they're handled by the browser, not by Leptos events.
- The `CloneButton` component is imported and rendered in `overview.rs`, `tree.rs`, `blob.rs`, and `commits.rs` — it appears visually but event handlers never fire.

## Possible causes (unconfirmed)

1. **Leptos 0.8.14 event delegation bug** — SSR serializes event callbacks with IDs, but hydration might fail to attach them for certain element types or in certain DOM positions.
2. **Server/client mismatch** — The component renders on SSR with `on:click` attributes. During hydration, if the SSR output and hydrate output don't match perfectly, Leptos may skip hydration for that component, leaving events unattached.
3. **Closure serialization issue** — The `move` closures capture `RwSignal` handles and `String`s. If Leptos's callback registry has an issue with non-`'static` or non-`Send` captures, callbacks might be silently dropped.
4. **Conflicting Tailwind styles** — Unlikely, as the button is visible and clickable (cursor changes).

## Next steps to debug

- Test `on:click` on a simple isolated component (no `<details>`, no popover, just a button with `on:click` that toggles a signal).
- Check if `on:click` works on elements outside a `<details>` (maybe `<details>` interferes with event delegation).
- Try upgrading/downgrading Leptos version.
- Look at Leptos 0.8.14 changelog for event system changes.
- Consider using a `<form>` + `<button type="submit">` with `ServerAction` as a workaround (form submit events use a different code path).
