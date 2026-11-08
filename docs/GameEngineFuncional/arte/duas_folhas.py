"""As duas folhas da cena «Two Sheets» (PH2D_GAME_SMOKE=5, plano GEF 26 V6): idle 128x32 (4x1 de 32 px, verde) e
run 288x96 (6x2 de 48 px, laranja). Fundo transparente: os pés de cada quadro tocam o fundo da célula, e o olho
branco anda com o número do quadro. Corra: python3 duas_folhas.py"""
import struct
import zlib


def png(path, w, h, f):
    raw = b"".join(b"\0" + b"".join(bytes(f(x, y)) for x in range(w)) for y in range(h))
    c = lambda t, d: struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)
    open(path, "wb").write(b"\x89PNG\r\n\x1a\n" + c(b"IHDR", ihdr) + c(b"IDAT", zlib.compress(raw)) + c(b"IEND", b""))


def boneco(cell, cor, n_quadros, colunas):
    """Um boneco INTEIRO por célula (cabeça, perna e pés ligados): a grelha que a transparência propõe é a certa."""
    def f(x, y):
        i = (y // cell) * colunas + x // cell
        u, v = x % cell, y % cell
        s = cell / 32.0
        bob = [0, 1, 2, 1][i % 4] * s
        cx, cy, r = cell / 2, 13 * s + bob, 9 * s
        passo = [0, 2, 4, 2][i % 4] * s
        pes = cell - 4 * s <= v < cell and cx - 6 * s - passo <= u <= cx + 6 * s + passo
        perna = abs(u - cx) <= 2 * s and cy <= v < cell
        ex = cx - 5 * s + 10 * s * (i % n_quadros) / max(n_quadros - 1, 1)
        olho = abs(u - ex) <= 1.5 * s and abs(v - (cy - 2 * s)) <= 1.5 * s
        if (u - cx) ** 2 + (v - cy) ** 2 <= r * r or pes or perna:
            return (255, 255, 255, 255) if olho else cor + (255,)
        return (0, 0, 0, 0)
    return f


png("idle.png", 128, 32, boneco(32, (110, 205, 140), 4, 4))
png("run.png", 288, 96, boneco(48, (230, 130, 80), 12, 6))
