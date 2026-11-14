# ADR-0202 — O compositor da CPU e os kernels espaciais correm no pool do `rayon`

- **Status:** Accepted — missão Painter 4K (dono 10/11, «todas as recomendações dos estudos»); desenho técnico delegado
  (padrão-ouro).
- **Data:** 2026-11-10
- **Linha:** `line/PainterWatercolor`
- **Estende:** a cerca do `rayon` em `ph2d-tool-painter` (ADR-0109, ADR-0172, ADR-0173); em `ph2d-painter-effects` é o
  1.º uso (dep nova).
- **Amparo:** `docs/Painter/missao_4k/BACKLOG.md` B19.7 (PX-11, PP-20, PL-23, IN-50, PA-72).

> Número recontado na integração de 13/11 (§5.0): a linha escreveu `0201`; a linha JointEditor tomou o `0201`.

## Contexto

Cada composição abria fios do SO com `std::thread::scope`: as faixas do `compor_regiao` (`compose/barreira.rs`), as do
arrasto (`cache.rs`), as do chão (`abaixo.rs`), o `encode`, o `flat_fill` e, dentro de cada ajuste, `par_rows` /
`par_pixels` do `ph2d-painter-effects`. A Color Lookup (`par_pixels`) corria DENTRO de cada faixa da composição: fios
dentro de fios. Medido no gate novo (1024×256, 4 camadas + B/C + Gaussian + Color Lookup, 32 núcleos): **1 044 fios do SO
abertos em 3 composições** (~350 por composição; ~10 µs cada).

## Decisão

1. Os sete sítios passam a `par_chunks_mut` do pool global do `rayon`, com a MESMA partição (faixas de linhas de
   `parallel_threads` / `available_parallelism`). Cada faixa escreve só as suas linhas e nada se reduz: o byte não depende
   da partição nem do número de operários (gates ao bit existentes, a 1/2/3/7/16/32 faixas).
2. O aninhamento (Color Lookup ou blur dentro de uma faixa) fica no mesmo pool: o operário que espera pelas sub-faixas
   rouba trabalho, sem abrir fio.
3. **Nenhuma espera bloqueante dentro de um job** (Bug #66): os sítios só esperam pelas próprias sub-faixas (`join` do
   `rayon`). Quem chama o compositor com uma tranca na mão não pode ter um job do pool à espera dessa tranca — a regra do
   #66 continua a valer para quem chama.

Gate: `ph2d-tool-painter a_composicao_nao_abre_fios_do_so` (TIDs novos em `/proc/self/task` durante 3 composições com o
pool aquecido = 0): vermelho 1 044 antes, verde depois.

## Alternativas recusadas

- **Pool próprio do compositor:** duplica operários com o pool global que a pintura já usa (ADR-0173/0200) e disputa núcleos.
- **Baixar o nº de faixas (`band_count`) mantendo `thread::scope`:** continua a pagar o `spawn` por chamada e não cura o
  aninhamento.

## Preço e risco

Acordar o pool custa 1–2 ms quando um operário está desagendado (ADR-0173); abaixo de `1 << 17` px (e `1 << 14` nos
kernels) tudo continua num núcleo, como antes.

## Extensão (B19b-cont 3) — a mesma família fora do compositor

O overlay da seleção, `for_dirty_rows_parallel` (cor do Transform e malha), o levante e o corpo do Transform e a
recomposição por camada do carimbo colorido (`stamp_color_cache`, `stamp_color_dynamic`) passam ao mesmo pool, com a
mesma partição; o piso `1 << 17` passa a ter UM dono (`compositor::parallel_threads`). Gate
`ph2d-tool-painter as_faixas_fora_do_compositor_nao_abrem_fios_do_so` (vermelho 72 · 128 · 96 fios → o controlo serial)
e `as_faixas_fora_do_compositor_dao_o_serial_ao_bit`. Fica de fora: os `thread::scope` do `ph2d-painter-brush`.
