# 07 — Plano das "joias da coroa" da timeline (sem tocar o fade)

> **Linha:** `line/anim` · **Data do estudo:** 2026-07-26 · **Estado:** PLANO (nada construído).
> Escopo: as 4 joias da coroa da pesquisa (expressões, extrapolação por-track,
> stagger, time-scale de seleção) + **Buffer curves** + **Time-Reverse Keyframes**.
> Pré-requisito cumprido: o Enio pediu para **estudar antes** se alguma feature
> pode danificar o **sistema de Clips/Strips/Containers/Fade** (precioso, custou a
> ajustar). Este doc abre com esse veredito.

---

## §0 — Veredito de risco: NENHUMA das 6 prejudica o fade (com 5 pinos)

O estudo (3 mapas paralelos sobre o pipeline de sampling, o sistema de fade, e a
superfície de edição) fixou a arquitetura em camadas. A composição, de fora para
dentro, é:

```
strip fold (Once/Loop/PingPong sobre a slice)   ← stack.rs:314
  → clip/container cut (length_override)         ← doc_extent.rs:170/179/188
    → Time-Remap clock (por-entidade)            ← clock.rs:77
      → Track::sample (flat-clamp nos extremos)  ← track.rs:633-638   ← A CAMADA MAIS INTERNA
```

### O conjunto INTOCÁVEL (o que o fade lê todo frame)
- `stack.rs`: `ramp` (541) · `weight_at` (489) · `blend_in/out` (404/456) ·
  `neighbour_reach_in/out` (424/439) · `fold` (314) ·
  `lead_start/end` + `source_time_with_lead` (260/267/282) · **o layout dos campos
  de `ClipStrip`/`ClipLane` + a invariante de ordenação por `t_start`**.
- `stack_hold.rs::hold_at` — o **complemento** `1 − Σ weight_at`; afeta todo fade
  solitário em silêncio.
- `stack_eval.rs` — a normalização `Σ(w·v)/Σ(w)` + `influence = den.min(1)·weight`
  (210-246) + a distinção **speaks/touched** (peso-zero-não-é-silêncio).
- `stack_frames.rs` — o clock de nesting (`resolve` empurra o frame do container,
  396-401; o `t` por-frame).
- **`DOC_VERSION` (`doc.rs:79`, hoje 13), postcard POSICIONAL, append-only.** Todo
  campo novo apenda no FIM e bumpa; nunca reordena (a regra `doc.rs:27-28`).

### A rota SEGURA para todo edit de keyframe
Passar pelas **tracks** do clip (`move_keys`/`scale_keys`/`reverse_about`/
`duplicate_keys` em `track.rs`, amostradas em `stack_eval.rs:158-162` /
`apply.rs:238`) via o choke point **`edit()` → `settle()`** (intent_apply.rs:559-577
→ intent_settle.rs:25-43, que re-ordena + re-resolve roving no MESMO passo de
undo). **Nunca** emitir uma das intents de strip/lane/container (a lista de §9) —
essas são despachadas por `edit_at(...)` e mexem na geometria que o fade lê.

> A frase-âncora do próprio código (intent_apply.rs:368-376):
> *"None of these touch the SELECTION: a strip is not a key, and the two never
> share an identity space."* — a separação já existe; o plano só a honra.

### Veredito por-feature

| Feature | Toca o fade? | Por quê / o pino |
|---|---|---|
| **Time-Reverse Keys** | **Não** | Edit de key via `edit()`; nunca `edit_at`/strip. |
| **Stagger/Distribute** | **Não** | `move_keys` por-track (objetos = tracks na view Keys, **não** strips). |
| **Time-scale de seleção** | **Não** (com cautela) | Reusa `ScaleSelectedKeys` (já existe). ⚠️ Ortogonal a `StretchStrip` — o gesto é sobre `TimelineHitKind::Key`, jamais sobre um strip. |
| **Buffer curves** | **Não** | Snapshot per-track no painel; restore é edit de key; buffer não persiste. |
| **Extrapolação por-track** | **Indiretamente, e é o pino** | Mora em `Track::sample` (interno). O `hold_at` do fade cruza para valores lidos pelo MESMO `Track::sample`, então extrapolação ≠ Hold muda o valor cruzado. **Default Pre=Post=Hold é BYTE-IDÊNTICO ao fade de hoje** — gate obrigatório. |
| **Expressões** | **Não** (por desenho) | Passe **pós-composição SEPARADO**: lê valores já compostos (`read_prop`) e escreve props dirigidas (`write_prop`). Nunca entra em `stack_eval`. Documento sem expr = byte-idêntico. |

### Bônus do estudo (de-risca metade do trabalho)
Já existem no `Track`/nas intents: **`scale_keys`** (track.rs:381), **`move_keys`**
(track.rs:338), **`reverse_about`** (track.rs:405), **`duplicate_keys`**
(track.rs:562), e as intents **`MoveSelectedKeys`**/**`ScaleSelectedKeys`**
(intent.rs:125/130). Então time-scale é **quase só UI**, e stagger/reverse são
camadas finas de intent+UI.

---

## §1 — Ordem recomendada (por risco × alavanca)

- **Wave A — primitivos prontos, risco baixo, ganho rápido** (nenhum schema, nenhum
  toque no fade): Time-Reverse · Stagger · Time-scale (UI) · Buffer curves.
- **Wave B — schema + sampling** (a única interação indireta com o fade, gateada):
  Extrapolação por-track. `DOC_VERSION 13→14`.
- **Wave C — subsistema novo (ADR ANTES)**: Expressões. `DOC_VERSION 14→15`.

> ⚠️ Os números de `DOC_VERSION` (14, 15) são **PROVISÓRIOS** — o número se CONTA na
> integração, não se escolhe (outra linha pode bumpar antes;
> [[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

---

## §2 — Time-Reverse Keyframes (Wave A)

- **O QUE:** reverte no TEMPO os keys selecionados em torno de um pivô (a extensão
  da seleção, ou o playhead), **migrando o `Interp` de cada segmento** para o key
  que passa a possuí-lo depois do espelho (ease-out↔ease-in). O AE tem
  *Time-Reverse Keyframes*.
- **ONDE:** `Track::reverse_about(span)` já faz t→span−t + migra interps + reverte
  os 3 vecs paralelos + resort (track.rs:405) — mas opera na track **INTEIRA**. O
  trabalho é um **`Track::reverse_keys(ids, pivot)`** que espelha só os `ids` em
  torno do pivô e migra os interps DENTRO do conjunto (irmão escopado do
  `reverse_about`). Intent nova `ReverseSelectedKeys` (intent.rs) aplicada via
  `for_selected_tracks` (intent_apply.rs:645).
- **SCHEMA:** nenhum.
- **PINO DE FADE:** edit de key via `edit()`→`settle()`; zero intent de strip.
- **UI:** row do menu R-click da seleção ("Reverse Keyframes") + hotkey.
- **GATES:** reverter 2× = identidade · o interp migra (o ease-out da borda vira
  ease-in) · **fade fingerprint (§8) intacto** · mutação (não migrar o interp) sangra.

---

## §3 — Stagger / Distribute (Wave A)

- **O QUE:** dois sabores, ambos sobre KEYS: **(a) Distribuir** — espalha os keys
  selecionados uniformemente entre o 1º e o último; **(b) Offset/cascata** — cada
  OBJETO (= track/binding na view Keys) recebe um deslocamento crescente `k·passo`,
  a cascata de motion-graphics. É o *Quick Offset* (AE 25.4, arrasta a seleção e ela
  se escalona) + *Sequence Layers*.
- **⚠️ ESCOPO QUE PROTEGE O PRECIOSO:** o stagger de OBJETOS é feito deslocando as
  KEYS de cada track (`move_keys` por-target com delta crescente), **NÃO** os
  strips. Escalonar *strips/clips* é a camada de composição (já temos `MoveStrip`) e
  fica **FORA** desta feature de propósito — a confusão "stagger mexe em strip" é
  exatamente o que arranharia o fade.
- **ONDE:** intent nova `StaggerSelectedKeys{ mode, step }` (ou `DistributeSelectedKeys`),
  aplicada com `move_keys` por-target numa ordem ESTÁVEL (por `entity`/RootOrder,
  não por ordem de clique). `move_keys` (track.rs:338) já rationaliza o delta e faz
  `merge_moved_over_stationary` + resort.
- **SCHEMA:** nenhum.
- **PINO DE FADE:** só `move_keys`; zero intent de strip.
- **UI:** o gesto do Quick Offset (Ctrl+Alt-arrasta a seleção → distribui) — o mais
  amado; + um item de menu "Stagger…" para o modo dialog. Começar pelo gesto.
- **GATES:** N tracks deslocados por `k·step` na ordem estável · distribuir espalha
  uniforme (o 1º e o último não se movem) · **fade fingerprint intacto** · a ordem de
  emissão faz todo `move_keys` pousar (sem colisão que o `merge` engula em silêncio).

---

## §4 — Time-scale de uma seleção de keys (Wave A — quase só UI)

- **O QUE:** box-select de keys → a seleção ganha uma **bounding-box com alças de
  TEMPO**; arrastar uma alça escala o tempo da seleção em torno do pivô (a borda
  oposta). O verbo de retiming mais amado (AE/Maya/Unreal).
- **ONDE:** **a intent `ScaleSelectedKeys{pivot_seconds, factor}` e o
  `Track::scale_keys` JÁ EXISTEM** (intent.rs:130, track.rs:381). Falta só a UI no
  painel: desenhar a bbox da seleção no dope-sheet + 2 alças (esquerda/direita), e o
  drag emite `ScaleSelectedKeys` ao vivo (bracket de undo, `held_button`).
- **⚠️ ORTOGONAL a `StretchStrip`** (o retime no nível do strip, intent.rs:553): o
  gesto vive sobre `TimelineHitKind::Key` no dope-sheet e **nunca** sobre a régua de
  strips — são superfícies diferentes. Este é o único ponto da Wave A onde um gesto
  mal-roteado poderia tocar o precioso; o gate crava a separação.
- **SCHEMA:** nenhum.
- **PINO DE FADE:** o gesto emite `ScaleSelectedKeys` (key), **nunca** `StretchStrip`
  — arch-gate sobre o painel: o handle de tempo da seleção de keys não resolve para
  `edit_at`/strip.
- **UI:** bbox da seleção + 2 alças de tempo; pivô = borda oposta; ao vivo.
- **GATES:** escalar 2× dobra a extensão da seleção · pivô correto (a borda oposta
  fica parada) · **fade fingerprint intacto** · **arch-gate: o gesto emite key-scale,
  não strip-stretch** (mutação: rotear pro strip → RED).

---

## §5 — Buffer curves (Wave A)

- **O QUE:** no graph editor, **"Store Buffer"** guarda a curva atual do track;
  depois de afinar, **"Swap Buffered"** troca a atual pela guardada (e vice-versa),
  e/ou desenha a buffered como **fantasma** para comparar A/B. Padrão Unreal.
- **ONDE:** estado do PAINEL (conveniência de edição, como onion/seleção). O
  snapshot é o `Vec<Key>` + `roving` do track (o mesmo shape que `graph.rs`/`speed.rs`
  já consomem via `&[KeyView]`); o fantasma sai do `sample_keys` (graph.rs:22) sem
  tocar o documento. O **swap/restore** escreve os keys de volta num bracket de undo
  (uma intent `RestoreTrackCurve{target, keys}` OU delete-all+insert-all no bracket).
- **SCHEMA:** **nenhum** — o buffer **NÃO persiste** (é sessão de edição, classe do
  onion/seleção; `TimelineState`, não serializado).
- **PINO DE FADE:** restore é edit de key (bracket); zero intent de strip.
- **UI:** 2 botões no graph editor (Store / Swap) + toggle do fantasma.
- **GATES:** store → editar → swap devolve a curva **byte-idêntica** · o fantasma
  desenha pela `sample_keys` (mesmo sampler do runtime) · **fade fingerprint intacto**.

---

## §6 — Extrapolação por-track: loopOut / cycle / pingpong / continue (Wave B)

> **✅ CONSTRUÍDA (2026-07-26, pendente de smoke).** Motor `ph2d-anim::extrap`
> (`Extrap{Hold,Loop,PingPong,Continue}` + `ExtrapSide`, `Track::pre/post`,
> consultado no flat-clamp de `Track::sample`) · intent `SetTrackExtrap` (edit(),
> nunca strip) · UI = duas cascatas no menu R-click da row → submenu de 4 modos
> (`ContextMenuKind::TimelineExtrap`), TimeRemap num menu próprio sem as cascatas.
> **`DOC_VERSION 13→14`** (PROVISÓRIO — conta na integração). Default Hold/Hold =
> byte-idêntico, `fade_fingerprint` intacto. Smoke: `PH2D_EXTRAP_SMOKE=1`. Gates:
> `extrapolation.rs` (motor) · clock TimeRemap-inert · `track_extrapolation.rs`
> (compose strip-Loop × track-loopOut) · `extrapolation_seam.rs` (UI).

- **O QUE:** além dos keys, o track **CICLA** (loop) / **PINGPONG** / **CONTINUA**
  (extensão linear pelas tangentes de borda) / **SEGURA** (Hold — o comportamento de
  hoje). **Pre e Post independentes** (o loopIn/loopOut do AE, o *Pre/Post Infinity*
  do Unreal). Um ciclo infinito com 2 keys, sem duplicar nada — legendária.
- **ONDE:** a regra vive no flat-clamp de **`Track::sample` (track.rs:633-638)**, a
  camada mais interna. 2 campos novos em `Track` (`pre: Extrap`, `post: Extrap`,
  enum `{Hold, Loop, PingPong, Continue}`, default **Hold**) + um sub-módulo
  `extrap.rs` com a função PURA `fn extrapolate(keys, mode, t) -> value` (mapeia `t`
  de volta ao range: `rem_euclid` p/ Loop, reflexão p/ PingPong, `last + slope·(t−t1)`
  p/ Continue). `Track::sample` consulta `pre`/`post` no lugar do clamp direto.
- **⚠️ EXCLUIR o track de TIME REMAP:** `remap_through` (clock.rs:104-110) contorna
  o `Track::sample` fora do range e tem a PRÓPRIA regra (slope-1 / freeze no Hold).
  Consequência dupla: (a) a extrapolação nova é **inerte no track de TimeRemap por
  construção** (o clock nunca chega no sampler além do range), e (b) o painel **não
  oferece** o controle de extrapolação para `PropKind::TimeRemap`. Documentar nos
  dois lados.
- **SCHEMA:** **`DOC_VERSION 13→14`** — 2 campos apendados no serde do `Track`,
  default Hold ⇒ um documento v13 sem extrapolação re-serializa byte-idêntico; v13
  recusado no load (a regra `doc.rs:491-495`).
- **PINO DE FADE (o crítico desta wave):** o `hold_at`/`hold_source_time` do fade
  cruza para valores lidos pelo MESMO `Track::sample`, então uma extrapolação ≠ Hold
  muda o valor que o crossfade cruza. Por isso:
  1. **`fade fingerprint` com extrapolação DEFAULT (Hold/Hold) = byte-idêntico ao de
     hoje** — o gate que prova que a wave não moveu o precioso (é o `Track::sample`
     default que o fade continua vendo).
  2. Um gate separado mostra que loopOut só muda o valor quando **opt-in**.
  3. Uma cena que exercita **strip Loop + track loopOut juntos** (camadas diferentes,
     compõem) — prova que os dois não colidem.
- **UI:** controle por-track (menu R-click da row: "Extrapolation ▶ Pre / Post" com
  Hold/Loop/PingPong/Continue). Opcional: desenhar a curva extrapolada como fantasma
  pontilhado no graph (como o AE mostra o loopOut).
- **GATES:** cada modo reproduz o valor certo além dos keys (loop/pingpong/continue)
  · **default Hold byte-idêntico ao sample de hoje** · **fade fingerprint com default
  = idêntico** · TimeRemap ignora a extrapolação · rewind/scrub bit-exato · a cena
  strip-Loop×track-loopOut · mutações por modo.

---

## §7 — Expressões em propriedades (Wave C — ADR ANTES de construir)

> ✅ **CONSTRUÍDA (2026-07-26, pendente de smoke)** — [ADR-0144](../architecture/decisions/0144-timeline-expressions-frozen-ir-separate-post-composition-pass.md).
> Parser compartilhado (leaf `ph2d-expr-parse`, o Motion node delega) · passe
> `expr_pass.rs` pós-composição (early-out sem expr, `fade_fingerprint` intacto) ·
> `TargetBinding.expr` (`DOC_VERSION 14→15`) · intent `SetBindingExpr` · campo de
> texto no menu R-click da track (`expr_edit.rs`). `wiggle`/`time`/`value`/`Name.prop`.
> Smoke: `PH2D_EXPR_SMOKE=1`. Pick-whip = follow-up.

- **O QUE:** uma propriedade pode ser dirigida por uma FÓRMULA de tempo e/ou de
  OUTRAS propriedades — `wiggle(freq, amp)`, `time*v`, "linka a `X.position`". A
  feature mais amada do AE.
- **⚠️ NÃO existe mecanismo hoje** (confirmado: `ph2d-timeline`/`ph2d-anim` não têm
  expressão/driver). `ph2d-expr` é **IR + eval, SEM parser** (`Expr`, `eval`, `wgsl`,
  `stream`; `Func::Noise` existe); o parser VEX-lite é **crate-private** em
  `ph2d-node-motion-expression::parse` (o `motion.expression` dos Motion Nodes, outro
  sistema).
- **DESENHO DE ISOLAMENTO (o que protege o fade):** a expressão roda num **passe
  pós-composição SEPARADO**, em `apply.rs` **depois** que o stack compôs todas as
  props keyadas — lendo os valores **já compostos** da cena (`read_prop`,
  apply.rs:260) e escrevendo as props dirigidas (`write_prop`). **Nunca entra em
  `stack_eval`/no blend.** Documento sem expressão = **byte-idêntico**. Ciclos:
  avaliar contra os valores do frame **ANTERIOR** (modelo AE/Blender — dependência
  circular lê o último valor) ou ordem topológica quando acíclico.
- **ONDE:** campo novo `expr: Option<String>` por-binding/track (text param, o
  precedente é o text-param da Motion, que ficou FORA do contrato congelado) +
  um passe novo em `apply.rs`. Reusa `ph2d_expr::eval` + a trait `Bindings`. O
  parser: **decisão do ADR** (extrair/compartilhar o VEX-lite vs um novo pequeno).
- **SCHEMA:** **`DOC_VERSION 14→15`** (após a extrapolação) — append `expr`.
- **PINO DE FADE:** passe SEPARADO pós-apply; **fade fingerprint intacto**; arch-gate
  "expressão nunca chama `stack_eval`".
- **UI:** campo de texto na row/inspector (`ParamWidget::Text`, como a
  motion.expression) + **pick-whip** (arrastar pra linkar) — UI extra.
- **ADR OBRIGATÓRIO ANTES:** subsistema novo com alternativas reais (parser
  compartilhado vs novo · eval pré-frame vs pós-frame · política de ciclos · onde o
  texto mora · `wiggle` determinístico com tempo+seed). Rodar `pd-adr`.
- **GATES:** `time*10` = rampa · `wiggle` determinístico por seed · link A→B · ciclo
  A↔B lê o frame anterior sem explodir · **documento sem expr byte-idêntico** ·
  **fade fingerprint intacto** · **arch-gate: o passe nunca toca `stack_eval`**.

---

## §8 — O protocolo de segurança do fade (o guardião do precioso)

Um **gate compartilhado por TODAS as waves**: o **`fade_fingerprint`**. Uma cena
roteirizada com o crossfade em exercício — 2 strips sobrepostos (crossfade por
overlap) + um `lead_out` + um container aninhado — amostrada ao longo da
sobreposição e **hasheada**. Rodado **ANTES e DEPOIS de cada wave**; qualquer wave
que mova o hash **falha**. É a tradução executável de *"não pode ser afetado"*.

Complementos:
- **Nenhum edit de key novo emite intent de strip/lane/container** (arch-gate que
  enumera a superfície de §9 e prova que as intents novas não entram nela).
- **Append-only DOC_VERSION** (cada bump apenda no fim, v anterior recusado).
- Onde uma feature compartilha um sampler com o fade (a extrapolação, via
  `Track::sample`), o **default reproduz o hash byte a byte** — a interação indireta
  fica pinada, não confiada à prosa.

---

## §9 — Superfície de strip/lane/container (a NÃO tocar — de intent.rs)

Strips: `AddStrip`(472) · `RemoveStrip`(483) · `DuplicateStrip`(492) ·
`MoveStrip`(505) · `TrimStrip`(522) · `StretchStrip`(553) · `SetStripLoop`(566) ·
`SetStripSpeed`(584) · **`SetStripEase`**(605) · **`SetStripLead`**(630).
Lanes: `AddLane`/`RenameLane`/`RemoveLane`/`SetLaneMuted`/`SetLaneMode`/`SetLaneWeight`.
Containers: `AddContainer`/`RenameContainer`/`RemoveContainer`/`SetContainerLoop`/`SetContainerLength`.
Durações que clampam o clock: `SetSceneLength`/`SetClipLength`/`SetContainerLength`.
⚠️ `DeleteClip` alcança a superfície de strip via `repoint_strips_after_clip_removal`.

Todas despachadas por `edit_at(...)`. **Os edits de key das 6 features usam `edit()`
(sem `_at`) e a família `for_selected_tracks`/`ids_for` — jamais estas.**

---

## §10 — Ledger de schema (provisório; conta na integração)

| Wave | Feature | DOC_VERSION | O que apenda |
|---|---|---|---|
| A | Reverse/Stagger/Time-scale/Buffer | **13 (intocado)** | nada (edits de key / painel) |
| B | Extrapolação | **13→14** | `Track.pre`, `Track.post` (default Hold) |
| C | Expressões | **14→15** | `expr: Option<String>` por-binding |

`PROJECT_SCHEMA` **não** muda por conta destes (o `TimelineDoc` viaja como blob
dentro do `ProjectFile` e carrega a própria versão — o precedente de toda wave desta
timeline).
