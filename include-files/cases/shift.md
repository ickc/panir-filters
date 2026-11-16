# Shifting headings

``` {.include shift-heading-level-by=-1}
file-b.md
```

``` {.include shift-heading-level-by=0}
file-b.md
```

``` {.include shift-heading-level-by=2}
nest/one.md
```

Not a number: no shift at all.

``` {.include shift-heading-level-by=abc}
file-b.md
```

A number as Lua reads one:

``` {.include shift-heading-level-by=0x1}
file-b.md
```
