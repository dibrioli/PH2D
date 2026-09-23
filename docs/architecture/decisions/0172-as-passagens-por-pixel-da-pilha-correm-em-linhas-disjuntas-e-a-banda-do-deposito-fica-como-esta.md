# ADR-0172 — As passagens POR PIXEL da pilha correm em linhas disjuntas; a banda do depósito fica como está

- **Status:** Accepted
- **Data:** 2026-09-23
- **Linha:** `line/PainterWatercolor`
- **Amparo:** ADR-0109 (a cerca de contenção do `rayon` em `ph2d-tool-painter`, e os TRÊS invariantes
  que qualificam uma excepção) · ADR-0171 (o precedente mais próximo: o borrão de caixa da mesma pilha)

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — quem integrar reconta-o contra o `main` do
> dia, nunca o copia daqui.

## Contexto

Ordem do dono (2026-09-23), depois de aprovar a composição por quadro (*«FPS aceitável em torno de
40»*): *«vamos tentar tornar mais rápido, vamos chegar ao estado da arte»*.

⛔⛔ **A primeira régua media OUTRO programa.** As sondas `diag_*` da crate correm sob `cfg(test)`, e
ali o traço paga duas coisas que o app nunca paga: o **journal do undo** do canvas
(`capture_canvas` é `cfg(any(test, debug_assertions))`) e a **espia do esfregão**, que copia o campo
de deslocamento inteiro (`8 MB` a `1024²`) a cada lote. ⇒ a medição do produto passa a ser o
exemplo `crates/ph2d-tool-painter/examples/mede_a_pilha.rs`, que compila a biblioteca **sem**
`cfg(test)` e monta a pilha do dono pelas portas públicas que a cena `PH2D_COMPOSITE_SMOKE` usa.

⛔⛔ **E o perfilador também mentiu, numa direcção só.** Sem `perf` (o `perf_event_paranoid = 2`
recusa-o sem root) a amostragem foi feita por `gdb`, e o `gdb` torna a **criação de threads**
artificialmente lenta (cada `pthread_create` é um evento de `ptrace`): a primeira leitura pôs
**`49,6 %`** da thread principal em `pthread_create`. A troca que essa leitura pedia foi construída,
medida em A/B no build do produto, e **não mede ganho nenhum** (ver *Recusa medida* abaixo).

## Decisão

Três passagens da composição da pilha do Composite Brush passam a correr em **linhas disjuntas** na
equipa de threads (`rayon::par_chunks_mut`, `with_min_len(8)`):

1. a **tinta** de uma camada Brush (`composite_linhas::tinta`);
2. a **borracha** nos dois escopos (`composite_linhas::borracha`);
3. o **re-amostrar do esfregão** pelo campo de deslocamento (`warp::session::reamostra`, que serve
   também a sessão de warp da ferramenta Smear e do Deform — a MESMA lei, agora em linhas).

Os três invariantes do ADR-0109 valem **à letra**: cada pixel de saída é função PURA de planos que a
passagem só LÊ (o plano da camada, o `pre`, o campo de deslocamento) e do próprio pixel da tela;
nenhuma soma atravessa píxeis; nenhuma aleatoriedade entra ⇒ a saída é **byte-idêntica** para
qualquer número de threads e qualquer escalonamento. **Gates:** cada passagem é comparada, byte a
byte, contra o laço em série de antes copiado à letra (`a_tinta_em_paralelo_da_o_byte_da_serie` ·
`a_borracha_em_paralelo_da_o_byte_da_serie` · `o_reamostrar_em_paralelo_da_o_byte_da_serie`), com
CONTROLO de que a fixtura muda bytes.

Duas curas **exactas** sem paralelismo entram no mesmo passo:

- **o arredondamento sem biblioteca** (`composite_linhas::redondo_u8`): o repo compila para o
  `x86-64` BASE, onde `f32::round` é uma chamada ao `roundf` de software do `compiler_builtins`; a
  função dá o mesmo byte com um truncamento e uma comparação (gate nas fronteiras `k/2 ± 4096` ulps,
  nos extremos, em `NaN`/`±∞`, e em toda saída `v·255` de `65 536` valores de `v`);
- **o `hypot` que comparava contra o infinito** (`ph2d_painter_brush::accumulate_dab_smear`): o
  produto corre com `SEM_TECTO = ∞`, e `m > ∞` é falso para todo `m` — o `hypot` era pago POR TEXEL
  para nada. A guarda pergunta `tecto != ∞` (e não `is_finite()`), logo com `−∞`/`NaN` o ramo corre
  como corria.

## Medição (A/B no build do produto, corridas ALTERNADAS, mínimo de 5)

Pilha do dono, tela `1024²`, traço de `720 px`, drenagem a cada 16 eventos (um quadro). A máquina
estava sob carga de outras linhas (`load 26`–`50`), por isso os números valem pela DIFERENÇA na
mesma ronda, não pelo absoluto:

| ronda | load | passo 2: antes → agora (ms/quadro) | passo 8: antes → agora (ms/quadro) |
|---|---|---|---|
| 1 | 48 | `7,11 → 4,13` | `16,44 → 11,26` |
| 2 | 41 | `10,98 → 5,07` | `18,00 → 11,38` |
| 3 | 36 | `7,73 → 6,20` | `19,02 → 12,46` |
| 4 | 51 | `8,62 → 4,17` | `19,17 → 11,16` |

⇒ **−35 %** num traço rápido e **−45 %** num lento, e nenhuma passagem mudou um byte.

## ⛔ Recusa medida — a banda do DEPÓSITO na equipa de threads

A leitura do `gdb` pedia trocar o `std::thread::scope` do `band_split` (e dos quatro irmãos em
`height_walk` e `accumulate_batch`) pela equipa permanente do `rayon`. Foi construído (byte-idêntico,
`443/443` e `1334/1334` verdes) e medido isolado, na mesma ronda e com o resto das curas aplicado
dos dois lados:

| ronda | load | passo 2: sem → com (ms/quadro) | passo 8: sem → com (ms/quadro) |
|---|---|---|---|
| 2 | 29 | `4,41 → 4,27` | `11,39 → 11,51` |
| 3 | 27 | `4,40 → 4,51` | `12,62 → 11,79` |

⇒ **ganho zero, dentro do ruído.** O `49,6 %` era o `ptrace` a pagar a criação de threads, não o
produto. **Revertido** — uma excepção à cerca sem ganho medido é o que a cerca existe para impedir.
⚠️ *Sob `gdb`, toda sonda que cria threads mede o depurador.*

## O que fica fora, com o número

- ⭐⭐ **O alvo de CPU do build** (decisão do DONO, não desta linha): o mesmo exemplo compilado com
  `-C target-cpu=x86-64-v3` mede, na mesma ronda, `218 → 128` · `165 → 143` · `251 → 178` ms no
  passo 2 e `96 → 72` · `88 → 72` · `141 → 88` no passo 8 — **20–40 % em todo o pincel, sem uma linha
  de código**. É o `x86-64` base (2003) que transforma `round`/`floor` em chamadas de função. Subi-lo
  decide que processadores o app suporta; `x86-64-v2` (SSE4.2, 2009+) já traz as instruções de
  arredondamento e não foi medido.
- O **depósito** das camadas no plano (a maior fatia que sobra: ~`43` de ~`57` ms por traço no passo
  8) e o **campo do esfregão** (`sample_window` + `walk_dab`, em série na thread principal).
