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
+ 1 docs: ADR-0146 Accepted e CONSTRUÍDO
```

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
- **Expressão pura sem keys extrapola a strip** — herdado do ADR-0144 (ela não tem track, logo nenhuma strip a janela); ligar isso exige vínculo autorado + provavelmente `DOC_VERSION`. Fora desta wave.
