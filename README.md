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

While sed is frighteningly powerful, `sd` focuses on doing just one thing and doing it well.

Some cherry-picked examples, where `sd` shines:
- Replace newlines with commas:
  - sed: `sed ':a;N;$!ba;s/\r/,/g'` vs
  - sd: `sd -r '\r' ','`
- Extracting stuff out of strings with special characters
  - sd: `echo "{((sample with /path/))}" | sd -r '\{\(\(.*(/.*/)\)\)\}' '$1'`
  - sed
    - incorrect, but closest I could get after 15 minutes of struggle
    - `echo "{((sample with /path/))}" | sed 's/{((\.\*\(\/.*\/\)))}/\1/g'`

Note: although `sed` has a nicer regex syntax with `-r`, it is not portable and doesn't work on, say, MacOS or Solaris. 

