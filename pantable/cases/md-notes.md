# Markdown cells: notes and links

```table
---
markdown: true
caption: "A caption with a note.^[In the caption.]"
---
Inline note,Note and definition,Note defined elsewhere
"text^[an inline note]","text[^a]

[^a]: defined in the same cell","text[^b]"
"[^b]: defined in another cell","[a link][ref]","[ref]: https://example.org"
"[same cell][here]

[here]: https://example.com",,
```
