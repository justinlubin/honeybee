import ast
import dataclasses
import inspect


# Based on https://stackoverflow.com/a/77628177
def _attribute_info(cls):
    src = inspect.getsource(cls)
    tree = ast.parse(src)
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


def _emit_fact(fact_kind, cls, kwargs):
    types, docs = _attribute_info(cls)
    print(f"[{fact_kind}.{cls.__name__}]")
    if len(types) == 0:
        print("params = {}")
    for p in types:
        hb_type = _python_to_honeybee(types[p])
        print(f"params.{p} = {hb_type}")
    if cls.__doc__ is not None:
        print(f'info.overview = "{cls.__doc__}"')
    for p in docs:
        print(f'info.params.{p} = "{docs[p]}"')
    for k in kwargs:
        print(f'info.{k} = "{kwargs[k]}"')
    print()


def _emit_function(f, condition, kwargs):
    print(f"[Function.{f.__name__}]")
    params = f.__annotations__
    if len(params) == 0 or list(params)[-1] != "ret":
        raise ValueError("Need 'ret' as final param in function '{f.__name__}'")
    if len(params) == 1:
        print("params = {}")
    for p in params:
        cls = params[p]
        if not hasattr(cls, "__honeybee_object") or cls.__honeybee_object != "Type":
            raise ValueError(
                f"Cannot use non-Honeybee type '{cls.__name__}' in function '{f.__name__}'"
            )
        if p == "ret":
            print(f'ret = "{cls.__name__}"')
        else:
            print(f'params.{p} = "{cls.__name__}"')
    print("condition = [")
    for c in condition:
        c = c.replace('"', '\\"')
        print(f'    "{c}",')
    print("]")
    if f.__doc__ is not None:
        print(f'info.overview = "{f.__doc__}"')
    for k in kwargs:
        print(f'info.{k} = "{kwargs[k]}"')
    print()


# Based on https://stackoverflow.com/a/14412901
def Prop(*args, **kwargs):
    def wrap(cls):
        _emit_fact("Prop", cls, kwargs)
        cls.__honeybee_object = "Prop"
        return dataclasses.dataclass(cls)

    if len(args) == 1 and len(kwargs) == 0 and callable(args[0]):
        return wrap(args[0])
    else:
        return wrap


# Based on https://stackoverflow.com/a/14412901
def Type(*args, **kwargs):
    def wrap(cls):
        _emit_fact("Type", cls, kwargs)
        cls.__honeybee_object = "Type"
        return dataclasses.dataclass(cls)

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
