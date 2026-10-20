#!/usr/bin/env python3
"""W23 — a lei do VÃO do legado {focus, gap} no 0.18.1 no losango (vivo e redondo) e no rectângulo redondo,
das saídas saidas/w23_V_*.0181.json (entrada entradas/w23_vao_casos.json, gerada por gera_w23_vao_casos.py).

Para cada seta presa: A = o ponto vizinho, T = a ponta depois de o editor a re-ligar (passo `empurrar`), no
referencial LOCAL da forma (centro da CAIXA na origem, sem rotação). Cada hipótese de «forma alargada de g»
é um contorno fechado (polilinha fina); duas réguas:
  recta  — a 1.ª entrada da recta OBSERVADA A→T no contorno candidato, contra T (só o vão);
  lei    — a recta da lei XV7 (A → |focus|·K, K o vértice do losango / o canto da caixa do lado do sinal),
           1.ª entrada no contorno (gap 0 ⇒ o próprio ponto do foco; sem entrada ⇒ a ponta não mexe),
           contra T (o foco E o vão).
Uso: python3 mede_w23_vao.py
"""
import json
import math

import numpy as np
import statistics
import sys
from pathlib import Path

H = Path(__file__).resolve().parent
S = H / "saidas"
ENTRADA = H / "entradas" / "w23_vao_casos.json"
N_CURVA = 400  # pontos por canto (resolução ≪ 0,001 px nos tamanhos medidos)


def local(p, c, ang):
    x, y = p[0] - c[0], p[1] - c[1]
    co, si = math.cos(-ang), math.sin(-ang)
    return (x * co - y * si, x * si + y * co)


def raio(x, rnd):
    """getCornerRadius medido: type 2 = x/4; type 3 = x/4 até x = 128, depois 32."""
    if rnd["type"] == 3 and x > 128:
        return 32.0
    return 0.25 * x


def quad(a, v, b, n=N_CURVA):
    return [((1 - t) ** 2 * a[0] + 2 * t * (1 - t) * v[0] + t * t * b[0],
             (1 - t) ** 2 * a[1] + 2 * t * (1 - t) * v[1] + t * t * b[1]) for t in (i / n for i in range(n + 1))]


def cubica_vv(a, v, b, n=N_CURVA):
    """Cúbica com os DOIS controlos no vértice."""
    return [((1 - t) ** 3 * a[0] + 3 * t * (1 - t) * v[0] + t ** 3 * b[0],
             (1 - t) ** 3 * a[1] + 3 * t * (1 - t) * v[1] + t ** 3 * b[1]) for t in (i / n for i in range(n + 1))]


def fecha(p):
    return p + p[:1]


def offset_normal(poly, g):
    """Minkowski com um disco de raio g (convexo, sentido horário ou anti-horário): cada ponto pela normal,
    com tampa redonda nos vértices vivos (g > 0)."""
    pts = poly[:-1]
    n = len(pts)
    area = sum(pts[i][0] * pts[(i + 1) % n][1] - pts[(i + 1) % n][0] * pts[i][1] for i in range(n))
    sgn = 1 if area > 0 else -1
    normais = []
    for i in range(n):
        a, b = pts[i], pts[(i + 1) % n]
        dx, dy = b[0] - a[0], b[1] - a[1]
        L = math.hypot(dx, dy) or 1
        normais.append((sgn * dy / L, -sgn * dx / L))
    out = []
    for i in range(n):
        n0, n1 = normais[i - 1], normais[i]
        p = pts[i]
        a0, a1 = math.atan2(n0[1], n0[0]), math.atan2(n1[1], n1[0])
        da = (a1 - a0 + math.pi) % (2 * math.pi) - math.pi
        if g > 0 and abs(da) > 1e-3:
            k = max(2, int(abs(da) / 0.002))
            out += [(p[0] + g * math.cos(a0 + da * j / k), p[1] + g * math.sin(a0 + da * j / k)) for j in range(k + 1)]
        else:
            m = (n0[0] + n1[0], n0[1] + n1[1])
            ml = math.hypot(*m)
            cosh = (m[0] * n1[0] + m[1] * n1[1]) / ml
            out.append((p[0] + g * m[0] / ml / cosh, p[1] + g * m[1] / ml / cosh))
    return fecha(out)


# ---------------------------------------------------------------- os candidatos
def losango_vertices(hw, hh, ox=0.0, oy=0.0, ev=0.0, eh=0.0):
    return [(ox, -hh - ev), (hw + eh, oy), (ox, hh + ev), (-hw - eh, oy)]


def floor1(w, h):
    return math.floor(w / 2) + 1 - w / 2, math.floor(h / 2) + 1 - h / 2


def losango_redondo(w, h, g, rnd, curva, floor=True, raios_novos=False, ox_oy=None, radial=False):
    """O caminho do losango redondo (vértices em floor(w/2)+1, floor(h/2)+1; raios R(floor(w/2)+1) na
    horizontal e R(floor(h/2)+1) na vertical; arestas entre as pontas dos cantos), cada canto transladado
    de g no eixo (`radial` False) ou de g·û(V), V o vértice a partir do centro da caixa (`radial` True)."""
    hw, hh = w / 2, h / 2
    ox, oy = ox_oy if ox_oy else (floor1(w, h) if floor else (0.0, 0.0))
    T, R, B, L = losango_vertices(hw, hh, ox, oy)
    if raios_novos:
        vr, hr = raio(hw + g + ox, rnd), raio(hh + g + oy, rnd)
    else:
        vr, hr = raio(hw + ox, rnd), raio(hh + oy, rnd)
    cantos = [  # (a, v, b, translação)
        ((T[0] - vr, T[1] + hr), T, (T[0] + vr, T[1] + hr), (0, -g)),
        ((R[0] - vr, R[1] - hr), R, (R[0] - vr, R[1] + hr), (g, 0)),
        ((B[0] + vr, B[1] - hr), B, (B[0] - vr, B[1] - hr), (0, g)),
        ((L[0] + vr, L[1] + hr), L, (L[0] + vr, L[1] - hr), (-g, 0)),
    ]
    out = []
    for a, v, b, (tx, ty) in cantos:
        if radial:
            n = math.hypot(*v)
            tx, ty = g * v[0] / n, g * v[1] / n
        out += [(p[0] + tx, p[1] + ty) for p in curva(a, v, b)]
    return fecha(out)


def rect_redondo(hx, hy, r, curva, t=(0.0, 0.0)):
    """Rectângulo de meia-caixa (hx, hy), canto de r; cada canto transladado de (±t[0], ±t[1])."""
    tx, ty = t
    out = []
    for sx, sy in ((1, -1), (1, 1), (-1, 1), (-1, -1)):
        v = (sx * hx, sy * hy)
        if sx * sy < 0:
            a, b = (v[0] - sx * r, v[1]), (v[0], v[1] - sy * r)
        else:
            a, b = (v[0], v[1] - sy * r), (v[0] - sx * r, v[1])
        out += [(p[0] + sx * tx, p[1] + sy * ty) for p in curva(a, v, b)]
    return fecha(out)


def candidatos(F, g):
    w, h = F["width"], F["height"]
    hw, hh = w / 2, h / 2
    rnd = F["roundness"]
    L = math.hypot(hw, hh)
    c = {}
    if F["type"] == "diamond" and rnd is None:
        base = fecha(losango_vertices(hw, hh))
        c["a Minkowski (disco g)"] = offset_normal(base, g)
        c["b semi-diagonais + g"] = fecha(losango_vertices(hw, hh, ev=g, eh=g))
        c["c mitra (arestas a g)"] = fecha(losango_vertices(hw, hh, ev=g * L / hw, eh=g * L / hh))
        ox, oy = floor1(w, h)
        c["d semi-diag + g, vértice floor+1"] = fecha(losango_vertices(hw, hh, ox, oy, ev=g, eh=g))
    elif F["type"] == "diamond":
        nosso = losango_redondo(w, h, 0.0, rnd, quad, floor=False)
        c["a Minkowski do redondo"] = offset_normal(nosso, g)
        ox, oy = floor1(w, h)
        c["b vivo (lei d do vivo)"] = fecha(losango_vertices(hw, hh, ox, oy, ev=g, eh=g))
        c["c cantos + g no eixo, cúbica"] = losango_redondo(w, h, g, rnd, cubica_vv)
        c["e raios do alargado, cúbica"] = losango_redondo(w, h, g, rnd, cubica_vv, raios_novos=True)
        c["f cantos + g·û(V), quad"] = losango_redondo(w, h, g, rnd, quad, radial=True)
        c["f' cantos + g·û(V), cúbica"] = losango_redondo(w, h, g, rnd, cubica_vv, radial=True)
    elif rnd is None:  # o controlo: a lei exacta da W10
        c["b caixa de Chebyshev + g"] = fecha([(hw + g, -hh - g), (hw + g, hh + g), (-hw - g, hh + g), (-hw - g, -hh - g)])
    else:
        r = raio(min(w, h), rnd)
        nosso = rect_redondo(hw, hh, r, quad)
        c["a Minkowski do redondo"] = offset_normal(nosso, g)
        c["b caixa de Chebyshev + g"] = fecha([(hw + g, -hh - g), (hw + g, hh + g), (-hw - g, hh + g), (-hw - g, -hh - g)])
        c["c lados + g, mesmo r"] = rect_redondo(hw + g, hh + g, r, quad)
        n = math.hypot(hw + g, hh + g)
        u = (g * (hw + g) / n, g * (hh + g) / n)
        c["d cantos + g·û(hw+g,hh+g), quad"] = rect_redondo(hw, hh, r, quad, u)
        c["d' idem, cúbica"] = rect_redondo(hw, hh, r, cubica_vv, u)
        c["e lados + g, r do alargado"] = rect_redondo(hw + g, hh + g, raio(min(w, h) + 2 * g, rnd), quad)
    return {k: np.asarray(v) for k, v in c.items()}


# ---------------------------------------------------------------- a recta e o contorno
def entrada(a, d, poly):
    """O menor t ≥ 0 com a + t·d no contorno (a 1.ª entrada vindo de A)."""
    P = poly if isinstance(poly, np.ndarray) else np.asarray(poly)
    p, q = P[:-1], P[1:]
    ex, ey = q[:, 0] - p[:, 0], q[:, 1] - p[:, 1]
    den = d[0] * ey - d[1] * ex
    ok = np.abs(den) > 1e-14
    den = np.where(ok, den, 1.0)
    wx, wy = p[:, 0] - a[0], p[:, 1] - a[1]
    t = (wx * ey - wy * ex) / den
    s = (wx * d[1] - wy * d[0]) / den
    m = ok & (s >= -1e-9) & (s <= 1 + 1e-9) & (t >= -1e-9)
    if not m.any():
        return None
    tb = float(t[m].min())
    return (a[0] + tb * d[0], a[1] + tb * d[1])


def sd(a, q):
    u = (q[0] - a[0], q[1] - a[1])
    return (u[0] * -a[1] - u[1] * -a[0]) / (math.hypot(*u) or 1e-300)


def mira(F, a, focus):
    """XV7: a recta passa por |focus|·K, K o vértice do losango (o canto da caixa) que mais se afasta do
    centro do lado do sinal."""
    if focus == 0:
        return (0.0, 0.0)
    hw, hh = F["width"] / 2, F["height"] / 2
    ks = [(0, -hh), (hw, 0), (0, hh), (-hw, 0)] if F["type"] == "diamond" else \
        [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)]
    r, o = abs(focus), math.copysign(1, focus)
    best, most = None, 0.0
    for k in ks:
        q = (k[0] * r, k[1] * r)
        s = o * sd(a, q)
        if s > most:
            most, best = s, q
    return best


def support(F, n):
    hw, hh = F["width"] / 2, F["height"] / 2
    nx, ny = abs(n[0]), abs(n[1])
    return max(hw * nx, hh * ny) if F["type"] == "diamond" else hw * nx + hh * ny


def dist_poly(p, poly):
    best = math.inf
    for (ax, ay), (bx, by) in zip(poly, poly[1:]):
        dx, dy = bx - ax, by - ay
        L2 = dx * dx + dy * dy
        t = 0 if L2 == 0 else max(0, min(1, ((p[0] - ax) * dx + (p[1] - ay) * dy) / L2))
        best = min(best, math.hypot(p[0] - ax - t * dx, p[1] - ay - t * dy))
    return best


# ---------------------------------------------------------------- a LEI (peças analíticas)
def pecas(F, g):
    """As peças do contorno alargado de g, no referencial local: (rectas, cúbicas).
    rectângulo vivo: caixa + g · losango vivo: vértices em floor(w/2)+1 / floor(h/2)+1, cada um + g no SEU eixo ·
    rectângulo redondo: cantos (quadrática, controlo no vértice, elevada a cúbica) transladados de
    ±g·û(hw+g, hh+g) · losango redondo: cantos (cúbica com os dois controlos no vértice) transladados de g·û(V)."""
    w, h = F["width"], F["height"]
    hw, hh = w / 2, h / 2
    rnd = F["roundness"]
    if rnd is None:
        if F["type"] == "rectangle":
            v = [(hw + g, -hh - g), (hw + g, hh + g), (-hw - g, hh + g), (-hw - g, -hh - g)]
        else:
            ox, oy = floor1(w, h)
            v = losango_vertices(hw, hh, ox, oy, ev=g, eh=g)
        return [(v[i], v[(i + 1) % 4]) for i in range(4)], []
    cs = []
    if F["type"] == "rectangle":
        r = raio(min(w, h), rnd)
        n = math.hypot(hw + g, hh + g)
        u = (g * (hw + g) / n, g * (hh + g) / n)
        for sx, sy in ((1, -1), (1, 1), (-1, 1), (-1, -1)):
            v = (sx * hw, sy * hh)
            if sx * sy < 0:
                a, b = (v[0] - sx * r, v[1]), (v[0], v[1] - sy * r)
            else:
                a, b = (v[0], v[1] - sy * r), (v[0] - sx * r, v[1])
            t = (sx * u[0], sy * u[1])
            a, v, b = [(q[0] + t[0], q[1] + t[1]) for q in (a, v, b)]
            cs.append((a, (a[0] + 2 / 3 * (v[0] - a[0]), a[1] + 2 / 3 * (v[1] - a[1])),
                       (b[0] + 2 / 3 * (v[0] - b[0]), b[1] + 2 / 3 * (v[1] - b[1])), b))
    else:
        ox, oy = floor1(w, h)
        T, R, B, L = losango_vertices(hw, hh, ox, oy)
        vr, hr = raio(hw + ox, rnd), raio(hh + oy, rnd)
        for a, v, b in (((T[0] - vr, T[1] + hr), T, (T[0] + vr, T[1] + hr)),
                        ((R[0] - vr, R[1] - hr), R, (R[0] - vr, R[1] + hr)),
                        ((B[0] + vr, B[1] - hr), B, (B[0] - vr, B[1] - hr)),
                        ((L[0] + vr, L[1] + hr), L, (L[0] + vr, L[1] - hr))):
            nv = math.hypot(*v)
            t = (g * v[0] / nv, g * v[1] / nv)
            a, v, b = [(q[0] + t[0], q[1] + t[1]) for q in (a, v, b)]
            cs.append((a, v, v, b))
    return [(cs[i][3], cs[(i + 1) % 4][0]) for i in range(4)], cs


def bez(c, t):
    m = 1 - t
    return tuple(m ** 3 * c[0][k] + 3 * m * m * t * c[1][k] + 3 * m * t * t * c[2][k] + t ** 3 * c[3][k] for k in (0, 1))


def dbez(c, t):
    m = 1 - t
    return tuple(3 * m * m * (c[1][k] - c[0][k]) + 6 * m * t * (c[2][k] - c[1][k]) + 3 * t * t * (c[3][k] - c[2][k])
                 for k in (0, 1))


def raiz_curva(c, P, Q, sementes=(0.5, 0.2, 0.8), tol=1e-3, iters=10):
    """UM ponto de cruzamento da cúbica com o segmento P→Q (Newton em (t, s) a partir de cada semente;
    aceita t, s ∈ [0, 1]) ou None. Medido: a contagem não depende do pormenor do Newton (sementes densas
    e tolerância 1e-9 dão o mesmo veredito) — o que conta é «uma raiz por curva»."""
    d = (Q[0] - P[0], Q[1] - P[1])
    for t0 in sementes:
        t, s = t0, 0.0
        ok = False
        for _ in range(iters):
            b = bez(c, t)
            f = (b[0] - P[0] - s * d[0], b[1] - P[1] - s * d[1])
            db = dbez(c, t)
            det = db[0] * -d[1] - -d[0] * db[1]
            if det == 0:
                break
            t += (-f[0] * -d[1] + f[1] * -d[0]) / det
            s += (-db[0] * f[1] + db[1] * f[0]) / det
            b = bez(c, t)
            if abs(b[0] - P[0] - s * d[0]) + abs(b[1] - P[1] - s * d[1]) < tol:
                ok = True
                break
        if ok and 0 <= t <= 1 and 0 <= s <= 1:
            return bez(c, t)
    return None


def corte_recta(a, b, P, Q):
    d = (Q[0] - P[0], Q[1] - P[1])
    e = (b[0] - a[0], b[1] - a[1])
    den = d[0] * e[1] - d[1] * e[0]
    if abs(den) < 1e-12:
        return None
    w_ = (a[0] - P[0], a[1] - P[1])
    s = (w_[0] * e[1] - w_[1] * e[0]) / den
    u = (w_[0] * d[1] - w_[1] * d[0]) / den
    return (P[0] + s * d[0], P[1] + s * d[1]) if 0 <= s <= 1 and 0 <= u <= 1 else None


def religa(F, A, ponta, focus, gap):
    """UMA re-ligação (local): a recta A → ponto do foco, como segmento LONGO; cada peça dá no máximo UM
    ponto; ≥ 2 pontos ⇒ o mais perto de A; 1 ⇒ o ponto do foco; 0 ⇒ a ponta não mexe. gap 0 ⇒ o foco."""
    m = mira(F, A, focus)
    if m is None:
        return ponta
    if gap == 0:
        return m
    rectas, curvas = pecas(F, gap)
    dx, dy = m[0] - A[0], m[1] - A[1]
    L = math.hypot(dx, dy)
    comp = math.hypot(A[0] - ponta[0], A[1] - ponta[1]) + math.hypot(*A) + 2 * max(F["width"], F["height"])
    Q = (A[0] + dx / L * comp, A[1] + dy / L * comp)
    pts = [p for p in (corte_recta(a, b, A, Q) for a, b in rectas) if p]
    pts += [p for p in (raiz_curva(c, A, Q) for c in curvas) if p]
    if len(pts) >= 2:
        return min(pts, key=lambda p: (p[0] - A[0]) ** 2 + (p[1] - A[1]) ** 2)
    return m if pts else ponta


def lei(F, A, T0, focus, gap):
    """O passo `empurrar` são DUAS re-ligações: a forma 1 px à direita (mundo), depois de volta."""
    co, si = math.cos(-F["angle"]), math.sin(-F["angle"])
    sh = (co, si)  # (1, 0) do mundo no referencial local
    a1 = (A[0] - sh[0], A[1] - sh[1])
    p1 = religa(F, a1, (T0[0] - sh[0], T0[1] - sh[1]), focus, gap)
    return religa(F, A, (p1[0] + sh[0], p1[1] + sh[1]), focus, gap)


# ---------------------------------------------------------------- a medida
def mede_caso(nome, entrada_caso):
    d = json.loads((S / f"w23_{nome}.0181.json").read_text())
    els = d["passos"][-1]["elementos"]
    F = els[0]
    hw, hh = F["width"] / 2, F["height"] / 2
    c = (F["x"] + hw, F["y"] + hh)
    ini = {e["id"]: e for e in entrada_caso["elementos"]}
    linhas = []
    for a in els[1:]:
        fim = a.get("endBinding") is not None
        b = a["endBinding"] if fim else a["startBinding"]
        pts = [(a["x"] + p[0], a["y"] + p[1]) for p in a["points"]]
        e0 = ini[a["id"]]
        p0 = [(e0["x"] + p[0], e0["y"] + p[1]) for p in e0["points"]]
        T, A = (pts[-1], pts[-2]) if fim else (pts[0], pts[1])
        T0 = p0[-1] if fim else p0[0]
        linhas.append((b, local(T, c, F["angle"]), local(A, c, F["angle"]), local(T0, c, F["angle"])))
    return F, linhas


def mede(nome, caso):
    F, linhas = mede_caso(nome, caso)
    dentro = nome.endswith("_dentro")
    g0 = linhas[0][0]["gap"]
    cand = {} if (dentro or g0 == 0) else candidatos(F, g0)
    geo = {k: [] for k in cand}
    r = {"n": len(linhas), "fica": 0, "foco": 0, "dfoc": 0.0, "lei": [], "longe": [], "F": F, "g": g0, "geo": geo}
    for b, T, A, T0 in linhas:
        focus, gap = b["focus"], b["gap"]
        q = lei(F, A, T0, focus, gap)
        r["lei"].append(math.hypot(q[0] - T[0], q[1] - T[1]))
        r["longe"].append(math.hypot(T[0] - T0[0], T[1] - T0[1]))
        if math.hypot(T[0] - T0[0], T[1] - T0[1]) < 1e-6:
            r["fica"] += 1
            continue
        u = (T[0] - A[0], T[1] - A[1])
        L = math.hypot(*u)
        r["dfoc"] = max(r["dfoc"], abs(sd(A, T) / support(F, (-u[1] / L, u[0] / L)) - focus))
        m = mira(F, A, focus)
        if m is not None and math.hypot(T[0] - m[0], T[1] - m[1]) < 1e-3:
            r["foco"] += 1  # a ponta no PONTO DO FOCO (gap 0, ou um só cruzamento)
            continue
        for k, poly in cand.items():  # o vão: a 1.ª entrada da recta OBSERVADA no contorno candidato
            e = entrada(A, u, poly)
            geo[k].append(math.inf if e is None else math.hypot(e[0] - T[0], e[1] - T[1]))
    return r


def fmt(v):
    if not v:
        return "     —"
    m = max(v)
    return f"{m:6.3f}" if m < 1e5 else "     ∞"


def main():
    casos = {cs["nome"]: cs for cs in json.loads(ENTRADA.read_text())["casos"]}
    fam = {}
    for nome in sorted(casos):
        partes = nome.split("_")
        chave = partes[1] + ("_" + partes[2] if partes[2] in ("r2", "r3") else "")
        fam.setdefault(chave, []).append((nome, mede(nome, casos[nome])))
    total = pior = 0
    for chave, rows in fam.items():
        ks = next((list(r["geo"]) for _, r in rows if r["geo"]), [])
        print(f"\n## {chave} — n setas · fica (não mexe) · foco (a ponta no ponto do foco) · |Δfocus| · LEI = "
              f"resíduo máx. da lei inteira (px) · vão: resíduo máx. da 1.ª entrada da recta observada por hipótese")
        print(f"{'caso':34} {'n':>4} {'fica':>4} {'foco':>4} {'|Δf|':>8} {'LEI':>7} | " + " | ".join(k[:24].ljust(24) for k in ks))
        for nome, r in rows:
            total += r["n"]
            pior = max(pior, max(r["lei"]))
            if nome.endswith("_dentro"):
                lo = r["longe"]
                print(f"{nome:34} {r['n']:4} {r['fica']:4} {r['foco']:4} {r['dfoc']:8.5f} {max(r['lei']):7.4f} | ponto fixo "
                      f"dentro: a ponta cai a mediana {statistics.median(lo):.2f} / máx. {max(lo):.2f} px do ponto gravado")
                continue
            print(f"{nome:34} {r['n']:4} {r['fica']:4} {r['foco']:4} {r['dfoc']:8.5f} {max(r['lei']):7.4f} | "
                  + " | ".join(fmt(r["geo"].get(k)).ljust(24) for k in ks))
    print(f"\nLEI: {total} setas, resíduo máximo {pior:.4f} px")


if __name__ == "__main__":
    main()
