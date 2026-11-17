# Captions, as text

```table
---
caption: Planets, as text
short-caption: Planets
---
Planet,Moons
Earth,1
```

A number, a boolean, a date, and an empty value are each made text:

```table
---
caption: 1.5
short-caption: 10
---
a,b
```

```table
---
caption: yes
short-caption: false
---
a,b
```

```table
---
caption: 2026-11-17
---
a,b
```

```table
---
caption:
---
a,b
```

Markdown stays text:

```table
---
caption: "*Not* **emphasis**"
---
a,b
```

`Caption`, capitalised, is not an option:

```table
---
Caption: Ignored
---
a,b
```
