# Kinds of file

```{include=src/no-newline.txt}
```

```{include=src/empty.txt startLine=1}
```

```{include=src/crlf.txt startLine=2}
```

```{include=src/utf8.txt}
```

```{include=src/five.txt}
old content is replaced
```

```{.c #id include=src/five.txt endLine=1 key=value}
```

```
no include: left alone
```

`inline code with include` {include=src/five.txt}
