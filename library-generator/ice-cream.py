# uv run ../library-generator/ice-cream.py => ../examples/gen-dessert/_suite-ice-cream.hblib.toml
# ./target/debug/honeybee interact --library ../examples/gen-dessert/_suite-ice-cream.hblib.toml --program ../examples/gen-dessert/gen_dessert.hb.toml


from dataclasses import dataclass

from lib import Function, Helper, Prop, Type


@Helper
def RUN(command):
    import subprocess

    return subprocess.run(
        command,
        shell=True,
        capture_output=True,
        text=True,
    ).stdout.split()


@Helper
def sample_names(filename):
    sn = RUN(f"cat {filename} | cut -d ',' -f1 | tail -n +2")
    return sn


################################################################################

@Prop
class BlizzardMachine:
    "Blizzard machine status"

    working: int
    "0-machine broken, 1-machine working"

@Prop
class LactoseIntolerant:
    "Allergen information"

    intolerant: int
    "0-lactose allowed, 1-lactose intolerant"

@Prop
class AvailableFlavor:
    "Ice-cream flavors"

    flavor: int
    "0-vanilla, 1-chocolate, 2-strawberry"

@Prop
class AvailableTopping:
    "Topping Varieties"

    topping: int
    "0-chocolate bar, 1-cookies, 2-fruit, 3-chocolate sauce, 4-whipped cream, 5-none"

@Type(var_name="MILK")
class Milk:
    "Milk"

    @dataclass
    class S:
        nonDairy: int
        "0-dairy, 1-coconut"

    @dataclass
    class D:
        pass

@Type(var_name="ICE")
class Ice:
    "Ice"

    @dataclass
    class S:
        pass

    @dataclass
    class D:
        pass

@Type(var_name="ICE_CREAM")
class IceCream:
    "Ice cream"

    @dataclass
    class S:
        flavor: int
        "0-vanilla, 1-chocolate, 2-strawberry"

    @dataclass
    class D:
        pass

@Type(var_name="MIX_IN_TOPPING")
class MixInTopping:
    "Mix-in/topping"

    @dataclass
    class S:
        kind: int
        "0-chocolate bar, 1-cookies, 2-fruit, 3-chocolate sauce, 4-whipped cream, 5-none"

    @dataclass
    class D:
        pass

@Type(var_name="FROZEN_DESSERT")
class FrozenDessert:
    "Frozen dessert"

    @dataclass
    class S:
        pass

    @dataclass
    class D:
        pass

@Function(
    "LactoseIntolerant { intolerant = 0 }",
    "ret.nonDairy = 0"
)
def get_dairy_milk(ret: Milk.S) -> Milk.D:
    """Get dairy milk"""
    return Milk.D()

@Function(
    "ret.nonDairy = 1"
)
def get_coconut_milk(ret: Milk.S) -> Milk.D:
    """Get coconut milk"""
    return Milk.D()

@Function()
def get_ice(ret: Ice.S) -> Ice.D:
    """Get ice"""
    return Ice.D()

@Function(
    "AvailableFlavor { flavor = ret.flavor }"
)
def get_ice_cream(milk: Milk, ice: Ice, ret: IceCream.S) -> IceCream.D:
    "Get ice cream"
    return IceCream.D

@Function(
    "AvailableTopping { topping = ret.kind }",
)
def get_mixin_topping(ret: MixInTopping.S) -> MixInTopping.D:
    "Get mix-in/topping"
    return MixInTopping.D

@Function()
def scoop(base: IceCream, topping: MixInTopping, ret: FrozenDessert.S) -> FrozenDessert.D:
    "Scoop of ice cream with one topping"
    return FrozenDessert.D

@Function(
    "BlizzardMachine { working = 1 }",
    "mixin.kind < 3",
)
def blizzard(base: IceCream, mixin: MixInTopping, ret: FrozenDessert.S) -> FrozenDessert.D:
    "Blizzard with one mix-in"
    return FrozenDessert.D
