# Plano 08 — Expressões como fonte de lane no blend (o padrão-ouro)

> ⚠️ **A cena `PH2D_EXPR_SMOKE` MORREU com o card de expressão (2026-07-30)** — o smoke do motor é o **`PH2D_EXPR_BLEND_SMOKE=1`**; ver [`14_a_autoria_de_expressoes_foi_retirada.md`](14_a_autoria_de_expressoes_foi_retirada.md).

> Executa [ADR-0146](../architecture/decisions/0146-timeline-expressions-are-a-first-class-lane-source-that-fades.md). Norte: **a expressão participa plenamente do fade/overlap/aditivo/container/prop-links, e o fade fica byte-idêntico onde não há expressão.** Custo não é restrição; a fidelidade do fade é inegociável.

A ordem é **dependência-dirigida e segurança-primeiro**: primeiro o andaime que prova a byte-identidade (nenhum comportamento novo), depois cada capacidade com seu gate RED-first. **O gate #1 (`the_fade_surface_is_byte_stable`, hash `0x69dca8811eb0f8f8`) roda em TODO commit da wave** — se ele mover sem re-pin justificado no MESMO commit, pare.

---

## W0 — O andaime da byte-identidade (nenhum comportamento novo)

**Objetivo:** o fork por presença + o `LinkFrame` fiado vazio, com a árvore **byte-idêntica**. Sem isto, todo o resto é arriscado.

1. `LinkFrame` = `BTreeMap<(u64,PropKind), f64>` + o `Name→entity` map (o `snap` do `expr_pass`, extraído para um tipo).
2. `eval_frame` ganha `links: &LinkFrame`, fiado pelo sítio único de recursão (`:142`) e pela recursão da referência aditiva (`:191-193`); `sample_stack`/`sample_stack_probed` ganham o mesmo parâmetro. **Frame vazio nunca é lido** ⇒ codegen muda, IEEE-754 não.
3. `apply.rs`: `let scheduled = any_global || any_clip` (o predicado **WIDENED**, corrige a assimetria `has_expr` de `:53`). **`!scheduled` roda `:57-115` verbatim e retorna** — o único caminho de um doc sem-fórmula, byte-idêntico e zero-alloc.
4. O `expr_pass::run` (`:127`) **fica** por ora (ainda chamado no ramo `scheduled`), para W0 não mudar comportamento nenhum.

**Gates (todos verdes, nenhum RED-first — é andaime):**
- `#1 the_fade_surface_is_byte_stable` (re-rodar antes/depois na árvore intocada).
- `#2 the_track_arm_adds_no_float_op` (mutação: rotear `Track` pela aritmética do `Expr` ⇒ hash move).
- `#3 no_expression_allocates_no_link_frame` (dhat: doc sem-fórmula não constrói `LinkFrame` nem topo-sort).

---

## W1 — A fonte de lane (per-clip vira fonte que fadeia) — O CORAÇÃO

**Objetivo:** no sítio de amostra (`:155-165`), a expressão per-clip é a contribuição da strip. Fade/overlap/aditivo/container passam a valer, de graça.

1. `enum AnimSource<'a> { Track(&Track), Expr { ir, value_track: Option<&Track> } }` no sítio de amostra; **o resto do arquivo verbatim.**
2. Os braços (ADR §2.3): `(None, tr, Some(ir))` ⇒ `value = tr.sample(t_src) ou rest`, `speaks = true` (**obrigatório**), `eval_expr(ir, value, t_src, seed, links)`. `seed = target.get()*SEED_SPACING`.
3. Referência aditiva (bloco `additive`, `:168-205`): `base = eval_expr(ir, value = reference(tr, src_in) ou rest, time = src_in, …)` — em tempo de clip. Delta = `E(t_src) − E(src_in)`; constante ⇒ 0 (Sum)/1 (Ratio).
4. **Remover** a rota per-clip do `expr_pass` para o caminho EMPILHADO (o `ExprWindow::Strips` fica obsoleto no playback).

**Gates (RED-first):**
- `#4` 2º fingerprint sobre região fadeada (`time*10` num crossfade 1 s; mutação sobrescreve ⇒ colapsa liga/desliga).
- `#9 an_expression_self_crossfades` (mesma clip, 2 strips; mutação mantém `PlaysTwice` ⇒ RED).
- `#7 an_additive_expression_contributes_a_delta` (mutação `base=0` ⇒ RED).
- `#10 lead_out_with_expr_fades_out` + `plays_twice_with_expr_drives_each_instance`.
- **(Hole A) `#5`** 3º fingerprint: canal keyado+fadeado NÃO-expressão **co-residente** com uma expressão (o caminho `scheduled==true`).

**Smoke (o fingerprint não vê — TEM de ser olhado):** `PH2D_EXPR_SMOKE` estendido — per-clip agora fadeia (era liga/desliga); `value` é per-strip; expressão pura **cobre** o canal (mascara lanes de baixo). Aprovação do Enio decide os defaults de comportamento.

---

## W2 — O sítio NÃO-EMPILHADO (o caso comum, C1)

**Objetivo:** um doc sem strips (animação keyada comum) dirige suas expressões. Sem isto, a correção W1 deixa o caso mais frequente **sem-dirigir em silêncio.**

1. Na rota solo/não-empilhada do `apply` (`:73`), canais de expressão resolvem via `eval_expr` sobre a amostra direta da track + o `LinkFrame` (o que o `ExprWindow::ActiveClip` faz hoje). **Dois sítios de amostra, declarados no ADR.**
2. `expr_pass::run` finalmente **deletado** aqui (o passe pós-composição some; `frame_solve.rs` já cobre os dois sítios).

**Gate (RED-first):** `#16 an_expression_drives_a_non_stacked_document` (mutação: rotear tudo por `eval_frame` ⇒ sem-dirigir, RED). **Nenhum teste atual cobre isto** — a fixture NÃO faz `add_lane`.

---

## W3 — Prop-links + ordem topológica (o escalonador `frame_solve.rs`)

**Objetivo:** `value + Sprite.x` acompanha a fonte fadeada (duplo fade); ciclos = um-frame-de-atraso determinístico.

1. `frame_solve.rs` (migração de `collect_links`/`resolve_link`/`topo_order`/`ExprBindings`): Fase 1 keyados verbatim (semeia `LinkFrame`), Fase 2 canais de expressão em ordem topológica (fonte antes do leitor), `frame.links.insert` **antes** de qualquer dependente ler.
2. **Ciclos (SCC):** semear do mundo, varrer `N_CYCLE=1` (o um-frame-de-atraso). Re-baselizar na descontinuidade (scrub/jump/load).
3. **(C4) A semente da Fase 1 lê Position pela TRAJETÓRIA** (espelhar `apply_path::read_rest`), não só "estender `read_prop`" — uma fonte Position **keyada** não é legível pelo `read_prop`. Morph idem. ⚠️ A extensão do `read_prop` **tem de ser inerte no caminho de captura de repouso** (`refresh_liveness_and_rest`), senão quebra a byte-identidade sem-fórmula de um canal Morph.

**Gates (RED-first):**
- `#8 a_prop_link_reads_the_faded_source` (mutação: compor o leitor antes da fonte ⇒ 1-frame-stale RED).
- `#14 the_scene_evaluates_in_dependency_order` (acíclico fresco; ciclo `N_CYCLE=1` estável).
- `#15 the_cross_os_hash_of_wiggle_plus_prop_link` (3-OS, estilo `physics_ecs_c9`). ⚠️ **só `wiggle`, NUNCA `sin`/`cos`** (transcendental do `std` não é cross-OS).
- **(Hole B) `#6`** corpus do fingerprint com Morph/Opacity/Position sob fade, OU gate provando a extensão do `read_prop` inerte no repouso.

**Notas de smoke:** prop-link é **inerte em lane aditiva** (cancela); um prop-link a Position/Morph só resolve se a fonte for legível.

---

## W4 — O global como transformação do canal (que NÃO fadeia)

**Objetivo:** a separação limpa ADR-0145. Um `binding.expr` global é aplicado como transformação final: `composed = eval_expr(global_ir, value = composed, time = cut_clock, …)`. Per-clip fadeia (fonte de lane); global não (fórmula do canal).

**Gate:** um driver global roda em todo lugar no cut clock; um per-clip janela com a strip. (Reusa/ajusta os gates de janela do ADR-0145.)

---

## W5 — Keying coerente (probe-through-expr + recusa honesta + C3)

**Objetivo:** `value + g(time)` key e pré-compensa; fórmulas puras/não-lineares recusam; **o caminho não-empilhado também pré-compensa.**

1. `invert_stack`: o probe substitui `v` **na expressão como `value`** e deixa a saída entrar no blend (o braço `(Some(p),_,Some(ir))`). O solve de três pontos existente decide keyabilidade **sem solver novo** (tabela ADR §2.3).
2. **(C3)** `key_value_in_active_clip` (`autokey.rs:304`): o early-return não-empilhado **não dispara** para canal de expressão; rota afim guarda `stored = want − g(t_key)`.
3. `refusal.rs`: `KeyRefusal::ExpressionDriven` (mensagem *"limpe/reescreva a fórmula"*, nunca "delete a lane").

**Gate (RED-first):** `#11` trio — `value_plus_g_of_time_keys_and_pre_compensates` (empilhado E não-empilhado); `a_pure_formula_refuses_ExpressionDriven`; `a_value_nonlinear_formula_refuses` (mutação: pular o 3º probe ⇒ chave errada, RED).

---

## W6 — O autokey/onion lê o `LinkFrame` persistido (a cura do seed==sample, C2)

**Objetivo:** o autokey não minta chave fantasma — **especialmente em canais com prop-link.**

1. **Persistir o `LinkFrame` composto no doc** (idioma `put_scratch`/`take_scratch`, `apply.rs:46/128`).
2. `shown_value`/`position_shown`/`pose_at` **leem** esse mapa para canais dirigidos — **uma derivação, `world == shown`**, nunca re-avaliam.
3. `pose_at` (onion) usa um `LinkFrame` degenerado (prop-links resolvem à pose viva): expressões locais são fantasmas exatos, cross-object são aproximados (limitação declarada e gateada).

**Gates (RED-first):**
- `#12 auto_key_mints_no_phantom_key_on_a_PROP_LINKED_channel` (**tem de ser prop-link entre objetos**, não expressão local; mutação: recomputar `shown_value` com `wiggle`/prop-link ⇒ fantasma por frame, RED).
- `#13 a_skipped_entity_is_left_alone_but_readable_by_a_prop_link` (mutação: ignorar `skip` ⇒ deriva ao pausar, RED).

---

## W7 — Completar o corpus de gates + aposentar o gate da isolação

1. Conferir que os Holes A/B (`#5`/`#6`) estão fechados sob mutação real.
2. **Aposentar `the_expression_pass_never_enters_the_blend`** (`tests/expressions.rs`) — decisão load-bearing (ADR §5.17): a isolação do ADR-0144 é trocada pela participação no fade *porque `#1`+`#2` provam o que ela afirmava*. Registrar a troca.
3. **Medir o custo** no gatilho nomeado: **centenas de canais com prop-link** (o escalonador re-roda `eval_frame` por canal + parse por-frame de 335 ns). O caminho sem-expressão é intocado.

---

## Matriz de smoke (o que o fingerprint NÃO pega — o Enio decide os defaults)

| Cenário | Env | Olhar por |
|---|---|---|
| Expressão per-clip fadeia com a strip | `PH2D_EXPR_SMOKE` | fade suave, não liga/desliga |
| `value` per-strip sob overlap | idem | as duas strips oscilam sobre os próprios valores e cruzam |
| Expressão pura cobre o canal (sparsity→mask) | idem | a lane de baixo/repouso fica mascarada |
| Prop-link acompanha fonte fadeada (duplo fade) | idem | `value + Sprite.x` segue Sprite fadeando E fadeia com a própria strip |
| Prop-link inerte em lane aditiva | idem | o termo de link cancela |
| Assimetria plays-twice (K recusa, playback dirige 2×) | idem | expressão toca em ambas as instâncias; K recusa com mensagem |
| Ciclo A↔B: um-frame-de-atraso; não-contrativo diverge | idem | estável acíclico; ciclo re-baseliza no scrub |

---

## Ordem de fechamento (Modo L)

Cada wave: `cargo check -p ph2d-timeline` no inner loop; no fim, `nextest` da crate + clippy + `#1` re-rodado. A linha **fecha, escreve o handoff de integração e PARA** — não integra nem pusha (ADR §0.7, ordem do Enio). O smoke inteiro é aprovado pelo Enio antes de qualquer integração.
