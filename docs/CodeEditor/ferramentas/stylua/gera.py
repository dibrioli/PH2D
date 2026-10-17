# Builds the I0d inputs exactly like crates/ph2d-app-code/tests/it/bancada.rs: bob() = bob.luau minus its
# first 2 lines (51 lines); juntado(n) = bob cycled to n lines, with the 2nd copy onward's `ph2d.property(`
# lines commented out. 7 500 = 147 copies + 3 comment lines (parses); ~3 000 is 3 009 = 59 whole copies,
# because 3 000 cuts a copy inside `update` and StyLua refuses it (rc 2, «expected `end`»).
# Usage: python3 -I gera.py <bob.luau> <out-dir>
import sys
fx, out = sys.argv[1], sys.argv[2]
b = open(fx, encoding="utf-8").read().splitlines()[2:]
def juntado(n):
    return "".join((("-- " + l) if i >= len(b) and l.startswith("ph2d.property(") else l) + "\n"
                   for i, l in ((i, b[i % len(b)]) for i in range(n)))
for name, n in (("bob51", 51), ("g3009", 3009), ("g7500", 7500)):
    open(f"{out}/{name}.luau", "w", encoding="utf-8").write(juntado(n))
