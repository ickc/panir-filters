# Empty tables are deleted

```table
```

Only blanks:

```table
   
```

Only delimiters:

```table
,,
,
```

Only quoted blanks:

```table
" "," "
```

Only YAML, closed by dots:

```table
---
caption: nothing
...
```

The paragraph after them stays.
