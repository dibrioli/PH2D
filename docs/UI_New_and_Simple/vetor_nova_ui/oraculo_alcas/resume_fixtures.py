#!/usr/bin/env python3
# A tabela das LEIS que as fixtures do Inkscape devolvem — cada linha põe ao lado do número do
# oráculo o que cada lei candidata preveria, para a escolha ser MEDIDA e não lida.
#
# uso:  python3 -I resume_fixtures.py
import json, math, os

AQUI = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(AQUI, "fixtures")


def load(n):
    p = os.path.join(FIX, n + ".json")
    return json.load(open(p)) if os.path.exists(p) else None


def unrot(p, deg, c=(400, 300)):
    a = -math.radians(deg)
    dx, dy = p[0] - c[0], p[1] - c[1]
    return (c[0] + dx * math.cos(a) - dy * math.sin(a), c[1] + dx * math.sin(a) + dy * math.cos(a))


def rect(nome, rot):
    f = load(nome)
    if not f:
        return
    print(f"\n## {nome}  (rect 300,240 200x120, rx=ry=10; o nó do rx no lado de cima, o do ry no lado direito)")
    print(f"{'exp':24} {'mod':6} {'rx':>9} {'ry':>9} | {'proj rx':>8} {'proj ry':>8}")
    for e in f["experiencias"]:
        d = e["depois"]
        t = unrot(e["alvo_doc"], rot)
        prx = min(max(500 - t[0], 0), 100)     # a projecção do alvo na aresta de cima, a partir do canto
        pry = min(max(t[1] - 240, 0), 60)      # idem na aresta direita
        print(f"{e['nome']:24} {e['mod']:6} {float(d.get('rx', 0)):9.3f} {float(d.get('ry', 0)):9.3f} | {prx:8.3f} {pry:8.3f}")


def star(nome, rot):
    f = load(nome)
    if not f:
        return
    print(f"\n## {nome}  (r1=100 r2=45, centro 400,300; arg em graus, ecrã y-para-baixo)")
    print(f"{'exp':24} {'mod':6} {'r1':>8} {'r2':>8} {'arg1':>8} {'arg2':>8} {'round':>6} | {'|alvo|':>8} {'ang alvo':>8}")
    for e in f["experiencias"]:
        d = e["depois"]
        t = unrot(e["alvo_doc"], rot)
        r = math.hypot(t[0] - 400, t[1] - 300)
        a = math.degrees(math.atan2(t[1] - 300, t[0] - 400))
        g = lambda k: float(d.get("sodipodi:" + k, d.get("inkscape:" + k, "nan")))
        print(f"{e['nome']:24} {e['mod']:6} {g('r1'):8.3f} {g('r2'):8.3f} {math.degrees(g('arg1')):8.2f} "
              f"{math.degrees(g('arg2')):8.2f} {g('rounded'):6.3f} | {r:8.3f} {a:8.2f}")


def arc(nome, rot):
    f = load(nome)
    if not f:
        return
    print(f"\n## {nome}  (rx=120 ry=80; start/end em graus; candidatas: atan2 NORMALIZADO · atan2 cru · ponto mais perto)")
    print(f"{'exp':20} {'mod':6} {'start':>8} {'end':>8} {'tipo':>6} | {'norm':>8} {'cru':>8} {'perto':>8}")
    for e in f["experiencias"]:
        d = e["depois"]
        t = unrot(e["alvo_doc"], rot)
        dx, dy = t[0] - 400, t[1] - 300
        norm = math.degrees(math.atan2(dy / 80, dx / 120)) % 360
        cru = math.degrees(math.atan2(dy, dx)) % 360
        best = min(range(36000), key=lambda k: (120 * math.cos(math.radians(k / 100)) - dx) ** 2
                   + (80 * math.sin(math.radians(k / 100)) - dy) ** 2) / 100
        s = math.degrees(float(d["sodipodi:start"])) % 360
        en = math.degrees(float(d["sodipodi:end"])) % 360
        print(f"{e['nome']:20} {e['mod']:6} {s:8.2f} {en:8.2f} {d.get('sodipodi:arc-type', '?'):>6} | {norm:8.2f} {cru:8.2f} {best:8.2f}")


def spiral(nome):
    f = load(nome)
    if not f:
        return
    print(f"\n## {nome}  (R=110, revolution=3, t0=0.1, expansion=1)")
    print(f"{'exp':24} {'mod':6} {'rev':>8} {'radius':>8} {'arg':>8} {'t0':>7} {'exp':>6} | {'|alvo|':>8} {'ang':>8}")
    for e in f["experiencias"]:
        d = e["depois"]
        t = e["alvo_doc"]
        r = math.hypot(t[0] - 400, t[1] - 300)
        a = math.degrees(math.atan2(t[1] - 300, t[0] - 400))
        g = lambda k: float(d.get("sodipodi:" + k, "nan"))
        print(f"{e['nome']:24} {e['mod']:6} {g('revolution'):8.3f} {g('radius'):8.3f} {math.degrees(g('argument')):8.2f} "
              f"{g('t0'):7.4f} {g('expansion'):6.3f} | {r:8.3f} {a:8.2f}")


rect("rect", 0)
rect("rect_rot25", 25)
star("star5", 0)
star("star5_rot25", 25)
star("polygon6", 0)
arc("arc_slice", 0)
arc("arc_slice_rot25", 25)
spiral("spiral")
