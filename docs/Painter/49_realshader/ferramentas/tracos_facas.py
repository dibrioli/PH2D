# As 10 espátulas do Rebelle (Oil/Knife), uma pincelada horizontal cada, mesma cor/tamanho/pressão.
# uso: tracos_facas.py > t_facas.jsonl ; T=t_facas.jsonl sessao.sh <tag> ...
import json
from tracos import p  # noqa: F401  (mesmo formato de ponto)
import tracos

FACAS = ["Knife Smooth", "Knife Smooth 2", "Knife Smooth 3", "Knife Textured", "Knife Oily",
         "Knife Dry", "Knife Dry 2", "Knife Grainy", "Knife Soft", "Knife Stain"]
tracos.E[:] = tracos.E[:3]
for i, nome in enumerate(FACAS):
    x0 = 70 if i < 5 else 650
    y = 90 + 150 * (i % 5)
    tracos.traco("Knife/" + nome, tracos.AZ, [(x0, y), (x0 + 450, y)], 90)
for e in tracos.E:
    print(json.dumps(e))
