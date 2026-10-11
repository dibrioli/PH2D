"""Fotografa o ecrã de um KWin VIRTUAL pelo ScreenShot2 (D-Bus da sessão virtual, nunca o do dono).

Uso: python3 -I foto_kwin.py <saida.png>   — precisa de KWIN_SCREENSHOT_NO_PERMISSION_CHECKS=1 no KWin.
"""

import os
import subprocess
import sys

import dbus


def main(out: str) -> None:
    bus = dbus.SessionBus()
    shot = dbus.Interface(
        bus.get_object("org.kde.KWin", "/org/kde/KWin/ScreenShot2"), "org.kde.KWin.ScreenShot2"
    )
    r, w = os.pipe()
    meta = shot.CaptureWorkspace({"native-resolution": True}, dbus.types.UnixFd(w))
    os.close(w)
    raw = bytearray()
    while chunk := os.read(r, 1 << 20):
        raw += chunk
    os.close(r)
    width, height, stride = int(meta["width"]), int(meta["height"]), int(meta["stride"])
    # QImage ARGB32 em little-endian: B, G, R, A por pixel.
    rows = b"".join(bytes(raw[y * stride : y * stride + width * 4]) for y in range(height))
    subprocess.run(
        ["magick", "-size", f"{width}x{height}", "-depth", "8", "bgra:-", out],
        input=rows,
        check=True,
    )


if __name__ == "__main__":
    main(sys.argv[1])
