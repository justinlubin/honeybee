" Based on the following tutorial:
"   https://vim.fandom.com/wiki/Creating_your_own_syntax_files

if exists("b:current_syntax")
  finish
endif

syn keyword ceaKeywords annotation analysis computation annotations goal ret
syn keyword ceaComputationKeyword computation nextgroup=ceaComputationName skipwhite
syn match ceaComputationName '[a-z][A-Za-z_\-0-9]*'
syn match ceaOp '[=<]'
syn match ceaParens '[\(\)]'
syn match ceaString '".*"'
syn match ceaNumber ' \d\+'
syn match ceaSelector '\.[a-z][A-Za-z_\-0-9]*'
syn match ceaFact '[A-Z][A-Za-z_\-0-9]*'
syn match ceaVar '[a-z][A-Za-z_\-0-9]*'

let b:current_syntax = "cea"

hi def link ceaKeywords Keyword
hi def link ceaComputationKeyword Keyword
hi def link ceaComputationName Special
hi def link ceaOp Statement
hi def link ceaParens Comment
hi def link ceaString String
hi def link ceaNumber Number
hi def link ceaSelector Function
hi def link ceaFact Type
