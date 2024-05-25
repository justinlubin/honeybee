" Based on the following tutorial:
"   https://vim.fandom.com/wiki/Creating_your_own_syntax_files

if exists("b:current_syntax")
  finish
endif

let b:current_syntax = "cea"

syn keyword ceaKeywords computation facts goal exists
hi def link ceaKeywords Keyword

syn keyword ceaRetKeyword ret
hi def link ceaRetKeyword Statement

syn keyword ceaComputationKeyword computation nextgroup=ceaComputationName skipwhite
hi def link ceaComputationKeyword Keyword

syn match ceaComputationName '[a-z][A-Za-z_\-0-9]*'
hi def link ceaComputationName Special
syn match _ceaComputationName '[a-z][A-Za-z_\-0-9]*'

syn keyword ceaForAllKeyword forall nextgroup=ceaForAllPlus
hi def link ceaForAllKeyword Keyword

syn match ceaForAllPlus '+'
hi def link ceaForAllPlus Keyword
syn match _ceaForAllPlus '+'

syn region ceaString start='"' end='"'
hi def link ceaString String

syn match ceaComment ";.*$"
hi def link ceaComment Comment

syn match ceaOp '[=<]'
hi def link ceaOp Statement

syn keyword ceaOpKeyword contains
hi def link ceaOpKeyword Statement

syn match ceaParens '[\(\)]'
hi def link ceaParens Comment

syn match ceaBrackets '[\[\]]'
hi def link ceaBrackets Comment

syn match ceaNumber ' \d\+'
hi def link ceaNumber Number

syn match ceaSelector '\.[a-z][A-Za-z_\-0-9]*'
hi def link ceaSelector Function

syn match ceaFact '[A-Z][A-Za-z_\-0-9]*'
hi def link ceaFact Type

syn match ceaVar '[a-z][A-Za-z_\-0-9]*'
" (No highlight for ceaVar.)

syn match ceaAnalysisTypeKeyword 'analysis type'
hi def link ceaAnalysisTypeKeyword Keyword

syn match ceaGroundFactKeyword 'ground fact'
hi def link ceaGroundFactKeyword Keyword
