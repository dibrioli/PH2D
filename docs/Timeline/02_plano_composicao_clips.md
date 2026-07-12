# Plano — Composição de clips (empilhar clips)

> **Decisão:** [ADR-0115](../architecture/decisions/0115-clip-composition-sequencer-overlap-crossfade-sparse-lanes.md)
> — leia-o antes deste plano. Ele carrega a pesquisa (5 frentes), as 9 regras, o conjunto de aceitação
> **congelado** e o **kill-criterion**. Este arquivo é só o **como**.
>
> **Status:** proposto — aguarda ratificação do Enio. Nada de código antes disso.

---

## §0 — Em uma frase

Uma **faixa de clips** na timeline: você arrasta instâncias de clip pra ela, **sobrepõe duas e o crossfade
aparece**; várias faixas empilham (Override / Additive); cada faixa só toca os canais que o clip dela keya.

O que NÃO é: o strip-stack do Blender (blend por strip, tweak mode, 5 modos). A pesquisa mostrou que o
próprio Blender está saindo dele — ADR-0115 §1.

---

## §1 — Estado de partida (o que já existe e é reaproveitado)

| já temos | onde | serve pra |
|---|---|---|
| `NamedClip` + `clips()` + dropdown de clip ativo | `ph2d-timeline/src/doc.rs`, ETAPA 3 | o **seletor** que elimina o tweak mode (ADR §2/R8) |
| bindings **document-wide** (mesmo `AnimTarget` em todo clip) | `doc.rs:140` | o **join key** entre clips — de graça |
| `remapped_time` (relógio por-entidade, modelo precomp AE) | `apply.rs:89` | o mapa de tempo *interno* ao clip (ADR §2/R6) |
| `Interp::BezierW` + solver Newton | `ph2d-anim/src/curve_weighted.rs` | a curva de ease/blend, sem motor novo |
| régua, `TimeView`, drag da loop-brace, box-select | `ph2d-panel-timeline` | a maquinaria de arrastar/redimensionar strip |
| voltas acumuladas na rotação (§6.1) | `gizmo/drag.rs` | `Rotation` vira escalar sem ambiguidade ±2π → blendável |
| `Playhead` + loop/ping-pong **por clip** | `playhead.rs`, `doc.rs` (v3) | transporte; **não** muda |

**Nada disso precisa ser construído.** O trabalho novo é o modelo de pilha, o avaliador e a faixa na UI.

---

## §2 — Fatia A — dados + avaliação (`ph2d-timeline`, headless)

**Entrega:** aceitação ADR §3.1–§3.8 + o kill-criterion **medido**.

| # | tarefa | nota |
|---|---|---|
| **A0** | **Hoistar o `remapped_time`**: O(B²) → O(B). Um passe que resolve o relógio **por entidade** (não por binding) antes do laço de escrita. | **Pré-requisito do kill-criterion**, não bônus. O baseline de hoje é quadrático (ADR §4) — empilhar em cima disso vira cúbico. Gate: bench antes/depois. |
| **A1** | `ClipLane` / `ClipStrip` + serde. `TimelineDoc.stack: Vec<ClipLane>` e `TargetBinding.rest: f32` **apendados**. `DOC_VERSION` 3→4; v3 **rejeitado** (postcard é posicional). | Módulo **irmão** `stack.rs` (isolamento, DIRETIVA §1) — `doc.rs` só ganha o campo. |
| **A2** | `strip.map(t) -> Option<f64>`: `t_start..t_end` → `src_in..src_out`, com `speed` e `loop_mode` (Once/Loop/PingPong/Hold). Fora do span → `None` (a faixa não cobre). | Teste por **valor amostrado**, não por campo existir (ADR §3.3). |
| **A3** | `strip.weight(t) -> f32`: a curva de ease. **Complementar por default** (soma 1 na sobreposição). `ease_in`/`ease_out` **é** `blend_in`/`blend_out` — um campo só. | Reusa `Interp`/`BezierW`. Curva default = ease S (Unity), não linear. |
| **A4** | `blend_op(PropKind)`: neutro + `⊕`/`⊖` por canal. Translation/Rotation somam; **Scale/Opacity multiplicam** (razão). | O bug que criou o COMBINE do Blender: 1.0 + 1.0 = 2.0. Uma função, não um `if` espalhado. |
| **A5** | **Avaliador**: por `(AnimTarget, t)` — dentro da faixa **normaliza** (`Σwv/Σw`) e devolve `(valor, coverage)`; entre faixas acumula bottom→top (Override lerpa; Additive soma o delta vs **primeiro frame do clip**). Faixa mutada não entra. | ADR §2/R3–R4. `coverage` e `valor` são grandezas **separadas** — é o que mata o "afundar pro default". |
| **A6** | `apply_from_doc` ramifica **num ponto só** ([apply.rs:62-71](../../crates/ph2d-timeline/src/apply.rs)): `stack` vazia → o `active_clip()` de hoje (byte-idêntico); senão → o avaliador. O relógio compõe **pra dentro**: `clip.remap(entity, strip.map(t))`. | ADR §2/R6. `remapped_time` recebe `&Clip`. |
| **A7** | `rest` capturado no `bind()`, re-capturável. É a base do fade-in "do nada" na faixa de baixo. | Capture Base State (Rive) / Base Pose (Unreal) — ADR §2/R5. |
| **A8** | **Autokey sob pilha**: inverte as faixas acima do clip ativo; não-inversível (`w→1`) → **recusa + toast**. | ADR §2/R9. Nunca mover o objeto em silêncio. |
| **A9** | **Gate R7** (executável): todo `PropKind` é um escalar blendável; um prop discreto no `stack` = **vermelho**. | `AnimValue::lerp` "blendaria" um `Bool` com step — errado e silencioso. |
| **A10** | Gates: zero-alloc (`no_alloc_bridge`), byte-identidade com `stack` vazia, bench 50 bindings × 4 faixas vs baseline. | **Kill-criterion** (ADR §4). |

## §3 — Fatia B — UI (`ph2d-panel-timeline`)

**Entrega:** aceitação ADR §3.9. **Mesma jornada que a Fatia A** — dados sem UI é fio órfão (DIRETIVA §1:
proibido armar e "fiar depois"; sem UI ninguém consegue criar um strip).

| # | tarefa | nota |
|---|---|---|
| **B1** | **Faixa de clips** no dope-sheet (rows novas, acima das tracks). Strip = retângulo com nome do clip; sobreposição desenha o X do crossfade. | Reusa `TimeView` + o pipeline de rows. |
| **B2** | **Criar strip**: arrastar do dropdown de clips pra faixa, ou botão "+ Strip" com o clip ativo. | 1 undo step. |
| **B3** | **Arrastar / redimensionar** (bordas = `src_in`/`src_out` por default; com modificador = trim). **Sobrepor dois = o crossfade nasce** (sem diálogo). | O gesto convergente (ADR §1.3). |
| **B4** | **Ease handle** no canto do strip (padrão Unreal: alça direta, não campo de Inspector). Vira read-only quando um vizinho define a duração (padrão Unity). | Ease e blend = a MESMA curva. |
| **B5** | **Cabeçalho da faixa**: nome, mute, **weight**, **mode** (Override/Additive). Blend e influence vivem na **faixa**, não no strip. | A lição central do Blender novo (ADR §1.1). |
| **B6** | R-click no strip: Delete · Duplicate · loop mode (Once/Loop/PingPong/Hold) · speed. | Espelha os menus de key que já existem. |
| **B7** | Toast de recusa do autokey (A8). | i18n EN, token — HR-15. |
| **B8** | Seam tests por `WidgetEvent` real; cada gesto = **1** undo step. | Gate anti-item-morto (o padrão do Delete Track). |

## §4 — Ordem, gates e fechamento

1. **A0** (hoist) → mede o baseline. Se o hoist não sair, **pare**: o kill-criterion não é avaliável.
2. **A1–A7** (modelo + avaliador) → aceitação §3.1–§3.6 verde.
3. **A8–A10** (autokey + gates) → aceitação §3.7–§3.8 verde + kill-criterion **medido**.
4. **B1–B8** → aceitação §3.9 verde.
5. **Gate batched** (1× no fim): `nextest-impacted` + clippy `--all-targets` + auditoria ≥2 lentes + LOC caps.
6. **Handoff de integração** (DIRETRIZ §1.5.9) e **PARE** — a linha não integra nem shippa (CLAUDE §0.7).

**LOC:** `apply.rs` cresce; o avaliador vai em `stack.rs` (irmão) desde o 1º commit — não deixar pra
depois do cap ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]).

## §5 — Fora de escopo (nomeado, não varrido pra debaixo do tapete)

- **Nesting** (container animado com relógio próprio — símbolo/precomp/artboard aninhado). É **o idioma 2D
  de reuso** e nós temos zero. **Próximo ADR** (ADR-0115 §5).
- **Blend por parâmetro / state machine** (Rive Blend 1D, Smart Bone do Moho). *Blend-paramétrico e
  frame-pick são a MESMA UX* — casa com os Motion Nodes **e** com o Flip. Follow-up.
- **Transition strip** explícito (o strip cinza do Blender/Maya pra *lacunas*). Sobreposição cobre o caso
  comum; lacuna fica pra depois se doer.
- Rig/skinning, Combine com rest-pose de esqueleto — deferidos pro fim de tudo (ADR-0108).
