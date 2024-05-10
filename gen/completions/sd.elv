
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
            cand -n 'Limit the number of replacements that can occur per file. 0 indicates unlimited replacements'
            cand --max-replacements 'Limit the number of replacements that can occur per file. 0 indicates unlimited replacements'
            cand -f 'Regex flags. May be combined (like `-f mc`).'
            cand --flags 'Regex flags. May be combined (like `-f mc`).'
            cand -p 'Display changes in a human reviewable format (the specifics of the format are likely to change in the future)'
            cand --preview 'Display changes in a human reviewable format (the specifics of the format are likely to change in the future)'
            cand -F 'Treat FIND and REPLACE_WITH args as literal strings'
            cand --fixed-strings 'Treat FIND and REPLACE_WITH args as literal strings'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
