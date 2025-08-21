import ast
import inspect
import re

import ctypes

props = {}
classes = {}

def deindent(s):
    # https://stackoverflow.com/a/2378988
    initial_indent = len(s) - len(s.lstrip())
    ret = ""
    for line in s.splitlines():
        ret += line[initial_indent:] + "\n"
    return ret


# Based on https://stackoverflow.com/a/77628177
def _attribute_info(cls):
    src = deindent(inspect.getsource(cls))
    tree = ast.parse(src.strip())
    for t in ast.walk(tree):
        if isinstance(t, ast.ClassDef):
            class_def = t
            break
    else:
        return None

    types = {}
    docs = {}
    last_attribute = None
    for expr in class_def.body:
        if isinstance(expr, ast.AnnAssign):
            last_attribute = ast.unparse(expr.target)
            types[last_attribute] = ast.unparse(expr.annotation)
        elif isinstance(expr, ast.Expr) and last_attribute is not None:
            value = expr.value.value
            if isinstance(value, str):
                docs[last_attribute] = value.strip()

    return types, docs


def _python_to_honeybee(type_name: str) -> str:
    if type_name == "str":
        return '"Str"'
    elif type_name == "int":
        return '"Int"'
    elif type_name == "bool":
        return '"Bool"'
    else:
        raise ValueError("Unable to convert to Honeybee type: " + type_name)


def _emit_fact(fact_kind, cls, parent_cls, kwargs):
    # emit @dataclass 
    types, docs = _attribute_info(cls)
    print(f"[{fact_kind}.{parent_cls.__name__}]")
    if len(types) == 0:
        print("params = {}")
    for p in types:
        hb_type = _python_to_honeybee(types[p])
        print(f"params.{p} = {hb_type}")
    if parent_cls.__doc__ is not None:
        doc_lines = parent_cls.__doc__.splitlines()
        if len(doc_lines) >= 3 and doc_lines[1].strip() == "":
            first = doc_lines[0]
            rest = "\n".join(doc_lines[2:])
            print(f'info.title = "{first}"')
            print(f'info.description = """{rest}"""')
        else:
            print(f'info.title = "{parent_cls.__doc__}"')
    for p in docs:
        print(f'info.params.{p} = "{docs[p]}"')
    for k in kwargs:
        print(f'info.{k} = "{kwargs[k]}"')
    code = inspect.getsource(parent_cls)
    new_code = ""
    for line in code.splitlines():
        if line.startswith("@Type") or line.startswith("@Prop"):
            continue
        new_code += line + "\n"
    print(f"info.code = '''{new_code.strip()}'''")
    print()

# assume lhs of = in conditions is ret/param.fieldm rhs can be ret/param.field or int/bool/string
# assume honeybee types cannot have other honeybee types as fields, so no param.field.field in condition- no chains of multiples dots
# !=, =, < are acceptable
def _emit_function(f, condition, kwargs):
    def type_of(arg):
        try:
            int(arg)
            return int
        except(ValueError):
            if arg == "true" or arg == "false":
                return bool
            # split on "."
            # for some reason re.split('.', arg) didnt work
            split = re.split(r'[.]', arg)
            # error if len != 2
            if len(split) == 1:
                return str
            if len(split) > 2:
                raise ValueError(
                    f"In type_of too many dots '{arg}'"
                )   
            # split left and right
            lhs = split[0]
            rhs = split[1]
            # make sure lhs is in params
            if lhs not in params:
                raise ValueError(
                    f"not in params"
                )   
            # get type of lhs
            lhs_cls = params[lhs]
            # if lhs is ret, get .S
            if lhs != "ret":
                lhs_cls = lhs_cls.S
            # make sure rhs is a field of lhs
            if rhs not in lhs_cls.__annotations__:
                raise ValueError(
                    f"class doesn't have that field"
                )  
            # get type of that field
            return lhs_cls.__annotations__[rhs]

    print(f"[Function.{f.__name__}]")
    params = f.__annotations__
    return_cls = params.pop("return")

    if return_cls.__name__ != "D" or not hasattr(return_cls, "__honeybee_parent"):
        raise ValueError(
            f"Must return dynamic Honeybee type in function '{f.__name__}'"
        )
        
    return_name = return_cls.__honeybee_parent.__name__

    if len(params) == 0 or list(params)[-1] != "ret":
        raise ValueError(f"Need 'ret' as final param in function '{f.__name__}'")
    if len(params) == 1:
        print("params = {}")
    for p in params:
        cls = params[p]
        if p == "ret":
            # ret needs to be x.S and return needs to be z.D and x must == z
            if cls.__name__ != "S" or not hasattr(cls, "__honeybee_parent"):
                raise ValueError(
                    f"'ret' must be static Honeybee type in function '{f.__name__}'"
                )
            ret_name = cls.__honeybee_parent.__name__
            if ret_name != return_name:
                raise ValueError(
                    f"'ret' and Return must be same Honeybee type in function '{f.__name__}'"
                )
            print(f'ret = "{return_name}"')
        else:
            if not hasattr(cls, "__honeybee_object") or cls.__honeybee_object != "Type":
                raise ValueError(
                    f"Cannot use non-Honeybee type '{cls.__name__}' in function '{f.__name__}'"
                )
            print(f'params.{p} = "{cls.__name__}"')
    print("condition = [")
    # CONDITION HERE
    for c in condition:
        c = c.replace('"', '\\"')
        c_original = c
        # take out spaces
        c = c.replace(" ", "")
        # split on '{' if len 2 this has to do with proposition
        c = re.split('{', c)
        if len(c) == 1:
            # binary operator
            c = re.split(r'[=|!=|<]', c[0])
            # if len of c is not two, that means one of those characters was not found or too many were found
            if len(c) != 2:
                raise ValueError(
                    f"expectd = OR != OR < in condition '{c_original}' function '{f.__name__}'"
                )
            #set lhs and rhs
            lhs = c[0]
            rhs = c[1]
            # now make sure lhs and rhs have same type
            lhs_type = type_of(lhs)
            rhs_type = type_of(rhs)
            if lhs_type is not rhs_type:
                raise ValueError(
                    f"lhs and rhs are not same type in function '{f.__name__}'"
                )
        else:
            # expecting c to be len 2 here- no reason for more than one {
            if len(c) != 2:
                raise ValueError(
                    f"condition should not have more than one opening curly brace in function '{f.__name__}'"
                )
            prop_name = c[0]
            # make sure prop name exists
            if prop_name not in props:
                raise ValueError(
                    f"prop name not in props in function '{f.__name__}'"
                )
            c = c[1]
            c = re.split('}', c)
            if len(c) != 2:
                raise ValueError(
                    f"prop conditions doesn't have one closing curly brace in function '{f.__name__}'"
                )
            if c[1] != '':
                raise ValueError(
                    f"extra stuff after end of condition in function '{f.__name__}'"
                )
            c = c[0]
            # split on =
            c = re.split('=', c)
            if len(c) != 2:
                raise ValueError(
                    f"no lhs, rhs in function '{f.__name__}'"
                )
            lhs = c[0]
            rhs = c[1]
            # make sure lhs is a field of prop name
            prop_cls = props[prop_name]
            if lhs not in prop_cls.__annotations__:
                raise ValueError(
                    f"field name not in prop in function '{f.__name__}'"
                )
            # get type of lhs
            lhs_type = prop_cls.__annotations__[lhs]
            # get type of rhs
            rhs_type = type_of(rhs)
            # compare types
            if lhs_type is not rhs_type:
                raise ValueError(
                    f"lhs and rhs are not same type in function '{f.__name__}'"
                )


        print(f'    "{c_original}",')
    print("]")
    if f.__doc__ is not None:
        doc_lines = f.__doc__.splitlines()
        if len(doc_lines) >= 3 and doc_lines[1].strip() == "":
            first = doc_lines[0]
            rest = "\n".join(doc_lines[2:])
            print(f'info.title = "{first}"')
            print(f'info.description = """{rest}"""')
        else:
            print(f'info.description = """{f.__doc__}"""')
    for k in kwargs:
        print(f'info.{k} = "{kwargs[k]}"')
    code = inspect.getsource(f)
    new_code = ""
    found_def = False
    for line in code.splitlines():
        if line.startswith("def"):
            found_def = True
        if not found_def:
            continue
        new_code += line + "\n"
    print(f"info.code = '''{new_code.strip()}'''")
    print()


# Based on https://stackoverflow.com/a/14412901
def Prop(*args, **kwargs):
    def wrap(cls):
        props[cls.__name__] = cls
        _emit_fact("Prop", cls, cls, kwargs)
        cls.__honeybee_object = "Prop"
        return cls

    if len(args) == 1 and len(kwargs) == 0 and callable(args[0]):
        return wrap(args[0])
    else:
        return wrap


# Based on https://stackoverflow.com/a/14412901
def Type(*args, **kwargs):
    def wrap(cls):
        classes[cls.__name__] = cls
        _emit_fact("Type", cls.S, cls, kwargs)
        cls.S.__honeybee_parent = cls
        cls.D.__honeybee_parent = cls

        if '__annotations__' not in dir(cls):
            raise ValueError("Doesn't have annotations")

        if 'static' not in cls.__annotations__:
            raise ValueError("Doesn't have static")
        # name of cls.__annotations__['static'].__name__ needs to be S
        if cls.__annotations__['static'].__name__ != 'S':
            raise ValueError("static type is not S")
        # cls.__annotations__['static'].__honeybee_parent should be same as cls
        if cls.__annotations__['static'].__honeybee_parent != cls:
            raise ValueError("static parent is not cls")
        
        if 'dynamic' not in cls.__annotations__:
            raise ValueError("Doesn't have dynamic")
        # name of cls.__annotations__['dynamic'].__name__ needs to be D
        if cls.__annotations__['dynamic'].__name__ != 'D':
            raise ValueError("dynamic type is not D")
        # cls.__annotations__['static'].__honeybee_parent should be same as cls
        if cls.__annotations__['dynamic'].__honeybee_parent != cls:
            raise ValueError("dynamic parent is not cls")
        cls.__honeybee_object = "Type"
        return cls

    if len(args) == 1 and len(kwargs) == 0 and callable(args[0]):
        return wrap(args[0])
    else:
        return wrap


def Function(*condition, **kwargs):
    def wrap(f):
        _emit_function(f, condition, kwargs)
        f.__honeybee_object = "Function"
        return f

    return wrap


helper_ran = False


def Helper(f):
    global helper_ran
    if not helper_ran:
        print("[[Preamble]]\ncontent = 'from dataclasses import dataclass'\n")
        helper_ran = True

    code = inspect.getsource(f)
    new_code = ""
    found_def = False
    for line in code.splitlines():
        if line.startswith("def"):
            found_def = True
        if not found_def:
            continue
        new_code += line + "\n"
    print(f"[[Preamble]]\ncontent='''{new_code.strip()}'''\n")
