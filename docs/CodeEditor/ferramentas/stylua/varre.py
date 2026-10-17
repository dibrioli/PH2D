# Size sweep (whole bob copies so each text parses), interleaved, rotated order. glibc build, our config.
# Usage: python3 -I varre.py <stylua> <config> <bob51.luau> <rounds>
import subprocess, sys, time
st, cf, bobp, rounds = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
b = open(bobp, encoding="utf-8").read().splitlines()
def juntado(n):
    return "".join((("-- " + l) if i >= len(b) and l.startswith("ph2d.property(") else l) + "\n"
                   for i, l in ((i, b[i % len(b)]) for i in range(n))).encode()
sizes = [51 * k for k in (1, 2, 4, 6, 8, 10, 12, 14, 16, 20, 30)]
txt = {n: juntado(n) for n in sizes}
res = {n: [] for n in sizes}
for r in range(rounds + 3):
    k = r % len(sizes); order = sizes[k:] + sizes[:k]
    for n in order:
        t0 = time.perf_counter_ns()
        p = subprocess.run([st, "--config-path", cf, "-"], input=txt[n], capture_output=True)
        dt = (time.perf_counter_ns() - t0) / 1e6
        assert p.returncode == 0, (n, p.stderr[:200])
        if r >= 3: res[n].append(dt)
def q(xs, p):
    xs = sorted(xs); return xs[min(len(xs) - 1, int(round(p * (len(xs) - 1))))]
print(f"{'lines':>6} {'n':>4} {'min':>7} {'p50':>7} {'p99':>7}  (ms)")
for n in sizes:
    xs = res[n]; print(f"{n:6} {len(xs):4} {min(xs):7.2f} {q(xs,.5):7.2f} {q(xs,.99):7.2f}")
