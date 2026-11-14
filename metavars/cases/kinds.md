---
author: Caleb *Hyde*
place: "%{city}"
city: Paris
draft: true
abstract: |
  First paragraph.

  Second paragraph.
subtitle: "%{author}"
---

By %{author} in %{place} today.

Not a field: %{author}. or (%{author}) or %{author}%{city}x.

A boolean, %{draft}, and blocks, %{abstract}, stay as they are.

- %{city}
- [%{author}](https://example.org)
