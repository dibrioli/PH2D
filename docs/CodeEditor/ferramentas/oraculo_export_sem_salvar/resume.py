"""Resumo do oráculo «@export sem salvar»: por passo, a linha do tempo das propriedades de script no Inspector."""
import json
import sys

caminho, quando, carga, versao = sys.argv[1:5]
print("# Oráculo: o @export aparece no Inspector do Godot sem salvar? (CodeEditor 1b)")
print(f"# corrido {quando} · {versao} (MIT) · loadavg {carga}")
print("# gerado por corre.sh — teclas reais (xdotool, 120 ms/tecla) no script aberto, nó seleccionado, sem salvar até ao 9")
print("# t_ms = relógio da sonda desde o início do passo; «fim das teclas» marca a última tecla")
print()
for l in open(caminho, encoding="utf-8"):
    d = json.loads(l)
    if "inicial" in d:
        print("## inicial:", json.dumps(d["inicial"]["cfg"], ensure_ascii=False))
        for e in d["inicial"]["lido"]["log"]:
            print("   props:", e.get("props"))
        print()
        continue
    if "passo" not in d:
        print("##", json.dumps(d, ensure_ascii=False))
        print()
        continue
    print("## passo", d["passo"])
    antes = None
    for e in d["lido"]["log"]:
        if "marca" in e:
            print(f"   {e['t_ms']:>6} ms  ── {e['marca']}")
            continue
        p = e["props"]
        mud = "" if p == antes else f"  props={p}"
        antes = p
        print(f"   {e['t_ms']:>6} ms  texto={e['texto_len']}{mud}  obj={e['obj'][:40]}")
    print("   avisos:", d["avisos"].get("avisos"))
    print("   abas:", d["abas"].get("abas"))
    print("   texto:", json.dumps(d["texto"].get("texto"), ensure_ascii=False))
    print()
