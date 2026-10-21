"""Resumo do oráculo «arrastar para o código»: por caso, o que a soltura ESCREVEU (as linhas novas contra a base do
script) e onde ficou o cursor. Uso: resume.py casos.jsonl <data> <loadavg> <godot> <abre>."""
import difflib
import json
import sys

casos, data, load, godot, abre = sys.argv[1:6]
print(f"# oráculo «arrastar para o código» — {data} — loadavg: {load}")
print(f"# godot: {godot} (MIT) ; o EDITOR numa Xvfb 1600x1000, arrastos reais por xdotool (XTest); sonda: sonda.gd")
print(f"# abre: {abre}")
print("# colunas: caso | modificador na soltura | destino (linha,coluna base 0) | o que ficou escrito | cursor depois")
print("# «sob» = o controlo sob o rato na origem/destino (prova de que o gesto apanhou o que se queria)")
base = {}
ultimo = None
for linha in open(casos, encoding="utf-8"):
    r = json.loads(linha)
    if "base" in r:
        ultimo = r["depois"]["texto"]
        base[r["base"]] = ultimo
        print(f"\n## base {r['base']}: {json.dumps(ultimo)}")
        continue
    if "avisos_solto" in r or "avisos_depois_de_renomear" in r:
        k = next(iter(r))
        print(f"{k}: {json.dumps(r[k]['avisos'], ensure_ascii=False)}")
        continue
    if "completar" in r:
        print(f"escrever {json.dumps(r['completar'])} dentro da função → sugestões: {r['r'].get('opcoes')}")
        continue
    if "ctrl_clique" in r:
        c = r["ctrl_clique"]
        print(f"Ctrl+clique sobre «Porta» de `$Porta` (Placar seleccionado antes) → seleção {c['selecao'].get('selecao')}"
              f" · script aberto {c['ecra'].get('script')} · Inspector {c['ecra'].get('inspector')}"
              f" · texto {json.dumps(c['texto'].get('texto'))} · cursor {c['texto'].get('cursor')}")
        continue
    if "selecao_antes" in r:
        print(f"seleção antes do arrasto: {r['selecao_antes']['selecao']}")
        continue
    if "erro" in r:
        print(f"{r['caso']} | ERRO {r['erro']}")
        continue
    d = r["depois"]
    novo = d["texto"]
    ref = ultimo if ultimo is not None else ""
    if r["caso"].startswith("depois de renomear") or r["caso"].startswith("undo"):
        print(f"{r['caso']} | texto: {json.dumps(novo, ensure_ascii=False)} | nomes: {r.get('nomes', '')}")
        continue
    mais = [l[1:] for l in difflib.ndiff(ref.splitlines(), novo.splitlines()) if l.startswith("+ ")]
    menos = [l[1:] for l in difflib.ndiff(ref.splitlines(), novo.splitlines()) if l.startswith("- ")]
    escrito = " ⏎ ".join(json.dumps(m.lstrip(" "), ensure_ascii=False) for m in mais) or "(nada)"
    if menos:
        escrito += "   [linhas que mudaram: " + " ⏎ ".join(json.dumps(m.lstrip(" "), ensure_ascii=False) for m in menos) + "]"
    so = r["sob_origem"]
    sd = r["sob_destino"]
    item = so.get("item", "").rsplit("/", 1)[-1] if so.get("item", "").startswith("/root") else so.get("item", "")
    sob = f"sob: {so.get('controlo')}/{item} → {sd.get('controlo')}{sd.get('linha_coluna', '')}"
    print(f"{r['caso']} | {r['mod']} | {r['destino']} | {escrito} | cursor {d['cursor']} | {sob}")
