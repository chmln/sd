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

**Smart, common-sense defaults**

Defaults follow common sense and are tailored for typical daily use. 

## Comparison to sed

While sed does a whole lot more, `sd` focuses on doing just one thing and doing it well.

Some cherry-picked examples, where `sd` shines:

- Simpler syntax for replacing all occurrences:
  - sd: `sd before after`
  - sed: `sed s/before/after/g`
- Replace newlines with commas:
  - sd: `sd '\n' ','`
  - sed: `sed ':a;N;$!ba;s/\n/,/g'`
- Extracting stuff out of strings containing slashes:
  - sd: `echo "sample with /path/" | sd '.*(/.*/)' '$1'`
  - sed: use different delimiters every time depending on expression so that the command is not completely unreadable
    - `echo "sample with /path/" | sed -E 's/.*(/.*/)/\1/g'`
    - `echo "sample with /path/" | sed -E 's|.*(/.*/)|\1|g'`
- In place modification of files:
  - sd: `sd before after file.txt`
  - sed: you need to remember to use `-e` or else some platforms will consider the next argument to be a backup suffix
    - `sed -i -e 's/before/after/g' file.txt`
    
## Benchmarks

**Simple replacement on ~1.5 gigabytes of JSON**

`hyperfine -w 3 'sed -E "s/\"/\'/g" *.json >/dev/null' 'sd "\"" "\'" *.json >/dev/null' --export-markdown out.md`

| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| `sed -E "s/\"/'/g" *.json >/dev/null` | 2.338 ± 0.008 | 2.332…2.358 |
| `sed "s/\"/'/g" *.json >/dev/null` | 2.365 ± 0.009 | 2.351…2.378 |
| `sd "\"" "'" *.json >/dev/null` | **0.997 ± 0.006** | 0.987…1.007 |

Result: ~2.35 times faster

**Regex replacement on a ~55M json file**:

```
hyperfine \
'sed -E "s:(\w+):\1\1:g" dump.json >/dev/null'\
"sed 's:\(\w\+\):\1\1:g' dump.json >/dev/null"\
'sd "(\w+)" "$1$1" dump.json >/dev/null'
```

| Command | Mean [s] | Min…Max [s] |
|:---|---:|---:|
| `sed -E "s:(\w+):\1\1:g" dump.json >/dev/null` | 11.315 ± 0.215 | 11.102…11.725 |
| `sed 's:\(\w\+\):\1\1:g' dump.json >/dev/null` | 11.239 ± 0.208 | 11.057…11.762 |
| `sd "(\w+)" "$1$1" dump.json >/dev/null` | **0.942 ± 0.004** | 0.936…0.951 |

Result: ~11.93 times faster

## Installation

### Cargo

[Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) is the Rust package manager.

You can install cargo by
```sh
curl https://sh.rustup.rs -sSf | sh
```

Then
```sh
cargo install sd
```

### Alpine Linux

```sh
apk add sd
```
Before installing, ensure the appropriate [repository](https://pkgs.alpinelinux.org/packages?name=sd) is enabled.

### Arch Linux

```sh
pacman -S sd
```

### Fedora

```sh
dnf install sd
```

### FreeBSD

```sh
pkg install sd
```

### Windows - Chocolatey

[Chocolatey](https://www.chocolatey.org/) is a Windows package manager.

First [install Chocolatey](https://chocolatey.org/docs/installation) then

```ps
choco install sd-cli
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
> sd 'window.fetch' 'fetch' http.js
```

That's it. The file is modified in-place.

To preview changes:

```sh
> sd -p 'window.fetch' 'fetch' http.js 
```

5. **Find & replace across project**

This example uses [fd](https://github.com/sharkdp/fd).

Good ol' unix philosophy to the rescue.

```sh
sd 'from "react"' 'from "preact"' $(fd --type file)
```

Same, but with backups (consider version control).

```bash
for file in $(fd --type file); do
  cp "$file" "$file.bk"
  sd 'from "react"' 'from "preact"' "$file"; 
done
```
