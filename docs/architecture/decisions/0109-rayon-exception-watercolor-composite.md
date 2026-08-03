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

---

## 5. Emenda 2 (2026-07-23) — o composite do WET PAINT entra na mesma exceção

- **Status:** ACEITO. **Escopo somado:** o composite de pigmento do Wet Paint
  (`tool/paint/wetpaint/composite.rs`), paralelizado **por LINHAS**.
  Continua **não** abrindo `rayon` para o resto do codebase — nem, em
  particular, para `ph2d-wet-paint`.

**Por que é o mesmo caso, e não um caso novo:** é a mesma forma de trabalho que
o §1 descreve — um **map puro por-pixel** que reconstrói a aparência por frame,
com piso O(janela) single-thread, no mesmo crate que este ADR já abriu. Medido
no canvas que o smoke abre (1024², folha cheia de tinta): **15,82 → 1,62 ms**
(glaze OFF) e **44,00 → 3,49 ms** (glaze ON), 9,8× e 12,6× em 32 vias. Antes
disso o composite era **metade do frame**; depois, 5%.

**A fronteira que isto respeita, e que vale a pena escrever:** o `ph2d-wet-paint`
é um **porte 1:1 com fingerprint** e sem dependências além do `libm` — ele não
pode ganhar um thread-pool sem virar outra coisa. Então o engine expõe **uma
LINHA** (`render::render_pigment_row_visual`) e **não spawna nada**; quem abre o
leque é o tool, onde a exceção já mora. O corpo por-linha é o corpo serial
literal — a região virou um laço que chama a linha — e as linhas nunca leem a
saída umas das outras.

**Byte-idêntico**, e não por argumento: a medição compara os dois buffers e
falha se um byte diferir (`measure_row_parallel_composite`), e a suíte do tool
(819 testes) roda sobre o caminho paralelo.

**Não paralelizado, de propósito:** o SOLVER do Wet Paint. Ele é **serial por
semântica** (o brake lê wetness viva escrita antes na mesma passada; o drying lê
o vizinho da esquerda pós-update; o advect subtrai dos cantos-fonte) — a mesma
razão que o ADR-0134 já registra. Depois desta emenda o passo de sim é **89% do
frame** com Pigment mixing ligado, e é lá que fica o teto.

---

## Emenda (2026-08-02) — o passe de SECAGEM do mapa de umidade, por linhas

- **Status:** ACEITO. **Escopo somado:** `dry_canvas_wet` (o decaimento do
  `canvas_wet`, em `tool/paint/watercolor_backdrop.rs`), paralelizado **por
  LINHAS**. Segue **não** abrindo `rayon` para o resto do codebase.

**O número que motiva.** O log do produto (`PH2D_PAINT_PERF`, Enio, canvas
4096², pincel 250) mostrou `secagem 10-16 ms` **em todo quadro** — o maior item
isolado do quadro do artista, e ele roda pinte-se ou não, porque o papel seca no
heartbeat. Medido pela porta do produto: **28,50 ms/quadro** sobre um rect de
12,85 M texels, ou **2,2 ns/texel**.

**⚠️ E três curas single-thread foram construídas ANTES desta e as três
mediram ~1,00×** — o que importa registrar, para ninguém as reconstruir: trocar
o snapshot do rect por uma **janela deslizante** (`up` de uma linha de scratch,
`left` de um escalar, `down`/`right` do mapa ainda não escrito) deu **1,02×**; o
**piso da erosão** e o **rect que encolhe** deram o mesmo. A hipótese era que a
alocação e a cópia do snapshot fossem o custo; a medição diz que o custo é o
**caminhar**. A janela deslizante foi **revertida**, porque a dependência que ela
cria entre linhas é exatamente o que impede o paralelo que funciona.

**Por que é o mesmo caso, e não um caso novo.** Os três invariantes do §2 valem
verbatim: cada linha de saída é função pura de um **snapshot imutável** (que já
existia — a lei do decaimento é independente de ordem por desenho, e o snapshot
é o que a garante), cada task escreve **só a própria fatia** de `canvas_wet`, e
não há RNG nem transcendental. É o mesmo crate que este ADR já abriu.

**A redução, que é o que a cerca de contenção do §2 nomeia.** Ela existe e é
**exatamente do tipo que a cerca isenta**: `max` sobre `u8` (o `wettest`, que
decide o teardown da sessão) e `min`/`max` sobre índices (a bbox do que sobrou
molhado). Associativas e comutativas sobre inteiros — não há soma em ponto
flutuante cuja ordem entre threads pudesse mudar um byte. É a mesma leitura que
o ADR-0147 fez para o gather do `advect`.

**Medido pela porta do produto**, mesma corrida, estado restaurado antes de cada
amostra: um passe **30,44 → 3,28 ms a 4096² (9,3×)** e 7,96 → 1,09 a 2048²
(7,3×); a sequência de secagem de 120 quadros, **28,50 → 2,93 ms/quadro
(9,7×)**. Piso do pool `DRY_PAR_MIN` em texels, no molde do limiar em bytes do
`plane_copy`.

**Byte-idêntico, e provado contra a rotina que SHIPAVA** — congelada sob
`cfg(test)` (`watercolor_dry_tests::decay_snapshot`) e comparada a cada passe,
varrendo o `step` (1/5/17/51) e **os dois lados do piso do pool**. ⚠️ A primeira
versão do gate usava só 128², abaixo do piso: ela nunca entrou na rota paralela
que a wave instala.
