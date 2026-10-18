"""Sonda do lote I9 + I12 + I14 (Onda 2 do CodeEditor): o que o `luau-lsp` 1.70.1 responde sobre NOSSAS entradas.

Lançado como o app (a API gerada em `<cache>/ph2d/luau-lsp/1.70.1/api/`, a configuração `ph2d_settings()` de
`crates/ph2d-code-lsp/src/client.rs`):
  I9  — as dicas do `bob.luau` com a configuração do app: posição, rótulo, `kind`, `paddingLeft/Right`,
        `textEdits`; quantas são `any`, truncadas ou genéricas soltas (o filtro do I7);
  I12 — os esqueletos candidatos, PREENCHIDOS, em modo normal e em `--!strict`: que diagnósticos sobram;
        e o que a completion dá numa linha vazia do topo e depois de `function ` (há modelos do servidor?);
  I14 — o `documentSymbol` sobre o bob + funções locais, de tabela, métodos e um local do topo (que
        símbolos o servidor conta, com que `kind` e que faixa).

Uso (da raiz da worktree):  python3 -I docs/CodeEditor/ferramentas/lspprobe_i9_i12_i14.py
Grava docs/CodeEditor/ferramentas/medicoes_onda2/I9_I12_I14_bruto.json e imprime o resumo.
"""
import glob, importlib.util, json, os

AQUI = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("lspprobe2", os.path.join(AQUI, "lspprobe2.py"))
P = importlib.util.module_from_spec(spec)
spec.loader.exec_module(P)

_params = P.initialize_params


def _como_o_app(extended):
    """O cliente Rust declara `snippetSupport: true` (client/json.rs); o lspprobe2 não."""
    p = _params(extended)
    p["capabilities"]["textDocument"]["completion"]["completionItem"]["snippetSupport"] = True
    return p


P.initialize_params = _como_o_app

API = os.path.expanduser("~/.cache/ph2d/luau-lsp/1.70.1/api")
APP = {"platform": {"type": "standard"}, "fflags": {"sync": False},
       "inlayHints": {"parameterNames": "none", "variableTypes": True,
                      "parameterTypes": True, "functionReturnTypes": True}}

EXTRA = (
    "\nlocal LIMITE = 3\n"
    "local function ajuda(v)\n  return v * 2\nend\n"
    "local M = {}\n"
    "function M.soma(a, b)\n  return a + b\nend\n"
    "function M:metodo()\n  local dentro = function() return 1 end\n  return dentro()\nend\n"
)

# Os esqueletos que o I12 geraria de `api_table::HOOKS` (e da `ph2d.property`), já PREENCHIDOS.
ESQUELETOS = {
    "init_anotado": "function init(self: ScriptSelf)\n  self.t = 0\nend\n",
    "update_anotado": "function update(self: ScriptSelf, dt: number)\n  self.t = (self.t or 0) + dt\nend\n",
    "on_signal_anotado": "function on_signal(self: ScriptSelf, signal: string)\n  print(signal)\nend\n",
    "update_anotado_retorno": "function update(self: ScriptSelf, dt: number): ()\n  self.t = (self.t or 0) + dt\nend\n",
    "update_sem_anotar": "function update(self, dt)\n  self.t = (self.t or 0) + dt\nend\n",
    "update_corpo_vazio": "function update(self: ScriptSelf, dt: number)\nend\n",
    "property_numero": 'ph2d.property("altura", 0)\n',
    "property_texto": 'ph2d.property("nome", "")\n',
    "property_opcoes": 'ph2d.property("forma", "sine", { options = { "sine", "square" } })\n',
}


def rotulo(h):
    l = h.get("label")
    return "".join(p.get("value", "") for p in l) if isinstance(l, list) else l


def main():
    d = sorted(glob.glob(os.path.join(API, "ph2d-*.d.luau")))
    j = sorted(glob.glob(os.path.join(API, "ph2d-docs-*.json")))
    assert d and j, "corra uma vez o app/oraculo para gerar a API em " + API
    P.DEFS = d[-1]
    s = P.Lsp(APP, extra_args=[f"--docs={j[-1]}"])
    bruto = {"defs": os.path.basename(d[-1]), "settings": APP}
    try:
        # I9
        uri = "file:///ph2d-project/bob.luau"
        texto = P.bob()
        s.open(uri, texto)
        P.diags(s, uri)
        n = texto.count("\n") + 1
        _, b, _ = s.req("textDocument/inlayHint", {"textDocument": {"uri": uri},
                                                    "range": {"start": {"line": 0, "character": 0},
                                                              "end": {"line": n, "character": 0}}})
        hints = b.get("result") or []
        linhas = texto.split("\n")
        bruto["I9_dicas"] = [{"line": h["position"]["line"], "character": h["position"]["character"],
                              "antes": linhas[h["position"]["line"]][:h["position"]["character"]][-24:],
                              "label": rotulo(h), "kind": h.get("kind"),
                              "paddingLeft": h.get("paddingLeft"), "paddingRight": h.get("paddingRight"),
                              "textEdits": len(h.get("textEdits") or []), "tooltip": bool(h.get("tooltip"))}
                             for h in hints]
        # I14
        uri2 = "file:///ph2d-project/simbolos.luau"
        s.open(uri2, texto + EXTRA)
        P.diags(s, uri2)
        _, b, _ = s.req("textDocument/documentSymbol", P.td(uri2))

        def achata(xs, nivel=0):
            out = []
            for x in xs or []:
                r = x.get("selectionRange") or x.get("range") or x.get("location", {}).get("range")
                out.append({"nivel": nivel, "name": x["name"], "kind": x["kind"], "detail": x.get("detail"),
                            "line": r["start"]["line"] if r else None})
                out += achata(x.get("children"), nivel + 1)
            return out
        bruto["I14_simbolos"] = achata(b.get("result"))
        # I12
        bruto["I12_esqueletos"] = {}
        for nome, corpo in ESQUELETOS.items():
            for modo, pre in (("normal", ""), ("estrito", "--!strict\n")):
                u = f"file:///ph2d-project/esq_{nome}_{modo}.luau"
                s.open(u, pre + corpo)
                ds = P.diags(s, u)
                bruto["I12_esqueletos"][f"{nome}/{modo}"] = [
                    {"code": x.get("code"), "line": x["range"]["start"]["line"], "message": x["message"][:140]}
                    for x in ds]
        for rot, txt in (("linha_vazia_topo", "ph2d.property(\"a\", 1)\n|\n"), ("depois_de_function", "function |\n"),
                         ("depois_de_function_up", "function up|\n")):
            t, pos = P.marca(txt)
            u = f"file:///ph2d-project/compl_{rot}.luau"
            s.open(u, t)
            _, b, _ = s.req("textDocument/completion", P.at(u, pos))
            r = b.get("result")
            items = r.get("items", []) if isinstance(r, dict) else (r or [])
            bruto[f"I12_completion_{rot}"] = sorted(
                {(i["label"], i.get("kind"), i.get("insertTextFormat"), (i.get("insertText") or "")[:60]) for i in items})
    finally:
        s.stop()
    saida = os.path.join(AQUI, "medicoes_onda2", "I9_I12_I14_bruto.json")
    with open(saida, "w", encoding="utf-8") as f:
        json.dump(bruto, f, ensure_ascii=False, indent=1)
    print("dicas:", len(bruto["I9_dicas"]))
    for h in bruto["I9_dicas"]:
        print(f"  {h['line']+1}:{h['character']} kind={h['kind']} padL={h['paddingLeft']} padR={h['paddingRight']} "
              f"edits={h['textEdits']} …{h['antes']!r} ⟨{h['label'][:50]}⟩")
    print("símbolos:")
    for x in bruto["I14_simbolos"]:
        print(f"  {'  '*x['nivel']}{x['name']} kind={x['kind']} linha={x['line']} detail={x['detail']}")
    print("esqueletos:")
    for k, v in bruto["I12_esqueletos"].items():
        print(f"  {k}: {[(x['code'], x['message'][:70]) for x in v]}")
    for rot in ("linha_vazia_topo", "depois_de_function", "depois_de_function_up"):
        it = bruto[f"I12_completion_{rot}"]
        print(f"completion {rot}: {len(it)} itens; snippets: {[i for i in it if i[2] == 2][:8]}; "
              f"com 'function'/'up': {[i[0] for i in it if 'function' in i[0] or i[0].startswith('up')][:8]}")
    print("gravado em", saida)


if __name__ == "__main__":
    main()
