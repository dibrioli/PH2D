# ⭐ W6c — o resumo da rodada: c2 = mediana do último terço da `série`; blocos antes = a última leitura antes do
# renascimento, depois = a última; renascimentos = as linhas do vigia; limpo = `alheios.txt` vazio.
import re, sys, pathlib, statistics as st
base = pathlib.Path(sys.argv[1])
linhas = []
for d in sorted(p for p in base.iterdir() if p.is_dir()):
    log = d / "app.log"
    if not log.exists(): continue
    var = d.name.split("_", 1)[1]
    alheios = (d / "alheios.txt").read_text().strip() if (d / "alheios.txt").exists() else ""
    serie, blocos_antes, blocos_depois, ren = [], None, None, 0
    for l in log.read_text(errors="replace").splitlines():
        m = re.search(r"série g\d+ w\d+ · t ([\d.]+) · c2 ([\d.]+) .* (\d+) blocos", l)
        if m:
            serie.append(float(m.group(2))); b = int(m.group(3))
            if blocos_antes is None or ren == 0: blocos_antes = b
            blocos_depois = b
        elif "a placa degradou a vista" in l:
            ren += 1
    if len(serie) < 30:
        linhas.append((var, d.name, None, ren, blocos_antes, blocos_depois, alheios, len(serie))); continue
    c2 = st.median(serie[2 * len(serie) // 3:])
    linhas.append((var, d.name, c2, ren, blocos_antes, blocos_depois, alheios, len(serie)))
por = {}
for var, nome, c2, ren, ba, bd, alh, n in linhas:
    limpo = not alh and c2 is not None
    print(f"{nome:28} {var:6} c2 {c2 if c2 is None else f'{c2:.3f}':>7} · renasce {ren} · blocos {ba} → {bd} · {n} leituras · {'LIMPO' if limpo else 'SUJO: ' + alh.replace(chr(10), ' ')[:80]}")
    if limpo: por.setdefault(var, []).append((c2, ren, ba, bd))
print()
for var, xs in sorted(por.items()):
    c2s = [x[0] for x in xs]
    print(f"{var:6} {len(xs)} limpos · c2 {' · '.join(f'{c:.3f}' for c in c2s)} · mediana {st.median(c2s):.3f} · ≤11,65 {sum(c <= 11.65 for c in c2s)}/{len(c2s)} · ≥12,2 {sum(c >= 12.2 for c in c2s)} · renasce {[x[1] for x in xs]} · blocos {[f'{x[2]}→{x[3]}' for x in xs]}")
