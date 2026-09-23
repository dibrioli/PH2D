#!/usr/bin/env python3
"""mostra.py — imprime uma fixture do oráculo Health como tabela compacta, passo a passo.

    python3 mostra.py fixtures/b_cooldown.json.gz            # todos os passos
    python3 mostra.py fixtures/b_cooldown.json.gz --acoes    # só passos com acção (+1 vizinho)

Colunas: passo · t(s) · acções · H antes→depois · Sh antes→depois · flags (antes|depois|fim)
· PrevDmg · PrevSh · PrevHeal · CD restante · TimeSinceLastHit · ShieldTimeRemaining.
Flags: D=IsJustDamaged H=IsJustHealed O=IsJustDodged S=IsShieldJustDamaged
       A=IsShieldActive C=IsDamageCooldownActive X=IsDead
"""
import gzip
import json
import sys

LETRAS = [("IsJustDamaged", "D"), ("IsJustHealed", "H"), ("IsJustDodged", "O"),
          ("IsShieldJustDamaged", "S"), ("IsShieldActive", "A"),
          ("IsDamageCooldownActive", "C"), ("IsDead", "X")]


def flags(pub):
    if pub is None:
        return "-------"
    return "".join(l if pub[k] else "." for k, l in LETRAS)


def num(v):
    if isinstance(v, str):
        return v
    return f"{v:g}" if abs(v - round(v)) > 1e-9 else str(int(round(v)))


def main():
    caminho = sys.argv[1]
    so_acoes = "--acoes" in sys.argv
    abrir = gzip.open if caminho.endswith(".gz") else open
    with abrir(caminho, "rt", encoding="utf-8") as fh:
        d = json.load(fh)
    passos = d["passos"]
    manter = set(range(len(passos)))
    if so_acoes:
        manter = set()
        for i, p in enumerate(passos):
            if p["acoes"]:
                manter.update({i - 1, i, i + 1})
    print(f"# {d['cabecalho']['cenario']['nome']} — {d['cabecalho']['cenario']['descricao']}")
    print(f"# controlos: {d['cabecalho']['controlos']}")
    print("passo  t(s)    acções                                 H(a→d)         Sh(a→d)      flags a|d|f               Prev  PrevSh PrevHeal CDrest  TSLH    ShTR")
    for i, p in enumerate(passos):
        if i not in manter:
            continue
        a = p["antes"]["publico"] if p["antes"] else None
        dp = p["depois"]["publico"]
        f = p["fim"]["publico"]
        acoes = ", ".join(f"{x[0]}({num(x[1])})" if len(x) > 1 else x[0] for x in p["acoes"])
        h = f"{num(a['Health']) if a else '?'}→{num(dp['Health'])}"
        sh = f"{num(a['ShieldPoints']) if a else '?'}→{num(dp['ShieldPoints'])}"
        print(f"{p['passo']:>5} {p['tempo_desde_inicio_ms']/1000:6.3f}  {acoes[:38]:<38} {h:<14} {sh:<12} "
              f"{flags(a)}|{flags(dp)}|{flags(f)}  {num(dp['PreviousDamageTaken']):>5} {num(dp['PreviousDamageToShield']):>6} "
              f"{num(dp['PreviousHealAmount']):>8} {dp['DamageCooldownRemaining']:6.3f} {num(round(dp['TimeSinceLastHit'],4)):>6} "
              f"{num(round(dp['ShieldTimeRemaining'],4)):>6}")


if __name__ == "__main__":
    main()
