#!/usr/bin/env python3
"""Gera entradas/w8_sondas.json: as sondas do `restoreElements`/`restoreAppState` (W8, esquema).
Cada sonda = elementos NOSSOS (mínimos, inválidos, legados, referências quebradas) que o oráculo
normaliza; a saída diz o que ele preenche, corrige ou recusa. Uso: python3 gera_w8_sondas.py <saida.json>"""
import json, sys

B = dict(x=10, y=20, width=100, height=50)
TIPOS = ["rectangle", "ellipse", "diamond", "text", "line", "arrow", "freedraw", "frame",
         "magicframe", "image", "embeddable", "iframe", "selection"]

def el(**kw):
    d = dict(B); d.update(kw); return d

s = []
# 1. mínimo por tipo: só type,x,y,width,height (sem id)
for t in TIPOS:
    s.append({"nome": f"minimo_{t}", "elementos": [el(type=t)]})
# 1b. com id, para ver que o id fica
s.append({"nome": "minimo_com_id", "elementos": [el(type="rectangle", id="meu-id")]})
# 2. tipo desconhecido
s.append({"nome": "tipo_desconhecido", "elementos": [el(type="star", id="s1"), el(type="rectangle", id="ok")]})
# 3. valores inválidos, um por sonda
inval = [
    ("opacity_150", dict(opacity=150)), ("opacity_menos5", dict(opacity=-5)), ("opacity_texto", dict(opacity="50")),
    ("opacity_0", dict(opacity=0)), ("roughness_7", dict(roughness=7)), ("roughness_menos1", dict(roughness=-1)),
    ("strokeStyle_wavy", dict(strokeStyle="wavy")), ("fillStyle_bogus", dict(fillStyle="bogus")),
    ("strokeWidth_0", dict(strokeWidth=0)), ("strokeWidth_menos3", dict(strokeWidth=-3)),
    ("roundness_tipo9", dict(roundness={"type": 9})), ("roundness_valor", dict(roundness={"type": 3, "value": 12})),
    ("angle_10", dict(angle=10)), ("angle_menos1", dict(angle=-1)),
    ("width_negativa", dict(width=-50, height=-20)), ("x_texto", dict(x="10")), ("x_null", dict(x=None)),
    ("strokeColor_invalida", dict(strokeColor="nao-e-cor")), ("backgroundColor_rgba", dict(backgroundColor="rgba(1,2,3,0.5)")),
    ("locked_texto", dict(locked="sim")), ("customData", dict(customData={"ph2d": {"nota": 1}})),
    ("link_javascript", dict(link="javascript:alert(1)")), ("link_sem_esquema", dict(link="example.com")),
    ("link_http", dict(link="https://example.com/a?b=1")),
    ("isDeleted", dict(isDeleted=True)),
    ("versao_dada", dict(id="v", version=5, versionNonce=7, seed=9, updated=1234)),
    ("versao_zero", dict(id="v0", version=0, versionNonce=0, seed=0)),
]
for n, kw in inval:
    kw = dict(kw); kw.setdefault("id", "r")
    s.append({"nome": f"invalido_{n}", "elementos": [el(type="rectangle", **kw)]})
# texto
txt = [
    ("fontFamily_99", dict(fontFamily=99)), ("fontFamily_texto", dict(fontFamily="Virgil")),
    *[(f"fontFamily_{i}", dict(fontFamily=i)) for i in range(1, 11)],
    ("textAlign_justify", dict(textAlign="justify")), ("verticalAlign_baseline", dict(verticalAlign="baseline")),
    ("sem_fontSize", dict()), ("sem_originalText", dict(text="abc")),
    ("originalText_diferente", dict(text="abc", originalText="xyz")),
    ("lineHeight_2", dict(lineHeight=2)), ("autoResize_false", dict(autoResize=False)),
    ("text_numero", dict(text=123)),
]
for n, kw in txt:
    kw = dict(kw); kw.setdefault("text", "Olá")
    s.append({"nome": f"texto_{n}", "elementos": [el(type="text", id="t", **kw)]})
# legado
for t in ["rectangle", "line", "diamond"]:
    for v in ["round", "sharp"]:
        s.append({"nome": f"legado_strokeSharpness_{v}_{t}", "elementos": [el(type=t, id="l", strokeSharpness=v)]})
# referências quebradas
s.append({"nome": "ref_containerId_ausente", "elementos": [el(type="text", id="t", text="x", containerId="nao-existe")]})
s.append({"nome": "ref_boundElements_ausentes", "elementos": [el(type="rectangle", id="r", boundElements=[{"id": "nada", "type": "arrow"}, {"id": "nada2", "type": "text"}])]})
s.append({"nome": "ref_binding_ausente", "elementos": [el(type="arrow", id="a", points=[[0, 0], [100, 0]], height=0, startBinding={"elementId": "nada", "focus": 0, "gap": 1})]})
s.append({"nome": "ref_binding_sem_reciproca", "elementos": [el(type="rectangle", id="r"), el(type="arrow", id="a", x=120, points=[[0, 0], [100, 0]], height=0, startBinding={"elementId": "r", "focus": 0, "gap": 1})]})
s.append({"nome": "ref_contentor_sem_reciproca", "elementos": [el(type="rectangle", id="r"), el(type="text", id="t", text="x", containerId="r")]})
s.append({"nome": "ref_frameId_ausente", "elementos": [el(type="rectangle", id="r", frameId="nada")]})
s.append({"nome": "ref_groupIds_texto", "elementos": [el(type="rectangle", id="r", groupIds="G")]})
# pontos
s.append({"nome": "pontos_primeiro_nao_zero", "elementos": [el(type="line", id="l", points=[[10, 10], [50, 30]])]})
s.append({"nome": "pontos_um_so", "elementos": [el(type="line", id="l", points=[[0, 0]])]})
s.append({"nome": "pontos_caixa_errada", "elementos": [el(type="arrow", id="a", width=999, height=999, points=[[0, 0], [100, 40]])]})
s.append({"nome": "pontos_negativos", "elementos": [el(type="line", id="l", points=[[0, 0], [-60, -30], [40, -10]])]})
s.append({"nome": "freedraw_pressoes_curtas", "elementos": [el(type="freedraw", id="f", points=[[0, 0], [5, 5], [10, 0]], pressures=[0.5], simulatePressure=False)]})
s.append({"nome": "seta_elbowed_sem_fixedPoint", "elementos": [el(type="rectangle", id="r"), el(type="rectangle", id="q", x=300), el(type="arrow", id="a", x=111, y=45, elbowed=True, points=[[0, 0], [188, 0]], height=0, startBinding={"elementId": "r", "focus": 0, "gap": 1}, endBinding={"elementId": "q", "focus": 0, "gap": 1})]})
s.append({"nome": "seta_elbowed_diagonal", "elementos": [el(type="arrow", id="a", elbowed=True, points=[[0, 0], [100, 50]])]})
s.append({"nome": "seta_pontas", "elementos": [el(type="arrow", id="a", points=[[0, 0], [100, 0]], height=0, startArrowhead="bogus", endArrowhead=None)]})
s.append({"nome": "seta_sem_pontas_dadas", "elementos": [el(type="arrow", id="a", points=[[0, 0], [100, 0]], height=0)]})
s.append({"nome": "linha_polygon", "elementos": [el(type="line", id="l", points=[[0, 0], [100, 0], [50, 50], [0, 0]], polygon=True)]})
# imagem
s.append({"nome": "imagem_sem_fileId", "elementos": [el(type="image", id="i")]})
s.append({"nome": "imagem_pending_escala", "elementos": [el(type="image", id="i", fileId="f", status="pending", scale=[-1, 1])]})
s.append({"nome": "imagem_crop", "elementos": [el(type="image", id="i", fileId="f", crop={"x": 1, "y": 2, "width": 4, "height": 4, "naturalWidth": 8, "naturalHeight": 8})]})
# ordem / índice fraccionário
s.append({"nome": "indice_ausente_5", "elementos": [el(type="rectangle", id=f"r{i}") for i in range(5)]})
s.append({"nome": "indice_fora_de_ordem", "elementos": [el(type="rectangle", id="r0", index="a5"), el(type="rectangle", id="r1", index=None), el(type="rectangle", id="r2", index="a1")]})
s.append({"nome": "indice_invalido", "elementos": [el(type="rectangle", id="r0", index="zz!"), el(type="rectangle", id="r1", index="a1")]})
s.append({"nome": "indice_duplicado", "elementos": [el(type="rectangle", id="r0", index="a1"), el(type="rectangle", id="r1", index="a1")]})
s.append({"nome": "indice_valido_mantido", "elementos": [el(type="rectangle", id="r0", index="a0"), el(type="rectangle", id="r1", index="a0V"), el(type="rectangle", id="r2", index="b0")]})
s.append({"nome": "indice_40", "elementos": [el(type="rectangle", id=f"r{i:02d}") for i in range(40)]})
# ids
s.append({"nome": "ids_duplicados", "elementos": [el(type="rectangle", id="dup"), el(type="ellipse", id="dup")]})
# contentor + texto, sem refreshDimensions e com
s.append({"nome": "texto_em_contentor_grande", "elementos": [el(type="rectangle", id="r", boundElements=[{"id": "t", "type": "text"}]), el(type="text", id="t", text="um texto muito comprido que nao cabe", containerId="r", width=5, height=5)]})
s.append({"nome": "texto_em_contentor_grande_refresh", "opts": {"refreshDimensions": True, "repairBindings": True}, "elementos": [el(type="rectangle", id="r", boundElements=[{"id": "t", "type": "text"}]), el(type="text", id="t", text="um texto muito comprido que nao cabe", containerId="r", width=5, height=5)]})
s.append({"nome": "texto_refresh_solto", "opts": {"refreshDimensions": True}, "elementos": [el(type="text", id="t", text="Olá mundo", fontSize=20, fontFamily=5, width=5, height=5)]})

# 2.ª leva (a 1.ª mostrou: lineHeight ausente sai da ALTURA; fontSize/fontFamily ausentes NÃO são preenchidos)
s.append({"nome": "texto_lineHeight_derivado_1linha", "elementos": [el(type="text", id="t", text="a", fontSize=20, fontFamily=5, height=37)]})
s.append({"nome": "texto_lineHeight_derivado_2linhas", "elementos": [el(type="text", id="t", text="a\nb", fontSize=20, fontFamily=5, height=50)]})
s.append({"nome": "texto_lineHeight_altura0", "elementos": [el(type="text", id="t", text="a", fontSize=20, fontFamily=5, height=0)]})
s.append({"nome": "texto_sem_fontFamily", "elementos": [el(type="text", id="t", text="a", fontSize=20, height=25)]})
s.append({"nome": "texto_completo_lineHeight_dado", "elementos": [el(type="text", id="t", text="a", fontSize=20, fontFamily=1, height=99, lineHeight=1.25)]})
s.append({"nome": "seta_binding_com_reciproca", "elementos": [el(type="rectangle", id="r", boundElements=[{"id": "a", "type": "arrow"}]), el(type="arrow", id="a", x=111, y=45, points=[[0, 0], [100, 0]], height=0, startBinding={"elementId": "r", "focus": 0.3, "gap": 4})]})
s.append({"nome": "linha_sem_caixa", "elementos": [{"type": "line", "id": "l", "x": 5, "y": 6, "points": [[0, 0], [30, 40]]}]})
# via convertToExcalidrawElements (o ESQUELETO da API): o que ele preenche, ao contrário do restore
conv = [
    ("rectangle", [{"type": "rectangle", "x": 0, "y": 0}]),
    ("ellipse", [{"type": "ellipse", "x": 0, "y": 0}]),
    ("diamond", [{"type": "diamond", "x": 0, "y": 0}]),
    ("text", [{"type": "text", "x": 0, "y": 0, "text": "Olá"}]),
    ("line", [{"type": "line", "x": 0, "y": 0}]),
    ("arrow", [{"type": "arrow", "x": 0, "y": 0}]),
    ("arrow_ligada", [{"type": "arrow", "x": 100, "y": 50, "start": {"type": "rectangle"}, "end": {"type": "ellipse"}}]),
    ("rectangle_label", [{"type": "rectangle", "x": 0, "y": 0, "label": {"text": "Olá caixa"}}]),
    ("frame", [{"type": "rectangle", "id": "c", "x": 10, "y": 10}, {"type": "frame", "children": ["c"]}]),
    ("freedraw", [{"type": "freedraw", "x": 0, "y": 0, "points": [[0, 0], [10, 10]]}]),
    ("image", [{"type": "image", "x": 0, "y": 0, "fileId": "f"}]),
    ("embeddable", [{"type": "embeddable", "x": 0, "y": 0, "link": "https://example.com"}]),
]
for n, els in conv:
    s.append({"nome": f"convert_{n}", "via": "convert", "elementos": els})

apps = [
    {"nome": "appstate_vazio", "appState": {}},
    {"nome": "appstate_nulo", "appState": None},
    {"nome": "appstate_lixo", "appState": {"viewBackgroundColor": 5, "gridSize": -1, "gridStep": 0, "zoom": {"value": 100}, "theme": "x", "scrollX": "a", "name": 3, "gridModeEnabled": "sim", "currentItemFontFamily": 99, "exportScale": 7}},
    {"nome": "appstate_legado_isSidebarDocked", "appState": {"isSidebarDocked": True}},
    {"nome": "appstate_ferramenta_desconhecida", "appState": {"activeTool": {"type": "star"}}},
    {"nome": "appstate_export", "appState": {"exportBackground": False, "exportEmbedScene": True, "exportWithDarkMode": True, "exportScale": 3, "viewBackgroundColor": "#123456", "gridSize": 40, "gridStep": 4, "gridModeEnabled": True, "name": "Quadro X"}},
]
json.dump({"sondas_elementos": s, "sondas_appstate": apps}, open(sys.argv[1], "w"), indent=1, ensure_ascii=False)
print(len(s), "sondas de elementos ·", len(apps), "de appState")
