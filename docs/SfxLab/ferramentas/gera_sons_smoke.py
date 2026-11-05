#!/usr/bin/env python3
"""Os três sons da cena de smoke de W1, bem diferentes de ouvido e no primeiro bloco do mixer.

Sintetizados aqui, sem amostra de terceiros: grave com queda de altura, sino agudo e chiado. Não são
material de produção; existem para o ouvido do dono distinguir qual saiu.

Corra: python3 docs/SfxLab/ferramentas/gera_sons_smoke.py
"""

import math
import random
import struct
import wave
from pathlib import Path

SR = 48_000
PASTA = Path(__file__).resolve().parent.parent / "arte" / "sons"


def grava(nome, amostras):
    PASTA.mkdir(parents=True, exist_ok=True)
    with wave.open(str(PASTA / nome), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes(
            b"".join(
                struct.pack("<h", int(max(-1.0, min(1.0, a)) * 32_000)) for a in amostras
            )
        )


def grave():
    n, fase, out = int(0.28 * SR), 0.0, []
    for i in range(n):
        t = i / SR
        fase += math.tau * (70 + 150 * math.exp(-t * 22)) / SR
        out.append(0.9 * math.exp(-t * 11) * math.sin(fase))
    return out


def agudo():
    n, out = int(0.35 * SR), []
    for i in range(n):
        t = i / SR
        out.append(
            math.exp(-t * 9)
            * (0.5 * math.sin(math.tau * 2093 * t) + 0.35 * math.sin(math.tau * 3136 * t))
        )
    return out


def chiado():
    n, out, rng, passa = int(0.18 * SR), [], random.Random(5), 0.0
    for i in range(n):
        t = i / SR
        ruido = rng.uniform(-1.0, 1.0)
        passa += 0.35 * (ruido - passa)
        out.append(0.8 * math.exp(-t * 20) * (ruido - passa))
    return out


if __name__ == "__main__":
    grava("coin_low.wav", grave())
    grava("coin_high.wav", agudo())
    grava("coin_noise.wav", chiado())
