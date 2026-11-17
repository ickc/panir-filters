# markdown as YAML gives it

The text "false" is true:

```table
---
markdown: "false"
---
*a*,b
```

YAML 1.1's `no` and `yes`:

```table
---
markdown: no
---
*a*,b
```

```table
---
markdown: yes
---
*a*,b
```

YAML 1.2's `False` and `True`:

```table
---
markdown: False
---
*a*,b
```

```table
---
markdown: True
---
*a*,b
```
