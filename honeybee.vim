" Based on the following tutorial:
"   https://vim.fandom.com/wiki/Creating_your_own_syntax_files

if exists("b:current_syntax")
  finish
endif

let b:current_syntax = "honeybee"

syn keyword hbKeywords computation facts goal exists
hi def link hbKeywords Keyword

syn keyword hbRetKeyword ret
hi def link hbRetKeyword Statement

syn keyword hbComputationKeyword computation nextgroup=hbComputationName skipwhite
hi def link hbComputationKeyword Keyword

syn match hbComputationName '[a-z][A-Za-z_\-0-9]*'
hi def link hbComputationName Special
syn match _hbComputationName '[a-z][A-Za-z_\-0-9]*'

syn keyword hbForAllKeyword forall nextgroup=hbForAllPlus
hi def link hbForAllKeyword Keyword

syn match hbForAllPlus '+'
hi def link hbForAllPlus Keyword
syn match _hbForAllPlus '+'

syn region hbString start='"' end='"'
hi def link hbString String

syn match hbComment ";.*$"
hi def link hbComment Comment

syn match hbOp '[=<]'
hi def link hbOp Statement

syn keyword hbOpKeyword contains
hi def link hbOpKeyword Statement

syn match hbParens '[\(\)]'
hi def link hbParens Comment

syn match hbBrackets '[\[\]]'
hi def link hbBrackets Comment

syn match hbNumber ' \d\+'
hi def link hbNumber Number

syn match hbSelector '\.[a-z][A-Za-z_\-0-9]*'
hi def link hbSelector Function

syn match hbFact '[A-Z][A-Za-z_\-0-9]*'
hi def link hbFact Type

syn match hbVar '[a-z][A-Za-z_\-0-9]*'
" (No highlight for hbVar.)

syn match hbAnalysisTypeKeyword 'analysis type'
hi def link hbAnalysisTypeKeyword Keyword

syn match hbGroundFactKeyword 'ground fact'
hi def link hbGroundFactKeyword Keyword
