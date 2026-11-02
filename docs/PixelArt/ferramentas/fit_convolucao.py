#!/usr/bin/env python3
"""Recupera as matrizes da Convolution Matrix a partir das respostas do Trial (offline).

Lei (rodada 2): out(q) = clamp(trunc0(sum_d w[d] * in(clamp(q - d))) / div) + bias), borda replicada.
`w[d]` = peso que um impulso em p põe na célula p + d (mapa de RESPOSTA, dx/dy).

Entradas gravadas em oracle_trial_convolution.json:
  r1 imp  : preto, 255 em (7,7) e (0,0)      r1 edge : x<7 a 255, resto 0
  r2 impg : 128, 255 em (7,7)                 r2 imp0 : 128, 0 em (7,7)     r2 stepg : x<7 a 64, resto 192

Uso: python3 -I fit_convolucao.py <fixture.json> [--write <saida.json>]
"""
import itertools
import json
import sys

N = 15
FIXTURE = "crates/ph2d-pixel-core/tests/fixtures/oracle_trial_convolution.json"


def grid(rows):
    return [[int(v) for v in r.split()] for r in rows]


def inputs():
    imp = [[0] * N for _ in range(N)]
    imp[7][7] = imp[0][0] = 255
    edge = [[255 if x < 7 else 0 for x in range(N)] for _ in range(N)]
    impg = [[128] * N for _ in range(N)]
    impg[7][7] = 255
    imp0 = [[128] * N for _ in range(N)]
    imp0[7][7] = 0
    stepg = [[64 if x < 7 else 192 for x in range(N)] for _ in range(N)]
    return {"imp": imp, "edge": edge, "impg": impg, "imp0": imp0, "stepg": stepg}


def trunc0(a, b):
    q = abs(a) // abs(b)
    return q if (a >= 0) == (b > 0) else -q


def conv(src, w, div, bias):
    cl = lambda v: 0 if v < 0 else (N - 1 if v > N - 1 else v)
    out = [[0] * N for _ in range(N)]
    for y in range(N):
        for x in range(N):
            s = 0
            for (dx, dy), k in w.items():
                s += k * src[cl(y - dy)][cl(x - dx)]
            v = trunc0(s, div) + bias
            out[y][x] = max(0, min(255, v))
    return out


def cases(fx, name):
    r1 = next(k for k in fx["kernels"] if k["name"] == name)
    r2 = next(k for k in fx["round2_gray"]["kernels"] if k["name"] == name)
    got = {"imp": grid(r1["imp"]), "edge": grid(r1["edge"])}
    for f in ("impg", "imp0", "stepg"):
        if f in r2:
            got[f] = grid(r2[f])
    return got


def misses(got, w, div, bias):
    src = inputs()
    n = 0
    for f, g in got.items():
        o = conv(src[f], w, div, bias)
        n += sum(o[y][x] != g[y][x] for y in range(N) for x in range(N))
    return n


def fit_positive(got, radius):
    """Pesos >= 0: lê-se a resposta do impg (fundo 128 => div = soma) e do imp (fundo 0)."""
    g = got["impg"]
    best = None
    for div in range(1, 4097):
        w = {}
        for dy in range(-radius, radius + 1):
            for dx in range(-radius, radius + 1):
                y, x = 7 + dy, 7 + dx
                if not (0 <= x < N and 0 <= y < N):
                    continue
                r = g[y][x] - 128
                k = round(r * div / 127)
                if k:
                    w[(dx, dy)] = k
        if not w or sum(w.values()) != div:
            continue
        m = misses(got, w, div, 0)
        if best is None or m < best[0]:
            best = (m, w, div)
        if m == 0:
            break
    return best


def fit_intervals(got, radius, smax=20000):
    """Pesos >= 0, div = soma S: cada peso w tem 3 leituras no impulso central —
    r1 imp `⌊255w/S⌋`, impg `128+⌊127w/S⌋`, imp0 `128-⌈128w/S⌉` — que o prendem num intervalo."""
    reads = []
    for dy in range(-radius, radius + 1):
        for dx in range(-radius, radius + 1):
            y, x = 7 + dy, 7 + dx
            obs = [] if (x <= radius and y <= radius) else [(255, got["imp"][y][x], 0)]
            if "impg" in got:
                obs.append((127, got["impg"][y][x] - 128, 0))
            if "imp0" in got:
                obs.append((128, 128 - got["imp0"][y][x], 1))
            reads.append(((dx, dy), obs))
    for s in range(1, smax):
        lo_all, hi_all, ok = {}, {}, True
        for d, obs in reads:
            lo, hi = 0, s
            for m, r, ceil in obs:
                # floor(m w / s) = r  <=>  r s <= m w < (r+1) s ; ceil: (r-1) s < m w <= r s
                if ceil:
                    lo = max(lo, ((r - 1) * s) // m + 1)
                    hi = min(hi, (r * s) // m)
                else:
                    lo = max(lo, -((-r * s) // m))
                    hi = min(hi, ((r + 1) * s - 1) // m)
            if lo > hi:
                ok = False
                break
            lo_all[d], hi_all[d] = lo, hi
        if not ok or not (sum(lo_all.values()) <= s <= sum(hi_all.values())):
            continue
        free = {d for d in lo_all if lo_all[d] < hi_all[d]}
        if len(free) == 0:
            w = {d: v for d, v in lo_all.items() if v}
            if misses(got, w, s, 0) == 0:
                return (0, w, s)
            continue
        # sobra: distribui pela simetria do mapa (os livres simétricos ficam iguais)
        need = s - sum(lo_all.values())
        orbits = {}
        for (dx, dy) in free:
            key = tuple(sorted({(abs(dx), abs(dy)), (abs(dy), abs(dx))}))
            orbits.setdefault(key, []).append((dx, dy))
        for pick in itertools.product([0, 1], repeat=len(orbits)):
            obs = list(orbits.values())
            if sum(len(o) for o, p in zip(obs, pick) if p) != need:
                continue
            w = dict(lo_all)
            for o, p in zip(obs, pick):
                for d in o:
                    w[d] += p
            w = {d: v for d, v in w.items() if v}
            if misses(got, w, s, 0) == 0:
                return (0, w, s)
    return None


def main():
    fx = json.load(open(sys.argv[1]))
    fitted = fx["law_measured_round2"]["fitted"]
    out = {}
    for k in fx["kernels"]:
        name = k["name"]
        got = cases(fx, name)
        if name in fitted and "response_weights_dx_dy" in fitted[name]:
            f = fitted[name]
            w = {tuple(int(c) for c in key.split(",")): v for key, v in f["response_weights_dx_dy"].items()}
            res = (misses(got, w, f["div"], f["bias"]), w, f["div"], f["bias"])
        elif name.startswith(("blur", "drunk")):
            b = fit_positive(got, 7)
            if b is None or b[0]:
                import re
                n = max(int(v) for v in re.findall(r"(\d+)x(\d+)", name)[0])
                b = fit_intervals(got, min(7, n // 2)) or b
            res = (b[0], b[1], b[2], 0) if b else None
        elif name.startswith("outline-transparent"):
            res = (misses(got, {(0, 0): 1}, 1, 0), {(0, 0): 1}, 1, 0)
        else:
            res = None
        if res is None:
            print(f"{name:28s} RECUSA (não recuperável)")
            continue
        m, w, div, bias = res
        cells = sum(len(v) for v in got.values()) * N
        print(f"{name:28s} desvios {m:4d}/{cells}  div {div:4d} bias {bias:4d}  n_pesos {len(w)}")
        out[name] = {"misses": m, "div": div, "bias": bias,
                     "w": {f"{dx},{dy}": v for (dx, dy), v in sorted(w.items(), key=lambda t: (t[0][1], t[0][0]))}}
    if "--write" in sys.argv:
        json.dump(out, open(sys.argv[sys.argv.index("--write") + 1], "w"), indent=1)




def rust_table(fit_json, fit_r3_json, out_rs):
    """Os kernels de 0 desvios como dados Rust (`filter/kernels.rs`), na ordem da lista do Trial; os da
    rodada 3 (`fit_convolucao_r3.py`) completam os que as rodadas 1–2 não fixaram."""
    d = json.load(open(fit_json))
    for name, e in json.load(open(fit_r3_json)).items():
        d[name] = dict(e, misses=0)
    order = [k["name"] for k in json.load(open(FIXTURE))["kernels"]]
    d = {n: d[n] for n in order if n in d}
    lines = [
        "//! Os kernels da Convolution Matrix — DADOS NOSSOS com o nome medido: cada matriz foi",
        "//! recuperada da resposta do Trial (`docs/PixelArt/ferramentas/fit_convolucao.py` sobre",
        "//! `tests/fixtures/oracle_trial_convolution.json` e `fit_convolucao_r3.py` sobre",
        "//! `oracle_trial_round3.json`, 0 células fora). Gerado; não editar à mão.",
        "",
        "use super::convolution::Kernel;",
        "",
        "#[rustfmt::skip]",
        "pub const KERNELS: &[Kernel] = &[",
    ]
    for name, e in d.items():
        if e["misses"] or name.startswith("outline-transparent"):
            continue
        w = {tuple(int(c) for c in k.split(",")): v for k, v in e["w"].items()}
        x0, x1 = min(p[0] for p in w), max(p[0] for p in w)
        y0, y1 = min(p[1] for p in w), max(p[1] for p in w)
        ws = [w.get((x, y), 0) for y in range(y0, y1 + 1) for x in range(x0, x1 + 1)]
        lines.append(
            f'    Kernel {{ name: {json.dumps(name)}, w: {x1 - x0 + 1}, h: {y1 - y0 + 1}, cx: {-x0}, cy: {-y0}, '
            f'div: {e["div"]}, bias: {e["bias"]}, weights: &{ws} }},'
        )
    lines.append("];")
    open(out_rs, "w").write("\n".join(lines) + "\n")


if __name__ == "__main__":
    if sys.argv[1] == "--rust":
        rust_table(sys.argv[2], sys.argv[3], sys.argv[4])
    else:
        main()
