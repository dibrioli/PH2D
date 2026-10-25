# As variantes corrigidas (dentro2/dentroazul/largocast/largoazul). Corre-se na pasta de saída depois de gera.py:
#   python3 -I gera_variantes.py   (importa gera.py da mesma pasta; escreve <nome>_{0,1,S,D}.json)
# Diferença para gera.py: o SET_BRUSH vai DUAS vezes (o 1.º traço depois de uma troca pinta ao tamanho anterior —
# na 1.ª corrida da `dentro` a faixa saiu com 37 px em vez de 52) e o _0 aquece o pincel num canto (50..60, 600).
import importlib.util, sys
from pathlib import Path
AQUI = Path(__file__).resolve().parent
sys.argv = ["gera.py", "."]
spec = importlib.util.spec_from_file_location("g", AQUI / "gera.py"); g = importlib.util.module_from_spec(spec); spec.loader.exec_module(g)
def dup(ev):
    out = []
    for e in ev:
        if e["event_type"] == "SET_BRUSH":
            out.append(dict(e)); out.append({"event_type": "SET_BRUSH_COLOR", "color": e["color"]})
        out.append(e)
    return out
for nome, cor, larga in (("dentro2", g.CAST, False), ("dentroazul", g.AZUL, False),
                         ("largocast", g.CAST, True), ("largoazul", g.AZUL, True)):
    g.w(f"{nome}_0", g.novo() + dup(g.traco((50, 600), (60, 600), 40.0, cor)) + g.bm(f"{nome}_0"))
    g.w(f"{nome}_S", [{"event_type": "SIMULATION", "repeats": 240}] + g.bm(f"{nome}_S"))
    g.w(f"{nome}_D", [{"event_type": "DRY"}] + g.bm(f"{nome}_D"))
    if larga:  # faixa ~84 px: dois traços vermelhos de 60 px em y=305 e y=335; o 2.º traço em y=325, todo dentro
        red = dup(g.traco((300, 305), (700, 305), 60.0, g.VERM)) + dup(g.traco((300, 335), (700, 335), 60.0, g.VERM))
        seg = dup(g.traco((433, 325), (850, 325), 40.0, cor))
    else:      # faixa 52 px em y=320; o 2.º traço em y=330 (a borda de baixo dele sai 5 px para o papel)
        red = dup(g.traco((300, 320), (700, 320), 60.0, g.VERM))
        seg = dup(g.traco((433, 330), (850, 330), 40.0, cor))
    g.w(f"{nome}_1", red + seg + g.bm(f"{nome}_1"))
print("ok")
