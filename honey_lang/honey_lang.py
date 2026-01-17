from dataclasses import dataclass
import ast
import inspect
import re


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


@dataclass
class Parameter:
    name: str
    default: str
    comment: str


def _parse_parameter(line: str, next_line: str) -> Parameter:
    info = next_line.strip().split("=", 1)
    return Parameter(
        name=info[0].strip(),
        default=info[1].strip(),
        comment=line.strip()[len("# PARAMETER:") :].strip(),
    )


def _emit_met_sig(kind, cls):
    assert kind in {"InputProp", "InputType", "OutputType"}

    if kind == "InputProp":
        print(f"[Prop.P_{cls.__name__}]")
    else:
        print(f"[Type.{cls.__name__}]")

    types, docs = _attribute_info(cls)

    if kind == "OutputType":
        if "path" not in types:
            raise ValueError(f"Must have attribute 'path' in Output '{cls.__name__}'")

        if types.pop("path") != "str":
            raise ValueError(
                f"Attribute 'path' must be of type str in Output '{cls.__name__}'"
            )

    if len(types) == 0:
        print("params = {}")

    for p in types:
        hb_type = _python_to_honeybee(types[p])
        print(f"params.{p} = {hb_type}")

    if cls.__doc__ is not None:
        doc_lines = cls.__doc__.splitlines()
        if len(doc_lines) >= 3 and doc_lines[1].strip() == "":
            first = doc_lines[0]
            rest = "\n".join(doc_lines[2:])
            print(f'info.title = "{first}"')
            if kind != "InputType":
                print(f'info.description = """{rest}"""')
        else:
            print(f'info.title = "{cls.__doc__}"')

    if kind != "InputType":
        for p in docs:
            print(f'info.params.{p} = "{docs[p]}"')

    if kind != "InputProp":
        code = ""
        for line in inspect.getsource(cls).splitlines():
            if line.startswith("@Input") or line.startswith("@Output"):
                line = "@dataclass"
            code += line + "\n"
        print(f"info.code = '''{code.strip()}'''")

    if kind == "InputProp":
        print()
        print(f"[Function.F_{cls.__name__}]")
        print("params = {}")
        print(f'ret = "{cls.__name__}"')
        print("condition = [")
        arg_string = ", ".join(f"{p} = ret.{p}" for p in types)
        print(f'    "P_{cls.__name__} {{ {arg_string} }}"')
        print("]")

    print()


def _emit_function_sig(f, condition, kwargs):
    print(f"[Function.{f.__name__}]")

    params = f.__annotations__.copy()

    if "return" in params:
        if params["return"] is not None:
            raise ValueError(f"Must return nothing in function '{f.__name__}'")
        params.pop("return")

    if len(params) == 0 or list(params)[-1] != "__hb_ret":
        raise ValueError(f"Need '__hb_ret' as final param in function '{f.__name__}'")

    if len(params) == 1:
        print("params = {}")

    for p in params:
        cls = params[p]

        if not p.startswith("__hb_"):
            raise ValueError(
                f"Parameter '{p}' must start with '__hb_' in function '{f.__name__}'"
            )

        p = p[len("__hb_") :]

        if not hasattr(cls, "__honeybee_type"):
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
        doc_lines = f.__doc__.splitlines()
        if len(doc_lines) >= 3 and doc_lines[1].strip() == "":
            first = doc_lines[0]
            rest = "\n".join(doc_lines[2:])
            print(f'info.title = "{first}"')
            print(f'info.description = """{rest}"""')
        else:
            print(f'info.description = """{f.__doc__}"""')

    for k in kwargs:
        if k in {"title", "description"}:
            raise ValueError(
                f"Cannot use reserved keyword '{k}' in function '{f.__name__}'"
            )
        print(f'info.{k} = "{kwargs[k]}"')

    code = ""
    found_def = False
    initial_indent = None

    hyper_parameters = {}
    it = iter(inspect.getsource(f).splitlines())
    while True:
        try:
            line = next(it)

            if line.startswith("def"):
                found_def = True
                continue

            if not found_def:
                continue

            if initial_indent is None:
                initial_indent = len(line) - len(line.lstrip())

            if line.strip().startswith("# PARAMETER:"):
                param = _parse_parameter(line, next(it))
                hyper_parameters[param.name] = param
                continue

            code += line[initial_indent:] + "\n"
        except StopIteration:
            break

    print("info.hyperparameters = [")
    for param in hyper_parameters.values():
        print(
            f"    {{name = '{param.name}', default = '{param.default}', comment = '{param.comment}' }},\n"
        )
    print("]")

    code = re.sub(
        pattern='"""(.|\n)*?"""\\s*',
        repl="",
        string=code,
        count=1,
    ).strip()

    print(f"info.code = '''{code}'''")
    print()


def Input(cls):
    _emit_met_sig("InputType", cls)
    _emit_met_sig("InputProp", cls)
    cls.__honeybee_type = True
    return cls


def Output(cls):
    _emit_met_sig("OutputType", cls)
    cls.__honeybee_type = True
    return cls


def Function(*condition, **kwargs):
    def wrap(f):
        _emit_function_sig(f, condition, kwargs)
        return f

    return wrap


helper_ran = False


def Helper(obj):
    global helper_ran
    if not helper_ran:
        imports = "from dataclasses import dataclass\n"
        with open(inspect.getsourcefile(obj), "r") as f:
            for line in f:
                line = line.strip()
                if line.startswith("import") or line.startswith("from"):
                    imports += line + "\n"
                else:
                    break

        print(f"[[Preamble]]\ncontent = '''{imports.strip()}'''\n")

        parameters = {}
        with open(inspect.getsourcefile(obj), "r") as f:
            while True:
                try:
                    it = iter(f)
                    line = next(it).strip()
                    if line.startswith("# PARAMETER:"):
                        param = _parse_parameter(line, next(it))
                        parameters[param.name] = param

                except StopIteration:
                    break

        parameters_string = ""
        for param in parameters.values():
            parameters_string += f"# PARAMETER: {param.comment} (default: {param.default})\n{param.name} = {param.default}\n\n"

        print(f"[[Preamble]]\ncontent = '''{parameters_string.strip()}'''\n")

        helper_ran = True

    code = ""
    for line in inspect.getsource(obj).splitlines()[1:]:
        code += line + "\n"
    print(f"[[Preamble]]\ncontent='''{code.strip()}'''\n")

    return obj
