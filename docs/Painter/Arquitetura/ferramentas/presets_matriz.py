#!/usr/bin/env python3
"""Matriz «pincel de fábrica × knob» do Painter (F1, 31/10).

Corre com:  python3 -I docs/Painter/Arquitetura/ferramentas/presets_matriz.py <raiz-do-repo>

Lê (só leitura):
  * assets/brushes/**/*.ph2d-brush.png — o JSON do preset vem no chunk `iTXt` de chave `ph2d-brush`
    (comprimido por zlib; formato em crates/ph2d-brush-presets/src/png.rs:96-110);
  * crates/ph2d-wet-paint/src/tuning/defs.rs — os 144 knobs (chave, grupo, min, max, passo, fábrica do MOTOR);
  * crates/ph2d-tool-painter/src/tool/paint/wetpaint_settings.rs — `painter_default_knobs()`: o padrão do
    PRODUTO (WetKnobs::DEFAULT), que é o que um preset recebe quando a chave falta (wetpaint_preset.rs:310-320);
  * crates/ph2d-tool-painter/src/tool/paint/mao.rs — as 8 linhas da mão fora do registo (tuplas chave/min/max/passo/fábrica);
  * crates/ph2d-painter-brush/src/spec_default.rs — `BrushSpec::default()` (campos escalares do `brush`);
  * docs/Painter/Arquitetura/F1_censo_wet_paint.md §1 — a coluna «quem ESCREVE (UI)» de cada knob.

Escreve:
  * docs/Painter/Arquitetura/F1_presets_de_fabrica.md
  * docs/Painter/Arquitetura/fotos/f1_presets/matriz.tsv
  * docs/Painter/Arquitetura/fotos/f1_presets/mapa_de_calor.png (precisa de PIL)
"""
import collections
import glob
import json
import os
import re
import struct
import sys
import zlib

RAIZ = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else os.getcwd())
P = lambda *a: os.path.join(RAIZ, *a)  # noqa: E731
SAIDA_DOC = P("docs/Painter/Arquitetura/F1_presets_de_fabrica.md")
SAIDA_DIR = P("docs/Painter/Arquitetura/fotos/f1_presets")
TOL = 1e-9


# ---------------------------------------------------------------- leitura do PNG
def ler_preset(caminho):
    b = open(caminho, "rb").read()
    assert b[:8] == b"\x89PNG\r\n\x1a\n", caminho
    i = 8
    while i < len(b):
        n, = struct.unpack(">I", b[i:i + 4])
        tipo, dados = b[i + 4:i + 8], b[i + 8:i + 8 + n]
        i += 12 + n
        if tipo == b"iTXt" and dados.startswith(b"ph2d-brush\0"):
            r = dados[len(b"ph2d-brush\0"):]
            comp, r = r[0], r[2:]
            r = r[r.index(b"\0") + 1:]  # língua
            r = r[r.index(b"\0") + 1:]  # chave traduzida
            return json.loads(zlib.decompress(r) if comp else r)
    raise SystemExit(f"sem iTXt ph2d-brush: {caminho}")


# ---------------------------------------------------------------- padrões tirados do código
def sem_comentarios(txt):
    return re.sub(r"//[^\n]*", "", txt)


def ler_knobs():
    src = sem_comentarios(open(P("crates/ph2d-wet-paint/src/tuning/defs.rs")).read())
    num = r"(-?[\d.]+)"
    pat = re.compile(r"k\(\s*Knob::(\w+),\s*KnobGroup::(\w+),\s*\"(\w+)\",\s*" + r",\s*".join([num] * 4))
    knobs = collections.OrderedDict()
    for m in pat.finditer(src):
        nome, grupo, chave, mn, mx, passo, fab = m.groups()
        knobs[chave] = dict(nome=nome, grupo=grupo, min=float(mn), max=float(mx), passo=float(passo),
                            motor=float(fab), produto=float(fab), fonte="defs.rs")
    m = re.search(r"pub const KNOB_COUNT: usize = (\d+);", open(P("crates/ph2d-wet-paint/src/tuning.rs")).read())
    assert len(knobs) == int(m.group(1)), (len(knobs), m.group(1))
    por_nome = {v["nome"]: k for k, v in knobs.items()}
    ws = open(P("crates/ph2d-tool-painter/src/tool/paint/wetpaint_settings.rs")).read()
    corpo = ws[ws.index("const fn painter_default_knobs"):]
    corpo = corpo[:corpo.index("\n}\n")]
    n = 0
    for nome, v in re.findall(r"k\[Knob::(\w+) as usize\] = (-?[\d.]+);", corpo):
        knobs[por_nome[nome]]["produto"] = float(v)
        knobs[por_nome[nome]]["fonte"] = "wetpaint_settings.rs (produto)"
        n += 1
    assert n >= 5, n
    return knobs, por_nome


def ler_mao():
    src = open(P("crates/ph2d-tool-painter/src/tool/paint/mao.rs")).read()
    dyn = open(P("crates/ph2d-painter-brush/src/dynamics.rs")).read()
    const = {c: float(v) for c, v in re.findall(r"pub const (\w+): f32 = ([\d.]+);", dyn)}
    out = {}
    pat = re.compile(r"const \w+: \(&str, f64, f64, f64, f64\) = \(\s*\"(\w+)\",\s*([^,]+),\s*([^,]+),\s*([^,]+),\s*([^,)]+),?\s*\);")
    for chave, *vs in pat.findall(src):
        vals = []
        for v in vs:
            v = v.strip()
            m = re.match(r"ph2d_painter_brush::dynamics::(\w+) as f64", v)
            vals.append(const[m.group(1)] if m else float(v))
        out[chave] = dict(nome=chave, grupo="Mão (fora do registo)", min=vals[0], max=vals[1], passo=vals[2],
                          motor=vals[3], produto=vals[3], fonte="mao.rs")
    assert len(out) == 8, sorted(out)
    return out


EXTRAS_ENGINE = {
    # wetpaint_preset.rs:97-109, 325-392: chaves do mapa `engine` que não são knobs.
    "erase": dict(min=0.0, max=1.0, produto=0.4, nota="slider Erase do cartão (WetKnobs::DEFAULT.erase)"),
    "glazeLayering": dict(min=0.0, max=1.0, produto=0.0, nota="bool; ausente = OFF"),
    "pigmentMixing": dict(min=0.0, max=1.0, produto=1.0, nota="bool; ausente = ON (K-M)"),
    "paintTexture": dict(min=0.0, max=1.0, produto=0.0, nota="bool; ausente = OFF"),
    "realOil": dict(min=0.0, max=1.0, produto=0.0, nota="bool; ausente = OFF"),
}


def ler_brush_default():
    src = sem_comentarios(open(P("crates/ph2d-painter-brush/src/spec_default.rs")).read())
    out = {}
    for campo, v in re.findall(r"^\s+(\w+): (-?[\d.]+|true|false),\s*$", src, re.M):
        out[campo] = 1.0 if v == "true" else 0.0 if v == "false" else float(v)
    return out


def ler_censo_ui():
    ui = {}
    for linha in open(P("docs/Painter/Arquitetura/F1_censo_wet_paint.md")):
        if not re.match(r"^\| \d+ \| `", linha):
            continue
        cel = [c.strip() for c in linha.strip().strip("|").split(" | ")]
        m = re.search(r"\(`(\w+)`\)", cel[1])
        if m:
            ui[m.group(1)] = cel[5]
    return ui


# ---------------------------------------------------------------- famílias D1..D22 (censo §4, em chaves do preset)
FAMILIAS = collections.OrderedDict([
    ("D1", ("massa de pigmento por dab", ["pigmentPerDab", "volumes.opacity", "intensity", "pressureOpacity"])),
    ("D2", ("arrasto ao longo do traço", ["drag", "smear", "rewetDrag", "waterDrag", "smearWater"])),
    ("D3", ("recolha pela ponta", ["pickup", "tipReservoir", "tipClean", "wetLift"])),
    ("D4", ("água por dab", ["volumes.water", "waterPerDab", "smearWater"])),
    ("D5", ("secagem", ["evaporation", "baseEvaporation", "retention", "dryer", "dryerLevel", "dryerEdge"])),
    ("D6", ("absorção do papel", ["paperGate", "gateSaturation", "capillary", "capillaryGate"])),
    ("D7", ("passo", ["spacing", "capsuleSpacing", "pigmentSpacing", "capsuleMaxPx", "brush.spacing"])),
    ("D8", ("re-dissolver o assentado", ["rewet", "rewetPaint", "solubility"])),
    ("D9", ("orla escura / sedimento", ["edgeDarkening", "edgeSoftMass", "heavy_edge"])),
    ("D10", ("nivelamento", ["leveling", "levelClamp", "viscosity", "brake", "brakeReach", "levelingOwnStroke",
                             "knifeLevel", "dryerLevel"])),
    ("D11", ("força da borracha", ["erase", "eraser"])),
    ("D12", ("papel visto na tinta", ["paperVisibility", "visualGrain", "paintTexture", "paperRelief", "paperPress",
                                      "extGranulation"])),
    ("D13", ("altura do relevo", ["emboss", "oilDepth", "knifeRidge", "bristleHeight", "bristleRemove"])),
    ("D14", ("aderência do assentado", ["extStaining", "solubility"])),
    ("D15", ("pincel seco", ["extDryBrush"])),
    ("D16", ("velatura", ["glaze", "glazeLayering", "pigmentMixing", "live_water"])),
    ("D17", ("estabilização", ["pulledString", "brush.stabilizer"])),
    ("D18", ("escala com o diâmetro", ["embossSizeExp", "embossSizeRef", "textureSizeExp", "textureSizeRef",
                                       "textureSizeExpAcross", "bristleLaneSizeExp"])),
    ("D19", ("abertura do molhado alheio", ["wetOpen", "wetOpenPush", "wetOpenEdge"])),
    ("D20", ("material Principled (célula)", ["metallic", "roughness", "iorLevel", "coat", "coatRoughness", "sheen",
                                              "subsurface", "subsurfaceRadius"])),
    ("D21", ("cobertura sobre a de baixo", ["opaque", "wetCover", "dryCover", "transparency", "transmission",
                                            "transmissionMode", "realOil", "pigmentPerDab", "volumes.opacity"])),
    ("D22", ("brilho especular", ["oilGloss", "coat", "coatRoughness", "roughness"])),
])


def achatar(o, pre=""):
    if isinstance(o, dict):
        for k, v in o.items():
            yield from achatar(v, f"{pre}.{k}" if pre else k)
    elif isinstance(o, list):
        if all(isinstance(x, (int, float)) and not isinstance(x, bool) for x in o) and len(o) <= 4:
            for i, x in enumerate(o):
                yield f"{pre}[{i}]", x
        else:
            yield pre, json.dumps(o, sort_keys=True)
    else:
        yield pre, o


def num(v):
    if isinstance(v, bool):
        return 1.0 if v else 0.0
    if isinstance(v, (int, float)):
        return float(v)
    return None


def fmt(v):
    if v is None:
        return "—"
    if isinstance(v, float):
        return f"{v:g}"
    return str(v)


def main():
    knobs, por_nome = ler_knobs()
    mao = ler_mao()
    brush_def = ler_brush_default()
    ui = ler_censo_ui()
    # tabela de colunas: chave -> def
    cols = collections.OrderedDict()
    for k, d in knobs.items():
        cols[k] = dict(d, cls="knob", ui=ui.get(k, "?"))
    for k, d in mao.items():
        cols[k] = dict(d, cls="mão", ui="cartão Mão")
    for k, d in EXTRAS_ENGINE.items():
        cols[k] = dict(nome=k, grupo="engine extra", passo=None, motor=d["produto"], fonte="wetpaint_preset.rs",
                       cls="extra", ui=d["nota"], **d)
    for fam, (_, membros) in FAMILIAS.items():
        for m in membros:
            assert m in cols or m.startswith(("volumes.", "brush.")) or m in ("heavy_edge", "live_water"), (fam, m)

    # presets
    presets = []
    for f in sorted(glob.glob(P("assets/brushes/**/*.ph2d-brush.png"), recursive=True)):
        j = ler_preset(f)
        rel = os.path.relpath(f, P("assets/brushes"))
        partes = rel.split(os.sep)
        plano = dict(achatar({k: v for k, v in j.items() if k != "engine"}))
        presets.append(dict(arq=rel, media=j["media"], pasta=partes[0], cat=partes[1], grupo=partes[2],
                            nome=j["name"], engine=j.get("engine", {}), plano=plano))
    assert presets, "nenhum preset"

    desconhecidas = collections.Counter(k for p in presets for k in p["engine"] if k not in cols)

    # colunas não-engine: volumes.*, brush.* escalares, topo
    outras = collections.OrderedDict()
    for p in presets:
        for k, v in p["plano"].items():
            if k in ("schema", "media", "name", "group"):
                continue
            outras.setdefault(k, None)
    def default_outro(k, vals):
        if k.startswith("brush.") and k.count(".") == 1 and k[6:] in brush_def:
            return brush_def[k[6:]], "spec_default.rs"
        if k in ("volumes.opacity", "volumes.water"):
            return 1.0, "ausente = o de agora; referência 1,0"
        if k == "heavy_edge":
            return 0.0, "ausente = 0"
        if k == "live_water" or k == "tilt.on":
            return 0.0, "ausente = OFF"
        c = collections.Counter(json.dumps(v) for v in vals if v is not None)
        if not c:
            return None, "—"
        return json.loads(c.most_common(1)[0][0]), "moda (sem padrão escalar no código)"

    # efeito por preset
    def valor(p, k):
        """(cru, efectivo) — cru None se ausente."""
        if k in cols:
            cru = p["engine"].get(k)
            if p["media"] == "WetPaint" or cols[k]["cls"] == "mão" or k in mao or k in (
                    "pressureSize", "pressureSizeCurve", "pressureOpacity", "pressureOpacityCurve", "pressureFloor",
                    "pressureCeiling", "pressureSmoothing", "pulledString", "dabRotation", "tiltStart", "tiltFull",
                    "tiltWidth", "tiltLength", "tiltSmooth", "tiltLag"):
                ef = cru if cru is not None else (cols[k]["produto"] if p["media"] == "WetPaint" else cols[k]["motor"])
                return cru, ef
            return cru, cru  # knob do Wet num preset de outro meio: não se aplica
        return p["plano"].get(k), p["plano"].get(k)

    colunas_outras = {}
    for k in outras:
        vals = [p["plano"].get(k) for p in presets]
        colunas_outras[k] = default_outro(k, vals)

    def fora(k, ef, padrao):
        a, b = num(ef), num(padrao)
        if a is None or b is None:
            return ef is not None and padrao is not None and ef != padrao
        return abs(a - b) > TOL * max(1.0, abs(b))

    # ---- estatística por coluna
    stats = collections.OrderedDict()
    todas = list(cols) + list(outras)
    for k in todas:
        if k in cols:
            padrao = cols[k]["produto"]
        else:
            padrao = colunas_outras[k][0]
        usam, presentes, faixa, alem = [], 0, [], []
        for p in presets:
            cru, ef = valor(p, k)
            if cru is not None:
                presentes += 1
            if ef is None:
                continue
            if fora(k, ef, padrao):
                usam.append(p)
            n = num(ef)
            if n is not None and (cru is not None):
                faixa.append(n)
            if k in cols and cru is not None and num(cru) is not None:
                d = cols[k]
                if num(cru) < d["min"] - TOL or num(cru) > d["max"] + TOL:
                    alem.append((p, cru))
        stats[k] = dict(padrao=padrao, usam=usam, presentes=presentes,
                        mn=min(faixa) if faixa else None, mx=max(faixa) if faixa else None, alem=alem)

    os.makedirs(SAIDA_DIR, exist_ok=True)
    # ---- TSV
    with open(os.path.join(SAIDA_DIR, "matriz.tsv"), "w") as t:
        t.write("arquivo\tmeio\tcategoria\tnome\t" + "\t".join(todas) + "\n")
        t.write("#padrão\t\t\t\t" + "\t".join(fmt(stats[k]["padrao"]) for k in todas) + "\n")
        for p in presets:
            linha = [p["arq"], p["media"], p["cat"], p["nome"]]
            for k in todas:
                cru, _ = valor(p, k)
                linha.append("" if cru is None else fmt(cru))
            t.write("\t".join(linha) + "\n")

    # ---- mapa de calor
    png_ok = mapa_de_calor(presets, todas, cols, stats, valor, num)

    # ---- documento
    escrever_doc(presets, cols, mao, outras, colunas_outras, stats, desconhecidas, png_ok, valor, fora)


def mapa_de_calor(presets, todas, cols, stats, valor, num):
    try:
        from PIL import Image, ImageDraw, ImageFont
    except ImportError:
        return False
    ks = [k for k in todas if stats[k]["usam"] and num(stats[k]["padrao"]) is not None]
    cel, esq, topo = 9, 230, 190
    W, H = esq + cel * len(ks) + 10, topo + cel * len(presets) + 40
    img = Image.new("RGB", (W, H), (255, 255, 255))
    d = ImageDraw.Draw(img)
    fonte = ImageFont.load_default()
    NA, NEUTRO = (225, 225, 225), (247, 247, 247)
    BAIXO, ALTO = (33, 102, 172), (178, 24, 43)  # divergente azul–vermelho (ColorBrewer RdBu)

    def cor(x):
        x = max(-1.0, min(1.0, x))
        alvo = ALTO if x > 0 else BAIXO
        a = abs(x) ** 0.6
        return tuple(int(NEUTRO[i] + (alvo[i] - NEUTRO[i]) * a) for i in range(3))

    for j, k in enumerate(ks):
        padrao = num(stats[k]["padrao"])
        if k in cols:
            esc = cols[k]["max"] - cols[k]["min"]
        else:
            esc = (stats[k]["mx"] or 0) - (stats[k]["mn"] or 0)
        esc = esc if esc > TOL else max(1.0, abs(padrao))
        for i, p in enumerate(presets):
            cru, ef = valor(p, k)
            n = num(ef)
            c = NA if n is None else (NEUTRO if abs(n - padrao) <= TOL * max(1, abs(padrao)) else cor((n - padrao) / esc))
            x0, y0 = esq + j * cel, topo + i * cel
            d.rectangle([x0, y0, x0 + cel - 1, y0 + cel - 1], fill=c)
        rot = Image.new("RGBA", (topo - 4, cel), (255, 255, 255, 0))
        ImageDraw.Draw(rot).text((0, -2), k[:34], fill=(40, 40, 40), font=fonte)
        rot = rot.rotate(90, expand=True)
        img.paste(rot, (esq + j * cel, 2), rot)
    meio_ant = None
    for i, p in enumerate(presets):
        y = topo + i * cel
        if p["media"] != meio_ant:
            d.line([0, y, W, y], fill=(80, 80, 80))
            meio_ant = p["media"]
        d.text((4, y - 2), f"{p['media'][:3]} {p['cat'][:8]:<8} {p['nome'][:22]}", fill=(30, 30, 30), font=fonte)
    y = topo + cel * len(presets) + 10
    # A fonte por omissão do PIL não tem acentos: legenda em ASCII.
    d.text((4, y), "azul = abaixo do padrao | vermelho = acima (normalizado pela faixa do painel) | "
                   "branco = no padrao | cinza = nao se aplica ao meio", fill=(30, 30, 30), font=fonte)
    img.save(os.path.join(SAIDA_DIR, "mapa_de_calor.png"))
    return True


def escrever_doc(presets, cols, mao, outras, colunas_outras, stats, desconhecidas, png_ok, valor, fora):
    L = []
    w = L.append
    w("# F1 — Os pincéis de fábrica × os knobs (31/10)\n")
    w("> **Gerado** por `python3 -I docs/Painter/Arquitetura/ferramentas/presets_matriz.py "
      "/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` — não editar à mão; a fonte é o "
      "código e os PNG. Matriz completa: [`fotos/f1_presets/matriz.tsv`](fotos/f1_presets/matriz.tsv) · mapa de "
      "calor: [`fotos/f1_presets/mapa_de_calor.png`](fotos/f1_presets/mapa_de_calor.png).\n")
    w("**De onde vem cada número.** O JSON de cada preset está no chunk `iTXt` `ph2d-brush` do PNG "
      "(`crates/ph2d-brush-presets/src/png.rs:96-110`, zlib). Padrão de um knob do registo = o do **produto** "
      "(`WetKnobs::DEFAULT`, `tp/wetpaint_settings.rs` `painter_default_knobs`): é o que o preset recebe quando "
      "a chave falta (`tp/wetpaint_preset.rs:310-320` parte de `WetKnobs::DEFAULT`); difere da fábrica do motor "
      "(`wp/tuning/defs.rs`) em 7 knobs (Pigment 78, EdgeDarkening 12, PaperGate 0,4, Felt 0,03, BristleSize 2, "
      "BristleCount 2000, Emboss 0). Linhas da mão fora do registo: tuplas de `tp/mao.rs:90-125`. Chaves extra "
      "do mapa `engine` (`erase`, `glazeLayering`, `pigmentMixing`, `paintTexture`, `realOil`): "
      "`tp/wetpaint_preset.rs:97-109,325-392`. Campos `brush.*`: `BrushSpec::default` "
      "(`ph2d-painter-brush/src/spec_default.rs`) quando o campo é escalar de topo; nos aninhados, a MODA dos "
      "presets (marcado). «Painel não alcança» = o valor gravado sai de `min..max` (o `WetKnobs::set` PRENDE-O "
      "ao carregar — o motor nunca o recebe) ou o knob não tem escritor de UI (coluna «quem ESCREVE» do censo = "
      "«só preset»).\n")
    # (a)
    w("## (a) Pincéis por modo e categoria\n")
    por = collections.OrderedDict()
    for p in presets:
        por.setdefault(p["media"], collections.Counter())[p["cat"]] += 1
    w("| modo | n | categorias |")
    w("|---|---|---|")
    for m, c in por.items():
        w(f"| {m} | **{sum(c.values())}** | " + " · ".join(f"{k} {v}" for k, v in c.items()) + " |")
    for m in ("Impasto", "Composite"):
        if m not in por:
            w(f"| {m} | **0** | nenhum preset de fábrica (o `media` do formato só tem Digital/Watercolor/WetPaint em uso; "
              f"{'o Impasto é `PaintMedia::Impasto` sem pasta em `assets/brushes`' if m == 'Impasto' else 'o Composite não é um `media` de preset'}) |")
    w(f"\nTotal: **{len(presets)}** ficheiros `*.ph2d-brush.png`.\n")
    if desconhecidas:
        w("Chaves do mapa `engine` que NÃO são knob, linha da mão nem extra conhecido (lidas por `chave_antiga` ou "
          "ignoradas): " + ", ".join(f"`{k}` ×{n}" for k, n in desconhecidas.items()) + "\n")
    # (b)
    w("## (b) Cada knob: quantos pincéis o tiram do padrão, faixa usada, alcance do painel\n")
    w("«fora» = pincéis cujo valor EFECTIVO difere do padrão (Wet Paint: ausente = padrão do produto; Digital/"
      "Watercolor só têm as linhas da mão no `engine`). «grav.» = presets que gravam a chave. Faixa = min..max dos "
      "valores gravados.\n")
    w("| chave | grupo | UI (censo) | faixa do painel | padrão | grav. | fora | faixa usada | painel não alcança |")
    w("|---|---|---|---|---|---|---|---|---|")
    for k, d in cols.items():
        s = stats[k]
        nao = []
        if s["alem"]:
            nao.append("FORA DA FAIXA (preso no load): " + ", ".join(f"{p['nome']}={fmt(v)}" for p, v in s["alem"]))
        if "só preset" in str(d.get("ui", "")) and s["usam"]:
            nao.append(f"sem painel ({len(s['usam'])} pincéis o põem)")
        ui_txt = str(d.get("ui", "")).replace("|", "/")[:40]
        w(f"| `{k}` | {d['grupo']} | {ui_txt} | {fmt(d['min'])}..{fmt(d['max'])} | {fmt(d['produto'])} | "
          f"{s['presentes']} | **{len(s['usam'])}** | {fmt(s['mn'])}..{fmt(s['mx'])} | {'; '.join(nao) or '—'} |")
    w("\n### Campos fora do mapa `engine` (volumes, topo, `brush.*`)\n")
    w("| campo | padrão (fonte) | grav. | fora | faixa usada |")
    w("|---|---|---|---|---|")
    for k in outras:
        s = stats[k]
        pd, fonte = colunas_outras[k]
        pd_txt = fmt(pd)[:30]
        w(f"| `{k}` | {pd_txt} ({fonte}) | {s['presentes']} | **{len(s['usam'])}** | {fmt(s['mn'])}..{fmt(s['mx'])} |")
    # (c)
    wet = [p for p in presets if p["media"] == "WetPaint"]
    nunca = [k for k, d in cols.items() if not stats[k]["usam"]]
    um = [(k, stats[k]["usam"][0]) for k in cols if len(stats[k]["usam"]) == 1]
    w("\n## (c) Knobs que nenhum pincel usa e knobs de um só pincel\n")
    w(f"**Nenhum pincel os tira do padrão ({len(nunca)}) — candidatos a sair ou a virar constante:**\n")
    w(", ".join(f"`{k}`" for k in nunca) + "\n")
    w(f"**Usados por UM só pincel ({len(um)}):**\n")
    w("| chave | pincel | valor | padrão |")
    w("|---|---|---|---|")
    for k, p in um:
        w(f"| `{k}` | {p['media']}/{p['cat']}/{p['nome']} | {fmt(valor(p, k)[1])} | {fmt(stats[k]['padrao'])} |")
    # (d)
    w("\n## (d) Famílias de duplicados D1..D22: que pincéis dependem de cada membro\n")
    w("Membro «usado» = valor efectivo fora do padrão. «≥ 2 membros» = pincéis que tiram do padrão MAIS DE UM "
      "membro da família — são eles que a migração para UMA lei tem de traduzir com conta, não com cópia.\n")
    multi = []
    for fam, (grandeza, membros) in FAMILIAS.items():
        w(f"### {fam} — {grandeza}\n")
        w("| membro | pincéis fora do padrão | quais |")
        w("|---|---|---|")
        por_preset = collections.Counter()
        for m in membros:
            if m not in stats:
                w(f"| `{m}` | 0 | (nenhum preset grava o campo) |")
                continue
            us = stats[m]["usam"]
            for p in us:
                por_preset[p["arq"]] += 1
            nomes = ", ".join(p["nome"] for p in us)
            w(f"| `{m}` | {len(us)} | {nomes if len(nomes) < 400 else nomes[:400] + '…'} |")
        dois = [a for a, n in por_preset.items() if n >= 2]
        nome_de = {p["arq"]: p["nome"] for p in presets}
        w(f"\n**≥ 2 membros: {len(dois)} pincéis**" + (": " + ", ".join(nome_de[a] for a in dois) if dois else "") + "\n")
        multi.append((fam, len(dois)))
    w("**Resumo:** " + " · ".join(f"{f} {n}" for f, n in multi) + "\n")
    # (e)
    oleo = [p for p in presets if p["cat"] in ("Oil", "ExpressOil")
            or num(p["engine"].get("realOil")) == 1.0]
    w("## (e) Pincéis de óleo: todos os knobs fora do padrão, lado a lado\n")
    w("Óleo = categoria `Oil`/`ExpressOil` ou `realOil = 1`. Célula vazia = no padrão.\n")
    ks = [k for k in list(cols) + list(outras) if any(fora(k, valor(p, k)[1], stats[k]["padrao"])
                                                         for p in oleo if valor(p, k)[1] is not None)]
    ks = [k for k in ks if not k.startswith(("brush.custom_falloff", "brush.dab_curve", "width_table"))]
    w("| knob | padrão | " + " | ".join(p["nome"] for p in oleo) + " |")
    w("|---|---|" + "---|" * len(oleo))
    for k in ks:
        cel = []
        for p in oleo:
            ef = valor(p, k)[1]
            cel.append(fmt(ef)[:12] if ef is not None and fora(k, ef, stats[k]["padrao"]) else "")
        w(f"| `{k}` | {fmt(stats[k]['padrao'])[:12]} | " + " | ".join(cel) + " |")
    w("\n## Regerar\n")
    w("```\npython3 -I /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor/docs/Painter/Arquitetura/"
      "ferramentas/presets_matriz.py /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor\n```\n")
    w("Saídas: este ficheiro, `fotos/f1_presets/matriz.tsv` (linha `#padrão` logo abaixo do cabeçalho; célula vazia = "
      "chave ausente no preset) e `fotos/f1_presets/mapa_de_calor.png`" + ("" if png_ok else " (NÃO gerado: falta PIL)") + ".\n")
    open(SAIDA_DOC, "w").write("\n".join(L))


if __name__ == "__main__":
    main()
