# Injetor de rato/teclado para o Rebelle, DENTRO do Wine (SendInput) — nao passa pelo servidor de ecra.
# uso: python.exe inj.py "acao; acao; ..."
#   move X Y · click X Y · rclick X Y · dclick X Y · drag X1 Y1 X2 Y2 [passos]
#   path X1 Y1 X2 Y2 X3 Y3 ... (arrasto por uma polilinha) · key ctrl+s · type texto · wait ms · wheel N
import ctypes, sys, time
from ctypes import wintypes

user32 = ctypes.windll.user32
INPUT_MOUSE, INPUT_KEYBOARD = 0, 1
MOVE, ABS, LDOWN, LUP, RDOWN, RUP, WHEEL = 0x1, 0x8000, 0x2, 0x4, 0x8, 0x10, 0x800
KEYUP = 0x2
ULONG_PTR = ctypes.c_size_t


class MOUSEINPUT(ctypes.Structure):
    _fields_ = [("dx", wintypes.LONG), ("dy", wintypes.LONG), ("mouseData", wintypes.DWORD),
                ("dwFlags", wintypes.DWORD), ("time", wintypes.DWORD), ("dwExtraInfo", ULONG_PTR)]


class KEYBDINPUT(ctypes.Structure):
    _fields_ = [("wVk", wintypes.WORD), ("wScan", wintypes.WORD), ("dwFlags", wintypes.DWORD),
                ("time", wintypes.DWORD), ("dwExtraInfo", ULONG_PTR)]


class _U(ctypes.Union):
    _fields_ = [("mi", MOUSEINPUT), ("ki", KEYBDINPUT), ("pad", ctypes.c_byte * 32)]


class INPUT(ctypes.Structure):
    _fields_ = [("type", wintypes.DWORD), ("u", _U)]


SW, SH = user32.GetSystemMetrics(0), user32.GetSystemMetrics(1)


def mouse(flags, x=0, y=0, data=0):
    i = INPUT(type=INPUT_MOUSE)
    if flags & ABS:
        i.u.mi.dx = int(x * 65535 / (SW - 1))
        i.u.mi.dy = int(y * 65535 / (SH - 1))
    i.u.mi.dwFlags = flags
    i.u.mi.mouseData = data
    user32.SendInput(1, ctypes.byref(i), ctypes.sizeof(INPUT))


def move(x, y):
    mouse(MOVE | ABS, x, y)
    time.sleep(0.01)


def click(x, y, down=LDOWN, up=LUP):
    move(x, y)
    mouse(down | MOVE | ABS, x, y)
    time.sleep(0.05)
    mouse(up | MOVE | ABS, x, y)
    time.sleep(0.1)


PASSO_S = 0.02


def path(pts, passos=12):
    move(*pts[0])
    time.sleep(0.1)
    mouse(LDOWN)
    time.sleep(0.1)
    for (a, b) in zip(pts, pts[1:]):
        for k in range(1, passos + 1):
            f = k / passos
            mouse(MOVE | ABS, a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f)
            time.sleep(PASSO_S)
    time.sleep(0.1)
    mouse(LUP)
    time.sleep(0.3)


VK = {"ctrl": 0x11, "shift": 0x10, "alt": 0x12, "enter": 0x0D, "esc": 0x1B, "tab": 0x09,
      "del": 0x2E, "space": 0x20, "back": 0x08, "up": 0x26, "down": 0x28, "left": 0x25, "right": 0x27}
for c in "abcdefghijklmnopqrstuvwxyz0123456789":
    VK[c] = ord(c.upper())
for n in range(1, 13):
    VK["f%d" % n] = 0x6F + n


def key(vk, up=False):
    i = INPUT(type=INPUT_KEYBOARD)
    i.u.ki.wVk = vk
    i.u.ki.dwFlags = KEYUP if up else 0
    user32.SendInput(1, ctypes.byref(i), ctypes.sizeof(INPUT))


def combo(s):
    ks = [VK[k] for k in s.lower().split("+")]
    for k in ks:
        key(k)
        time.sleep(0.03)
    for k in reversed(ks):
        key(k, True)
        time.sleep(0.03)
    time.sleep(0.15)


def typ(s):
    for ch in s:
        r = user32.VkKeyScanW(ord(ch))
        vk, sh = r & 0xFF, (r >> 8) & 1
        if sh:
            key(0x10)
        key(vk)
        key(vk, True)
        if sh:
            key(0x10, True)
        time.sleep(0.02)


for acao in " ".join(sys.argv[1:]).split(";"):
    p = acao.split()
    if not p:
        continue
    c, a = p[0], p[1:]
    if c == "move":
        move(float(a[0]), float(a[1]))
    elif c == "click":
        click(float(a[0]), float(a[1]))
    elif c == "rclick":
        click(float(a[0]), float(a[1]), RDOWN, RUP)
    elif c == "dclick":
        click(float(a[0]), float(a[1]))
        click(float(a[0]), float(a[1]))
    elif c == "drag":
        n = int(a[4]) if len(a) > 4 else 20
        path([(float(a[0]), float(a[1])), (float(a[2]), float(a[3]))], n)
    elif c == "drag2":
        n = int(a[4]) if len(a) > 4 else 20
        x0, y0, x1, y1 = [float(v) for v in a[:4]]
        move(x0, y0); time.sleep(0.1)
        mouse(LDOWN | MOVE | ABS, x0, y0); time.sleep(0.1)
        for k in range(1, n + 1):
            f = k / n
            mouse(MOVE | ABS, x0 + (x1 - x0) * f, y0 + (y1 - y0) * f); time.sleep(PASSO_S)
        time.sleep(0.1); mouse(LUP | MOVE | ABS, x1, y1); time.sleep(0.3)
    elif c == "down":
        move(float(a[0]), float(a[1])); time.sleep(0.05); mouse(LDOWN | MOVE | ABS, float(a[0]), float(a[1]))
    elif c == "up":
        mouse(LUP | MOVE | ABS, float(a[0]), float(a[1]))
    elif c == "pace":
        PASSO_S = float(a[0])
    elif c == "shot":
        # shot <ficheiro.bmp> [titulo]: captura por GDI (dentro do Wine) a janela cujo titulo contem <titulo>, ou o ecra todo.
        gdi32 = ctypes.windll.gdi32
        alvo = " ".join(a[1:]).lower()
        hw = [0]
        if alvo:
            CB2 = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
            def cb2(h, _):
                n = user32.GetWindowTextLengthW(h)
                if n:
                    b = ctypes.create_unicode_buffer(n + 1); user32.GetWindowTextW(h, b, n + 1)
                    if alvo in b.value.lower() and user32.IsWindowVisible(h):
                        hw[0] = h; return False
                return True
            user32.EnumWindows(CB2(cb2), 0)
        r = wintypes.RECT()
        if hw[0]:
            user32.GetWindowRect(hw[0], ctypes.byref(r)); x0, y0, w, h = r.left, r.top, r.right - r.left, r.bottom - r.top
        else:
            x0, y0, w, h = 0, 0, SW, SH
        sdc = user32.GetDC(0); mdc = gdi32.CreateCompatibleDC(sdc); bmp = gdi32.CreateCompatibleBitmap(sdc, w, h)
        gdi32.SelectObject(mdc, bmp); gdi32.BitBlt(mdc, 0, 0, w, h, sdc, x0, y0, 0x00CC0020)
        class BIH(ctypes.Structure):
            _fields_ = [("biSize", wintypes.DWORD), ("biWidth", wintypes.LONG), ("biHeight", wintypes.LONG), ("biPlanes", wintypes.WORD), ("biBitCount", wintypes.WORD), ("biCompression", wintypes.DWORD), ("biSizeImage", wintypes.DWORD), ("biXPelsPerMeter", wintypes.LONG), ("biYPelsPerMeter", wintypes.LONG), ("biClrUsed", wintypes.DWORD), ("biClrImportant", wintypes.DWORD)]
        bi = BIH(); bi.biSize = ctypes.sizeof(BIH); bi.biWidth = w; bi.biHeight = -h; bi.biPlanes = 1; bi.biBitCount = 32; bi.biCompression = 0
        buf = ctypes.create_string_buffer(w * h * 4)
        gdi32.GetDIBits(mdc, bmp, 0, h, buf, ctypes.byref(bi), 0)
        import struct as st
        hdr = b"BM" + st.pack("<IHHI", 14 + 40 + w * h * 4, 0, 0, 54)
        bi.biHeight = -h
        with open(a[0], "wb") as f:
            f.write(hdr); f.write(bytes(bi)); f.write(buf.raw)
        gdi32.DeleteObject(bmp); gdi32.DeleteDC(mdc); user32.ReleaseDC(0, sdc)
        print("shot", a[0], x0, y0, w, h, "hwnd", hw[0])
    elif c == "rel":
        n = int(a[2]) if len(a) > 2 else 1
        for _ in range(n):
            mouse(MOVE, 0, 0) if False else None
            i = INPUT(type=INPUT_MOUSE); i.u.mi.dx = int(float(a[0])); i.u.mi.dy = int(float(a[1])); i.u.mi.dwFlags = MOVE
            user32.SendInput(1, ctypes.byref(i), ctypes.sizeof(INPUT)); time.sleep(PASSO_S)
    elif c == "fg":
        alvo = " ".join(a).lower(); hw = [0]
        CB3 = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
        def cb3(h, _):
            n = user32.GetWindowTextLengthW(h)
            if n:
                b = ctypes.create_unicode_buffer(n + 1); user32.GetWindowTextW(h, b, n + 1)
                if alvo in b.value.lower() and user32.IsWindowVisible(h):
                    hw[0] = h; return False
            return True
        user32.EnumWindows(CB3(cb3), 0)
        if hw[0]:
            r1 = user32.SetForegroundWindow(hw[0]); user32.SetActiveWindow(hw[0]); user32.BringWindowToTop(hw[0])
            fgw = user32.GetForegroundWindow()
            print("fg", hw[0], "SetForegroundWindow", r1, "foreground agora", fgw, "capture", user32.GetCapture())
        else:
            print("fg: janela nao achada")
    elif c == "path":
        v = [float(t) for t in a]
        path(list(zip(v[0::2], v[1::2])))
    elif c == "key":
        combo(a[0])
    elif c == "type":
        typ(" ".join(a))
    elif c == "wait":
        time.sleep(float(a[0]) / 1000)
    elif c == "rect":
        alvo = " ".join(a)
        achados = []
        CB = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
        def cb(h, _):
            n = user32.GetWindowTextLengthW(h)
            if n:
                b = ctypes.create_unicode_buffer(n + 1)
                user32.GetWindowTextW(h, b, n + 1)
                if alvo.lower() in b.value.lower() and user32.IsWindowVisible(h):
                    r = wintypes.RECT()
                    user32.GetWindowRect(h, ctypes.byref(r))
                    c = wintypes.RECT()
                    user32.GetClientRect(h, ctypes.byref(c))
                    pt = wintypes.POINT(0, 0)
                    user32.ClientToScreen(h, ctypes.byref(pt))
                    achados.append("%s | janela %d,%d,%d,%d | cliente %d,%d %dx%d" % (b.value, r.left, r.top, r.right, r.bottom, pt.x, pt.y, c.right, c.bottom))
            return True
        user32.EnumWindows(CB(cb), 0)
        print("\n".join(achados) or "nenhuma")
    elif c == "wheel":
        mouse(WHEEL, data=int(a[0]) * 120)
print("ok", SW, SH)
