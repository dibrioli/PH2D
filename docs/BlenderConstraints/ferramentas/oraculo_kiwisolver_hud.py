"""Oráculo kiwisolver do HUD que se arruma (W5.5) — CORRE-SE, nunca se lê o fonte (python-kiwisolver 1.5.1, BSD-3-Clause,
`pacman -Qi python-kiwisolver`).

Uso:
    python -I docs/BlenderConstraints/ferramentas/oraculo_kiwisolver_hud.py \
        crates/ph2d-hud-layout/fixtures/kiwisolver_1_5_1_hud.json

Monta o MESMO sistema que `ph2d_hud_layout::Arranjo` (doc do módulo; ADR-0194), na MESMA ordem de inserção, e grava
as caixas de cada quadro. Cada caso corre também com os elementos inseridos ao contrário: o ótimo tem de ser o mesmo
(`unico`) — com os pesos da espec original (todos WEAK) muitos não o são, e isso fica gravado (`unico_tudo_weak`).
"""

import json
import math
import sys
import time

import kiwisolver as kiwi

REQ, STRONG, MEDIUM, WEAK = (kiwi.strength.required, kiwi.strength.strong, kiwi.strength.medium, kiwi.strength.weak)
FORCA = {"Requerida": REQ, "Forte": STRONG, "Media": MEDIUM, "Fraca": WEAK}
EDICAO = 1.0e9
# (posição de quem segue, largura de quem segue, posição de âncora, largura de âncora) — o desempate do lib.rs.
FIXO = 2_000_000.0  # acima do STRONG do «dentro do canvas»: o que o artista e as âncoras fixam manda (W5.5)
PESOS = (0.125, 0.3125, 0.5, 0.6875, FIXO, FIXO)  # + a posição de quem só é âncora e o tamanho do que não é texto
PESOS_TUDO_WEAK = (1.0, 1.0, 1.0, 1.0, 1.0, 1.0)
CAMPOS = ("x", "y", "w", "h")
GRELHA = 256.0


def g(x):
    """A grelha diádica do lib.rs (`f64::round`: metade para longe do zero)."""
    return math.copysign(math.floor(abs(x) * GRELHA + 0.5), x) / GRELHA


def validas(n, relacoes):
    out = []
    for rel, f in relacoes:
        (tipo, d), = rel.items()
        a, b = d["a"], d.get("b")
        if a >= n or (b is not None and (b >= n or b == a)):
            continue
        out.append((tipo, d, f))
    return out


def poe(s, c):
    try:
        s.addConstraint(c)
        return True
    except kiwi.UnsatisfiableConstraint:
        return False


def relacao(v, tipo, d):
    a = v[d["a"]]
    if tipo == "LarguraDoConteudo":
        return None
    b = v[d["b"]]
    if tipo == "Centrado":
        k = "x" if d["eixo"] == "X" else "y"
        m = "w" if k == "x" else "h"
        return a[k] + 0.5 * a[m] == b[k] + 0.5 * b[m]
    lado, g_ = d["lado"], g(d["folga"])
    if tipo == "AoLado":
        return {"Direita": a["x"] == b["x"] + b["w"] + g_, "Esquerda": a["x"] + a["w"] + g_ == b["x"],
                "Acima": a["y"] == b["y"] + b["h"] + g_, "Abaixo": a["y"] + a["h"] + g_ == b["y"]}[lado]
    return {"Direita": a["x"] + a["w"] + g_ <= b["x"], "Esquerda": a["x"] >= b["x"] + b["w"] + g_,
            "Acima": a["y"] + a["h"] + g_ <= b["y"], "Abaixo": a["y"] >= b["y"] + b["h"] + g_}[lado]


class Arranjo:
    def __init__(self, canvas, elementos, relacoes, pesos=PESOS, ao_contrario=False):
        self.s = s = kiwi.Solver()
        self.caixas = [e["caixa"] for e in elementos]
        rels = validas(len(elementos), relacoes)
        part, segue, ancora, texto = set(), set(), set(), set()
        for tipo, d, _ in rels:
            part.add(d["a"])
            if "b" not in d:
                texto.add(d["a"])
            if "b" in d:
                part.add(d["b"])
                segue.add(d["a"])
                ancora.add(d["b"])
        # Três degraus (W5.5): só segue (cede primeiro) · segue e é âncora · só âncora (fica onde a âncora o pôs).
        self.v = {}
        self.larguras = []
        if not part:
            return
        self.cv = {k: kiwi.Variable("canvas." + k) for k in CAMPOS}
        for k in CAMPOS:
            s.addEditVariable(self.cv[k], EDICAO)
            s.suggestValue(self.cv[k], g(canvas[k]))
        cv = self.cv
        ordem = sorted(part, reverse=ao_contrario)
        for i in ordem:
            c = {k: g(x) for k, x in elementos[i]["caixa"].items()}
            v = self.v[i] = {k: kiwi.Variable("e%d.%s" % (i, k)) for k in CAMPOS}
            if i in segue:
                pp, pw = (pesos[0], pesos[1]) if i not in ancora else (pesos[2], pesos[3])
            else:
                pp, pw = pesos[4], pesos[3]
            if i not in texto:
                pw = pesos[5]  # só o texto muda de tamanho (W5.5)
            for cn in ((v["x"] == c["x"]) | pp, (v["y"] == c["y"]) | pp, (v["w"] == c["w"]) | pw,
                       (v["h"] == c["h"]) | (FIXO if pesos is PESOS else STRONG), (v["w"] >= 0) | REQ, (v["h"] >= 0) | REQ,
                       (v["x"] >= cv["x"]) | STRONG, (v["y"] >= cv["y"]) | STRONG,
                       (v["x"] + v["w"] <= cv["x"] + cv["w"]) | STRONG,
                       (v["y"] + v["h"] <= cv["y"] + cv["h"]) | STRONG):
                poe(s, cn)
        self.conteudo = {}
        for tipo, d, f in rels:
            a = d["a"]
            if tipo != "LarguraDoConteudo":
                poe(s, relacao(self.v, tipo, d) | FORCA[f])
                continue
            if a not in self.conteudo:
                cvar = self.conteudo[a] = kiwi.Variable("conteudo%d" % a)
                s.addEditVariable(cvar, EDICAO)
                s.suggestValue(cvar, g(elementos[a]["conteudo"] or 0.0))
            cvar, w = self.conteudo[a], self.v[a]["w"]
            l = {"a": a, "cs": [(w == cvar) | FORCA[f], (w <= cvar) | REQ], "postas": [False, False]}
            if elementos[a]["conteudo"] is not None:
                l["postas"] = [poe(s, c) for c in l["cs"]]
            self.larguras.append(l)

    def atualiza(self, canvas, conteudos):
        if not self.v:
            return [dict(c) for c in self.caixas]
        s = self.s
        for k in CAMPOS:
            s.suggestValue(self.cv[k], g(canvas[k]))
        for l in self.larguras:
            a = l["a"]
            c = conteudos[a] if a < len(conteudos) else None
            if c is not None:
                s.suggestValue(self.conteudo[a], g(c))
                if not any(l["postas"]):
                    l["postas"] = [poe(s, cn) for cn in l["cs"]]
            elif any(l["postas"]):
                for cn, p in zip(l["cs"], l["postas"]):
                    if p:
                        s.removeConstraint(cn)
                l["postas"] = [False, False]
        s.updateVariables()
        return [{k: self.v[i][k].value() for k in CAMPOS} if i in self.v else dict(self.caixas[i])
                for i in range(len(self.caixas))]


def corre(caso, **kw):
    arr = Arranjo(caso["canvas"], caso["elementos"], caso["relacoes"], **kw)
    return [arr.atualiza(q["canvas"], q["conteudos"]) for q in caso["quadros"]]


def igual(r1, r2):
    return all(abs(a[k] - b[k]) <= 1e-9 for q1, q2 in zip(r1, r2) for a, b in zip(q1, q2) for k in CAMPOS)


# ---- os casos (entradas NOSSAS) ----
def C(x, y, w, h):
    return {"x": float(x), "y": float(y), "w": float(w), "h": float(h)}


def E(x, y, w, h, conteudo=None):
    return {"caixa": C(x, y, w, h), "conteudo": None if conteudo is None else float(conteudo)}


def aolado(a, b, lado, folga=0.0):
    return {"AoLado": {"a": a, "b": b, "lado": lado, "folga": float(folga)}}


def naoinvade(a, b, lado, folga=0.0):
    return {"NaoInvade": {"a": a, "b": b, "lado": lado, "folga": float(folga)}}


def centrado(a, b, eixo):
    return {"Centrado": {"a": a, "b": b, "eixo": eixo}}


def largura(a):
    return {"LarguraDoConteudo": {"a": a}}


TELA = C(0, 0, 640, 360)


def quadros(n_el, *qs):
    """`qs` = (canvas, {índice: conteúdo}) por quadro."""
    out = []
    for canvas, cont in qs:
        out.append({"canvas": canvas, "conteudos": [cont.get(i) for i in range(n_el)]})
    return out


def casos():
    cs = []

    def caso(nome, familia, elementos, relacoes, qs):
        cs.append({"nome": nome, "familia": familia, "canvas": qs[0][0], "elementos": elementos,
                   "relacoes": relacoes, "quadros": quadros(len(elementos), *qs)})

    # ao lado — a moeda cola à direita da pontuação que cresce (99 → 12 345).
    pont, moeda = E(16, 312, 100, 32, 38.5), E(200, 316, 24, 24)
    caso("moeda_a_direita_da_pontuacao_que_cresce", "ao_lado", [pont, moeda],
         [[largura(0), "Forte"], [aolado(1, 0, "Direita", 8), "Requerida"]],
         [(TELA, {0: 38.5}), (TELA, {0: 51.25}), (TELA, {0: 77.0}), (TELA, {0: 102.75}), (TELA, {0: 128.375}),
          (TELA, {0: 38.5})])
    caso("vidas_a_esquerda_do_relogio", "ao_lado", [E(560, 312, 64, 32), E(400, 316, 40, 24), E(10, 10, 5, 5)],
         [[aolado(1, 0, "Esquerda", 6.5), "Requerida"]], [(TELA, {}), (C(0, 0, 800, 360), {})])
    caso("legenda_acima_da_barra", "ao_lado", [E(16, 16, 200, 12), E(30, 90, 120, 18)],
         [[aolado(1, 0, "Acima", 4), "Forte"]], [(TELA, {}), (C(0, 0, 640, 240), {})])
    caso("dica_abaixo_do_botao_fraca", "ao_lado", [E(300, 200, 80, 30), E(250, 120, 140, 20)],
         [[aolado(1, 0, "Abaixo", 10), "Fraca"]], [(TELA, {})])

    # pilha — Abaixo encadeado.
    caso("tres_linhas_empilhadas", "pilha", [E(16, 312, 160, 32), E(16, 250, 160, 24), E(16, 100, 160, 24)],
         [[aolado(1, 0, "Abaixo", 4), "Requerida"], [aolado(2, 1, "Abaixo", 4), "Requerida"]],
         [(TELA, {}), (C(0, 0, 640, 480), {}), (TELA, {})])
    caso("pilha_media_com_rotulos", "pilha", [E(400, 300, 200, 40, 150), E(410, 10, 90, 20, 70.5), E(5, 5, 50, 20)],
         [[largura(0), "Forte"], [aolado(1, 0, "Abaixo", 2.5), "Media"], [aolado(2, 1, "Abaixo", 2.5), "Media"],
          [largura(1), "Forte"]],
         [(TELA, {0: 150.0, 1: 70.5}), (TELA, {0: 180.0, 1: 33.0})])

    # centrado.
    caso("titulo_centrado_no_painel", "centrado", [E(120, 100, 400, 200), E(0, 260, 180, 30, 96)],
         [[centrado(1, 0, "X"), "Requerida"], [largura(1), "Forte"]],
         [(TELA, {1: 96.0}), (TELA, {1: 250.0}), (TELA, {1: 96.0})])
    caso("icone_centrado_em_y", "centrado", [E(20, 140, 300, 80), E(330, 10, 32, 32)],
         [[centrado(1, 0, "Y"), "Forte"]], [(TELA, {}), (C(0, 0, 640, 200), {})])
    caso("centrado_nos_dois_eixos", "centrado", [E(200, 120, 240, 120), E(0, 0, 50, 30)],
         [[centrado(1, 0, "X"), "Requerida"], [centrado(1, 0, "Y"), "Requerida"]], [(TELA, {})])

    # não invade — a pontuação cresce contra o minimapa à direita.
    caso("pontuacao_contra_o_minimapa", "nao_invade", [E(16, 312, 100, 32, 60), E(504, 240, 120, 104)],
         [[largura(0), "Media"], [naoinvade(0, 1, "Direita", 8), "Requerida"]],
         [(TELA, {0: 60.0}), (TELA, {0: 300.0}), (TELA, {0: 500.0}), (TELA, {0: 60.0})])
    # O minimapa segura a largura dele (o conteúdo, Forte); a pontuação quer a dela só a Media ⇒ ela cede primeiro.
    mapa = {0: 200.0, 1: 120.0}
    caso("tela_estreita_encolhe_a_pontuacao", "nao_invade", [E(16, 312, 100, 32, 200), E(504, 240, 120, 104, 120)],
         [[largura(0), "Media"], [largura(1), "Forte"], [naoinvade(0, 1, "Direita", 8), "Requerida"]],
         [(TELA, mapa), (C(0, 0, 400, 360), mapa), (C(0, 0, 300, 360), mapa), (C(0, 0, 250, 360), mapa),
          (TELA, mapa)])
    caso("nao_invade_ja_satisfeito", "nao_invade", [E(16, 16, 50, 20), E(300, 16, 50, 20)],
         [[naoinvade(0, 1, "Direita", 8), "Requerida"]], [(TELA, {})])
    caso("nao_invade_esquerda_acima_abaixo", "nao_invade",
         [E(300, 150, 60, 60), E(280, 160, 40, 20), E(310, 190, 30, 40), E(320, 140, 20, 30)],
         [[naoinvade(1, 0, "Esquerda", 4), "Forte"], [naoinvade(2, 0, "Acima", 4), "Media"],
          [naoinvade(3, 0, "Abaixo", 4), "Requerida"]], [(TELA, {})])

    # largura com força — conteúdo largo × estreito, Forte × Fraca.
    # O vizinho está colado à borda direita e segura a largura (Forte): só a força do conteúdo decide quem cede.
    vizinhos = [E(16, 312, 100, 32, 60), E(560, 312, 80, 32, 80)]
    for forca in ("Forte", "Fraca"):
        caso("conteudo_estreito_" + forca.lower(), "largura_com_forca", [E(16, 312, 100, 32, 60)],
             [[largura(0), forca]], [(TELA, {0: 60.0})])
        caso("conteudo_largo_contra_vizinho_media_" + forca.lower(), "largura_com_forca", vizinhos,
             [[largura(0), forca], [largura(1), "Forte"], [naoinvade(0, 1, "Direita", 8), "Media"]],
             [(TELA, {0: 60.0, 1: 80.0}), (TELA, {0: 600.0, 1: 80.0})])
    caso("conteudo_some_e_volta", "largura_com_forca", [E(16, 312, 100, 32, 60)], [[largura(0), "Forte"]],
         [(TELA, {0: 60.0}), (TELA, {}), (TELA, {0: 200.0}), (TELA, {0: 60.0})])
    caso("conteudo_maior_que_a_tela_media", "largura_com_forca", [E(16, 312, 100, 32, 900)], [[largura(0), "Media"]],
         [(TELA, {0: 900.0}), (C(0, 0, 1280, 360), {0: 900.0})])

    # requerida impossível e índices maus — a relação é ignorada, nunca pânico. ⚠️ Só a contradição de IGUALDADES se
    # deteta: a de DESIGUALDADES requeridas (AoLado + NaoInvade opostos) passa e viola outra requerida (medido: w = −8,
    # e a resposta muda com a ordem de inserção) — fica fora da fixture; ADR-0194 §Preço.
    caso("requerida_impossivel_e_ignorada", "impossivel", [E(16, 312, 100, 32), E(200, 316, 24, 24)],
         [[aolado(1, 0, "Direita", 8), "Requerida"], [aolado(1, 0, "Direita", 20), "Requerida"],
          [aolado(7, 0, "Abaixo", 1), "Requerida"], [centrado(1, 1, "X"), "Requerida"]], [(TELA, {})])
    return cs


def main():
    saida = sys.argv[1]
    out = {"cabecalho": {
        "oraculo": "kiwisolver %s (python-kiwisolver 1.5.1-1.1, BSD-3-Clause — `pacman -Qi`), corrido, nunca lido"
                   % kiwi.__version__,
        "gerador": "docs/BlenderConstraints/ferramentas/oraculo_kiwisolver_hud.py",
        "comando": "python -I docs/BlenderConstraints/ferramentas/oraculo_kiwisolver_hud.py "
                   "crates/ph2d-hud-layout/fixtures/kiwisolver_1_5_1_hud.json",
        "sistema": "ADR-0194 / ph2d_hud_layout (doc do módulo): stays %s, edição %g, grelha 1/%g, ordem fixa"
                   % (PESOS, EDICAO, GRELHA),
        "data": time.strftime("%Y-%m-%d"),
    }, "casos": []}
    for caso in casos():
        r = corre(caso)
        caso["unico"] = igual(r, corre(caso, ao_contrario=True))
        caso["unico_tudo_weak"] = igual(corre(caso, pesos=PESOS_TUDO_WEAK),
                                        corre(caso, pesos=PESOS_TUDO_WEAK, ao_contrario=True))
        for q, caixas in zip(caso["quadros"], r):
            q["caixas"] = caixas
        out["casos"].append(caso)
        print("%-45s %-18s unico=%s tudo_weak=%s" % (caso["nome"], caso["familia"], caso["unico"],
                                                     caso["unico_tudo_weak"]))
    json.dump(out, open(saida, "w"), indent=1)


main()
