# ARQUIVO — CLAUDE.md (história, 23 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`CLAUDE.md`](../../../CLAUDE.md) em 2026-10-02, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-507,531-542` do original.
>
> ⚠️ **A única alteração ao corpo:** 14 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

- **Timeline** — dope-sheet + graph editor + transporte sobre `ph2d-core::Playhead` (`ph2d-panel-timeline` +
  `ph2d-timeline` + `ph2d-anim`): curvas com handles bézier e weighted tangents, roving keys, time remap, record,
  clips + **composição** ([ADR-0115](../../architecture/decisions/0115-clip-composition-sequencer-overlap-crossfade-sparse-lanes.md)),
  **nesting** ([ADR-0133](../../architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md)),
  duração explícita, motion path ([ADR-0141](../../architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md)), onion ([ADR-0142](../../architecture/decisions/0142-timeline-onion-ghost-poses-non-destructive-pose-at.md)),
  retiming, extrapolação, **sinais** ([ADR-0143](../../architecture/decisions/0143-timeline-signals-a-marker-emits-a-decoupled-event-not-a-call.md)) e expressões
  ([ADR-0144](../../architecture/decisions/0144-timeline-expressions-frozen-ir-separate-post-composition-pass.md) / [0151](../../architecture/decisions/0151-timeline-expressions-are-per-clip-so-a-strip-windows-them.md) / [0152](../../architecture/decisions/0152-timeline-expressions-are-a-first-class-lane-source-that-fades.md)).
  ⚠️ O `TimelineDoc` viaja como **blob dentro do `ProjectFile` e carrega a própria versão** — é por isso que ele evolui
  sem mover o `PROJECT_SCHEMA` — e desde 25/09 essa porta (gravar · reinstalar · reencontrar o objecto de uma track
  depois de um respawn) vive na crate-folha [`ph2d-timeline-persist`](../../../crates/ph2d-timeline-persist), que desceu da
  shell na integração da rodada 03. ⚠️ A **AUTORIA** de expressões foi **retirada** (o motor ficou; registro em
  [doc 14](../../Timeline/14_a_autoria_de_expressoes_foi_retirada.md)) — remover a feature **não** removeu o schema.
  **Aberto:** a expressão **PURA** (sem keys) extrapola a strip — ligar exige vínculo autorado (produto + provável
  `DOC_VERSION`) · **W4.T4** (o dock da timeline dentro do Motion) aguarda re-smoke: duas linhas discordaram, o código
  do `main` shipou com cap e gates, e a nota da rejeição saiu do `layout.rs` · o catálogo de receitas morreu, a pesquisa não.
  **Smokes:** `PH2D_NEST_SMOKE=1..3` · `PH2D_PATH_SMOKE=1|2` · `PH2D_ONION_SMOKE` · `PH2D_TIMESCALE_SMOKE` ·
  `PH2D_STAGGER_SMOKE` (⚠️ **Ctrl**+drag, o KDE rouba o Alt) · `PH2D_BUFFER_SMOKE` · `PH2D_EXTRAP_SMOKE` ·
  `PH2D_SIGNAL_SMOKE` · `PH2D_EXPR_BLEND_SMOKE` · `PH2D_MORPH_FADE_SMOKE`. ⚠️ `PH2D_EXPR_SMOKE` **morreu** com o card.
  ⚠️ **Flake conhecida e PRÉ-EXISTENTE:** `the_cost_of_depth_is_linear_not_explosive` é gate de RAZÃO sensível a carga —
  re-rode sozinho antes de suspeitar de um merge.
  **Ler:** [`docs/Timeline/`](../../Timeline) · [`BUGS_timeline.md`](../../Timeline/BUGS_timeline.md) ·
  [handoffs](../../Timeline/handoffs/README.md) · [história](../estado-2026-08-18/timeline.md)

