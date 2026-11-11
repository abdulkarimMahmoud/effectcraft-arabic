# Arabic RTL

EffectCraft's Arabic interface (`ar`) mirrors the whole chrome — menus, dock, panels, dialogs,
tables, fields and popups — while the timeline's time axis, the graph editor and the viewer keep
their left-to-right axes, exactly as Arabic After Effects does.

## How it works

- **Language.** `crates/engine/src/settings/prefs/choices.rs` lists `("العربية", "ar")`.
  `crate::i18n::is_rtl` / `current_is_rtl` report whether the resolved interface language is
  right-to-left (`ar`).
- **Style.** `theme::apply_visuals` sets `Style::right_to_left` from `current_is_rtl()`, so egui's
  own widgets (buttons, combo boxes, scroll areas, popups) mirror their layout. The timeline,
  graph editor and viewer draw with explicit coordinates and never use `ui.horizontal`, so their
  axes stay left-to-right.
- **Rows.** `widgets::h_row` / `h_row_wrapped` are the RTL-aware `ui.horizontal` /
  `ui.horizontal_wrapped` for dialog rows; `widgets::labelled_row` / `grid_row` swap a label and
  its control in RTL; `i18n::end_layout` puts dialog buttons at the reading end.
- **Dock.** `dock_ui::dock_style` sets `DockStyle::right_to_left` from `current_is_rtl()`; the
  craft-libs dock mirrors its chrome and new-dock placement accordingly.
- **Icons.** `icons::paint_mirrored` flips directional chevrons and arrows in RTL. Media
  transport (`Play`, `StepBack`, `StepFwd`, `First`, `Last`) is never mirrored.
- **Text.** `i18n::bidi::visual_order` reorders mixed Arabic/Latin runs (UAX #9) for owned label
  strings; editable fields are never reordered. IBM Plex Sans Arabic (OFL) is bundled in
  `theme.rs` so the web build draws Arabic without system fonts.

## The egui fork patch (submenus)

The egui fork's `SubMenu::show` (`crates/egui/src/containers/menu.rs`) hardcodes
`.align(RectAlign::RIGHT_START)` and `FocusDirection::Right`, so submenu popups open to the right
even in RTL. Until the fork is patched, Arabic submenus open to the right while their arrows
(pointing left, via `widgets::submenu_button`) already point the right way.

The patch, in the fork's `menu.rs`:

1. In `SubMenu::show`, replace `.align(RectAlign::RIGHT_START)` with
   `.align(if ui.style().right_to_left { RectAlign::LEFT_START } else { RectAlign::RIGHT_START })`.
2. In `SubMenu::show`, replace the `FocusDirection::Right` enter check with
   `FocusDirection::Left` when `right_to_left`.
3. In `MenuState::handle_keyboard`, the `FocusDirection::Left => anchor_widget` arm (exit the
   submenu) becomes `FocusDirection::Right` in RTL.

`RectAlign::LEFT_START` already exists in the fork (`emath/src/rect_align.rs`), and
`Popup::get_best_align` already flips popups at the screen edge via `find_best_align` /
`symmetries()` / `MENU_ALIGNS`, so no other fork change is needed.

## What stays left-to-right

- Timeline time axis, playhead, keyframe positions, graph editor, viewer/3D coordinates.
- The anchor-point 3×3 spatial grid (`comp-anchor`) — it is a map of the comp frame.
- Western digits (`0123`) for timecode and coordinates.
- Brand names (EffectCraft, After Effects) stay in English.
