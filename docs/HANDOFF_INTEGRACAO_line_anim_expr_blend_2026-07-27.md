# Handoff de integração — `line/anim` · Expressões como fonte de lane que FADEIA (ADR-0146)

**Data:** 2026-07-27 · **Linha:** `line/anim` · **Estado:** FECHADA, gateada, aguardando smoke do Enio + ordem de integração (§0.7 — a linha NÃO integra nem pusha sozinha).

## Uma frase

Uma expressão de propriedade da timeline deixou de ser um passe pós-composição que **sobrescreve** e passou a ser **o valor que a strip entrega ao blend** — então **fade / crossfade / aditivo / container** valem para ela de graça, pela mesma máquina — e **o fade extraordinário fica byte-idêntico onde não há expressão** (fingerprint `0x69dca8811eb0f8f8` intacto).

## O que muda para o artista (SMOKE — o fingerprint não vê isto)

- Uma expressão numa strip agora **fadeia com a strip** (antes ligava/desligava seco), **cruza** com a vizinha e **soma** numa lane aditiva.
- `value + Sprite.x` **acompanha o Sprite mesmo enquanto ELE fadeia**, e a própria strip do seguidor fadeia o resultado (**duplo fade**).
- Uma expressão pura (sem keys) agora **cobre o canal** (mascara lanes de baixo).
- ⚠️ **`value` virou per-STRIP** (o modelo do After Effects: uma strip é uma camada) — mudança de comportamento para expressões per-clip que já existam. **TEM de ser vista num smoke.**
- **Keying:** `value + g(time)` **key e pré-compensa** (guarda `want − g(t)`); fórmula pura (`wiggle`) ou não-linear (`value*value`) **recusa** com mensagem *"clean or rewrite the formula"* (`KeyRefusal::ExpressionDriven`), nunca "delete a lane".
- **Sem chave-fantasma:** um canal prop-linkado parado não minta mais key por frame.
- **Onion:** um fantasma de expressão LOCAL agora é exato (antes mostrava a track crua); prop-link entre objetos é aproximado (lê a pose viva da fonte — limitação declarada).

## Como smokar

Não há env-smoke novo — é a timeline normal. Sugerido:
1. Um objeto com per-clip expr `time*10` numa strip que fadeia → ver o **fade suave** (não liga/desliga).
2. `Sprite.x` num seguidor + Sprite fadeando num crossfade → o seguidor **acompanha o valor fadeado**.
3. Uma lane aditiva com expressão constante → contribui **0** (o termo cancela).
4. Autokey armado, cena parada com um canal `value + Sprite.x` → **nenhuma key fantasma**.
5. Keyar (K) um canal `value + wiggle` → **recusa** com a mensagem da fórmula; keyar `value + 100` → **pré-compensa**.
6. `PH2D_EXPR_SMOKE=1` (o smoke de expressão pré-existente) segue válido.

## Commits (10 de feature + 2 de higiene, todos LOCAIS)

```
c2d5827da W0  -- LinkFrame fiado vazio + fork scheduled (byte-identidade)
9450c2724 W1  -- per-clip vira fonte de lane que FADEIA (o coração)
70bf418c1 W2  -- o sítio NÃO-EMPILHADO dirige expressões (C1)
54a3bae91 W3  -- prop-links leem a fonte FADEADA (ordem topológica; ciclo = 1-frame-delay)
41b306076 W3f -- o mesmo nas 2 VISTAS de edição (Containers/Keys)
d880fe3a5 W4  -- global = transformação do canal (NÃO fadeia); doc do expr_pass
0e4099e77 W5  -- keying coerente (probe-through-expr + C3 não-empilhado + reason)
149ed53ef W6  -- autokey/onion lê o LinkFrame PERSISTIDO (cura do fantasma, C2)
1865ac27f W7  -- corpus (#15 cross-OS, #6 Hole B) + aposenta #17 + mede o custo
546a14cf2     -- split doc_scratch.rs (LOC cap)
82ead2be5     -- cena de smoke PH2D_EXPR_BLEND_SMOKE
+ 1 docs: ADR-0146 Accepted e CONSTRUÍDO
```

## Correções pós-smoke (Enio, 2026-07-27 — reports do 1º smoke da wave)

```
81a5f1b95  fix: o gizmo do loop obedece o CONTAINER aberto (nao trava)
7358c19cc  fix: projeto sem timeline abre com 4s autorados (veu + corte das expressoes)
c05f5111d  fix(smoke): o expr-blend smoke autora a duracao dos clips (veu na aba Keys)
ef9d51a87  fix: Arrange e um escopo INDEPENDENTE (Bugs 6+8) -- fim do "single clip is the timeline"
3066d5e56  feat: a expressao PURA obedece a janela da composicao (o item "Aberto" abaixo, FECHADO)
```

Quatro reports, **três eram um por-linha e um por-composicao**:

- **Loop gizmo TRAVA dentro de um container** (`81a5f1b95`): a brace do loop e DESENHADA
  do `container_loop` (snapshot §487), mas o drag streamava `SetLoop`, cujo handler parkeia
  no loop do clip/Arrange via `keys_mode` — escrevia onde nada e lido. Agora o drag ramifica
  em `snap.container_open` → `SetContainerLoop`, o MESMO split do toggle Loop/PingPong
  (`timeline_bridge.rs`) e do `duration_drag` (`length_scope`). Gate + mutacao no `loop_drag`.
- **"Duracao padrao do clip deve ser 4s mas abre em 0 / veu so aparece se digitar" + "expressoes
  tocam fora da area valida do clip"** (`7358c19cc`): eram **UM** — uma composicao de duracao 0.
  Um `Ctrl+O` de projeto sem animacao instalava `TimelineState::new()` DERIVADO (`scene_length`
  None, clips sem override), e a sessao abria Dur 0, sem veu, e as expressoes PURAS extrapolando
  (nada as cortava). `install_from_project` agora usa `with_default_duration()` no caso vazio,
  a MESMA composicao de 4s do boot. **MEDIDO:** uma expressao per-clip JA congela no corte
  autorado (t=3 sobre clip de 2s → 20, nao 30) via `cut_source`/`stack_frames` — o vazamento
  era so a AUSENCIA de duracao. Gate novo (`a_per_clip_expression_freezes_at_the_clips_authored_cut`)
  + gate do load (`an_empty_project_opens_with_the_default_four_second_composition`), 2 mutacoes.
  A cena de smoke tambem autora `scene_length=6s` (casava conteudo [0,6) com scene de 4s).
- **"Ao deletar as strips de Arrange as expressoes dos clips foram deletadas" — SEM defeito, SEM
  data loss (verdict b).** `remove_strip` (`stack_edit.rs:133`) so tira a strip da lane; NAO toca
  `NamedClip.expr`. A formula continua no clip (o campo Expression da track a mostra) — ela apenas
  PARA de dirigir porque uma expressao per-clip so dirige enquanto uma strip toca o clip (o design
  da wave, ADR-0145/0146: *"value virou per-STRIP"*). So a **lixeira Delete Clip** (`transport_clips`)
  apaga a expressao (dropa o clip inteiro). **Decisao de PRODUTO do Enio:** manter o per-strip
  (uma expressao e uma camada, fadeia com a strip) ou querer uma expressao GLOBAL que sobrevive
  (a UI hoje so autora per-clip via `SetBindingExpr` → `set_clip_expr`). Nada a corrigir sem essa
  ordem — nao ha perda de dados.
```

**2ª rodada (Enio, 2026-07-28) — três correções de smoke:**

- **O véu não aparecia antes de tocar na caixa de Dur** (`c05f5111d`): não era bug de produto (boot + clips do usuário já mostram o véu — repro headless confirmou). Era o **smoke**: a cena `PH2D_EXPR_BLEND_SMOKE` montava os clips por `doc.add_clip` cru (derivado, sem duração), então a aba Keys abria sem véu. A cena passou a autorar `set_clip_length_override(i, Some(6.0))` (+ gate de fonte). Smoke OK.
- **Arrange é um escopo INDEPENDENTE (Bugs 6+8)** (`ef9d51a87`): as duas queixas tinham a mesma raiz — o atalho *"sem stack o clip É a timeline"*. Três cortes: `keys_mode = shows_keys()` (a aba, não `&& stacked()`) ⇒ Keys=escopo do clip, Arrange=escopo da cena, **independentes** · `apply_scene` (aditivo) força o caminho de stack ⇒ Arrange vazio blenda p/ **rest** (toca NADA), o solo single-clip fica intacto p/ Keys · `view_authored_end` sem o fallback no-stack→clip (Arrange lê só `scene_length`). 3 gates, 3 mutações sangram. Smoke OK.
- **Vincular a expressão pura** (`3066d5e56`): ver o item ~~Aberto~~→**FECHADO** abaixo.

## Impacto de integração — **LIMPO**

- **Nenhum `DOC_VERSION`, nenhum `PROJECT_SCHEMA`.** A per-clip expr usa `NamedClip.expr` (já existe desde ADR-0144); o `composed_links` mora na **scratch** (não serializada, `PartialEq` sempre igual → sem passo de undo espúrio). `KeyRefusal::ExpressionDriven` é runtime.
- **Nenhum contrato congelado tocado** (§6): `NodeOp`/`OpResolver`/`NodeManifest`/`Tool` intactos (isto é timeline, não nodes/tools).
- **Nenhum ADR novo além do 0146.** ⚠️ **O número 0146 é PROVISÓRIO** — conta na integração a partir do `main` do dia; se colidir, quem chega primeiro fica com ele (gate `architecture_adr_numbers_are_unique`).
- **Foundational tocado, todo aditivo:** `frame_solve.rs` (novo, o escalonador da Fase 2) + `doc_scratch.rs` (novo, split). `StackScratch` ganhou o campo `composed_links`. `key_value_in_active_clip`/`invert_stack` viraram `Result<f32, KeyRefusal>` (o reason viaja ao shell).
- **Superfície pública mudada (crate `ph2d-timeline`):** `key_value_in_active_clip(doc, e, prop, want)` → `(..., want, t) -> Result<f32, KeyRefusal>`; `KeyRefusal::ExpressionDriven` novo. O único chamador de shell (`timeline_bridge::key_value_for`) já foi atualizado (`.ok()?` + passa `t_secs`).
- **1 arquivo de shell tocado:** `shells/desktop/src/render_loop/timeline_bridge.rs` (o chamador de `key_value_in_active_clip`). Compila.

## Gates (16 no corpus, todos mutação-provados; RED-first onde marcado)

O fade (a joia): `#1 fade_fingerprint` (0x69dca8811eb0f8f8, formula-free) · `#3 no_expression_allocates_no_link_frame` (dhat) · `#4 an_expression_fades_with_its_strip` · `#5 a_keyed_fade_co_resident...` (Hole A) · `#6 a_multi_channel_keyed_fade_co_resident...` (Hole B — prova: ScaleX↔ScaleY deixa #6 RED e #5+fingerprint verdes).
Participação: `#7 additive delta` · `#8 prop_link_reads_the_faded_source` · `#9 self_crossfades` · `#10 lead_out fades` · `#16 non_stacked drives`.
Keying: `#11` trio (`value_plus_g_of_time_keys_and_pre_compensates` · `a_pure_formula_refuses_ExpressionDriven` · `a_value_nonlinear_formula_refuses`).
Fantasma: `#12 auto_key_mints_no_phantom_key_on_a_PROP_LINKED_channel` · `#13 a_skipped_entity_is_left_alone_but_readable_by_a_prop_link` · `#20 the_onion_ghost_evaluates_a_local_expression`.
Ordem/determinismo: `#14 the_scene_evaluates_in_dependency_order` · **`#15 the_cross_os_hash_of_wiggle_plus_prop_link`** (0x6ed2_84e3_8f4f_28f9 — ⚠️ roda na matriz 3-OS do nextest; SÓ wiggle, nunca sin/cos).
Aposentado: `#17 the_expression_pass_never_enters_the_blend` — decisão load-bearing registrada (ADR §5.17): a isolação do ADR-0144 é trocada pela PARTICIPAÇÃO (per-clip compõe DENTRO do blend), e a byte-identidade fica provada direto pelos fingerprints.

**Custo medido** (`#[ignore]` `measure_hundreds_of_prop_link_channels`): 300 prop-links ENCADEADOS = **1,86 ms/frame (debug)**, teto do gatilho; o caminho sem-formula paga zero (gate #3). Sem cap.

## Fechamento

- `cargo test -p ph2d-timeline` = **436/0** (⚠️ o flake pré-existente `the_cost_of_depth_is_linear_not_explosive` de `nesting_clock.rs` é sensível a carga — re-rode sozinho antes de suspeitar).
- `cargo clippy -p ph2d-timeline --all-targets` limpo · `cargo check -p ph2d-host-desktop` limpo.
- LOC caps (`architecture_workspace_file_loc_cap` + shell `file_loc_caps`) verdes.

## Aberto (declarado, não bloqueante)

- **C4:** `read_prop` NÃO foi estendido para Position/Morph — um prop-link a `Nome.position`/`Nome.morph` resolve 0, e a aresta-de-volta de um ciclo nesses canais semeia 0. Limitação declarada (estender quebraria a byte-identidade do repouso; ver #6). Prop-links a canais scalar de transform (X/Y/rot/scale) funcionam.
- **Onion cross-object:** o fantasma de um prop-link entre objetos lê a pose da fonte NESTE playhead, não no tempo do fantasma (o onion não tem grafo cross-time). Expressão local é exata.
- **Ciclo não-contrativo** (ganho ≥ 1) diverge ENTRE frames (padrão da indústria; re-baseliza na descontinuidade). `N_CYCLE=1`.
- ~~**Expressão pura sem keys extrapola a strip**~~ — **FECHADO** (`3066d5e56`, "vincular a expressão pura", ordem do Enio 2026-07-28). **Medido headless:** a raiz é única — um clip de expressão pura (fórmula sem keys) tinha `clip_end_seconds == 0` (a fórmula não tem última chave que a encerre), com dois sintomas: em Keys/solo `clip_cut` (só `length_override`) rodava a fórmula no playhead cru p/ sempre (`x(6)=600` p/ `time*100`); em Arrange o strip nascia com slice 0 (`source_length == clip_end_seconds == 0`) e lia clip-time 0 p/ sempre (`E(0)=0`). ⚠️ **O produto JÁ estava correto** — todo clip autorado ganha 4s (`with_default_duration`/AddClip), o que janela a pura nos dois lados; o buraco só reabre ao **limpar o Dur** (=0) ou num save legado. O fix torna correto POR CONSTRUÇÃO: um clip que carrega expressão e não keya nada ganha `DEFAULT_DURATION_SECONDS` como fim de composição mesmo sem override — `clip_end_seconds` (janela do strip) e `clip_cut` (corte solo/Keys) pela mesma medida (`clip_keyed_end`). **Inerte** p/ clip formula-free (`!expr.is_empty()` nunca dispara ⇒ fingerprint `0x69dca8811eb0f8f8` byte-idêntico) e keyado (o fim keyado vence). **Sem `DOC_VERSION`** — o campo `expr` já existe (v16), é comportamento derivado; bumpar recusaria todo projeto salvo à toa (a política do ADR-0146 §6). 4 gates red-first (`pure_expression_window.rs`), 4 mutações, 4 sangram (600 · 0 · 4 · 4); o hash cross-OS `#15` autora 8s p/ manter as 121 amostras distintas (pin intacto). ⚠️ **Pendente de smoke:** limpar o Dur de um clip de expressão pura e conferir que ela congela no fim da composição em vez de rodar sem fim.
