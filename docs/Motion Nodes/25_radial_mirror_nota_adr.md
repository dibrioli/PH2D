# 25 — Nota-ADR: Radial Array + Mirror (M3 — array polar + simetria)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: 2/1/8). Adiciona o **array radial** (a distribuição polar que faltava)
e o **mirror/simetria** (o modifier que faltava). A cena boot também **exercita o voronoi a count 180** —
o número que o Enio viu cair pra 20fps — pra conferir ao vivo a paralelização.

---

## 1. O problema

Tínhamos distribuições retangulares (grid), espiral (fibonacci), blue-noise (scatter), hexagonal (lattice) e
CVT (voronoi) — mas nenhuma **radial/polar** (anéis, sunburst, clock-face, radial cloner do C4D). E nenhum
**mirror/simetria** (o modifier universal que espelha um layout). Além disso, o Enio pediu **ver o voronoi no
próximo exemplo pra checar se a perf melhorou** (a paralelização rayon).

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.distribute_radial` (o array polar):** o padrão-ouro é o **radial array regular** (anéis
concêntricos, pontos igualmente espaçados em ângulo). Veredito:

- `count` pontos divididos o mais uniformemente possível entre `rings` anéis (raios de `inner` a `radius`);
  em cada anel, ângulo = `k/n_ring + spin`. Um **`spin` value input** (graus) gira o array inteiro → um
  `value.lfo` balança. HR-5: os ângulos usam `cos_sin_cycles` (seno parabólico copiado de `motion.orbit`,
  ~0.09% erro), **sem `sin`/`cos`**. Testado por: **anel único = raio constante + espaçamento igual** (chords
  iguais; falsifica uma espiral) · anéis concêntricos entre inner/radius · **count exato** mesmo dividindo
  desigual · **spin gira** (¼ de volta leva +x → +y). `Effect::Pure`, Source.

**`motion.mirror` (o espelho):** o padrão-ouro é a **reflexão de eixo** que duplica (Symmetry/Mirror de todo
pacote 2D/3D). Veredito:

- Cada elemento fica, e uma cópia refletida é adicionada: eixo **vertical** `(x,y)→(2cx−x, y)`, **horizontal**
  `(x,y)→(x, 2cy−y)`, com `(cx,cy)` = centroide. `count → 2·count`, as duas metades espelho. **Só `P` é
  refletido**; as outras colunas (`size`/`tint`/`id`) copiam no gêmeo — espelho do *layout* (exato p/ uma
  distribuição posicional; `vel`/`rot` de um sim são duplicados, não flipados — nota honesta). HR-5:
  aritmética, **sem trig/sqrt**. Testado por: **conta dobra + reflete x** (vertical) / **reflete y**
  (horizontal), centroide preservado (simétrico), cook duplica todas as colunas. `Effect::Pure`, Transform.

## 3. O que foi adicionado (fatia)

**`ph2d-node-motion-distribute-radial` (drop-crate, o ARRAY RADIAL):** `(spin?) → out`. `count` pontos em
`rings` anéis, `spin` value input gira. `Pure`, Source, `trig.rs` (cos_sin_cycles). Display "Radial Array".

**`ph2d-node-motion-mirror` (drop-crate, o ESPELHO):** `(in) → out`. Reflete `P` + duplica todas as colunas
no eixo `axis` (V/H) pelo centroide. `Pure`, Transform, sem trig/sqrt. Display "Mirror".

**Cena boot — DUAS cenas** (`motion_demo_strobe.rs`, 11 nós):

```
ESQUERDA (radial): distribute_radial → move(−6) → tint(âmbar) → output   lfo → spin
DIREITA  (mirror): voronoi(180) → mirror → move(+6) → tint(ciano) → output  lfo → relax
```

- **radial** (x≈−6): 48 pontos em 3 anéis, `spin` (`value.lfo` ±180°) balança o array.
- **voronoi(180) + mirror** (x≈+6): **o voronoi roda a count 180 com `relax` animado** (Lloyd re-rodado todo
  frame — agora **paralelizado**, suave a esse count; era o caso que caía pra 20fps), e `motion.mirror`
  reflete no eixo vertical → colmeia simétrica de **360 pontos**. **É o teste de perf ao vivo.**

**Testes (11 unit + 3 integração):** radial (7, inclui 2 do trig: anel-único, concêntrico, count-exato,
spin-gira; falsificados); mirror (4: dobra+reflete-x, reflete-y, simétrico, cook duplica colunas). Integração
no shell: `the_radial_array_swings_round` (viaja + esquerda) · `the_voronoi_is_mirrored_and_relaxes`
(**360 pontos** = 180 espelhado + simétrico [skew≈0] + relaxação viva + direita) ·
`the_default_document_replays_deterministically`.

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-distribute-radial`, tipo `motion.distribute_radial` | nova | nome novo |
| crate `ph2d-node-motion-mirror`, tipo `motion.mirror` | nova | nome novo |
| `trig.rs` (copiado de `motion.orbit`) em radial | local | nenhum (leaf) |
| `ph2d-node-registry-init` regenerado (56 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita, 2 cenas, 11 nós) | shell | módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo (só path crates). Machete verde.

## 5. O que fica

As distribuições (grid/fibonacci/scatter/lattice/voronoi/radial) e os deformers auto-contidos do M3
(twist/morph/bend/look_at/four_point_warp/spherize) estão **completos**. O que resta:
- **Distribuição:** `motion.distribute-path` (curva — integra vector.*; DEFERIDO). N-fold radial symmetry
  (kaleidoscope) seria uma extensão do mirror.
- **Deformer:** `motion.slit-scan` (amostragem temporal; DEFERIDO).
- **Simulação:** `pin_constraint` (port de constraints; DEFERIDO) · spatial hash · colisão · **motor GPU**
  (`docs/plans/2026-07-gpu-resident-node-pipeline.md`).
- Straggler do M2: `motion.delay` (precisa do time-scope do editor, como `trail`/`time_remap`; DEFERIDO).

> O vocabulário de arranjo do M3 fecha: distribuições (retangular, espiral, blue-noise, hex, CVT, radial),
> deformers (rotação, interpolação, arc, orientação, corner-pin, lente), e o modifier de simetria. O que resta
> depende de outros módulos (path/vector, slit-scan/delay temporais, pin) ou é o motor GPU.
