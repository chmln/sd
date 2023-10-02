
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'sd' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'sd'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'sd' {
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Limit the number of replacements')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'Regex flags. May be combined (like `-f mc`).')
            [CompletionResult]::new('--flags', 'flags', [CompletionResultType]::ParameterName, 'Regex flags. May be combined (like `-f mc`).')
            [CompletionResult]::new('-p', 'p', [CompletionResultType]::ParameterName, 'Output result into stdout and do not modify files')
            [CompletionResult]::new('--preview', 'preview', [CompletionResultType]::ParameterName, 'Output result into stdout and do not modify files')
            [CompletionResult]::new('-F', 'F ', [CompletionResultType]::ParameterName, 'Treat FIND and REPLACE_WITH args as literal strings')
            [CompletionResult]::new('--fixed-strings', 'fixed-strings', [CompletionResultType]::ParameterName, 'Treat FIND and REPLACE_WITH args as literal strings')
            [CompletionResult]::new('-r', 'r', [CompletionResultType]::ParameterName, 'Recursively replace files')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', 'V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
