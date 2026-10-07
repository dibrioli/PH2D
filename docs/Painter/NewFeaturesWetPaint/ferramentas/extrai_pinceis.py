# Extrai os chunks de texto (Parameters/ParametersEx/BrushID/Name) de cada *.rebelle-brush.png
# para JSON — DADOS do produto (o que o Brush Creator mostra), nunca o executável. As texturas
# embutidas (Texture_*) NÃO são extraídas: são assets do alvo.
import sys, os, struct, zlib, json
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
n = 0
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
            od = os.path.join(out, rel); os.makedirs(od, exist_ok=True)
            json.dump(rec, open(os.path.join(od, f.replace('.rebelle-brush.png', '.json')), 'w'), indent=1, sort_keys=True)
            n += 1
print("pinceis", n)
