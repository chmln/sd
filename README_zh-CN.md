# sd - `搜索`与`替换`

`sd` 是一个直观的查找与替换命令行工具。

## 主要优点

为什么要使用它而不是现有的任何工具？

*更好的正则表达式* &nbsp; `sd` 使用您已经熟悉的来自 JavaScript 和 Python 的正则表达式语法。不用再去处理 `sed` 或 `awk` 的生僻语法 - 立即提高生产力。

*字符串文本模式* &nbsp; 非正则表达式的查找和替换。不再需要反斜杠或记住哪些字符是特殊的并且需要转义。

*易读易写* &nbsp; 查找和替换表达式被拆分开来，这样更容易阅读和编写。不再需要处理未闭合和转义的斜杠。

*智能、符合常识的默认设置* &nbsp; 默认设置遵循常识，并且针对典型的日常使用进行了调整。

## 与 sed 相比

虽然 sed 可以做更多的事情，但 sd 专注于做一件事情，并且做得很好。以下是一些精选的例子，展示了 sd 的优势所在。

替换所有出现的内容的更简单语法：
  - sd: `sd before after`
  - sed: `sed s/before/after/g`

将换行符替换为逗号：
  - sd: `sd '\n' ','`
  - sed: `sed ':a;N;$!ba;s/\n/,/g'`

从包含斜杠的字符串中提取内容：
  - sd: `echo "sample with /path/" | sd '.*(/.*/)' '$1'`
  - sed: `echo "sample with /path/" | sed -E 's/.*(\\/.*\\/)/\1/g'`
    
    使用 sed，你可以使用不同的分隔符来改善，但仍然有些混乱：
    
    `echo "sample with /path/" | sed -E 's|.*(/.*/)|\1|g'`

原地修改文件：
  - sd: `sd before after file.txt`
  - sed: `sed -i -e 's/before/after/g' file.txt`
    
    在使用 sed 时，需要记住使用 `-e`，否则某些平台会将下一个参数视为备份后缀。

## 基准测试

**在大约 1.5GB 大小的 JSON 文件上进行简单的替换**

```sh
hyperfine --warmup 3 --export-markdown out.md \
  'sed -E "s/\"/'"'"'/g" *.json > /dev/null' \
  'sed    "s/\"/'"'"'/g" *.json > /dev/null' \
  'sd     "\"" "'"'"'"   *.json > /dev/null'
```

| 命令 | 平均 [s] | 最小耗时…最大耗时 [s] |
|:---|---:|---:|
| `sed -E "s/\"/'/g" *.json > /dev/null` | 2.338 ± 0.008 | 2.332…2.358 |
| `sed    "s/\"/'/g" *.json > /dev/null` | 2.365 ± 0.009 | 2.351…2.378 |
| `sd     "\"" "'"   *.json > /dev/null` | **0.997 ± 0.006** | 0.987…1.007 |

结果：速度提高了大约 2.35 倍

**对一个约 55M 大小的 JSON 文件进行正则表达式替换**:

```sh
hyperfine --warmup 3 --export-markdown out.md \
  'sed -E "s:(\w+):\1\1:g"    dump.json > /dev/null' \
  'sed    "s:\(\w\+\):\1\1:g" dump.json > /dev/null' \
  'sd     "(\w+)" "$1$1"      dump.json > /dev/null'
```

| 命令 | 平均 [s] | 最低…最高 [s] |
|:---|---:|---:|
| `sed -E "s:(\w+):\1\1:g"    dump.json > /dev/null` | 11.315 ± 0.215 | 11.102…11.725 |
| `sed    "s:\(\w\+\):\1\1:g" dump.json > /dev/null` | 11.239 ± 0.208 | 11.057…11.762 |
| `sd     "(\w+)" "$1$1"      dump.json > /dev/null` | **0.942 ± 0.004** | 0.936…0.951 |

结果：速度提高了大约 11.93 倍

## 安装

通过 [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html) 使用 `cargo install sd` 命令安装，或通过各种包管理器安装。

[![Packaging status](https://repology.org/badge/vertical-allrepos/sd-find-replace.svg?exclude_unsupported=1)](https://repology.org/project/sd-find-replace/versions)

## 快速指南

1. **字符串文字**模式。默认情况下，表达式被视为正则表达式。使用 `-F` 或 `--fixed-strings` 可以禁用正则表达式。

   ```sh
   > echo 'lots((([]))) of special chars' | sd -s '((([])))' ''
   lots of special chars
   ```

2. **基本正则表达式的使用** - 让我们去掉一些末尾的空白符

   ```sh
   > echo 'lorem ipsum 23   ' | sd '\s+$' ''
   lorem ipsum 23
   ```

3. **捕获组**

   索引捕获组：

   ```sh
   > echo 'cargo +nightly watch' | sd '(\w+)\s+\+(\w+)\s+(\w+)' 'cmd: $1, channel: $2, subcmd: $3'
   cmd: cargo, channel: nightly, subcmd: watch
   ```

   命名捕获组：

   ```sh
   > echo "123.45" | sd '(?P<dollars>\d+)\.(?P<cents>\d+)' '$dollars dollars and $cents cents'
   123 dollars and 45 cents
   ```

   在不太可能出现歧义的情况下，通过使用 `${var}` 而不是 `$var` 来解决。这里有一个例子：

   ```sh
   > echo '123.45' | sd '(?P<dollars>\d+)\.(?P<cents>\d+)' '$dollars_dollars and $cents_cents'
    and

   > echo '123.45' | sd '(?P<dollars>\d+)\.(?P<cents>\d+)' '${dollars}_dollars and ${cents}_cents'
   123_dollars and 45_cents
   ```

4. **在文件中查找并替换**

   ```sh
   > sd 'window.fetch' 'fetch' http.js
   ```

   就是这样，文件将直接在原地修改。

   预览更改：

   ```sh
   > sd -p 'window.fetch' 'fetch' http.js
   ```

5. **在整个项目中查找并替换**

   这个例子使用了 [fd](https://github.com/sharkdp/fd)。

   好的 Unix 哲学来拯救我们了。

   ```sh
   fd --type file --exec sd 'from "react"' 'from "preact"'
   ```

   同理，但带有备份（考虑版本控制）。

   ```bash
   fd --type file --exec cp {} {}.bk \; --exec sd 'from "react"' 'from "preact"'
   ```

### 特殊情况

sd 会将以 `-` 开头的每个参数解释为（可能是未知的）标志。
    
尊重常见的惯例，使用 `--` 来表示标志的结束：

```bash
$ echo "./hello foo" | sd "foo" "-w"
error: Found argument '-w' which wasn't expected, or isn't valid in this context

USAGE:
    sd [OPTIONS] <find> <replace-with> [files]...

For more information try --help
$ echo "./hello foo" | sd "foo" -- "-w"
./hello -w
$ echo "./hello --foo" | sd -- "--foo" "-w"
./hello -w
```

### 转义特殊字符

要转义 `$` 字符，需使用 `$$`：

```bash
❯ echo "foo" | sd 'foo' '$$bar'
$bar
```

### 帮助

使用方法
```shell
sd [OPTIONS] <FIND> <REPLACE_WITH> [FILES]...
   [选项]     <查找>  <替换为>       [文件列表]...

参数：
  
  <FIND>
          要搜索的正则表达式或字符串（如果使用 `-F` 选项）

  <REPLACE_WITH>
          替换每个匹配项的内容。除非处于字符串模式，否则您可以使用类似 $1、$2 等捕获值

  [FILES]...
          文件路径。这是可选项， - sd 也可以从标准输入 STDIN 中读取。
          请注意，sd 默认会直接修改文件。请参阅文档中的示例。

选项:
  -p, --preview
          以可阅读的方式显示更改（具体格式的细节可能会在将来更改）

  -F, --fixed-strings
          将 FIND 和 REPLACE_WITH 参数视为文字字符串

  -n, --max-replacements <LIMIT>
          限制每个文件的替换次数。0 表示无限制替换
          [默认值为：0]

  -f, --flags <FLAGS>
          正则表达式标志。可以组合使用（如 `-f mc`）。

          c - 区分大小写

          e - 禁用多行匹配

          i - 不区分大小写

          m - 多行匹配

          s - 使 `.` 匹配换行符

          w - 仅匹配完整单词

  -h, --help
          打印帮助信息（使用 '-h' 可以查看摘要）

  -V, --version
          打印版本信息
```