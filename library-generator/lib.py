import ast
import inspect


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


def _emit_function(f, condition, kwargs):
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
            print(f'ret = "{return_name}"')
        else:
            if not hasattr(cls, "__honeybee_object") or cls.__honeybee_object != "Type":
                raise ValueError(
                    f"Cannot use non-Honeybee type '{cls.__name__}' in function '{f.__name__}'"
                )
            print(f'params.{p} = "{cls.__name__}"')
    print("condition = [")
    for c in condition:
        c = c.replace('"', '\\"')
        print(f'    "{c}",')
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
        _emit_fact("Type", cls.S, cls, kwargs)
        cls.S.__honeybee_parent = cls
        cls.D.__honeybee_parent = cls
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
