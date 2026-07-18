# Plano — nesting (containers de animação aninhados)

> Implementa o [ADR-0133](../architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md).
> Pesquisa: [`03_pesquisa_nesting.md`](03_pesquisa_nesting.md). Molde: [`02_plano_composicao_clips.md`](02_plano_composicao_clips.md).
> **Nada aqui começa antes do aceite do ADR pelo Enio.**

---

## §0 — Em uma frase

Um **container** é um clip promovido a cena: por dentro tem sua própria pilha de strips, por fora
é instanciado num `ClipStrip` como qualquer clip — e **o relógio é sempre do pai**.

## §1 — Estado de partida (o que já existe e é reaproveitado)

| Peça | Onde | O que dá de graça |
|---|---|---|
| Composição de tempo *outer-then-inner* | `stack_eval::strip_source_time` | a lei do relógio, já escrita e gateada |
| Recusa nomeada | `KeyRefusal` + `sole_strip_of` | *"toca duas vezes"* / *"não toca"* já respondidos |
| Hoist do relógio | `clock.rs::ClockIndex` | o pré-requisito de custo, **já pago** |
| Strip com todos os overrides | `ClipStrip` | `speed`/`src_in`/`src_out`/`loop_mode`/`ease`/`lead_in` |
| Crossfade por sobreposição | `ClipLane` | vale igual para container |
| Duas metades de UI | `Tab::{Keys, Arrange}` | *"uma régua mede um relógio"*, um nível acima |
| Dois `Playhead` reais | `App.playhead` + `App.clip_playhead` | as duas réguas do AE, sem infra nova |
| Organização sem tempo | `GroupedChildren` | o *grupo* do Harmony; o container é o irmão dele |

**Não se constrói:** mecanismo de instanciação novo, segundo grafo de cena, cache de container,
relógio autônomo.

---

## §2 — Fatia 0 — **a pergunta do z** (medição, antes de qualquer código de feature)

⚠️ **É a fatia que pode matar o desenho, e por isso vem primeiro.** O aviso é do Spine: o motor 2D
esqueletal mais maduro do mercado nunca implementou nesting em 10 anos porque *nesting e ordem de
desenho global brigam*. Nós temos a mesma tensão viva — o z-order é projeção da árvore única
([ADR-0110](../architecture/decisions/0110-vector-nodes-are-ecs-entities-one-hierarchy.md)).

**Duas perguntas, as duas por medição, nenhuma por opinião:**

1. **Onde a sub-árvore de cada instância entra na pilha de z?** Um container instanciado 3× tem 3
   lugares na pilha, e a árvore hoje tem um `RootOrder` por raiz.
2. **Quanto custa compor a IMAGEM de N instâncias?** É a armadilha (a) da pesquisa — cada nível
   não-colapsado materializa um raster intermediário. **O ADR mediu o custo de AVALIAR
   (~0,3% do frame), não o de COMPOR**, e é aqui que a diferença aparece.

**Saída da fatia:** um número, e a decisão (A)/(B) do ADR **resolvida por ele** — instância única
(barata, não é o multiplicador) vs instanciável N vezes (o que o briefing pediu).

**Aceitação:** um harness que compõe N ∈ {1, 4, 16} instâncias × profundidade ∈ {1, 2, 3} e
publica a tabela. Sem gate de barra: é **medição para decidir**, não regressão a proteger — o gate
de perf nasce na Fatia 2, contra o número que esta fatia produzir.

---

## §3 — Fatia 1 — dados + ciclo (`ph2d-timeline`, headless)

1. `ClipStrip.clip: u16` → `ClipStrip.source: StripSource{Clip(u16), Container(u16)}`.
2. `TimelineDoc` ganha a lista de containers (irmã da de clips), cada um dono da própria pilha.
3. **`DOC_VERSION` 7 → 8**, com migração total de v7 (`clip` → `Source::Clip`). Load rejeita
   versão desconhecida, como hoje.
4. Ciclo, **duas camadas independentes**: DFS ancestral na criação do link (`NestRefusal::WouldCycle`)
   + re-checagem no load com **rejeição**, nunca auto-reparo.

**Aceitação** (gates 1, 4, 5, 6 do ADR — cada um nasce VERMELHO):
- `a_container_instance_is_a_strip_and_reads_the_parents_clock`
- `linking_a_container_into_itself_is_refused_at_the_gesture`
- `a_cyclic_document_is_rejected_at_load_not_repaired`
- `a_v7_document_loads_with_its_strips_pointing_at_clips`

⚠️ **Gate POR CAMADA** ([[feedback_layered_defenses_need_per_layer_gates]]): neutralizar o DFS da
criação não pode ficar verde porque o load segurou, e vice-versa. Cada um tem mutação própria.

## §4 — Fatia 2 — o relógio recursivo (`ph2d-timeline`, headless)

1. A cadeia ganha o elo do container; `key_home` e a amostragem compõem pela **mesma** função em
   qualquer profundidade.
2. A recusa propaga pela recursão.

**Aceitação** (gates 2, 3, 8):
- `the_clock_composes_outer_then_inner_at_every_depth` — a 3 níveis, autoria e leitura dão o mesmo
  instante ([[feedback_derived_coordinate_seed_must_match_sample]], agora recursivo)
- `a_container_playing_twice_refuses_the_key_and_names_why`
- **perf**: 8 containers × profundidade 3 em **< 2×** o custo do mesmo número de bindings achatado.
  Bar em **RATIO**, nunca wall-clock — o `ci-test` compila em `opt-level=1` e o wall-clock mede o
  perfil, não a lei (e a própria baseline varia ~25% entre execuções).

## §5 — Fatia 3 — UI (`ph2d-panel-timeline` + shell)

1. **Breadcrumb** persistente (Animate/Harmony), com entrar/sair.
2. **As duas réguas juntas** (o mecanismo do AE): a de baixo = tempo do pai, a de cima = o da
   fonte. Nada de rótulo em texto — nenhum produto rotula, e a régua se explica por estar alinhada.
3. `Tab::{Keys, Arrange}` **não muda de significado**; passa a valer no nível em que a breadcrumb
   pôs você.

**Aceitação** (gate 7): seam que **CLICA** — entrar publica breadcrumb + as duas réguas, sair
restaura ([[feedback_widget_is_done_when_a_test_clicks_it]]). Tokens e i18n, zero string
hardcoded (HR-15). Ids novos registrados no `WidgetStore` — o X da timeline já foi esse bug uma
vez nesta linha (`9a67beb2`).

---

## §6 — Ordem, gates e fechamento

- **Fatia 0 é bloqueante.** Ela pode devolver (A) e reescrever as fatias 1-3.
- Fatias 1 → 2 são sequenciais (2 depende do dado de 1). 3 depende de 2.
- Gate batched **1× no fechamento** (CLAUDE §2): `nextest-impacted` + clippy `--all-targets` +
  auditoria ≥2 lentes. ⚠️ Os gates de LOC moram na `ph2d-editor-core` e **não rodam** com
  `cargo test -p ph2d-timeline` — rodar na árvore combinada.
- ⚠️ `stack.rs` está em **548 LOC** e `stack_eval.rs` em **549** (medido 2026-07-18); o cap é
  **700** (`architecture_workspace_file_loc_cap`, workspace-wide, nenhum dos dois em allowlist).
  Sobram ~150 linhas em cada, e o `StripSource` + a recursão comem isso. **Orce o split desde a
  Fatia 1** (o padrão da casa: módulo irmão, não allowlist). ⚠️ Rode `cargo fmt` **antes** de
  medir — o fmt re-expande ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]).
- **A linha fecha, escreve o handoff (DIRETRIZ §1.5.9) e PARA.** Não integra, não pusha.

## §7 — Fora de escopo (nomeado, não varrido pra debaixo do tapete)

| Item | Por quê | O gatilho que o acorda |
|---|---|---|
| Relógio próprio por container (*movie clip*) | maior fonte de confusão documentada; mata o scrub determinístico | máquina de estados / interação em runtime |
| Cache de saída do container | sem consumidor, cachear é escolher um modo de falha antes de ter o problema | o kill-criterion ser excedido por instâncias IDÊNTICAS (aí o desconto tem nome: *Master Pose Component* do Unreal) |
| Teto de profundidade | **não temos o recurso que o justifique** (§0.0); ninguém no mercado publica um medido | alguém medir memória ou recursão de pilha e escrever o número |
| Vetor/pintura dentro do container | o interior deste ADR é **animação** | ordem do Enio |
| Overrides ricos por instância (tint, blend) | o mínimo universal é transform + tempo + visibilidade; o resto é curadoria (o AE só expõe o que o autor marcou) | pedido real de artista |
