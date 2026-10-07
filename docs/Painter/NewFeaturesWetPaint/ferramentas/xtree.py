# Lista as janelas X MAPEADAS do ecrã virtual (filhas da raiz, inclui popups override-redirect) com geometria,
# via libX11 por ctypes — não há xwininfo nesta máquina. uso: xtree.py :1 [filtro WxH]
import ctypes, sys
X = ctypes.CDLL("libX11.so.6")
X.XOpenDisplay.restype = ctypes.c_void_p
X.XDefaultRootWindow.argtypes = [ctypes.c_void_p]; X.XDefaultRootWindow.restype = ctypes.c_ulong
X.XQueryTree.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.POINTER(ctypes.c_ulong), ctypes.POINTER(ctypes.c_ulong), ctypes.POINTER(ctypes.POINTER(ctypes.c_ulong)), ctypes.POINTER(ctypes.c_uint)]
class XWA(ctypes.Structure):
    _fields_ = [("x", ctypes.c_int), ("y", ctypes.c_int), ("width", ctypes.c_int), ("height", ctypes.c_int), ("border_width", ctypes.c_int), ("depth", ctypes.c_int), ("visual", ctypes.c_void_p), ("root", ctypes.c_ulong), ("class_", ctypes.c_int), ("bit_gravity", ctypes.c_int), ("win_gravity", ctypes.c_int), ("backing_store", ctypes.c_int), ("backing_planes", ctypes.c_ulong), ("backing_pixel", ctypes.c_ulong), ("save_under", ctypes.c_int), ("colormap", ctypes.c_ulong), ("map_installed", ctypes.c_int), ("map_state", ctypes.c_int), ("all_event_masks", ctypes.c_long), ("your_event_mask", ctypes.c_long), ("do_not_propagate_mask", ctypes.c_long), ("override_redirect", ctypes.c_int), ("screen", ctypes.c_void_p)]
X.XGetWindowAttributes.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.POINTER(XWA)]
X.XFetchName.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.POINTER(ctypes.c_char_p)]
d = X.XOpenDisplay(sys.argv[1].encode()); root = X.XDefaultRootWindow(d)
filtro = sys.argv[2] if len(sys.argv) > 2 else None
def walk(w, prof=0):
    r = ctypes.c_ulong(); p = ctypes.c_ulong(); ch = ctypes.POINTER(ctypes.c_ulong)(); n = ctypes.c_uint()
    if not X.XQueryTree(d, w, ctypes.byref(r), ctypes.byref(p), ctypes.byref(ch), ctypes.byref(n)): return
    for i in range(n.value):
        c = ch[i]; a = XWA(); X.XGetWindowAttributes(d, c, ctypes.byref(a))
        if a.map_state == 2 and a.width > 1 and a.height > 1:
            nm = ctypes.c_char_p(); X.XFetchName(d, c, ctypes.byref(nm))
            g = f"{a.width}x{a.height}+{a.x}+{a.y}"
            if filtro is None or filtro in g:
                print(f"0x{c:x} {g} or={a.override_redirect} {nm.value.decode(errors='replace') if nm.value else ''}")
        if prof < 2: walk(c, prof + 1)
walk(root)
