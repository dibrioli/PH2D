"""Foto da tela VIRTUAL do KWin pelo ScreenShot2 do D-Bus DELA (nunca o do dono: recusa o bus de quem
chamou o roteiro, passado em DONO_BUS). Uso: python3 -I foto_kwin.py <saida.png>"""
import os, subprocess, sys
import dbus

saida = sys.argv[1]
bus_addr = os.environ.get("DBUS_SESSION_BUS_ADDRESS", "")
if not bus_addr or bus_addr == os.environ.get("DONO_BUS", ""):
    sys.exit("RECUSA: este D-Bus pode ser o do dono")
r, w = os.pipe()
obj = dbus.SessionBus().get_object("org.kde.KWin", "/org/kde/KWin/ScreenShot2")
meta = obj.CaptureWorkspace({"native-resolution": True}, dbus.types.UnixFd(w),
                            dbus_interface="org.kde.KWin.ScreenShot2")
os.close(w)
dados = b""
while True:
    b = os.read(r, 1 << 20)
    if not b:
        break
    dados += b
largura, altura, passo = int(meta["width"]), int(meta["height"]), int(meta["stride"])
linhas = b"".join(dados[y * passo:y * passo + largura * 4] for y in range(altura))
subprocess.run(["magick", "-size", f"{largura}x{altura}", "-depth", "8", "bgra:-", saida],
               input=linhas, check=True)
print(f"foto {os.path.basename(saida)} {largura}x{altura}")
