"""As assinaturas de TODOS os nomes que o runtime abre (o censo do `ph2d-code-help`, 225 a 20/10), pelo balão
do luau-lsp 1.70.1 sobre `<biblioteca>.<membro>` (e `<nome>` para os globais soltos). A fonte que quem
escreve a página de ajuda copia; o gate `oraculo_lsp::a_assinatura_da_pagina_e_a_do_assistente` confere-a.

Uso (da raiz da worktree):  python3 -I docs/CodeEditor/ferramentas/lspprobe_assinaturas.py <censo.txt>
  <censo.txt>: linhas `<página> <n> <membro> <membro> …` (o resumo do vermelho do G1).
Grava docs/CodeEditor/ferramentas/medicoes_onda2/ajuda_assinaturas.json: {"math.sin": "<balão cru>" | null}.
"""
import glob, importlib.util, json, os, sys

AQUI = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("lspprobe2", os.path.join(AQUI, "lspprobe2.py"))
P = importlib.util.module_from_spec(spec)
spec.loader.exec_module(P)
API = os.path.expanduser("~/.cache/ph2d/luau-lsp/1.70.1/api")


def main():
    nomes = []
    for linha in open(sys.argv[1]):
        partes = linha.split()
        if len(partes) < 3:
            continue
        pagina, membros = partes[0], partes[2:]
        nomes += [m if pagina == "globals" else f"{pagina}.{m}" for m in membros]
    texto = "".join(f"local _{i} = {n}\n" for i, n in enumerate(nomes))
    P.DEFS = sorted(glob.glob(os.path.join(API, "ph2d-*.d.luau")))[-1]
    s = P.Lsp(P.BASE, extra_args=[f"--docs={sorted(glob.glob(os.path.join(API, 'ph2d-docs-*.json')))[-1]}"])
    uri = "file:///ph2d-project/assinaturas.luau"
    s.open(uri, texto)
    out = {}
    for i, n in enumerate(nomes):
        col = len(f"local _{i} = ") + len(n) - 1
        _, h, _ = s.req("textDocument/hover", {"textDocument": {"uri": uri}, "position": {"line": i, "character": col}})
        r = h.get("result")
        out[n] = r["contents"]["value"] if r else None
    s.stop()
    dest = os.path.join(AQUI, "medicoes_onda2", "ajuda_assinaturas.json")
    json.dump(out, open(dest, "w"), ensure_ascii=False, indent=1)
    sem = [n for n, v in out.items() if not v]
    print(f"{len(out)} nomes, {len(sem)} sem balão: {sem}")


if __name__ == "__main__":
    main()
