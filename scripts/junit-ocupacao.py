#!/usr/bin/env python3
"""Para onde foi o tempo de uma suíte, a partir do junit do nextest (um ou vários, p. ex. as fatias da noite).

    python3 scripts/junit-ocupacao.py target/nextest/default/junit.xml [--janela 30] [--top 25]

Mostra parede, soma, os limiares ≥ 60/10/1 s, a soma por binário, os maiores, e a ocupação por janela de
tempo (vagas ocupadas): é assim que se vê um teste a bloquear o escalonador.
"""
import argparse
import collections
import datetime as d
import xml.etree.ElementTree as ET


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("junit", nargs="+")
    ap.add_argument("--janela", type=int, default=30)
    ap.add_argument("--top", type=int, default=25)
    ap.add_argument("--janelas", type=int, default=12, help="quantas janelas mostrar a partir do início")
    a = ap.parse_args()
    linhas = []
    falhas = []
    for f in a.junit:
        for ts in ET.parse(f).getroot().iter("testsuite"):
            for tc in ts.iter("testcase"):
                if tc.find("skipped") is not None:
                    continue
                ini = tc.get("timestamp")
                ini = d.datetime.fromisoformat(ini.replace("Z", "+00:00")).timestamp() if ini else None
                linhas.append((ts.get("name"), tc.get("name"), float(tc.get("time") or 0), ini))
                if tc.find("failure") is not None or tc.find("error") is not None:
                    falhas.append((ts.get("name"), tc.get("name")))
    soma = sum(x[2] for x in linhas)
    com_ini = [x for x in linhas if x[3] is not None]
    t0 = min(x[3] for x in com_ini) if com_ini else 0
    fim = max(x[3] + x[2] for x in com_ini) if com_ini else 0
    print(f"testes {len(linhas)} · soma {soma:.0f} s · parede {fim - t0:.0f} s · falhas {len(falhas)}")
    for b, n in falhas[:40]:
        print(f"  ✗ {b} {n}")
    for lim in (60, 10, 1):
        s = [x for x in linhas if x[2] >= lim]
        print(f"≥ {lim:>2} s: {len(s):5d} testes · {sum(x[2] for x in s):7.0f} s · {100 * sum(x[2] for x in s) / max(soma, 1):.1f} %")
    por = collections.defaultdict(lambda: [0.0, 0])
    for x in linhas:
        por[x[0]][0] += x[2]
        por[x[0]][1] += 1
    print("por binário:")
    for b, (s, n) in sorted(por.items(), key=lambda kv: -kv[1][0])[:15]:
        print(f"  {s:7.0f} s  {n:5d}  {b}")
    print("maiores:")
    for x in sorted(linhas, key=lambda x: -x[2])[: a.top]:
        print(f"  {x[2]:7.1f} s  {x[0][:30]} :: {x[1][-70:]}")
    print(f"ocupação ({a.janela} s por janela): início · arrancaram · vagas ocupadas")
    for i in range(a.janelas):
        ja, jb = t0 + i * a.janela, t0 + (i + 1) * a.janela
        arr = sum(1 for x in com_ini if ja <= x[3] < jb)
        ocup = sum(max(0, min(jb, x[3] + x[2]) - max(ja, x[3])) for x in com_ini) / a.janela
        print(f"  {i * a.janela:6d} s  {arr:6d}  {ocup:5.1f}")


if __name__ == "__main__":
    main()
