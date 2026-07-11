# 27 — Nota-ADR: Sort + Cull (M3 — os operadores ESTRUTURAIS do stream)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: 2/1/8). Adiciona os dois operadores **estruturais** que faltavam:
**reordenar** o stream (`motion.sort`) e **podá-lo** (`motion.cull`). Juntos dão os *reveals ordenados* — a
capacidade que nenhum nó tinha até agora.

---

## 1. O problema

Todo nó até aqui **transforma valores em-lugar** (move/twist/tint/…), **faz crescer a contagem**
(clone/mirror/kaleidoscope) ou é fonte. **Nenhum reordena** e **nenhum poda**. Sem reordenar+podar não há
*reveal ordenado* — o efeito mais básico de motion graphics: uma grade que aparece do centro pra fora, um
texto que dissolve em pontos aleatórios, um build-on por proximidade. É um buraco estrutural, não cosmético.

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.sort` (reordenar por chave):** o padrão-ouro é o **Sort SOP** (Houdini / TouchDesigner) — reordena
pontos por atributo/eixo/distância/expressão, sort **estável**, ascendente. Veredito:

- Cada elemento recebe uma **chave** de `key` — **Radial** (distância² de `(center_x, center_y)`; ao quadrado
  → sem `sqrt` e a MESMA ordem), **X**, **Y**, **Random** (hash splitmix do índice → embaralhamento
  determinístico) ou **Index** (identidade). A **permutação** é ordenada de forma **estável** (ascendente;
  `descending` inverte), e TODAS as colunas (`P`/`size`/`tint`/`id`/…) são reordenadas pela mesma permutação —
  a instância viaja inteira. Contagem inalterada. HR-5: chaves aritméticas (Radial usa distância²) + o hash
  inteiro, **sem trig**. `Effect::Pure`, categoria Utility. Testado por: Radial ordena por distância (raios
  monotônicos, mais perto primeiro) · X ordena por x + descending inverte · **Random é embaralhamento
  determinístico** (mesma seed = mesma ordem, É permutação, ≠ identidade) · cook reordena todas as colunas
  juntas.

**`motion.cull` (podar por predicado):** o padrão-ouro é o **Blast / Delete SOP** (Houdini) — deleta pontos
por critério. Veredito:

- Mantém os elementos que passam no predicado e **filtra todas as colunas**. Dois modos: **Fraction** mantém
  os primeiros `amount·n` (revela na ORDEM upstream → casa com o `sort`); **Falloff** mantém onde a coluna
  `falloff` ≥ `amount` (máscara espacial → casa com `motion.falloff`; coluna ausente = 1). `invert` mantém o
  complemento. Os índices sobreviventes coletam cada coluna → as instâncias mantidas ficam intactas numa
  contagem menor (o **primeiro nó que ENCOLHE** a contagem — mirror/kaleidoscope crescem, sort reordena). Um
  `amount` **value input** (desconectado → o param) anima o reveal, então um `value.lfo` varre. HR-5: contagem
  e comparação apenas. `Effect::Pure`, categoria Utility. Testado por: Fraction mantém os primeiros `amount·n`
  · endpoints 0=vazio/1=tudo · invert mantém o complemento · Falloff mantém acima do limiar · cook: o value
  input dirige a contagem + filtra todas as colunas.

**A composição é o ponto:** `sort` sozinho "não faz nada visível" — ele define a ORDEM que o `cull` revela.
`sort(Radial) → cull(fração crescente)` = **wipe do centro pra fora**; `sort(Random) → cull` = **dissolve**.

## 3. O que foi adicionado (fatia)

**`ph2d-node-motion-sort` (drop-crate, REORDENAR):** `(in) → out`. Permutação estável por `key` (Radial/X/Y/
Random/Index), `descending`, `center`/`seed`. `Pure`, Utility, `hash.rs` (splitmix p/ a chave Random).
Display "Sort".

**`ph2d-node-motion-cull` (drop-crate, PODAR):** `(in, amount?) → out`. Mantém `mode` (Fraction/Falloff),
`amount` (value input anima), `invert`. Filtra todas as colunas. `Pure`, Utility, aritmética pura. Display
"Cull".

**Cena boot — DUAS cenas + um lfo COMPARTILHADO** (`motion_demo_strobe.rs`, 13 nós):

```
ESQUERDA (wipe radial):  grid → sort(Radial) → cull → tint(âmbar) → move(−6) → output
DIREITA  (dissolve):     grid → sort(Random) → cull → tint(ciano) → move(+6) → output
                                            lfo → amount de AMBOS os culls (fan-out de valor)
```

- Duas grades 10×10 idênticas; a ÚNICA diferença é a `key` do sort. Um `value.lfo` (5 s, amount ∈ [0.15,
  0.95]) dirige os **dois** culls → à esquerda a grade preenche **do centro pra fora** (sort Radial), à direita
  **dissolve em pontos espalhados** (sort Random). Mostra também o **fan-out de valor** (um clock, dois
  reveals). É o `sort`+`cull` legível: mesma poda, ordens diferentes.

**Testes (11 unit + 3 integração):** sort (6, inclui 2 do hash: Radial-ordena, X+descending, Random-shuffle-
determinístico, cook-reordena-colunas); cull (5: Fraction-primeiros-N, endpoints, invert-complemento, Falloff-
limiar, cook-filtra-colunas). Integração no shell: `the_radial_wipe_grows_from_the_centre` (contagem varia +
frame esparso = cluster central [span<2] vs cheio largo [span>3.5] + esquerda) · `the_random_dissolve_stays_
spread` (contagem varia + esparso AINDA espalhado [span>3] provando ordem aleatória ≠ radial + direita) ·
`the_default_document_replays_deterministically`.

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-sort`, tipo `motion.sort` | nova | nome novo |
| crate `ph2d-node-motion-cull`, tipo `motion.cull` | nova | nome novo |
| `hash.rs` (splitmix, copiado de scatter/boids) em sort | local | nenhum (leaf) |
| `ph2d-node-registry-init` regenerado (60 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita, 2 cenas + lfo compartilhado, 13 nós) | shell | módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo (só path crates). Machete verde. **Nota:** o gate
`no_tofu_glyphs` pegou 3 setas `→` novas em string literals de teste (cull) — trocadas por `->`. (O gate só
escaneia string literals, não comentários — as setas nos doc-comments da cena passam.)

## 5. O que fica

Com sort+cull, o vocabulário do M3 fecha o eixo **estrutural** (reordenar/podar) além de arranjo/deformação/
simetria/empacotamento. O poço de nós **auto-contidos** (só `ph2d-nodegraph`) está **esgotado**:
- **Distribuição:** `motion.distribute-path` (curva — integra `vector.*`; DEFERIDO).
- **Deformer:** `motion.slit-scan` (amostragem temporal; DEFERIDO).
- **Simulação:** `pin_constraint` (port de constraints; DEFERIDO) · spatial hash (acelera collide/boids) ·
  colisão contra bordas · **motor GPU** (`docs/plans/2026-07-gpu-resident-node-pipeline.md`).
- Straggler do M2: `motion.delay` (precisa do time-scope do editor; DEFERIDO).

> Todo próximo nó exige **outro módulo** (path/vetor, timeline/keyframes, time-scope) ou é a fronteira **GPU**
> (linha foundational dedicada, exige ADR). Hora natural de **integrar** as 12 fatias acumuladas.
