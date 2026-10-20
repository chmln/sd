# sd - 搜索与替换工具

`sd` 是一个直观易用的命令行查找替换工具。

## 为什么选择 sd？

相比现有的同类工具（如 `sed`），sd 有以下优势：



* **直观的正则表达式**：使用你已经熟悉的 JavaScript / Python 正则语法，无需再去记忆 `sed` 或 `awk` 的各种奇葩语法陷阱，上手即用。

* **字符串字面量模式**：支持非正则的纯字符串查找替换，再也不用纠结哪些字符需要转义。

* **易读易写**：查找和替换的表达式是分开的，结构清晰，不用再处理未闭合的斜杠和转义问题。

* **智能默认值**：默认参数符合日常使用习惯，开箱即用。

## 与 sed 对比

虽然 sed 功能强大，但 sd 专注做好 "查找替换" 这一件事。以下是几个典型场景的对比：

**替换所有匹配项：**



* sd: `sd before after`

* sed: `sed s/before/after/g`

**将换行符替换为逗号：**



* sd: `sd -A '\n' ','`

* sed: `sed ':a;N;$!ba;s/\n/,/g'`

> 注意：因为 
>
> `\n`
>
>  是跨行匹配，需要加 
>
> `-A`
>
> （跨模式）参数。

**提取包含斜杠的字符串内容：**



* sd: `echo "sample with /path/" | sd '.*(/.*/)' '$1'`

* sed: `echo "sample with /path/" | sed -E 's/.*(\\/.*\\/)/\1/g'`

**原地修改文件：**



* sd: `sd before after file.txt`

* sed: `sed -i -e 's/before/after/g' file.txt`

## 安装

通过 [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) 安装：



```
cargo install sd
```

也可以通过各平台的包管理器安装（见英文 README 中的徽章链接）。

## 快速指南

### 1. 字符串字面量模式

默认使用正则表达式，加 `-F` 或 `--fixed-strings` 可切换为纯字符串模式：



```
\> echo 'lots(((\[]))) of special chars' | sd -F '(((\[])))' ''

lots of special chars
```

### 2. 基础正则使用

比如去掉行尾多余的空格：



```
\> echo 'lorem ipsum 23   ' | sd '\s+\$' ''

lorem ipsum 23
```

### 3. 捕获组

按位置索引捕获：



```
\> echo 'cargo +nightly watch' | sd '(\w+)\s+\\+(\w+)\s+(\w+)' 'cmd: \$1, channel: \$2, subcmd: \$3'

cmd: cargo, channel: nightly, subcmd: watch
```

按名称捕获：



```
\> echo "123.45" | sd '(?P\<dollars>\d+)\\.(?P\<cents>\d+)' '\$dollars dollars and \$cents cents'

123 dollars and 45 cents
```

如果遇到歧义，可以用 `${var}` 代替 `$var`：



```
\> echo '123.45' | sd '(?P\<dollars>\d+)\\.(?P\<cents>\d+)' '\${dollars}\_dollars and \${cents}\_cents'

123\_dollars and 45\_cents
```

### 4. 在文件中查找替换



```
\> sd 'window.fetch' 'fetch' http.js
```

文件会被直接原地修改。

如果想先预览改动，加 `-p` 参数：



```
\> sd -p 'window.fetch' 'fetch' http.js
```

### 5. 跨项目批量替换

配合 [fd](https://github.com/sharkdp/fd) 使用：



```
fd --type file --exec sd 'from "react"' 'from "preact"'
```

## 处理模式说明

默认情况下，sd **逐行处理**输入：



* 内存占用低（同一时间只加载一行）

* 流式输出（处理完就输出，不用等全部完成）

* `^` 和 `$` 匹配每行的开头和结尾

如果需要跨行匹配（比如替换 `\n` 或多行模式），加 `-A` / `--across` 参数：



```
\> echo -e "hello\nworld" | sd -A '\n' ','

hello,world
```

## 转义特殊字符

要转义 `$` 字符，使用 `$$`：



```

❯ echo "foo" | sd 'foo' '\$\$bar'

\$bar
```

