
use builtin;
use str;

set edit:completion:arg-completer[sd] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'sd'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'sd'= {
            cand -n 'Limit the number of replacements'
            cand -f 'Regex flags. May be combined (like `-f mc`).'
            cand --flags 'Regex flags. May be combined (like `-f mc`).'
            cand -p 'Output result into stdout and do not modify files'
            cand --preview 'Output result into stdout and do not modify files'
            cand -F 'Treat FIND and REPLACE_WITH args as literal strings'
            cand --fixed-strings 'Treat FIND and REPLACE_WITH args as literal strings'
            cand -r 'Recursively replace files'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
