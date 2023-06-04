complete -c sd -s n -d 'Limit the number of replacements' -r
complete -c sd -s f -l flags -d 'Regex flags. May be combined (like `-f mc`).' -r
complete -c sd -s p -l preview -d 'Output result into stdout and do not modify files'
complete -c sd -s F -l fixed-strings -d 'Treat FIND and REPLACE_WITH args as literal strings'
complete -c sd -s r -d 'Recursively replace files'
complete -c sd -s h -l help -d 'Print help (see more with \'--help\')'
complete -c sd -s V -l version -d 'Print version'
