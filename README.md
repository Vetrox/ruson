# Sea of Nodes - Simple

This is a rust implementation of the Sea of Nodes - Simple example.

# Sea of Nodes

Cliff Click introduced the C2 HotSpot compiler. It was the first of a kind JIT compiler and to this day it is still the core idea behind
the [HotSpot JIT](https://github.com/openjdk/jdk/tree/master/src/hotspot/share/opto)

# Optimization techniques

| ID          | Short explanation             |
|-------------|-------------------------------|
| T_CONSTFLD  | `Typ(#1+#2)=Typ(#3)`          |
| T_CONSTPROP | Infer types through def edges |