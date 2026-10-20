"""Sonda da página de ajuda (Onda 2 do CodeEditor, item 5): o que o luau-lsp 1.70.1 responde a `hover` e a
`definition` em cada ESPÉCIE de nome da API — a função da biblioteca (`math.sin`), a global (`print`), o método
de um valor (`s:upper()`, `("x"):upper()`), a tabela por outro nome (`local m = math; m.sin`), a constante
(`math.pi`), a nossa (`ph2d.set`, `motion.fire`), o tipo nosso (`PoseField`), o gancho (`update`) e a própria
biblioteca (`string`). A pergunta: o balão já diz o nome QUALIFICADO (a porta única para a página)?
Lançado COMO O APP (a API gerada em `<cache>/ph2d/luau-lsp/1.70.1/api/` + `--docs=`).

Uso (da raiz da worktree):  python3 -I docs/CodeEditor/ferramentas/lspprobe_ajuda.py
Grava docs/CodeEditor/ferramentas/medicoes_onda2/ajuda_bruto.json e imprime a tabela.
"""
import glob, importlib.util, json, os

AQUI = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("lspprobe2", os.path.join(AQUI, "lspprobe2.py"))
P = importlib.util.module_from_spec(spec)
spec.loader.exec_module(P)

API = os.path.expanduser("~/.cache/ph2d/luau-lsp/1.70.1/api")
TEXTO = """ph2d.property("amplitude", 1.0)
local m = math
local campo: PoseField = "x"
function update(self, dt)
  local s = "abc"
  local a = math.sin(dt) + m.cos(dt) + math.pi + math.huge
  print(s:upper(), ("x"):upper(), string.format("%d", 1), typeof(s), tostring(a))
  table.insert({}, 1)
  local b = buffer.create(4)
  local v = vector.create(1, 2, 3)
  local n = bit32.band(1, 3) + utf8.len(s) + os.clock()
  local co = coroutine.wrap(function() end)
  local ok = pcall(error, "x")
  debug.traceback()
  ph2d.set(self.id, campo, a)
  motion.fire("go")
end
"""

# (rótulo, agulha, deslocamento dentro dela)
CASOS = [
    ("math.sin", "math.sin", 5),
    ("m.cos (math por outro nome)", "m.cos", 2),
    ("math.pi (constante)", "math.pi", 5),
    ("math.huge (constante)", "math.huge", 5),
    ("math (a biblioteca)", "math.sin", 0),
    ("print (global)", "print(", 0),
    ("s:upper (método de um local)", "s:upper", 2),
    ("(\"x\"):upper (método de um literal)", "(\"x\"):upper", 6),
    ("string.format", "string.format", 7),
    ("string (a biblioteca)", "string.format", 0),
    ("typeof", "typeof(", 0),
    ("tostring", "tostring(", 0),
    ("table.insert", "table.insert", 6),
    ("buffer.create", "buffer.create", 7),
    ("vector.create", "vector.create", 7),
    ("bit32.band", "bit32.band", 6),
    ("utf8.len", "utf8.len", 5),
    ("os.clock", "os.clock", 3),
    ("coroutine.wrap", "coroutine.wrap", 10),
    ("pcall", "pcall(", 0),
    ("error (passado como valor)", "error,", 0),
    ("debug.traceback", "debug.traceback", 6),
    ("ph2d.set", "ph2d.set", 5),
    ("ph2d.property", "ph2d.property", 5),
    ("ph2d (a tabela)", "ph2d.set", 0),
    ("motion.fire", "motion.fire", 7),
    ("PoseField (tipo nosso)", "PoseField", 0),
    ("update (gancho)", "function update", 9),
    ("self.id (campo do motor)", "self.id", 5),
]


def resumo(v):
    if v is None:
        return None
    if isinstance(v, dict) and "contents" in v:
        c = v["contents"]
        t = c.get("value") if isinstance(c, dict) else json.dumps(c)
        return (t or "").replace("\n", "⏎")[:110]
    return json.dumps(v)[:120]


def main():
    d = sorted(glob.glob(os.path.join(API, "ph2d-*.d.luau")))
    j = sorted(glob.glob(os.path.join(API, "ph2d-docs-*.json")))
    assert d and j, "corra uma vez o app/oraculo para gerar a API em " + API
    P.DEFS = d[-1]
    out = {"loadavg": open("/proc/loadavg").read().split()[:3], "texto": TEXTO, "casos": {}}
    s = P.Lsp(P.BASE, extra_args=[f"--docs={j[-1]}"])
    out["args"] = s.args
    uri = "file:///ph2d-project/ajuda.luau"
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
    dest = os.path.join(AQUI, "medicoes_onda2", "ajuda_bruto.json")
    json.dump(out, open(dest, "w"), ensure_ascii=False, indent=1)
    for k, v in out["casos"].items():
        dfn = v["definition"]
        aqui = bool(dfn) and any(t.get("uri", "").endswith("ajuda.luau") for t in dfn) if isinstance(dfn, list) else dfn
        print(f"{k:<36} def={'aqui' if aqui is True else json.dumps(dfn)[:30]:<32} hover={resumo(v['hover'])}")
    print("loadavg", out["loadavg"], "->", out["loadavg_fim"])


if __name__ == "__main__":
    main()
