# Bugs do módulo Painter — registro + soluções

> Log vivo de bugs **não-triviais** do Painter (sintoma → causa-raiz → tentativas que falharam → solução).
> O objetivo não é listar todo fix (isso o git já faz), mas registrar os bugs cuja **causa enganava** —
> aqueles em que a aparência levou a vários rounds na pista errada. Cada entrada termina em **lições
> generalizáveis** pra não repetir o erro de diagnóstico.

| # | Bug | Área | Estado | Data |
|---|---|---|---|---|
| [1](#bug-1--offset-de-curva-as-quinas-não-ficavam-paralelas-nem-cruzavam) | Offset de curva — quinas (não-paralelas, depois não-cruzavam) | Stroke shape-editor (Curve/Circle/Polygon/Free Hand) | ✅ Resolvido | 2026-06-29 |
| [2](#bug-2--per-layer-color-fps-despenca--artefatos-retangulares-retângulo-virtual) | Per-Layer Color — FPS despenca (✅) + artefatos retangulares ("retângulo virtual", 🔴 **ABERTO**) | Stamp path (CPU) + GPU preview | 🔶 Parcial: perf resolvido · artefato ABERTO | 2026-06-29 |

---

## Bug #1 — Offset de curva: as quinas não ficavam paralelas (nem cruzavam)

**Crates/arquivos:** [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/) →
[`tool/paint/curve_offset.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_offset.rs),
[`tool/paint/curve_join.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_join.rs) (novo),
[`tool/paint/curve_trim.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_trim.rs),
[`tool/paint/curve.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve.rs).
**Feature:** o slider **Offset** (card Offset + checkbox **Trim**) do editor de traço — gera a curva paralela
de Curve / Circle-convertido / Polygon-convertido / Free Hand, pra fora e pra dentro, aberta e fechada.

### Sintoma (e como ele evoluiu — o próprio sintoma enganou)

O Offset funcionava nos **trechos curvos** mas falhava nas **quinas**, e a descrição do sintoma mudou a cada
round porque cada fix resolvia uma camada e expunha a próxima:

1. *"O Polygon convertido em curva não offseta direito; parece o algoritmo antigo."*
2. *"Funciona com lados retos; só ao criar ponto novo e curvar é que piora."*
3. *"Lados curvos ficam mais distantes que os lados retos."* (quinas **encurtadas** vs. curvas)
4. *"O handle Free/Aligned/Symmetric piora; Auto/Vector é melhor."*
5. *"As quinas ficam pontudas e **não se cruzam**."* (sintoma final, decisivo)

### Causa-raiz (a verdadeira, achada só no fim)

Havia **duas** causas, em camadas:

- **Camada A — undershoot da quina.** O offset deslocava cada âncora pelo **normal médio normalizado** (unitário)
  × `d`. Num vértice suave isso é exato, mas numa **quina** (descontinuidade de tangente — handle colapsado/Free)
  a curva paralela verdadeira fica na **interseção das duas arestas offsetadas**, a `d / cos δ`, não a `d`.
  Resultado: a quina ficava **mais curta** que os trechos curvos por um fator `cos δ`. Isso explica os sintomas
  3 e 4 (Auto/Vector mantêm tangente contínua → sem descontinuidade → sem undershoot; Free/Aligned criam a
  descontinuidade → undershoot).

- **Camada B — a quina nunca cruzava.** Mesmo corrigindo a distância (miter na interseção), o algoritmo ainda
  produzia **um único vértice por quina**. E **um ponto único nunca se auto-cruza.** O padrão-ouro CAD é
  **offset-then-trim**: cada aresta é offsetada de forma independente, numa quina **côncava** as duas arestas
  **ultrapassam** uma a outra (cruzam), e um passo de **Trim** corta a orelha. Fundir a quina num ponto (mesmo
  na distância certa) **evita** justamente o cruzamento que o resultado pro precisa. Esse é o sintoma 5.

### Tentativas que falharam — e por quê (as lições estão aqui)

| # | Tentativa | Por que pareceu certo | Por que falhou |
|---|---|---|---|
| 1 | Offsetar âncoras ao longo da **tangente** Bézier (não do chord) | A teoria do "chord dá distância desigual" era correta | **Nenhuma mudança visível**: o `offset_curve` já roda sobre pontos **densificados**, onde tangente≈chord. O fix estava num lugar que já era no-op. |
| 2 | **Polyline offset** (offset por segmento de reta + miter join) | Deu "distância correta" | Perdeu as âncoras Bézier/pontos visíveis (Enio rejeitou) **e** ainda artefatava nas quinas. |
| 3 | Restaurar **densificação CAD** com pontos visíveis | Resolveu "ver os múltiplos pontos" nas curvas | Não tocava nas quinas: a densificação refina **dentro** de spans suaves; a quina é uma **junção entre** segmentos. |
| 4 | **Miter** simétrico: `vertex_normal` devolve `(n₁+n₂)/(1+n₁·n₂)` (a interseção) com miter-limit | Corrigiu o undershoot (Camada A); zero regressão em suave/círculo | Ainda **um vértice único** → continuava pontudo, sem cruzar (Camada B intacta). |
| 5 | Miter **assimétrico** (convexa clampa, côncava alcança a interseção sem clamp) | Distâncias 100% corretas em todos os casos | Ainda **um vértice único** por quina → **não cruzava**. Um ponto não se auto-cruza, ponto final. |

**A lição-mãe:** "distância visualmente correta" **não** é prova de que o algoritmo está certo. As tentativas 4 e 5
acertavam a distância e ainda assim estavam erradas na **topologia** (sem cruzamento). Só o sintoma reformulado
pelo Enio — *"não se cruzam"* — revelou que o problema era de **estrutura de saída** (1 ponto vs. 2), não de
posição. Ver [feedback_measure_perf_symptom_scale](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_measure_perf_symptom_scale.md)
e [feedback_tool_unit_green_integration_dead](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_tool_unit_green_integration_dead.md).

### A solução final (offset-then-trim, padrão CAD)

A arquitetura **já estava pronta** para o cruzamento e ninguém tinha percebido: em
[`curve.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve.rs) o **Trim age só no *spine* pintado**
(`trim_offset_spine`), deixando as **âncoras livres pra cruzar** (comentário explícito: *"the anchors may
cross; the crossed loop just isn't painted"*). Faltava o `offset_curve` **produzir** o cruzamento.

Criei [`curve_join.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_join.rs) (irmão do `curve_trim`),
onde o novo `offset_curve` decide **por quina**:

- **Vértice suave** (`n_in ≈ n_out`, dot > `SMOOTH_DOT`): **1 âncora** no normal unitário → círculos / Auto /
  Vector ficam byte-idênticos (sem regressão).
- **Quina convexa** (lado de **fora** da curva — um gap): **1 âncora** no **miter** `(n₁+n₂)/(1+n₁·n₂)`,
  clampada a `MITER_LIMIT` (um espinho convexo não se auto-cruza, então o Trim não o limparia → tem que ser
  limitado).
- **Quina côncava** (lado de **dentro** — as arestas se sobrepõem): **DIVIDE em 2 âncoras** `P_in = V+d·n_in`
  e `P_out = V+d·n_out`. Na côncava elas caem em **lados opostos** do vértice → as duas arestas offsetadas
  **ultrapassam** → o `flatten_spine` gera um spine **auto-cruzado** → o **Trim corta a orelha**. O conector
  reto (handles colapsados) entre `P_in` e `P_out` é exatamente a orelha.

Convexo vs. côncavo é decidido pelo sinal do giro vezes o sinal de `d`:
`côncavo ⇔ (n₁×n₂)·d < 0` (com gate `dot < SMOOTH_DOT` pra um bend suave nunca fragmentar).

**Plumbing do `remap`.** Como uma quina côncava agora vira **2 âncoras**, a saída do `offset_curve` tem tamanho
variável. Ele devolve um `remap: Vec<usize>` (índice de saída → índice de entrada; o split mapeia 2 saídas →
1 entrada), e o [`offset_curve_refined`](../../crates/ph2d-tool-painter/src/tool/paint/curve_offset.rs) **compõe**
esse `remap` com o `origin` da densificação, pra o **bake** continuar carregando handle-kinds + seleção através
do split. O bake materializa o cruzamento na curva editável — o usuário **vê** os pontos cruzados.

**HR-5 (transcendental-free):** tudo é produto vetorial + a rotação complex-multiply do `SegXform`. Nada de
`atan2`/`sin`/`cos`. Não puxa kurbo (usa transcendentais; e há o gate `vello_kurbo_only_in_ph2d_vector`).

### O que NÃO era a causa (red herrings registrados)

- **Miter-join no convexo.** Necessário e correto, mas **insuficiente** — só corrige a distância (Camada A),
  não a topologia (Camada B).
- **Mais densidade de pontos na quina.** Não resolve: a quina é uma **junção**, não falta de amostras. Mais
  pontos só agrupa amostras perto do vértice (errado). Confirmado na literatura (Levien: subdivisão e junções
  são problemas **separados**).
- **Tipo de handle.** Free/Aligned "pioravam" só porque criavam a descontinuidade de tangente que disparava o
  undershoot; não era um bug do handle.

### Arquivos e commits (ordem cronológica da saga)

| Commit | O que fez |
|---|---|
| `3a3f6071` | (tentativa 1) offset ao longo da tangente — no-op por causa da densificação |
| `803a7c76` | (tentativa 2) polyline offset — revertido (perdia Bézier; artefatos) |
| `d9e6e5ab` | (tentativa 3) densificação CAD com pontos visíveis; sem simplificação automática |
| `c6e600ab` | (tentativa 4) miter simétrico corrige o undershoot da quina |
| `7d7d7a7d` | (tentativa 5) miter assimétrico: convexo clampa, côncavo alcança a interseção |
| `99f3aef0` | **solução** — `curve_join.rs`: côncava **divide em 2 âncoras** → spine cruza → Trim corta |

### Verificação

Testes em [`curve_join.rs`](../../crates/ph2d-tool-painter/src/tool/paint/curve_join.rs) (`cargo test -p ph2d-tool-painter --lib curve_join`):
`a_concave_corner_splits_into_two_overshooting_anchors` (prova: 4 âncoras, `remap=[0,1,1,2]`, `P_in`/`P_out`
em lados opostos; convexo no mesmo canto = 3 âncoras), `a_convex_corner_stays_one_sharp_miter_anchor`,
`offsetting_a_circle_stays_concentric...` (suave nunca fragmenta), `the_convex_miter_reaches_the_true_distance_then_clamps`,
`a_smooth_vertex_miter_stays_unit`, `side_normals_follow_the_handle_tangent_not_the_chord`.
**Smoke do Enio (2026-06-29):** "Perfeito tanto para fora quanto para dentro! Tanto curvas como quinas! Curvas
abertas ou fechadas!"

### Lições generalizáveis

1. **Reformule o sintoma antes de iterar.** O salto de "quinas curtas" para "quinas não cruzam" mudou a classe
   do problema (posição → topologia). Cada round na pista errada custou um commit.
2. **"Parece certo" ≠ "está certo".** Distância visualmente correta escondeu um defeito topológico por 2 fixes.
3. **Junção ≠ amostragem.** Offset de traçado = problema de *stroking*: a parte suave é subdivisão; a quina é
   uma **junção** (miter/round/bevel ou split-and-trim). São mecanismos distintos.
4. **Cheque o que a arquitetura já permite.** O Trim-só-no-spine já deixava as âncoras cruzarem; a correção era
   *upstream* (produzir o cruzamento), não mexer no Trim/dispatch.
5. **Saída de tamanho variável precisa de `remap`.** Ao trocar 1↔N na saída de uma função no meio de um
   pipeline, propague um mapa índice→origem pra os consumidores a jusante (bake/seleção/kinds) não quebrarem.

---

## Bug #2 — Per-Layer Color: FPS despenca + artefatos retangulares ("retângulo virtual")

**Crates/arquivos:**
- **Perf:** [`ph2d-painter-brush/src/stamp_color/accumulate.rs`](../../crates/ph2d-painter-brush/src/stamp_color/accumulate.rs)
  (kernel **fundido** `accumulate_color_stamps_fused`), [`tool/paint/stamp_color_cache.rs`](../../crates/ph2d-tool-painter/src/tool/paint/stamp_color_cache.rs);
  coalescing de ponteiro no shell ([`input_dispatch/painter_canvas_input.rs`](../../shells/desktop/src/input_dispatch/painter_canvas_input.rs)
  + [`render_loop/mod.rs`](../../shells/desktop/src/render_loop/mod.rs)) + `StrokeMethod::coalesces_canvas_motion`.
- **Artefato:** [`ph2d-render/src/individual.rs`](../../crates/ph2d-render/src/individual.rs) —
  `clear_all_mips_transparent` em `create_entry_empty` (clear-on-alloc do slot de preview).

**Feature:** Per-Layer Color (camadas-como-pincel) — N camadas capturadas como Shape, cada uma com sua cor,
compostas em z-order e estampadas ao longo do traço.

Dois sintomas reportados **juntos**, com **causas-raiz diferentes** (essa foi a primeira armadilha):

1. **FPS despenca** ao desenhar (9 FPS) **e** o contador **"Raw" SOBE enquanto o FPS cai** (paradoxo).
2. **Artefatos retangulares:** fatias da imagem do brush **aparecem e somem** em "cantos de retângulos invisíveis".

### Problema A — Perf (estrutural, bem-comportado)

**Medição primeiro** (harness `per_layer_perf` em [`tool/paint/tests.rs`](../../crates/ph2d-tool-painter/src/tool/paint/tests.rs),
`--release`): split de fases revelou **um único kernel = 96.5%** do custo por-Move — `accumulate_color_stamp_coverage`,
`O(D·N·S)` (D dabs × N camadas × footprint (2r)²), refeito pra forma inteira a cada pointer-Move. **Refutou** a
teoria do handoff (que culpava bbox/recompose/upload — D/H≈1.0 provou que **não** é bbox-bound). O "Raw sobe" é a
assinatura: as estampas rodam **fora** da janela de encode que o Raw mede, então `frame_cpu_ms` (Raw) **cai**
enquanto o wall-clock total (FPS) **sobe**.

**Fix:** (1) **kernel fundido alpha-only** — todos os N stamps compartilham `size`, então as coords bilineares
são computadas **1×/pixel** (não ×N) e só o canal alpha é amostrado (o caminho descarta o RGB) → **3.2–4.5×**,
byte-idêntico (gate `fused_per_layer_accumulate_is_bit_identical_to_sequential`). (2) **Coalescing de ponteiro
por-frame** dos métodos de forma (Curve/Line/Circle/Polygon) — colapsa o storm de re-estampa por-evento bruto em
1/frame (incrementais resamplam o segmento, ficam de fora por design). **Limite aceito:** com pincel grande × N16
× canvas grande uma estampa única ainda é ~110 ms — o caso extremo fica para a **migração GPU** do accumulate
(decisão do Enio: sem mitigações CPU de spacing/pincel/camada).

### Problema B — Artefatos (a causa que enganou por ~5 rounds)

A **descrição do sintoma evoluiu** e cada reformulação reposicionou a causa:

1. *"Listras retangulares ao desenhar."* → suspeita: upload parcial de GPU (`preview_upload_bbox`).
2. *"Persiste com `PH2D_PAINT_FULL_UPLOAD=1`."* → upload parcial **descartado**; reclassifiquei como **tearing
   por perf** (§3-D) — **errado** (tearing seria persistente, não "primeiras vezes").
3. *"Fatias da forma, transientes, só nas primeiras vezes; depois nunca mais."* (mockup do Enio) → re-suspeita:
   base stale no cache `composited` CPU. Implementei `reseed_preview_base` (full recompose no início de cada
   sessão de forma). **Não resolveu.**
4. *"Existe um **retângulo virtual** onde o traço é feito; ele sofre o artefato só na **PRIMEIRA vez** que aquela
   região é desenhada na sprite; depois fica limpo pra sempre, mesmo redesenhando."* (a observação decisiva).

A observação 4 é a assinatura inequívoca de **leitura de memória GPU não-inicializada**: garbage até a região ser
escrita a 1ª vez; uma vez escrita, válida para sempre. E **imune ao FULL_UPLOAD e ao reseed** porque ambos mexem
em buffers **CPU já semeados** — e se o stack é GPU-elegível, o `gpu_owns_preview` **desliga o caminho CPU inteiro**.

### Estado do artefato: 🔴 **ABERTO** (perf resolvido, artefato persiste)

A assinatura é **leitura de memória GPU não-inicializada** (retângulo virtual; garbage só na 1ª vez que a região é
desenhada; limpo pra sempre depois). Mas o artefato **sobrevive a TODAS as tentativas até aqui** — então o buffer
culpado ainda **não** foi identificado.

### Tentativas que falharam — e por quê

| # | Tentativa | Por que pareceu certo | Por que falhou |
|---|---|---|---|
| 1 | `PH2D_PAINT_FULL_UPLOAD` (upload full do slot CPU) | "Se é upload parcial, full resolve" | Persistiu → não é cobertura do upload; e no stack GPU-elegível o `gpu_owns_preview` desliga o caminho CPU. |
| 2 | Reclassificar como **tearing por perf** (§3-D) | FPS=9 → frames rasgados é plausível | "Primeiras vezes, depois nunca" **contradiz** tearing (seria persistente). |
| 3 | `reseed_preview_base` (full recompose no início da sessão de forma) | Forçaria base correta | Re-semeia o `composited` **CPU**; não tocou a textura **GPU**. Persistiu. |
| 4 | **Clear-on-alloc do slot** (`clear_all_mips_transparent` em `create_entry_empty`) | O slot exibido nascia sem clear (`acquire_empty`) → garbage | **Persistiu.** O slot estava no caminho certo (GPU), mas ou não é a fonte, ou é **sobrescrito** por uma cópia de buffer ainda-não-inicializado a montante (`out`/premul). |

**A lição-mãe (até aqui):** três fixes miraram buffers que se provaram **já semeados** (CPU `composited`, slot via
FULL_UPLOAD, slot empty via clear-on-alloc). Cada "não mudou" me fez reclassificar a causa — **cedo demais** em #1 e
#2. A análise estática mostra TODOS os buffers semeados, o que é paradoxal frente ao sintoma real → falta reproduzir.

### Próximo passo — `out`/premul VERIFICADOS limpos (2026-06-29); análise estática ESGOTADA

O suspeito eram as texturas internas do compositor GPU (`out` + premul, STORAGE sem clear). **Verificado nos
shaders: NÃO são a fonte.**
- **`out`:** `cs_flat` ([`layer_composite.wgsl`](../../crates/ph2d-render/src/shaders/layer_composite.wgsl)) inicia
  `acc = vec4(0,0,0,0)` (transparente — **não** lê o `out` anterior) e faz `textureStore` em **todo** texel; o
  `try_drive` do painter sempre dispara `seed_full` → região = canvas → `out` totalmente escrito todo frame sujo.
- **premul:** `cs_main` ([`preview_premul.wgsl`](../../crates/ph2d-render/src/shaders/preview_premul.wgsl)) escreve
  todo texel in-bounds; `run()` despacha o canvas inteiro. Totalmente escrito.

**Conclusão:** TODOS os buffers do pipeline se provam semeados na análise estática (CPU composited, slot, layer
textures, `out`, premul, mips, mapas), e mesmo assim o sintoma (uninitialized-read) é real. **A análise estática
esgotou** — errei o buffer 4× chutando. O próximo passo **TEM** que ser reprodução runtime, não um 5º chute:
1. **Diagnóstico magenta:** limpar o slot (ou `out`) com cor-debug viva. Artefato magenta ⇒ aquele buffer é
   amostrado não-escrito; artefato garbage ⇒ é sobrescrito por uma fonte ainda não-rastreada.
2. **Captura GPU** (Xcode frame capture) do frame do artefato — vê o conteúdo real da textura amostrada.
3. **Re-examinar caminhos NÃO-preview:** a `draw_repeat_image` (preview 3×3), os overlays on-canvas
   (`painter_bridge_overlays`), ou o cursor/brush-ring — os slivers no mockup estavam perto dos gizmos, não
   necessariamente no resultado pintado.

### O que JÁ foi descartado (red herrings registrados)

- **Upload parcial de GPU / `preview_upload_bbox`** (§3-A): cobertura provada consistente; FULL_UPLOAD descartou.
- **Tearing por perf** (§3-D): contradito pelo "primeiras vezes, depois nunca".
- **Cache `composited` CPU stale / drag-preview restore:** auto-consistentes (trail-freedom verdes); reseed não resolveu.
- **Slot do `IndividualTextureStore` não-inicializado:** clear-on-alloc landou (correto e defensivo — guard verde) mas
  **não** resolveu o artefato → o slot não era a fonte (ou é sobrescrito a montante).
- **`regen_mips` / `ensure_slice`:** ambos os caminhos de cópia regeneram mips; `ensure_slice` sobe o buffer inteiro.

### Verificação (do que LANDOU)

- **Perf (✅ resolvido):** harness `per_layer_perf` (`#[ignore]`, `--release`) + gate de paridade byte
  `fused_per_layer_accumulate_is_bit_identical_to_sequential`; **3.2–4.5×**; smoke do Enio: "FPS melhor".
- **Defensivos que LANDARAM mas NÃO resolveram o artefato (mantidos por serem corretos):** `reseed_preview_base`
  (full recompose por sessão de forma) + clear-on-alloc do slot (guard `acquire_empty_slot_reads_back_transparent_not_garbage`
  verde — um slot vazio agora lê all-zero, antes garbage). 6/6 `individual_readback` verdes.
- **Artefato:** ❌ persiste (Enio 2026-06-29). **Continua ABERTO.**

### Lições generalizáveis (parciais — o artefato ainda ensina)

1. **Dois sintomas juntos podem ter causas independentes.** Perf (kernel CPU) e artefato (textura GPU) foram
   reportados juntos; só medindo/traçando cada um isoladamente eles se separaram. A perf foi resolvida; o artefato não.
2. **"Não mudou" não autoriza reclassificar a causa.** Reclassifiquei cedo demais (#1 upload → #2 tearing) — cada
   "não resolveu" só prova que **aquele buffer** estava ok, não que a CLASSE mudou. Confirme QUAL caminho/buffer o
   fix afeta antes de descartar a hipótese.
3. **Análise estática "tudo semeado" + sintoma vivo = reproduza, não chute.** Errei o buffer 2× chutando. Com todos os
   buffers parecendo semeados no código, o passo certo é REPRODUZIR (teste headless / diagnóstico magenta), não um 3º chute.
4. **Texturas wgpu nascem com lixo.** Toda textura amostrável-antes-do-1º-write-completo precisa de clear-on-alloc;
   limpe **todos** os níveis de mip. (O clear do slot é correto mesmo não tendo resolvido este bug.)
5. **Meça antes de culpar.** O split de fases (96.5% num kernel) refutou a teoria do handoff em uma medição. Ver
   [feedback_measure_perf_symptom_scale](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_measure_perf_symptom_scale.md).

---

## Como adicionar um bug aqui

Uma seção `## Bug #N — <título>` + linha na tabela do topo. Foque nos bugs cuja **causa enganou** (vários rounds
na pista errada); fix trivial fica só no git. Sempre termine em **lições generalizáveis**.
