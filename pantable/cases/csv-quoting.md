# CSV as Python's csv module reads it

Quoted fields with a comma, a line break, and a doubled quote; empty
fields; a trailing comma:

```table
a,"b, c","d
e","f ""g"" h"
,,"",
1,2,3,4,
```

A quote inside an unquoted field, text after a closing quote, a blank
before an opening quote:

```table
a"b,"c"d,x "y",z
1,2,3,4
```

A blank line is an empty row:

```table
a,b

c,d
```

A field of only blanks is not empty:

```table
" ",a
```
