# DDNSupd UI guidelines

Independent implementation informed by the pinned TalkOps design reference.
Use Svelte 5 runes, SvelteKit static output, TypeScript and Tailwind. Keep
API, session, translations and display components in separate modules.

- Slate backgrounds and cards, teal primary actions, readable light/dark themes.
  Persist theme and locale; apply theme before paint from a local script.
- One topbar, responsive side navigation and one footer. Main content uses
  container queries. Tables keep identity/status/actions visible on phones.
- Use one icon system. Give icon-only actions accessible labels. Do not use
  color alone for status. Use native forms, labels, keyboard focus and dialogs.
- Reusable page headers, cards, inputs, buttons, badges, table skeletons and
  error/empty states. A failed request must not appear as an empty successful list.
- Records have a URL-addressable side inspector; narrow screens use a full
  panel. Preserve Ctrl/Cmd-click behavior. Dialogs trap focus and restore it.
- Search, filters and sort are explicit. Paginated lists sort/filter on server.
- Initial skeletons are visible immediately. Polling preserves current data.
  Respect prefers-reduced-motion. Motion durations: 120, 180 and 260 ms.
- All visible product strings use equal DE/EN key sets. User-provided resource
  names are displayed as text. Never render provider-supplied HTML.
- Destructive actions identify the object and need confirmation. Credential
  forms never prefill saved secrets. Connection tests are read-only.
- Show actual provider state and evidence level. API acceptance does not mean
  global DNS propagation. Do not insert sample operational data into live views.
- Browser checks: 360, 768 and 1280 px; both themes and languages; keyboard
  operation, login/logout, loading, empty/error states and record workflow.
