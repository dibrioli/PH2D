# Interleaved timing of StyLua over stdin: each round runs every variant once, in a rotated order.
# Wall time = Popen() .. communicate() returns (spawn + read stdin + format + write stdout + exit).
# Usage: python3 -I cronometra.py <stylua-glibc> <stylua-musl> <config> <inputs-dir> <rounds>
import subprocess, sys, time, os
gl, mu, cf, ind, rounds = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5])
txt = {n: open(f"{ind}/{n}.luau", "rb").read() for n in ("bob51", "g3009", "g7500")}
ref = {}
V = [("true", ["/usr/bin/true"], None)]
for tag, b in (("glibc", gl), ("musl", mu)):
    V.append((f"{tag}:--version", [b, "--version"], None))
    for n in txt:
        V.append((f"{tag}:{n}", [b, "--config-path", cf, "-"], n))
for n in ("bob51", "g7500"):
    V.append((f"glibc:{n}+verify", [gl, "--config-path", cf, "--verify", "-"], n))
for n in ("bob51", "g3009", "g7500"):
    V.append((f"glibc:{n}+threads1", [gl, "--config-path", cf, "--num-threads", "1", "-"], n))
V.append(("glibc:--version+threads1", [gl, "--num-threads", "1", "--version"], None))
res = {v[0]: [] for v in V}
for r in range(rounds + 3):
    k = r % len(V)
    for name, argv, inp in V[k:] + V[:k]:
        data = txt[inp] if inp else None
        t0 = time.perf_counter_ns()
        p = subprocess.Popen(argv, stdin=subprocess.PIPE if inp else subprocess.DEVNULL,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        out, err = p.communicate(data)
        dt = (time.perf_counter_ns() - t0) / 1e6
        assert p.returncode == 0, (name, p.returncode, err[:200])
        if inp:
            ref.setdefault(inp, out)
            assert out == ref[inp], f"{name}: output differs between runs/builds"
        if r >= 3:  # 3 warm-up rounds discarded
            res[name].append(dt)
def q(xs, p):
    xs = sorted(xs); return xs[min(len(xs) - 1, int(round(p * (len(xs) - 1))))]
print(f"{'variant':28} {'n':>4} {'min':>8} {'p50':>8} {'p99':>8} {'max':>8}  (ms)")
for name, *_ in V:
    xs = res[name]
    print(f"{name:28} {len(xs):4} {min(xs):8.2f} {q(xs,.5):8.2f} {q(xs,.99):8.2f} {max(xs):8.2f}")
for n, o in ref.items():
    print(f"out {n}: {o.count(b'\n')} lines, {len(o)} bytes")
