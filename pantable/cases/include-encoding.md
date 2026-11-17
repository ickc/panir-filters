# Encodings

UTF-8 with a byte-order mark, read as UTF-8 (the mark stays) and as
`utf-8-sig`:

```table
---
include: bom.csv
---
```

```table
---
include: bom.csv
include-encoding: utf-8-sig
---
```

Latin-1, by two of its names:

```table
---
include: latin1.csv
include-encoding: latin-1
---
```

```table
---
include: latin1.csv
include-encoding: ISO-8859-1
---
```

UTF-16 with a byte-order mark:

```table
---
include: utf16.csv
include-encoding: utf-16
---
```

UTF-8, spelled another way:

```table
---
include: plain.csv
include-encoding: UTF8
---
```
