"""Sonda do Acabamento (Onda 2 do CodeEditor, item 4): o que o luau-lsp 1.70.1 responde a `definition`
e a `hover` em cada ESPÉCIE de palavra de código — a regra do Ctrl+clique (`goto.rs`) é «sítio neste
script · `self.<propriedade>` · senão o balão», então a mão do Ctrl+rato aparece onde o balão tiver
o que dizer. Lançado COMO O APP (a API gerada em `<cache>/ph2d/luau-lsp/1.70.1/api/` + `--docs=`).

Uso (da raiz da worktree):  python3 -I docs/CodeEditor/ferramentas/lspprobe_acabamento.py
Grava docs/CodeEditor/ferramentas/medicoes_onda2/acabamento_bruto.json e imprime a tabela.
"""
import glob, importlib.util, json, os

AQUI = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("lspprobe2", os.path.join(AQUI, "lspprobe2.py"))
P = importlib.util.module_from_spec(spec)
spec.loader.exec_module(P)

API = os.path.expanduser("~/.cache/ph2d/luau-lsp/1.70.1/api")
TEXTO = """ph2d.property("amplitude", 1.0)
local brilho = 1
function update(self, dt)
  self.t = (self.t or 0) + dt
  local d = brilho * self.amplitude * math.sin(self.t)
  if d > 0 and self.ampiltude then
    pirnt(d, nil, true)
  end
  ph2d.set(self.id, "x", d)
end
"""

# (rótulo, agulha, deslocamento dentro dela)
CASOS = [
    ("palavra-chave local", "local brilho", 0),
    ("palavra-chave function", "function update", 0),
    ("palavra-chave if", "if d > 0", 0),
    ("palavra-chave then", "then\n", 0),
    ("palavra-chave end", "end\n  ph2d.set", 0),
    ("palavra-chave and", "and self.ampiltude", 0),
    ("palavra-chave or", "or 0", 0),
    ("nil", "nil, true", 0),
    ("true", "true)", 0),
    ("número 1.0", "1.0)", 0),
    ("número 0", "0 and", 0),
    ("local: declaração", "local brilho", 6),
    ("local: uso", "= brilho", 2),
    ("parâmetro: declaração", "dt)\n", 0),
    ("parâmetro: uso", "+ dt", 2),
    ("self (parâmetro)", "self, dt", 0),
    ("self (uso)", "self.amplitude", 0),
    ("self.amplitude (propriedade)", "self.amplitude", 5),
    ("self.ampiltude (erro)", "self.ampiltude", 5),
    ("self.t (escrito no script)", "self.t * 0" if False else "math.sin(self.t", 14),
    ("self.id (campo do motor)", "self.id", 5),
    ("math (tabela global)", "math.sin", 0),
    ("math.sin", "math.sin", 5),
    ("ph2d (tabela)", "ph2d.set", 0),
    ("ph2d.set", "ph2d.set", 5),
    ("ph2d.property", "ph2d.property", 5),
    ("update (função do script)", "function update", 9),
    ("pirnt (global desconhecido)", "pirnt(", 0),
]


def resumo(v):
    if v is None:
        return None
    if isinstance(v, dict) and "contents" in v:
        c = v["contents"]
        t = c.get("value") if isinstance(c, dict) else json.dumps(c)
        return (t or "").replace("\n", "⏎")[:90]
    return json.dumps(v)[:120]


def main():
    d = sorted(glob.glob(os.path.join(API, "ph2d-*.d.luau")))
    j = sorted(glob.glob(os.path.join(API, "ph2d-docs-*.json")))
    assert d and j, "corra uma vez o app/oraculo para gerar a API em " + API
    P.DEFS = d[-1]
    out = {"loadavg": open("/proc/loadavg").read().split()[:3], "texto": TEXTO, "casos": {}}
    s = P.Lsp(P.BASE, extra_args=[f"--docs={j[-1]}"])
    out["args"] = s.args
    uri = "file:///ph2d-project/acabamento.luau"
    s.open(uri, TEXTO)
    for rotulo, agulha, off in CASOS:
        pos = P.pos_de(TEXTO, agulha, off)
        doc = {"textDocument": {"uri": uri}, "position": pos}
        _, dfn, _ = s.req("textDocument/definition", doc)
        _, hov, _ = s.req("textDocument/hover", doc)
        out["casos"][rotulo] = {
            "pos": pos,
            "definition": dfn.get("result", dfn.get("error")),
            "hover": hov.get("result", hov.get("error")),
        }
    s.stop()
    out["loadavg_fim"] = open("/proc/loadavg").read().split()[:3]
    dest = os.path.join(AQUI, "medicoes_onda2", "acabamento_bruto.json")
    json.dump(out, open(dest, "w"), ensure_ascii=False, indent=1)
    for k, v in out["casos"].items():
        dfn = v["definition"]
        aqui = bool(dfn) and any(t.get("uri", "").endswith("acabamento.luau") for t in dfn) if isinstance(dfn, list) else dfn
        print(f"{k:<30} def={'aqui' if aqui is True else json.dumps(dfn)[:40]:<42} hover={resumo(v['hover'])}")
    print("loadavg", out["loadavg"], "->", out["loadavg_fim"])


if __name__ == "__main__":
    main()
