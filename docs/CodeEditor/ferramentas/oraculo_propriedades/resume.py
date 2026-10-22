#!/usr/bin/env python3
# Junta a saída crua da sonda (stdout+stderr do Godot, numa só corrente) num caso JSON por linha:
# {"caso": rótulo, "r": resposta, "stderr": [linhas que o motor imprimiu entre o @@CASO e o @@R]}.
import json, re, sys
# o id de instância (`<Classe#123>`) muda de corrida para corrida: normaliza-se para `#ID`.
ID = re.compile(r"#-?\d+>")
casos, cur = [], None
for ln in open(sys.argv[1], encoding="utf-8", errors="replace"):
    ln = ln.rstrip("\n")
    ln = ID.sub("#ID>", ln)
    if ln.startswith("@@CASO "):
        cur = {"caso": ln[7:], "r": None, "stderr": []}
        casos.append(cur)
    elif ln.startswith("@@R "):
        cur["r"] = json.loads(ln[4:])
    elif cur is not None and cur["r"] is None and ln.strip():
        cur["stderr"].append(ln.strip())
    elif ln.strip():
        casos.append({"fora_de_caso": ln.strip()})
for c in casos:
    print(json.dumps(c, ensure_ascii=False))
