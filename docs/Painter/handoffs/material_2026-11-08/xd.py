# Uso: xd.py <cena.json> <chave> <x0> <y0> [indice_bloco] -> script xdotool em stdout
import json, sys
from cena import lerp_poly
c=json.load(open(sys.argv[1])); k=sys.argv[2]; X=int(sys.argv[3]); Y=int(sys.argv[4])
strokes=c[k] if len(sys.argv)<6 else c[k][int(sys.argv[5])]
for s in strokes:
    p=lerp_poly([tuple(q) for q in s])
    print(f"mousemove {round(X+p[0][0])} {round(Y+p[0][1])}"); print("sleep 0.05"); print("mousedown 1"); print("sleep 0.03")
    for x,y in p[1:]:
        print(f"mousemove {round(X+x)} {round(Y+y)}"); print("sleep 0.006")
    print("sleep 0.03"); print("mouseup 1"); print("sleep 0.15")
