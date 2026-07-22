# HANDOFF DE INTEGRAÇÃO — `line/Vector` (2026-07-21)

**Para:** o agente INTEGRADOR (DIRETRIZ §1.5.3–1.5.4).
**De:** a linha `line/Vector`, que fechou o **Expand** (Outline Stroke · Power Stroke · **Offset
vivo**), o **Chamfer**, e o **undo da pilha de efeitos**.
**Estado:** ✅ **fechada, TODOS os smokes aprovados pelo Enio.** A linha NÃO integrou e NÃO pushou.

> ⚠️ Este documento é o que evita conflito e regressão. Ele **não** duplica o detalhe técnico —
> esse mora em [`HANDOFF_line_vector_TROCA_2026-07-20_offset_vivo.md`](HANDOFF_line_vector_TROCA_2026-07-20_offset_vivo.md)
> (o modelo do Offset vivo, §0) e em [`Vector Module/BUGS_vector.md`](Vector%20Module/BUGS_vector.md) #18
> (a saga, com os 4 padrões que ela pagou).

---

## 1. Identidade

| | |
|---|---|
| **Branch / worktree** | `line/Vector` — `Worktrees/line-Vector/` |
| **HEAD** | `35c0d32d` |
| **Merge-base com `main`** | `5cc54941` |
| **Commits à frente** | **45** · **0 atrás** (a `main` não andou desde o fork) |
| **Árvore** | limpa |
| **Suíte** | 898 unit (bins) + 933 com integração · **0 falhas** · clippy `--all-targets` limpo · `cargo check --workspace --all-targets` limpo · typos ok |

⚠️ **`0 atrás` é de HOJE.** Se outra linha integrar antes desta, rebase primeiro (DIRETRIZ §1.5.2.3)
e re-rode `check --workspace` na árvore combinada — um merge textualmente limpo pode estar
semanticamente quebrado.

---

## 2. Foundational / compartilhado tocado, e por quê

Tudo aditivo salvo onde marcado. Nenhuma crate nova.

| Arquivo | O quê | Aditivo? |
|---|---|---|
| `crates/ph2d-ecs/src/vec_offset.rs` | **arquivo NOVO** — o componente `VecOffset{d, join, side}` (o Offset vivo) | ✅ novo |
| `crates/ph2d-ecs/src/lib.rs` | `mod vec_offset;` + `pub use vec_offset::VecOffset;` | ✅ 2 linhas |
| `crates/ph2d-ecs/src/scene/registry.rs` | registro do `VecOffset` — **e o contador subiu 32 → 33** | ⚠️ **ver §3** |
| `crates/ph2d-editor-core/src/ids/chrome/vector.rs` | 23 ids novos (seção Expand + Apply de efeitos + os 2 modos de quina) | ✅ novos |
| `crates/ph2d-editor-core/src/interaction/state/store_core.rs` | `Store::set_slider_value` — o painel republica o slider quando a seleção muda de offset | ✅ método novo |
| `crates/ph2d-editor-core/tests/node_id_collisions.rs` | os ids novos entram na varredura de colisão | ✅ |
| `crates/ph2d-i18n/src/lib.rs` | 3 chaves: `panel.vector.section.expand`, `…mode.fillet`, `…mode.chamfer` | ✅ |
| `crates/ph2d-vec-scene/` | `stroke_style.rs` / `width_profile.rs` / `stroke_plan.rs` (**arquivos novos**, extraídos do `lib.rs` pelo teto de LOC) + `path_ops::curve_bbox_in_frame` (pub novo) + `effect.rs`/`corner_live.rs`/`geometry.rs` (aditivo) | ⚠️ **ver §3** |
| `crates/ph2d-vec-boolean/` | `expand.rs` + `expand_ribbon.rs` (**novos**) — `offset_path` / `outline_stroke` / `power_stroke` / `MIN_OFFSET` | ✅ novos |
| `crates/ph2d-vec-render/src/lib.rs` | `pub type LiveGeometry` novo; **`dispatch` ganhou o parâmetro `live` (5→6 args)**; `corner.rs` e `markers.rs` **REMOVIDOS** | ⚠️ **ver §3** |
| `crates/ph2d-vec-edit/` | `corner_tool.rs` (novo); `corner_handle.rs` encolheu (a alça do Node saiu — virou ferramenta) | ⚠️ **ver §3** |
| `crates/ph2d-panel-vector/` | seção **Expand** (`paint_expand.rs` novo) + `populate_style.rs` (novo, split de LOC) + Apply na seção Effects + os 2 chips de quina | ✅ do módulo |
| `crates/ph2d-tool-vector/` | `params.rs` (a lei da forma do slider) + `params_text.rs` (novo, split de LOC) | ✅ do módulo |
| `shells/desktop/src/` | `offset_live.rs`, `vec_expand.rs`, `vec_convert.rs`, `fx_undo_smoke.rs`, `build_smoke_expand.rs` (novos) + `input_dispatch.rs` / `render_loop/mod.rs` / `vec_gizmo_view.rs` / `project.rs` / `undo.rs` (edições pontuais) | ⚠️ **ver §3** |
| `.typos.toml` | **+2 palavras pt-BR** (`itens`, `instrumentos`) no fim da lista | ⚠️ **ver §3** |

---

## 3. O que pode COLIDIR com outra linha (grepe isto)

### 3.1 ⚠️ `ComponentRegistry` — o contador **SOMA entre linhas**

```rust
// crates/ph2d-ecs/src/scene/registry.rs
assert_eq!(reg.len(), 33);   // esta linha diz 33 (era 32)
```

**O valor se CONTA, não se escolhe.** Se outra linha também registrou um componente, o número certo
é a **contagem da árvore combinada** — nunca "um dos lados". O próprio comentário no arquivo já
narra a vez anterior em que isto mordeu (duas linhas diziam 27 por motivos diferentes; a árvore
tinha 28). [[feedback_numbers_that_sum_across_lines_count_dont_pick]]

O registro em si é **append-only** (uma linha `reg.register::<crate::VecOffset>(…)` no fim do bloco)
— o merge textual é trivial; só o `assert_eq!` exige aritmética.

### 3.2 ⚠️ `ph2d_vec_render::dispatch` mudou de assinatura (5 → 6 args)

```rust
pub fn dispatch(scene, view, xforms, live: &LiveGeometry, camera, target)
//                                   ^^^^ NOVO (4º parâmetro)
```

**Um chamador**, na shell (`render_loop/mod.rs`). Se outra linha tocou aquele sítio, o conflito é
textual: passe `self.offset_live.live()`. Mesma forma para as quatro funções de pick que ganharam
o mesmo parâmetro:

```
vec_gizmo_view::{contains_world, contains_path, pick_all_at_world, pick_in_world_rect}
envelope_gesture::press
```

### 3.3 ⚠️ Símbolos REMOVIDOS (não os ressuscite num merge)

| Removido | Onde foi parar |
|---|---|
| `ph2d_vec_render::draw_corner_handles` + `src/corner.rs` | a alça de quina virou **ferramenta** (`ph2d-vec-edit::corner_tool`) |
| `ph2d_vec_render::markers` (`src/markers.rs` + tests) | virou `ph2d_vec_scene::stroke_plan` — a receita *"o que este traço desenha?"* passou a ter **dois** consumidores (quem pinta e quem **assa**, o Outline Stroke), e duas cópias divergiriam em silêncio. A geometria pura das pontas nunca saiu de `ph2d_vec_scene::marker` |
| `ph2d-vec-edit::corner_handle_tests.rs` | virou `corner_tool_tests.rs` |
| `ProjectUndo::forget_last` | existiu por ~1 dia dentro desta linha e saiu com o modelo que a motivava |
| `shells/desktop/src/vec_expand_retune_tests.rs` | idem — o modelo destrutivo do Offset morreu |
| `shells/desktop/tests/a_retune_replaces_its_own_undo_step.rs` | idem |
| `shells/desktop/tests/the_live_offset_preview_is_a_gesture_to_the_settle.rs` | idem |

Se um merge trouxer qualquer um destes de volta, **é resíduo** — o produto não os tem.

### 3.4 Ids novos (23) — literais para grep de mesmo-símbolo

Todos por `hash_node_id("…")`, prefixo `vector.` — colisão com outra linha é improvável mas o
`node_id_collisions` a pega. Os nomes:

```
vector.expand.{offset, offset_num, offset_path, outline_stroke, power_stroke,
               join.{miter,round,bevel}, side.{outer,inner,both},
               w_start, w_start_num, w_mid, w_mid_num, w_end, w_end_num, w_pos, w_pos_num}
vector.section.expand · vector.fx.apply · vector.mode.fillet · vector.mode.chamfer
```

### 3.5 `.typos.toml` — **só ADICIONE**

Duas palavras pt-BR no fim de `[default.extend-words]` (`itens`, `instrumentos`). A lista é
compartilhada e funde contra a `main` de HOJE: **acrescente, nunca remova o que outra linha pôs**.
⚠️ Chave duplicada mata o gate no parse do TOML — se as duas linhas adicionarem a mesma palavra,
deixe **uma**. [[feedback_duplicate_allowlist_key_kills_the_gate_at_parse]]

### 3.6 Schemas — **NENHUM bump nesta linha**

`PROJECT_SCHEMA` = **26** · `VEC_SCENE_SCHEMA_VERSION` = **13** · `DOC_VERSION` = **8** — os três
**exatamente como na `main`**. Um componente ECS novo é chaveado por `stable_type_id` (hash do
NOME), então registrá-lo **não move layout nenhum** e bumpar seria jogar fora todo projeto salvo
para melhorar uma mensagem de erro. Se outra linha bumpar, **conte** — não escolha um lado.

---

## 4. Contratos congelados encostados

**NENHUM.** Confirmado por grep:

- **Nodes** (ADR-0039, `NodeOp`/`OpResolver`/`NodeManifest`) — intocados.
- **Tools** (ADR-0040/0041, `Tool=12`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent=4`) — intocados;
  os dois modos novos de quina são **modos da tool vetorial**, não variants de contrato.
- **Vector data-model** (ADR-0056..0068, `ph2d-vector-doc`/`-traits`) — **não tocados**; o gate
  `architecture_vector_contract_surface` só varre essas duas crates, e o motor novo (`ph2d-vec-*`)
  tem contrato próprio, **ainda não congelado**.

---

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

Rodei, na árvore da linha: `cargo test` (bins + integração), `clippy --all-targets`,
`cargo check --workspace --all-targets`, `cargo fmt`, `typos`. **Não** rodei:

- **`cargo machete`** — a linha **não acrescentou dependência nenhuma** (nenhum `Cargo.toml` de
  crate mudou), então o risco é de outra linha, não desta.
- **`cargo deny` / `cargo audit`** — sem deps novas, mesma leitura; mas o **RUSTSEC pode ter
  publicado** desde o fork ([[project_integration_prefork_lines_ship_drift]]).
- **fmt/typos PRÉ-fork** — arquivos que esta linha não tocou podem estar sujos na `main` combinada.
- **clippy latente cross-crate** — a árvore combinada tem lints que nenhuma linha sozinha vê.

Orce **2–4 iterações** de ship ([[project_integrator_ship_catches_latents_budget_iterations]]).

---

## 6. Ordem, dependências e o que smoke-testar

### 6.1 Ordem

**Os 45 commits são uma sequência única e devem entrar em ordem** — há DUAS reversões de código DENTRO
da linha (`43a6f4d0` tira um memo de preview; `8e6b1fff` reverte uma regressão de overlay minha)
e um `revert` fora de ordem deixaria o produto num estado que nunca existiu. `--ff-only` resolve.

**Dependência externa: nenhuma.** A linha não espera nada de outra.

### 6.2 O que JÁ foi smokado e APROVADO pelo Enio

| Smoke | O quê | Veredito |
|---|---|---|
| `PH2D_BUILD_SMOKE=17` | Outline Stroke · Power Stroke · **Offset vivo** (o modelo novo: preview em tempo real, `Apply Offset` materializa) | ✅ aprovado |
| idem, passo do clique | **o pick segue o desenho** (clicar na banda crescida seleciona; o vão da encolhida não pega) | ✅ aprovado |
| `PH2D_BUILD_SMOKE=15` / `=16` | Fillet / Chamfer como ferramentas | ✅ aprovado |
| `PH2D_BUILD_SMOKE=20` | **o undo da pilha de efeitos**, auto-dirigido e auto-verificável | ✅ 15/15, veredito verde |

⚠️ **Todos exigem `--release`.** O motor é ~16× mais lento em debug, e "trava alguns segundos" já
foi lido como bug do produto quando era o build.

### 6.3 O que o integrador deve RE-smokar na árvore combinada

Um só, e é o que cobre a superfície inteira desta linha:

```
cd <árvore combinada> && PH2D_BUILD_SMOKE=17 cargo run --release -p ph2d-host-desktop
```

E o probe do undo, que **se verifica sozinho** (não precisa de olho — leia a última linha):

```
PH2D_BUILD_SMOKE=20 timeout 60 cargo run --release -p ph2d-host-desktop 2>&1 | grep VEREDITO
```

Esperado: `VEREDITO: o undo da pilha de efeitos FUNCIONA nos 7 gestos (0 de 15 conferências falharam)`.

### 6.4 O que NÃO foi smokado (nomeado, não contrabandeado)

- **Multi-seleção com offsets DIFERENTES** — arrastar o slider escreve o MESMO `d` em todas
  (o slider é um número só); materializar honra o de cada uma. Coerente, sem UI que o diga.
- **Offset → efeito da pilha nessa ORDEM** (offset e *depois* ondular) — só pelo caminho
  destrutivo (Apply Offset, depois o efeito). Se virar pedido, a resposta é o `LiveGeometry`
  alimentar um 2º estágio, **não** mexer no contrato da pilha.
- **Ghost da curva original** por baixo do offset — não existe; a visibilidade `Show/Hide` da
  árvore continua a ser da FONTE (correto).

---

## 7. Duas coisas que o integrador deve saber para não "consertar"

1. **A caixa do gizmo NÃO segue o offset, de propósito.** O `d` é distância de MUNDO, então escalar
   a forma não escala a banda; uma caixa que a incluísse faria o gizmo derivar do dedo durante o
   arrasto — a armadilha das 5 tentativas revertidas do ADR-0128. É também o default do Illustrator
   ("Use Preview Bounds" desligado). O **modo Node** também lê a FONTE (as âncoras são as autoradas,
   ADR-0121). A divisão *quem lê a derivada, quem lê a fonte* está escrita no cabeçalho de
   `shells/desktop/src/offset_live.rs`.
2. **O `undo_tests.rs::putting_or_removing_any_effect_round_trips_through_undo` FICA**, mesmo agora
   que o probe do smoke 20 cobre o mesmo terreno: ele é rápido, roda em CI, e prova o ida-e-volta
   do **estado** por tipo de efeito. O que ele **não** pode provar — e por isso o probe existe — é
   *"o meu clique virou um passo?"*.

---

## Resumo para o Enio

> Linha `Vector` pronta (HEAD `35c0d32d`, 45 commits, 0 atrás da `main`, árvore limpa, todos os
> smokes aprovados). Foundational tocado: `ph2d-ecs` (componente `VecOffset` + registro, **contador
> 32→33 — SOMA entre linhas**), `ph2d-editor-core` (23 ids + 1 método de store), `ph2d-i18n` (3
> chaves), `ph2d-vec-{scene,boolean,render,edit}` (aditivo, salvo `dispatch` que ganhou 1 parâmetro
> e 2 módulos removidos), `.typos.toml` (+2 palavras, só adicionar). **Zero contrato congelado.
> Zero bump de schema. Zero dependência nova.** Aguardo ordem de integração.
