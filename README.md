# sd - s[earch] & d[isplace]

`sd` is an intuitive find & replace CLI.

## The Pitch

Why use it over any existing tools?

**Painless regular expressions**

`sd` uses regex syntax that you already know from JavaScript and Python. Forget about dealing with quirks of `sed` or `awk` - get productive immediately.

**String-literal mode**

Non-regex find & replace. No more backslashes or remembering which characters are special and need to be escaped.

**Easy to read, easy to write**

Find & replace expressions are split up, which makes them easy to read and write. No more messing with unclosed and escaped slashes.

## Comparison to sed

While sed does a whole lot more, `sd` focuses on doing just one thing and doing it well.

Some cherry-picked examples, where `sd` shines:
- Replace newlines with commas:
  - sd: `sd '\r' ','`
  - sed: `sed ':a;N;$!ba;s/\r/,/g'`
- Extracting stuff out of strings with special characters
  - sd: `echo "{((sample with /path/))}" | sd '\{\(\(.*(/.*/)\)\)\}' '$1'`
  - sed
    - incorrect, but closest I could get after 15 minutes of struggle
    - `echo "{((sample with /path/))}" | sed 's/{((\.\*\(\/.*\/\)))}/\1/g'`

Note: although `sed` does have a nicer regex syntax with `-r`, it is a non-portable GNU-ism and thus doesn't work on MacOS, BSD, or Solaris. 

## Installation

```sh
cargo install sd
```

## Quick Guide

1. **String-literal mode**. By default, expressions are treated as regex. Use `-s` or `--string-mode` to disable regex.


```sh
> echo 'lots((([]))) of special chars' | sd -s '((([])))' ''
lots of special chars
```


2. **Basic regex use** - let's trim some trailing whitespace

```sh
> echo 'lorem ipsum 23   ' | sd '\s+$' ''
lorem ipsum 23
```

3. **Capture groups**

Indexed capture groups:

```sh
> echo 'cargo +nightly watch' | sd '(\w+)\s+\+(\w+)\s+(\w+)' 'cmd: $1, channel: $2, subcmd: $3'
cmd: cargo, channel: nightly, subcmd: watch
```

Named capture groups:

```sh
> echo "123.45" | sd '(?P<dollars>\d+)\.(?P<cents>\d+)' '$dollars dollars and $cents cents'
123 dollars and 45 cents
```

In the unlikely case you stumble upon ambiguities, resolve them by using `${var}` instead of `$var`. Here's an example:

```sh
> echo '123.45' | sd '(?P<dollars>\d+)\.(?P<cents>\d+)' '$dollars_dollars and $cents_cents'
 and 
> echo '123.45' | sd '(?P<dollars>\d+)\.(?P<cents>\d+)' '${dollars}_dollars and ${cents}_cents'
123_dollars and 45_cents
```

4. **Find & replace in a file**

```sh
> sd 'window.fetch' 'fetch' -i http.js
```

That's it. The file is modified in-place.

To do a dry run, just use stdin/stdout:

```sh
> sd 'window.fetch' 'fetch' < http.js 
```

5. **Find & replace across project**

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
