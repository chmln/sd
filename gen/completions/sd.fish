complete -c sd -s n -l max-replacements -d 'Limit the number of replacements that can occur per file. 0 indicates unlimited replacements' -r
complete -c sd -s f -l flags -d 'Regex flags. May be combined (like `-f mc`).' -r
complete -c sd -s p -l preview -d 'Display changes in a human reviewable format (the specifics of the format are likely to change in the future)'
complete -c sd -s F -l fixed-strings -d 'Treat FIND and REPLACE_WITH args as literal strings'
complete -c sd -s h -l help -d 'Print help (see more with \'--help\')'
complete -c sd -s V -l version -d 'Print version'
