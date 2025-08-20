FormulaList ::= $\epsilon$ | Formula, FormulaList <br>
Formula ::= BinaryOperator | AtomicProposition <br>
BinaryOperator ::= Atom Symbol Atom <br>
AtomicProposition ::= prop { field = Atom} <br>
Atom ::= Selector | b | s | i <br>
Symbol ::= < | = | != <br>
Selector = Var.field <br>
Var = param | ret <br><br>

i is int <br>
b is bool <br>
s is string <br>
field in AtomicProposition and Selector are same <br>
prop is the name of a Proposition <br>
param is the name of a parameter to the function <br>
ret is a function parameter referring to return