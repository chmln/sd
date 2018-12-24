# sd - s[earch] & d[isplace]

`sd` is a simple, user-friendly find & replace command line tool.

## Features

**Painless regular expressions**

Use regex syntax that you already know from JavaScript, Python, and Rust. No need to learn  special syntax or eccentrisms of `sed` or `awk`. Easily access your captured groups with `$1`, `$2`.

**String-literal mode**

In string-literal mode, you don't need to escape any special characters - its simply unnecessary.

**Easy to read, easy to write**

Find & replace expressions are split up and in most cases unescaped, which contributes to readability and makes it easier to spot errors in your regexes.

## Comparison to sed

Although `sed` has a nice regex syntax with `-r`, it is not portable and doesn't work on, say, MacOS or Solaris. Also, `sed` is far more powerful. Focusing on just finding and replacing allows `sd` to make this common task far more straightforward.

Some cherry-picked examples, where `sd` shines:
- Replace newlines with commas:
  - `sed ':a;N;$!ba;s/\r/,/g'` vs
  - `sd -r '\r' ','`




