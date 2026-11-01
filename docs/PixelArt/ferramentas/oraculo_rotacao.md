# Oráculo da rotação (Fast e RotSprite) — recusas MEDIDAS (01/11)

Fixtura: `crates/ph2d-pixel-core/tests/fixtures/oracle_trial_rotation.json` (Aseprite Trial, caixa preta;
condutor `trial_rotacao.py`). Gate: `crates/ph2d-pixel-core/tests/it/rotation_oracle.rs`. Clean-room: só a
fixtura, a literatura pública (RotSprite de Xenowhirl) e o nosso código.

Sondas (Python, exactas em `Fraction`):
- `sonda_rotacao_fast.py` — réplica da lei Fast de `affine_fast.rs` sobre os 69 casos de bloco ≠ 180°;
  reproduz o resíduo do gate **célula a célula (36 em 10 casos)**.
- `sonda_rotsprite.py` — variantes da reconstrução do RotSprite sobre os 18 casos de bloco.

## 1. Fast: o desempate das 36 células

Medido: há **854** células de empate exacto (o centro cai numa linha da grelha da fonte) nos blocos;
o «um fio à esquerda» acerta **818**. As 36 que falham ficam nos mesmos 10 casos, e DENTRO de cada caso
o Trial escolhe às vezes um lado e às vezes o outro (ex.: r86 a72, componente `u`, `du/dx = 1/3`: a coluna
12 quer o `floor` exacto, as colunas 15 e 18 querem o lado de baixo, menos (15,13)). Isto é a assinatura
de aritmética de PONTO FIXO acumulada por linha (erro de ±1 ulp de 16.16), não de uma regra geométrica.

| Variante testada (tie → lado) | células erradas (69 blocos) | casos |
|---|---:|---:|
| **«um fio à esquerda» (actual)** | **36** | 10 |
| deslocamento infinitesimal em QUALQUER direcção do ecrã (720 direcções) | ≥ 36 (melhor 36, 180°–187°) | — |
| desempate pelo sinal da derivada, por componente (tabela de sinais) | não separa: as duas escolhas aparecem para o mesmo sinal | — |
| 16.16 acumulado da caixa: início exacto/floor/round/trunc × passo trunc/floor/round/away × linha exacta/acumulada (48) | ≥ 272 | ≥ 31 |
| varredura por linha à Allegro (borda esquerda interpolada em 16.16, `fixmul`/`fixdiv` round/floor/trunc, acumular linha/pixel) (36) | ≥ 320 | ≥ 36 |
| idem + centro `0x7fff`/`0x8000` em x e y, extensão da fonte `w·1` ou `w·1 − 1 ulp`, 1.º pixel `ceil(l−½)`/`round(l)` (576) | **40** (melhor: `fixmul` round, `fixdiv` trunc, centro x `0x7fff`, y `0x8000`, extensão exacta, 1.º pixel `round`) | 13 |

⇒ **Não achado.** O mais perto de uma explicação é a varredura de ponto fixo com centro em `x + ½ − 1 ulp`
(dá o «fio à esquerda» de graça), a 4 células do actual; o que falta é o percurso exacto da borda (que
canto é o topo, onde reinicia o passo) — a próxima sonda deve variar ISSO, não a direcção do desempate.
Os 36 continuam o resíduo declarado (`FAST_RESIDUAL`).

## 2. RotSprite: distância ao Trial

Antes (gate, `ROTSPRITE_GAP`): **1 554** células no total — blocos 623 (r86 363, r75 260), `size_` 471,
`xform_` 105. O EPX dos blocos de ids distintos SEM borda é o nearest puro (nenhum vizinho igual): o
×8 actual equivale ao Fast amostrado no centro, por isso não encolhe a cobertura como o Trial.

Variantes (18 blocos, mapeamento = inversa exacta do giro em volta do centro, sem assentar):

| borda transparente | escala | amostra | total | nota |
|---:|---:|---|---:|---|
| 0 | 2×/4×/8× | `floor` (canto) | 602 | ≈ o actual (623 com assentar + homografia) |
| 0 | 8× | `round` (centro da sub-célula) | 576 | |
| 0 | 4× | `round` | 565 | |
| 0 | 2× | `round` | 520 | |
| 1 | 8× | `floor` | 579 | |
| 1 | 8× | `round` | 558 | |
| 1 | 4× | `floor` / `round` | 586 / 549 | |
| **1** | **2×** | **`round`** | **513** | melhor; k20 (inclinação) 0–1; mas sobe r75 a45 (20→24) e a150 (20→35) |

⇒ **Nenhuma variante baixa TODOS os casos**, e o gate só aceita baixar; a melhor (−110 nos blocos) troca
casos. Falta medir as mesmas variantes nos `size_`/`xform_` em Rust antes de mudar o motor.
`ROTSPRITE_GAP` fica como está (1 554).
