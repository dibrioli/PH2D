"""Corre DENTRO do Krita (o plugin `ph2dbateria` executa-o depois de a janela nascer): pinta bateria.json com cada
preset por eventos de CANETA espontâneos — `QWindowSystemInterface::handleTabletEvent` por ctypes, o mesmo caminho
que um tablet real percorre (ferramenta de mão livre, sensores de pressão e velocidade, suavização por omissão).

Porque não a API: `Node.paintLine` só aceita QPoint inteiro e cada chamada é um traço novo (o espaçamento e a
dinâmica recomeçam em cada segmento); `paintPath` não tem pressão. `QApplication.sendEvent` com QTabletEvent não
pinta (evento não espontâneo, sem timestamp). Medido 2026-10-19 — 02_ORACULOS.md §Krita.

Trabalho em JSON (PH2D_KRITA_JOB): {"bateria", "saida", "presets": [nomes] | null, "passo_ms", "dt_ms", "paira"}.
Saída: <saida>/<nome_seguro>_<cena>.png (RGBA, o documento exportado) + <saida>/krita_presets.json (nome → ficheiro).
Retoma: o preset cujas 3 PNG já existem é saltado.
"""
import ctypes
import json
import os
import re
import sys
import time
import traceback

from krita import InfoObject, Krita, ManagedColor
from PyQt6 import sip
from PyQt6.QtCore import QPoint, QPointF, QTimer
from PyQt6.QtGui import QInputDevice, QPointingDevice
from PyQt6.QtWidgets import QWidget

JOB = json.load(open(os.environ["PH2D_KRITA_JOB"]))
BAT = json.load(open(JOB["bateria"]))
W, H = BAT["tela"]
PASSO_MS = int(JOB.get("passo_ms", 10))
DT_MS = int(JOB.get("dt_ms", 10))
PAIRA = int(JOB.get("paira", 10))
SAIDA = JOB["saida"]
os.makedirs(SAIDA, exist_ok=True)
LOG = open(os.path.join(SAIDA, "krita_log.txt"), "a")


def log(*a):
    print(*a, file=LOG, flush=True)


gui = ctypes.CDLL("libQt6Gui.so.6")


class PF(ctypes.Structure):
    _fields_ = [("x", ctypes.c_double), ("y", ctypes.c_double)]


_hte = getattr(gui, "_ZN22QWindowSystemInterface17handleTabletEventEP7QWindowmPK15QPointingDeviceRK7QPointFS7_"
                    "6QFlagsIN2Qt11MouseButtonEEddddddS8_INS9_16KeyboardModifierEE")
_hte.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_void_p, ctypes.POINTER(PF), ctypes.POINTER(PF),
                 ctypes.c_int] + [ctypes.c_double] * 6 + [ctypes.c_int]
_hte.restype = ctypes.c_bool
_prox = getattr(gui, "_ZN22QWindowSystemInterface36handleTabletEnterLeaveProximityEventEP7QWindowmPK15QPointingDevice"
                     "bRK7QPointFS7_6QFlagsIN2Qt11MouseButtonEEdddddS8_INS9_16KeyboardModifierEE")
_prox.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_void_p, ctypes.c_bool, ctypes.POINTER(PF),
                  ctypes.POINTER(PF), ctypes.c_int] + [ctypes.c_double] * 5 + [ctypes.c_int]
_reg = getattr(gui, "_ZN22QWindowSystemInterface19registerInputDeviceEPK12QInputDevice")
_reg.argtypes = [ctypes.c_void_p]
DEV = QPointingDevice("ph2d-caneta", 4242, QInputDevice.DeviceType.Stylus, QPointingDevice.PointerType.Pen,
                      QInputDevice.Capability.Position | QInputDevice.Capability.Pressure, 1, 3)
_reg(sip.unwrapinstance(DEV))

k = Krita.instance()
JANELA = k.activeWindow()
JANELA.qwindow().resize(2400, 1500)
JANELA.qwindow().move(0, 0)
RELOGIO = [10_000]
EST = {}


def nome_seguro(s):
    return re.sub(r"[^A-Za-z0-9._+-]", "_", s)


def canvas():
    return [x for x in JANELA.qwindow().findChildren(QWidget)
            if x.metaObject().className() in ("KisOpenGLCanvas2", "KisQPainterCanvas") and x.isVisible()][0]


def para_janela(x, y):
    """Imagem → coordenadas da janela de topo (zoom 1, o centro do canvas é o centro da imagem — medido)."""
    a = EST["alvo"]
    o = a.mapTo(a.window(), QPoint(0, 0))
    return o.x() + a.width() / 2 + (x - W / 2), o.y() + a.height() / 2 + (y - H / 2)


def caneta(x, y, p, botoes, proximidade=None):
    top = EST["alvo"].window()
    lx, ly = para_janela(x, y)
    g = top.mapToGlobal(QPointF(lx, ly))
    RELOGIO[0] += DT_MS
    if proximidade is not None:
        return _prox(sip.unwrapinstance(top.windowHandle()), RELOGIO[0], sip.unwrapinstance(DEV), proximidade,
                     PF(lx, ly), PF(g.x(), g.y()), 0, 0, 0, 0, 0, 0, 0)
    return _hte(sip.unwrapinstance(top.windowHandle()), RELOGIO[0], sip.unwrapinstance(DEV), PF(lx, ly),
                PF(g.x(), g.y()), botoes, float(p), 0, 0, 0, 0, 0, 0)


def cor(rgb):
    c = ManagedColor("RGBA", "U8", "")
    c.setComponents([rgb[2] / 255, rgb[1] / 255, rgb[0] / 255, 1.0])  # BGRA
    return c


def pinta_preset(nome, preset):
    base = os.path.join(SAIDA, nome_seguro(nome))
    for cena in BAT["cenas"]:
        png = f"{base}_{cena['nome']}.png"
        if os.path.exists(png):
            continue
        doc = k.createDocument(W, H, f"bat_{cena['nome']}", "RGBA", "U8", "", 300.0)
        doc.setBatchmode(True)
        view = JANELA.addView(doc)
        view.canvas().setZoomLevel(1.0)
        view.setCurrentBrushPreset(preset)
        yield 300
        EST["alvo"] = canvas()
        caneta(W / 2, H / 2, 0, 0, proximidade=True)
        yield 50
        for op in cena["ops"]:
            if op["op"] == "pincel":
                view.setCurrentBrushPreset(preset)  # base = o mesmo preset (bateria.py)
            elif op["op"] == "cor":
                view.setForeGroundColor(cor(op["rgb"]))
            elif op["op"] == "traco":
                view.setBrushSize(float(op["size_px"]))
                pts = op["pts"]
                RELOGIO[0] += 500
                for _ in range(PAIRA):
                    caneta(pts[0][0], pts[0][1], 0.0, 0)
                    yield PASSO_MS
                caneta(*pts[0], 1)
                yield PASSO_MS
                for x, y, p in pts[1:]:
                    caneta(x, y, p, 1)
                    yield PASSO_MS
                caneta(pts[-1][0], pts[-1][1], 0.0, 0)
                yield 50
                doc.waitForDone()
            elif op["op"] == "foto" and op["nome"] == "A":
                doc.waitForDone()
                doc.refreshProjection()
                ok = doc.exportImage(png, InfoObject())
                log("foto", nome, cena["nome"], ok)
        doc.setModified(False)
        doc.close()
        yield 100


def passos():
    todos = k.resources("preset")
    nomes = JOB.get("presets") or sorted(todos)
    mapa = {n: {"ficheiro": nome_seguro(n), "origem": todos[n].filename()} for n in nomes if n in todos}
    json.dump(mapa, open(os.path.join(SAIDA, "krita_presets.json"), "w"), indent=1, ensure_ascii=False)
    yield 500
    for n in nomes:
        if n not in todos:
            log("AUSENTE", n)
            continue
        t0 = time.time()
        try:
            yield from pinta_preset(n, todos[n])
        except Exception:
            log("FALHA", n, traceback.format_exc())
        log("ok", n, f"{time.time() - t0:.1f}s")
    log("FIM")
    LOG.close()
    os._exit(0)


G = passos()


def anda():
    try:
        d = next(G)
    except StopIteration:
        return
    QTimer.singleShot(d, anda)


QTimer.singleShot(1500, anda)
