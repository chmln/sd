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

## Guide

1. By default, expressions are treated as literals.

```sh
> echo "lots((([]))) of special chars" | sd "((([])))" ""
lots of special chars
```

Use `-r` or `--regex` to enable regex.

2. Basic regex use - let's trim some trailing whitespace

```sh
> echo "lorem ipsum 23   " | sd -r '\s+$' ''
lorem ipsum 23
```

3. Capturing useful information. 

Indexed capture groups:

```sh
> echo "cargo +nightly watch" | sd -r '(\w+)\s+\+(\w+)\s+(\w+)' 'cmd: $1, channel: $2, subcmd: $3'
cmd: cargo, channel: nightly, subcmd: watch
```

Named capture groups:

```sh
> echo "123.45" | sd -r '(?P<dollars>\d+)\.(?P<cents>\d+)' '$dollars dollars and $cents cents'
123 dollars and 45 cents
```

If you stumble upon any ambiguities, just use `${1}` instead of `$1`:

```sh
> echo "123.45" | sd -r '(?P<dollars>\d+)\.(?P<cents>\d+)' '$dollars_dollars and $cents_cents'
 and 
> echo "123.45" | sd -r '(?P<dollars>\d+)\.(?P<cents>\d+)' '${dollars}_dollars and ${cents}_cents'
123_dollars and 45_cents
```

You may choose to always use `${1}` or only add as necessary.


4. Find & replace in files

```sh
> sd "window.fetch" "fetch" -i http.js
```

That's it.

Do a dry run:

```sh
> sd "window.fetch" "fetch" < http.js | less
```

5. Find & replace across your project

Good ol' unix philosophy to the rescue.

```sh
fd -t f --exec sd 'from "react"' 'from "preact"' -i {}
```

Same, but with backups (consider version control).

```bash
for file in $(fd -t f); do
  cp "$file" "$file.bk"
  sd 'from "react"' 'from "preact"' -i "$file"; 
done
```
