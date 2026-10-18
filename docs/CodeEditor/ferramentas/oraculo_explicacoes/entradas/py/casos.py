import traceback
class Onda:
    def __init__(self): self.amplitude = 1.0
def soma(a, b): return a + b
def dobro(a: int) -> int: return a * 2
casos = {
 "nome_global": lambda: pirnt("x"),
 "membro": lambda: Onda().ampiltude,
 "prop_tipo": lambda: (1.5).z,
 "nargs": lambda: soma(1),
 "tipo_arg": lambda: "texto" + 2,
 "chamada_nil": lambda: (None)(),
 "metodo_nil": lambda: None.mover(),
}
for k, f in casos.items():
    print("===", k)
    try: f()
    except Exception as e: print("".join(traceback.format_exception_only(e)).rstrip())
