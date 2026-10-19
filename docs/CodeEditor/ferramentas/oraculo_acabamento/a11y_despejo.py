"""Despeja a árvore AT-SPI (papel, nome acessível, descrição, estados, caixa no ecrã) de cada app no bus
de acessibilidade da sessão dbus PRIVADA do oráculo. Uso: python3 -I a11y_despejo.py <saida.txt> [max_nos]
O ambiente (DBUS_SESSION_BUS_ADDRESS, DISPLAY, XDG_RUNTIME_DIR) vem do corre.sh — nunca o do dono."""
import sys
import gi
gi.require_version("Atspi", "2.0")
from gi.repository import Atspi

saida, maxn = sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 20000
linhas, n = [], 0


def caixa(o):
    try:
        e = o.get_extents(Atspi.CoordType.SCREEN)
        return f"[{e.x},{e.y} {e.width}x{e.height}]"
    except Exception:
        return "[?]"


def estados(o):
    try:
        ss = o.get_state_set().get_states()
        return ",".join(Atspi.StateType(s).value_nick for s in ss if Atspi.StateType(s).value_nick in
                        ("focused", "selected", "focusable", "selectable", "expanded", "collapsed", "showing", "active"))
    except Exception:
        return "?"


def anda(o, prof):
    global n
    if n >= maxn or o is None:
        return
    n += 1
    try:
        papel, nome = o.get_role_name(), o.get_name()
        desc = o.get_description()
    except Exception as ex:
        linhas.append("  " * prof + f"<erro {ex}>")
        return
    linhas.append("  " * prof + f"{papel} «{nome}»" + (f" desc«{desc}»" if desc else "") + f" {caixa(o)} {{{estados(o)}}}")
    try:
        k = o.get_child_count()
    except Exception:
        k = 0
    for i in range(k):
        try:
            anda(o.get_child_at_index(i), prof + 1)
        except Exception as ex:
            linhas.append("  " * (prof + 1) + f"<erro filho {i}: {ex}>")


d = Atspi.get_desktop(0)
linhas.append(f"desktop: {d.get_child_count()} apps")
for i in range(d.get_child_count()):
    anda(d.get_child_at_index(i), 0)
open(saida, "w").write("\n".join(linhas) + "\n")
print(f"{n} nós -> {saida}")
