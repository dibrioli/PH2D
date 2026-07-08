# ADR-0109 — Exceção sancionada ao "sem rayon": composite óptico do Watercolor paralelizado (byte-idêntico, replay-safe)

- **Status:** ACEITO (Enio, 2026-07-07).
- **Data:** 2026-07-07.
- **Escopo:** habilita `rayon` (thread-pool data-parallel) em `ph2d-tool-painter`, restrito ao composite
  óptico de aquarela — o loop por-pixel (`tool/paint/watercolor_render.rs::apply_watercolor`) **e** o
  `box_blur` separável que ele usa (`tool/paint/watercolor_field.rs`). **Não** abre `rayon`/threading para
  o resto do codebase.
- **Não afeta:** nenhum contrato congelado (Nodes [ADR-0039](0039-nodegraph-contract-freeze-w2t4.md),
  Tools [ADR-0040](0040-tool-as-isolated-feature-crate.md), Vector). Nenhuma mudança de ABI. **Resultado da
  pintura byte-idêntico** (prova em §3).

## 1. Contexto

O motor de aquarela reconstrói a aparência do traço opticamente por frame (Beer–Lambert por-canal + rewet +
granulação + un-premultiply). Auditoria de perf (2026-07-07, `docs/Painter/12_…` + medições de ablação em
release) mostrou que **com brush > 200 px e vários recursos ligados** o custo é dominado por trabalho
**O(janela)** single-thread: o loop por-pixel do composite (piso ~6 ms/frame a R=220) e o bake do pen-up
(238 ms com tudo ligado) — queda de FPS visível + freeze no soltar da caneta.

As alavancas byte-idênticas **single-thread** foram esgotadas: o cache do substrato (paper_h canvas-anchored)
landou e removeu o custo do papel ([commit anterior]); o reuso de buffers foi **medido e refutado** (os spikes
são compute-bound, não de alocação). O que sobrava para cortar o piso O(janela) sem mudar o resultado era
**paralelismo**.

O codebase mantém, **de propósito**, uma disciplina "sem rayon" documentada em três crates
(`ph2d-imageio-png`, `ph2d-vector-doc`, `ph2d-render`). O motivo canônico está no `ph2d-vector-doc`:
*"the default features (rayon parallel SIMD) are off — pure-Rust path keeps the crate small **for
deterministic replay**."* Ou seja: a regra existe para garantir **replay bit-reproduzível** (gate
`replay-hash` no CI), não por medo de paralelismo em si.

## 2. Decisão

Paralelizar **apenas** o loop de saída do composite de aquarela sobre **linhas de saída disjuntas**, via
`rayon::par_chunks_mut`, e registrar isto como **exceção explícita** à disciplina "sem rayon".

Por que esta exceção é segura para o replay determinístico (os 3 invariantes que a qualificam):

1. **Sem redução entre pixels.** Cada pixel de saída é função **pura** de entradas imutáveis (base/ground
   congelados, campos de cobertura/blur, cache do substrato, LUTs). Não há acumulação/soma cuja ordem entre
   threads pudesse mudar o resultado em float.
2. **Sem estado mutável compartilhado.** Cada task escreve **só** os pixels da sua própria linha
   (`row[gx*4..]`, fatias disjuntas do canvas). O cache do substrato é preenchido num **pré-passo serial**
   antes do paralelo, então o loop paralelo só o **lê**.
3. **Sem RNG / sem transcendental no hot-loop.** O value-noise é hash inteiro determinístico; nenhuma fonte
   de não-determinismo por-thread.

Como IEEE-754 é determinístico por-operação e as operações por-pixel são idênticas às da versão serial, a
saída é **bit-idêntica independentemente do número de cores ou do agendamento** — logo o `replay-hash`
permanece estável. **É isto que qualifica a exceção:** ela não introduz o não-determinismo que a regra
"sem rayon" existe para prevenir.

**Cerca de contenção (para não erodir a regra):**
- `rayon` entra **só** em `ph2d-tool-painter`, com comentário no `Cargo.toml` apontando este ADR.
- Uso restrito ao `apply_watercolor`. Qualquer novo uso de `rayon`/threading (nesta ou em outra crate),
  **em especial se envolver redução/acumulação cuja ordem importe** (somar contribuições, folds, `reduce`),
  **exige novo ADR** — não se ampara neste.
- O comentário no código (`watercolor_render.rs`) cita ADR-0109 e os 3 invariantes.

## 3. Prova de byte-identidade + ganho (medido)

**Byte-identidade:** suíte byte-exata do watercolor **33/33** verde (inclui o pin de ±1 byte
`watercolor_incremental_composite_matches_full_recompose`) + suíte completa da crate **479/479**. A
paralelização é memoização/distribuição de uma função por-pixel pura → não há como divergir.

**Ganho (sonda de ablação, R=220 px, canvas 2048², release; antes = pós-cache-do-substrato):**

| Config (R=220, spread 8) | frame avg | frame max | commit (bake) |
|---|---|---|---|
| plain / paper — antes | ~6,0 ms | ~18 ms | ~44 ms |
| plain / paper — **depois** | **~1,8 ms** | **~5,5 ms** | **~7,6 ms** |
| TUDO — antes | 6,7 ms | 40 ms | 198 ms |
| TUDO — **depois** | **3,5 ms** | **20 ms** | **116 ms** |
| TUDO (spread 48) commit — antes → depois | | | 121 → **30 ms** |

Brush grande com papel volta a 60 fps confortável; o freeze do pen-up cai à metade/um-quarto. **Sem
regressão no brush pequeno** (R=16 TUDO: commit 17 → 11 ms) — o overhead do thread-pool em janelas pequenas
é desprezível.

**`box_blur` paralelizado (2026-07-07, sob este ADR).** As far-fields do soak eram o remanescente serial;
`box_blur` agora distribui os dois passos sobre o eixo independente (horizontal por-linha, vertical
por-coluna via buffer transposto — relayout de memória, aritmética idêntica, prefixo da mesma origem).
Byte-idêntico (units + suíte watercolor 33/33). Ganho medido (R=220, TUDO): commit **116 → 44 ms**
(spread 8), **30 → 15 ms** (spread 48); frame max **20 → 10 ms**. Acumulado vs baseline original:
commit 238 → 44 ms (spread 8) / 157 → 15 ms (spread 48); frame max 51 → 10 ms — 60 fps confortável com
tudo ligado, sem regressão no brush pequeno.

## 4. Alternativas rejeitadas

- **Manter single-thread.** Deixa o piso O(janela) e o freeze do bake — o problema reportado.
- **Reduzir resolução/janela / estender o downsample das blurs a spread baixo / GPU.** Todas **mudam o
  resultado da pintura** — vetado pela restrição explícita do Enio ("nada que comprometa o resultado atual").
- **Reuso de buffers (sem paralelismo).** Medido, **sem ganho** (spikes são compute-bound). Descartado.
