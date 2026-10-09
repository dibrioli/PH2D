"""O oráculo da RAIZ: a `libstemmer` 3.1.1 instalada (Snowball C, BSD-3, API pública `sb_stemmer_*`)
sobre uma amostra FIXA do vocabulário oficial do Snowball (cada 10.ª palavra de `snowball-data/<lang>/voc.txt`)
mais as palavras das nossas entradas. Grava `língua<TAB>palavra<TAB>raiz` com cabeçalho.

Uso: python3 -I grava_raizes.py <voc_en.txt> <voc_pt.txt> <saida.tsv>
"""
import ctypes, json, pathlib, sys

lib = ctypes.CDLL("libstemmer.so.0")
lib.sb_stemmer_new.restype = ctypes.c_void_p
lib.sb_stemmer_new.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
lib.sb_stemmer_stem.restype = ctypes.POINTER(ctypes.c_ubyte)
lib.sb_stemmer_stem.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_int]
lib.sb_stemmer_length.argtypes = [ctypes.c_void_p]

def stemmer(name):
    st = lib.sb_stemmer_new(name.encode(), b"UTF_8")
    assert st, name
    def f(w):
        b = w.encode()
        p = lib.sb_stemmer_stem(st, b, len(b))
        return bytes(p[: lib.sb_stemmer_length(st)]).decode()
    return f

voc_en, voc_pt, out = sys.argv[1:4]
here = pathlib.Path(__file__).parent
ours = set()
for f in ("notas_w7.json", "apostrofos.json"):
    for n in json.loads((here / "entradas" / f).read_text()):
        ours.update(w.strip(".,;:!?&").lower() for w in n["text"].replace("’", "'").split())
extra_en = sorted(w for w in ours if w) + ["interfered", "internal", "international", "interval", "added"]
lines = ["# Oráculo: libstemmer 3.1.1 (Snowball, BSD-3), sb_stemmer_stem em UTF_8.",
         "# Amostra: cada 10.ª palavra de snowball-data/{english,portuguese}/voc.txt + as palavras das entradas.",
         "# Gerado por docs/MiroClone/ferramentas/facilitacao_oracle/grava_raizes.py — não edite à mão.",
         "# língua\tpalavra\traiz"]
for lang, path, extra in (("english", voc_en, extra_en), ("portuguese", voc_pt, ["informação", "informações", "ação", "trabalho"])):
    st = stemmer(lang)
    words = [w.strip() for w in open(path, encoding="utf-8") if w.strip()][::10] + extra
    seen = set()
    for w in words:
        if w in seen:
            continue
        seen.add(w)
        lines.append(f"{lang}\t{w}\t{st(w)}")
pathlib.Path(out).write_text("\n".join(lines) + "\n", encoding="utf-8")
print(len(lines) - 4, "palavras")
