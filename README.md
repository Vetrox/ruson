# Sea of Nodes - Simple

This is a rust implementation of the Sea of Nodes - Simple example.

# Sea of Nodes

Cliff Click introduced the C2 HotSpot compiler. It was the first of a kind JIT compiler and to this day it is still the core idea behind
the [HotSpot JIT](https://github.com/openjdk/jdk/tree/master/src/hotspot/share/opto)

# Useful resources

[Modern Compiler Implementation in Java](https://dx.doi.org/10.1017/CBO9780511811432)

# Lattices

$\bot \leq \mathcal{Int} \leq \top$

$\mathrm{join}(a, b) = \mathrm{min} \left \lbrace e: a\leq e \wedge b \leq e \right \rbrace$

$\mathrm{meet}(a, b) = \mathrm{max} \left \lbrace e: e \leq a \wedge e \leq b \right \rbrace$

$\mathrm{isA}(a, b) \Leftrightarrow \mathrm{meet}(a,b) = b$: Describes whether `a` is a subtype of `b` $\equiv$ `b` is a supertype of `a`.

## Examples

$\mathrm{join}(\bot, \top) = \mathrm{min} \left \lbrace e: \bot \leq e \wedge \top \leq e \right \rbrace = \top$

$\mathrm{meet}(\bot, \top) = \mathrm{max} \left \lbrace e: e \leq \bot \wedge e \leq \top \right \rbrace = \bot$

# Intuition

$\bot$ is treated as an unknown value. We have to honor what the programmer wrote and emit the code.

$\top \equiv $ all possible choices (constants). We can choose any value, as convenient.

# Optimization techniques

| ID          | Short explanation             |
|-------------|-------------------------------|
| T_CONSTFLD  | `Typ(#1+#2)=Typ(#3)`          |
| T_CONSTPROP | Infer types through def edges |