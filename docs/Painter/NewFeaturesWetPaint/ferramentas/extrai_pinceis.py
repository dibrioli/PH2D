# Extrai os chunks de texto (Parameters/ParametersEx/BrushID/Name) de cada *.rebelle-brush.png
# para JSON — DADOS do produto (o que o Brush Creator mostra), nunca o executável. As texturas
# embutidas (Texture_*) NÃO são extraídas: são assets do alvo.
#
# uso: python3 -I extrai_pinceis.py "<Rebelle 8>" <pasta>        → um JSON por pincel
#      python3 -I extrai_pinceis.py "<Rebelle 8>" <saida>.json   → a fixture versionada: cabeçalho +
#      um pincel por linha, ordenados pelo caminho (fixtures/rebelle_presets_281.json)
import sys, os, struct, zlib, json, datetime
root, out = sys.argv[1], sys.argv[2]
def chunks(d):
    i = 8
    while i < len(d):
        ln, = struct.unpack('>I', d[i:i+4]); typ = d[i+4:i+8]; body = d[i+8:i+8+ln]
        yield typ, body; i += 12 + ln
def texts(d):
    r = {}
    for typ, body in chunks(d):
        if typ == b'tEXt':
            k, v = body.split(b'\0', 1); r[k.decode()] = v.decode('latin1')
        elif typ == b'zTXt':
            k, rest = body.split(b'\0', 1); r[k.decode()] = zlib.decompress(rest[1:]).decode('utf8', 'replace')
    return r
recs = []
for base in ["Brushes", "Brushes Bristle"]:
    for dp, dn, fn in os.walk(os.path.join(root, base)):
        for f in fn:
            if not f.endswith('.rebelle-brush.png'): continue
            t = texts(open(os.path.join(dp, f), 'rb').read())
            rel = os.path.relpath(dp, root)
            rec = {"file": os.path.join(rel, f), "BrushID": t.get("BrushID"), "Name": t.get("Name"),
                   "Textures": json.loads(t["Textures"]) if "Textures" in t else None,
                   "Parameters": json.loads(t["Parameters"]) if "Parameters" in t else None,
                   "ParametersEx": json.loads(t["ParametersEx"]) if "ParametersEx" in t else None,
                   "outros": sorted(k for k in t if k not in ("BrushID","Name","Textures","Parameters","ParametersEx") and not k.startswith("Texture_"))}
            recs.append(rec)
            if not out.endswith('.json'):
                od = os.path.join(out, rel); os.makedirs(od, exist_ok=True)
                json.dump(rec, open(os.path.join(od, f.replace('.rebelle-brush.png', '.json')), 'w'), indent=1, sort_keys=True)
if out.endswith('.json'):
    recs.sort(key=lambda r: r["file"])
    cab = {"fonte": "Rebelle 8.3.4 (teste, sob Wine): os chunks de texto de cada *.rebelle-brush.png de fábrica",
           "data": datetime.date.today().isoformat(), "pinceis": len(recs),
           "comando": "cd docs/Painter/NewFeaturesWetPaint && python3 -I ferramentas/extrai_pinceis.py "
                      "\"$HOME/Apps/rebelle/wineprefix/drive_c/Program Files/Rebelle 8\" " + out,
           "nota": "só parâmetros em texto; as texturas embutidas (Texture_*) são assets do alvo e NÃO estão aqui"}
    with open(out, 'w') as o:
        o.write('{"cabecalho": ' + json.dumps(cab, ensure_ascii=False, sort_keys=True) + ',\n "pinceis": [\n')
        o.write(',\n'.join(json.dumps(r, ensure_ascii=False, sort_keys=True) for r in recs))
        o.write('\n]}\n')
print("pinceis", len(recs))
