# Handoff de integração **MESTRE** — `line/Vector`, a jornada da UI/UX

> **Para o agente integrador.** Esta linha fechou **84 commits** (`864229887..c3a1cef43`),
> **315 arquivos**, **+30.963 / −1.093**, em **onze waves** que já têm handoff próprio.
> Este documento **não os repete**: ele traz a **tabela de colisão**, os **pontos de merge
> sensíveis** e a **ordem de leitura**. Cada wave é lida no handoff dela, e só se o merge a tocar.
>
> **Base:** `main` de 2026-08-04 (`a4018d203`). **Tip:** `c3a1cef43`.
> **A linha está FECHADA e PARADA.** Não integra, não pusha, não faz ship.

---

## 1 — O que a linha entrega, em três frentes

| frente | waves | o que muda para o artista |
|---|---|---|
| **A — os TOKENS deixam de ser uma tabela de compilação** | W4b.1 · W4b.2 · W4c.1 → W4c.5 | um token de cor **segue outro** (alias, com ciclo detectado na PORTA) · o readout de **contraste WCAG** onde a escolha é feita · a **escala** (`spacing`/`radius`/`stroke`) vira autorável · um token numérico pode valer uma **fórmula** · a tabela **sai e entra em DTCG** (`.tokens.json`, o formato que Figma / Tokens Studio / Style Dictionary falam) |
| **B — a UI ganha ESTADOS e o Smart Animate** | W7 · W7r · W7c · W7m | uma cena de UI tem **poses nomeadas** (Default / Hover / Pressed…), o **modo de preview** dirige-as com o rato, a **curva** da transição é escolhida por um seletor, e a **MOLA** é uma opção ao lado das curvas |
| **C — a árvore autorada vira um PAINEL VIVO** | W6.2 · W8b.1 → W8b.4 · W2a · hierarquia | a forma **veste** um controle do catálogo · o app **escreve o código** do painel · o gerado é um painel que pinta, rola, responde ao ponteiro e **mexe na arte** · o texto **reflui** · o menu da Hierarquia não foge do cursor |

Mais **dois documentos de medição** que não movem código e são a razão de eles existirem:

- [`LEVANTAMENTO_vector_para_a_UI_do_app_2026-08-08.md`](LEVANTAMENTO_vector_para_a_UI_do_app_2026-08-08.md) — **438 construções de widget** contadas em 23 painéis; cobertura **67,1%**, e o buraco tem **duas naturezas** (18,3% é OMISSÃO — fiação; 14,6% é ESTRUTURAL — pede filhos autorados).
- [`docs/Runtime/`](../../Runtime/) — o plano do runtime, a medição do formato de arquivo resgatada da `line/runtime` (descartada), e o handoff da R0. **São docs, e viajam com esta linha** porque foi aqui que foram escritos.

---

## 2 — ⚠️ A TABELA DE COLISÃO

| item | main (04/08) | esta linha | ação do integrador |
|---|---:|---:|---|
| **`PROJECT_SCHEMA`** | **55** | **62** | ⚠️ **CONTE, não copie** — ver §2.1 |
| `VEC_SCENE_SCHEMA_VERSION` | 14 | **14** | intacto |
| `FLIP_SCHEMA_VERSION` | 13 | **13** | intacto |
| `DOC_VERSION` (timeline) | 17 | **17** | intacto |
| registro `ph2d-ecs` | 52 | **54** | +2 (`VecWidgetBind` · `VecWidgetValue`) |
| espelhos `ph2d-render` / `ph2d-script` | 53 / 53 | **55 / 55** | ⚠️ o contador é **TRÊS** — ver §2.2 |
| **contrato congelado** | 4/4 | **4/4** | ⚠️ **intacto**, conferido por `git diff` vazio em `ph2d-nodegraph` e `ph2d-core/src/tool.rs` |
| **ADRs novos** | — | **NENHUM** | ⇒ esta linha fica **FORA** de toda disputa de número da janela |
| **crates novas** | — | **5** | todas folhas ou quase-folhas; ver §2.3 |
| **deps EXTERNAS novas** | — | **NENHUMA** | o `Cargo.lock` só ganha os 5 pacotes de path; ver §2.3 |
| cenas de smoke | …60 | **61 · 62 · 63 · 64 · 65** | próxima livre: **66** |
| `cargo fmt --all -- --check` | — | **limpo** | conferido no tip |

### 2.1 ⚠️ O `PROJECT_SCHEMA` é PROVISÓRIO — a escada tem SETE degraus

A linha escreve **62** porque o `main` do dia da abertura dizia 55 e ela subiu sete vezes:

| degrau | wave | o que entrou |
|---:|---|---|
| **v56** | W7 | `ProjectState.ui_states` — a tabela de poses de UI |
| **v57** | W4b.1 | um token de cor passa a poder valer um **alias** |
| **v58** | W4c.1 | a camada **numérica** dos tokens (`spacing`/`radius`/`stroke`) |
| **v59** | W4c.3 | um token numérico passa a poder valer uma **fórmula** |
| **v60** | W4c.4 | `ph2d_ecs::BoundProp` ganha os tokens de **escala** |
| **v61** | W2a | `ph2d_ecs::VecTextParams.wrap_width` |
| **v62** | W7m | `ph2d_ui_state::HostStates.spring` |

⚠️ **Se qualquer outra linha bumpar nesta janela, 62 está ERRADO.** O valor se **CONTA** a
partir do `main` do dia da integração — some sete ao que ele disser, e reescreva os sete
comentários da escada em `shells/desktop/src/project.rs` **na mesma ordem**. Cada um deles já
traz `⚠️ **PROVISÓRIO**` escrito por mim, exactamente para este momento.
[[feedback_numbers_that_sum_across_lines_count_dont_pick]]

⚠️ **E há um modo de falha MUDO documentado nesta casa** (`line/FLIP` × `line/physics`,
2026-08-01): se a outra linha escrever **o mesmo literal**, o `project.rs` **não conflita** — o
git não tem opinião sobre o que o número significa, e um dos dois bumps **evapora com a suíte
verde**. Quem denuncia é o conflito no `project_schema_tests.rs` ao lado. **Confira o
`project_schema_tests.rs` mesmo que o `project.rs` funda limpo.**

### 2.2 ⚠️ O contador de componentes é TRÊS, e cada um roda numa suíte diferente

A MESMA contagem é afirmada em `ph2d-ecs` (52→**54**), `ph2d-render` (53→**55**) e
`ph2d-script` (53→**55**) — e cada uma só corre com `cargo test -p` da própria crate. Já ficou
**vermelho-latente** na `line/Vector` três vezes. Os dois espelhos contam +1 (o `Sprite` /
o `LuauScript`), e é por isso que dizem 55 e não 54.

Se outra linha registrar componentes nesta janela, **os três números sobem juntos**, e o
comentário acima do `assert_eq!` do `ph2d-ecs` já diz por escrito o que fazer:
*"escolher 'um dos lados' aqui é o erro que deixa o workspace vermelho com dois merges verdes."*

### 2.3 As cinco crates novas — e por que nenhuma delas é uma dep externa

| crate | LOC | deps externas | a contenção que a justifica |
|---|---:|---|---|
| **`ph2d-ui-state`** | 2 584 | `serde` | **folha**: sem relógio (o relógio é o `Playhead`), sem ECS, sem UI. E **sem motor próprio** de forma ou de cor — interpola pelo `ph2d-vec-blend`, senão divergiria do Blend/Morph que o artista já usa, e a divergência só apareceria numa screenshot |
| **`ph2d-tokens-dtcg`** | 1 110 | **`serde_json`** | a `ph2d-tokens` declara no próprio `Cargo.toml` *"design-data puro — zero runtime deps"*, e ela é a folha de que **44 widgets** dependem. Um parser de JSON no caminho de compilação de todos eles, para uma feature que corre **duas vezes na vida de um projeto** |
| **`ph2d-panel-authored`** | 1 024 | — | ⚠️ **sem `ph2d-i18n`, e a ausência é a decisão**: toda string que este painel mostra é o `Name` que o **artista** digitou. HR-15 governa strings que o **programa** escreve |
| **`ph2d-token-math`** | 307 | — | fora da `ph2d-tokens` porque `ph2d-expr-parse` arrasta o `ph2d-nodegraph`; é **injectada por fn-pointers** |
| **`ph2d-ui-codegen`** | 243 | **nenhuma, nem interna** | ⚠️ **não depende do `ph2d-editor-core`** — sem alcance ao catálogo, ela **não CONSEGUE** ter opinião sobre o que um `Slider` é. A contenção é **estrutural** (arch-gate sobre o `Cargo.toml`), não disciplinar — o molde do `ph2d-paint-gpu` contra o `ph2d-painter-brush` |

⚠️ **`serde_json` já está na árvore** (`ph2d-mcp`, `ph2d-asset`, `ph2d-tokens`) ⇒ o
`Cargo.lock` ganha **arestas**, e **zero pacote externo novo**. Conferido por `git diff` no
lock: os únicos `+name` são os cinco `ph2d-*` de path.

---

## 3 — Os pontos de merge sensíveis

### 3.1 ⚠️ `shells/desktop/src/project.rs` — a §2.1, e nada mais

É o único arquivo desta linha onde um merge limpo pode estar semanticamente errado. Ver §2.1.

### 3.2 ⚠️ As DUAS listas `default` do painel autorado

`ph2d-panel-authored` está registada em **cinco** sítios, e dois deles são listas `default` que
outra linha pode ter editado:

```
crates/ph2d-panel-registry-init/Cargo.toml:17   dep opcional
crates/ph2d-panel-registry-init/Cargo.toml:55   ⚠️ lista `default`
crates/ph2d-panel-registry-init/Cargo.toml:64   a feature
crates/ph2d-panel-registry-init/src/lib.rs:45   o `reg.push`
shells/desktop/Cargo.toml:397                   ⚠️ lista `default` DO SHELL
```

⚠️ **A lista do shell é a que importa**, e o precedente é o W2b da física: o shell põe
`default-features = false` na `registry-init` e **re-enumera** os painéis na lista dele — ligar
a feature só na crate de registry **não alcança ninguém**, e tudo a jusante funciona sobre um
painel que não existe, **sem erro e sem warning**. O gate
`every_panel_the_shell_drives_is_in_its_registry` mora onde o shell é compilado e pega isto.

**Uma lista compartilhada funde contra o `main` de HOJE: só ADICIONE.**
[[feedback_a_shared_list_is_merged_against_todays_main]]

### 3.3 ⚠️ `crates/ph2d-editor-core/src/widget/skin.rs` — +207 linhas

O catálogo de peles. É o arquivo foundational mais mexido da linha, e é onde a fatia seguinte
(os quatro widgets por OMISSÃO do levantamento) vai voltar. Se outra linha acrescentou um
`WidgetKind`, **os códigos são explícitos e nunca a ordem do enum** — a W6.2 estabeleceu isso
de propósito, e `from_code` devolve `None` para o desconhecido. Um add/add aqui resolve-se
mantendo os dois lados.

### 3.4 ⚠️ Os ids nasceram em arquivos IRMÃOS, não no `ids.rs`

`git diff main...HEAD -- crates/ph2d-editor/src/ids.rs` é **vazio**. Os ids desta linha moram em
seis arquivos **novos** sob `crates/ph2d-editor-core/src/ids/chrome/`
(`authored.rs` · `tokens.rs` · `vector_states.rs` · `vector_text.rs` · `vector_tokens.rs` ·
`vector_frame.rs`) — o padrão de isolamento que o ADR-0107 pede. **Não há colisão a resolver**;
o `node_id_collisions` confere-os.

---

## 4 — Gates

| gate | onde | estado |
|---|---|---|
| `cargo fmt --all -- --check` | workspace | ✅ limpo no tip |
| **`cargo nextest run --workspace`** | 13 166 testes | ✅ **13 166 / 13 166**, ver §4.1 |
| contrato congelado | `ph2d-nodegraph` · `ph2d-core/src/tool.rs` | ✅ **diff vazio** |
| `no_two_smoke_scenes_claim_the_same_level` | `shells/desktop/tests/` | ✅ (61–65 são novos e únicos) |
| `every_panel_the_shell_drives_is_in_its_registry` | `shells/desktop/tests/` | ✅ |
| `the_leaf_stays_dep_free` | `ph2d-tokens/tests/` | **novo** — é ele que impede a `ph2d-tokens` de ganhar `serde_json` |
| `no_effect_inside_debug_assert` | `ph2d-editor-core/tests/` | **novo** — ver §5.1 |

### 4.1 A bateria completa, rodada no tip — e o que ela achou

Primeira corrida: **13 163 passam, 3 falham**. Duas são **flakes de RAZÃO conhecidas** e uma era
**real e minha**.

| falha | dono | veredito |
|---|---|---|
| `ph2d-editor-core::no_magic_numeric` | **esta linha** | ⚠️ **REAL — corrigida**, ver abaixo |
| `ph2d-timeline::nesting_clock::the_cost_of_depth_is_linear_not_explosive` | timeline | flake **PRÉ-EXISTENTE e documentada** no CLAUDE.md (*"gate de RAZÃO sensível a carga — passa isolado"*); re-rodado sozinho: **6/6** |
| `ph2d-mesh::measure_normals::measure_normals_parallel_speedup` | `line/sculpt3d` | **mesma classe** (razão sob 13 k testes em paralelo); re-rodado sozinho: **3/3** |

⚠️ **A que era minha é exatamente o modo de falha que o §4 descreve dois parágrafos acima**, e vale
mais do que o conserto: **cinco literais de física** (`60.0` · `0.1` · `12.0` — as pontas da régua
da MOLA) em `ph2d-panel-vector`, apanhados por um gate que mora na **`ph2d-editor-core`**. Um
fechamento por `cargo test -p ph2d-panel-vector` **nunca o alcança** — a mesma causa estrutural que
a `line/motion-value`, a `line/physics` e esta linha já documentaram, **e eu caí nela na wave em
que a escrevi**.

**O conserto é o escape que o próprio gate oferece** (`// LITERAL-PX-OK: <razão>`), porque estes
números **não são valores de design**: são as pontas da régua de uma grandeza FÍSICA (rigidez em
unidades de mola, amortecimento adimensional). *Não existe token de escala para "quão dura é uma
mola", e inventar um poria uma constante de física dentro do design system.*

⚠️ **O marcador tem de ficar NA linha**, e o `rustfmt` já reflowou uma correção destas para fora
antes (a cicatriz da `line/motion-value`, 02/08) — conferido depois do `cargo fmt`: os cinco
continuam onde deviam. A "mutação" é trivial e foi observada: tirar o marcador devolve o gate ao
vermelho com os cinco sítios nomeados.

⚠️ **Rode a suíte do shell em DEBUG e em RELEASE.** Esta linha tem o precedente escrito: o
`b7cb03d4e` corrige um botão que **não fazia nada em release** porque a escrita morava dentro de
um `debug_assert!` — e a suíte de debug estava verde sobre ele.

⚠️ **Os gates de `shells/desktop/tests/` e `ph2d-editor-core/tests/` só correm na varredura
impactada.** Um fechamento por `cargo test -p` por crate **não os alcança** — a causa
estrutural que a `line/motion-value`, a `line/physics` e esta linha já documentaram. Rode
`--workspace` na árvore combinada.

---

## 5 — Duas correções que esta linha fez em código ALHEIO

### 5.1 ⚠️ O "Reset This Mode" dos tokens não fazia NADA em release (`b7cb03d4e`)

A escrita morava **dentro de um `debug_assert!`**, e o `debug_assert!` **apaga o argumento
inteiro em release** — não avalia, não avisa, não deixa vestígio. O botão pintava, respondia ao
clique e não mudava um byte no build que o artista usa.

O gate novo `no_effect_inside_debug_assert` varre a árvore por esta forma. **É de interesse do
repo inteiro, não desta linha.**

### 5.2 ⚠️ A row `Duplicate` da Hierarchy duplicava a ENTIDADE, não a FORMA (`3beeaadfb`)

Correção pontual, num arquivo que a linha já tocava.

---

## 6 — Os smokes

Todos com `--release`. **Cinco cenas novas**, e as três primeiras já foram aprovadas pelo Enio:

| cena | o que julga | veredito |
|---|---|---|
| **`PH2D_BUILD_SMOKE=61`** | os ESTADOS de UI + o Smart Animate (W7) | ✅ aprovado |
| **`PH2D_BUILD_SMOKE=62`** | a árvore autorada vira **painel vivo** (W6.3 / W8b) | ✅ aprovado |
| **`PH2D_BUILD_SMOKE=63`** | o texto **reflui** (W2a) | ✅ aprovado |
| **`PH2D_BUILD_SMOKE=64`** | a hierarquia aninhada | ✅ aprovado |
| **`PH2D_BUILD_SMOKE=65`** | ⭐ **a MOLA** — quatro pistas, mesma viagem, mesmas duas poses, **só o motor difere** | ✅ aprovado |

⚠️ **A cena `=65` imprime o pico de cada pista e escreve `!! PARE` se algum verdicto falhar.**
Ela é a única do lote que se auto-verifica; se a linha de veredito não aparecer, **pare**.

⚠️ **As cenas herdadas (`=20`..`=60`) têm de continuar iguais.** Esta linha mexeu em
`skin.rs`, no `slider`, no `checkbox`, no `dropdown` e no `color_swatch` — as peles alcançam
todo painel do app.

---

## 7 — ⚠️ Aberto — e uma decisão que é do Enio, não de engenharia

### 7.1 A MUDANÇA DE COMPORTAMENTO da W7m — o clamp era GLOBAL e virou POR CANAL

**A pergunta que a lei responde:** *passar do alvo significa alguma coisa neste canal?*
Posição e rotação recebem `t` cru; escala (negativa **espelha**), opacidade (alfa negativo),
tinta, largura e geometria recebem `tc` clampado.

⚠️ **A consequência:** `Back Out` (pico **1,100**) e `Elastic Out` (pico **1,3731**) passam
finalmente a **ultrapassar** na posição — que é o que aquelas curvas **são**. Isso muda o
comportamento de um easing **já shipado**.

**Reverter custa à mola o carregamento de momento** (uma mola que inverte a meio do voo tem de
poder ir `t < 0`), e a mola foi aprovada no smoke. **A decisão é do Enio**, e está no §4 do
[handoff da mola](HANDOFF_INTEGRACAO_line_Vector_mola_2026-08-08.md).

### 7.2 A fila de intents do painel autorado **cresce sem teto**

`AuthoredIntent` é empurrado a cada gesto e **ninguém fora dos testes da própria crate o
drena** — um arrasto de slider empurra um intent com **duas `String` por quadro** enquanto o
painel está aberto.

⚠️ **A ausência era uma cerca de Chesterton correta** (o `state.rs` a declarava: *"quem escuta
ainda não existe"*), **mas ela envelheceu**: a W8b.3 ligou a row à ARTE por outra rota
(`WidgetStore` → `VecViewState`). O doc do `drain_intents` foi corrigido em `c3a1cef43` para
dizer a verdade e nomear o preço. **As duas curas — ligar o ouvinte, ou parar de empurrar — são
da wave que o fizer.** Não é bloqueio de integração.

### 7.3 O que o levantamento nomeia como próximo, com o preço medido

| ordem | wave | ganho | custo |
|---|---|---|---|
| 1 | os **quatro por OMISSÃO** (`ColorSwatch` · `NumberInput` · `IconButton` · `LevelMeter`) | cobertura **67,1% → 85,4%** | fiação; o molde da W6.2 já existe |
| 2 | colapso de seção | **9 dos 23** painéis reais | comportamento, não tipo novo |
| 3 | a família da **LISTA** (filhos autorados) | 85,4% → **~100%** | **desenho novo** |
| 4 | multi-painel | N painéis por build | a decisão já está tomada (§3.1 do levantamento) |

⚠️ **A ordem é por RAZÃO ganho/custo, não por tamanho** — o (1) é o único item onde os **dois
lados** já estão medidos.

### 7.4 Os docs do Runtime viajam com esta linha

`docs/Runtime/00_plano_runtime.md` · `01_o_formato_medido.md` ·
`HANDOFF_runtime_R0_2026-08-08.md` são **docs**, não código. Estão aqui porque foi aqui que
foram escritos, e a `line/runtime` nova (a partir do `main`) vai lê-los. **Se esta linha não
integrar, aquele agente lê-os pelo caminho absoluto desta worktree** — é o que o prompt de
abertura dele manda fazer, e é por isso que ele proíbe copiá-los (uma cópia viraria add/add).

---

## 8 — A ordem de leitura dos handoffs por-wave

**Leia só o da wave que o merge tocar.** Por ordem cronológica:

| # | handoff | wave |
|---:|---|---|
| 1 | [`../../HANDOFF_INTEGRACAO_line_Vector_ui_states_2026-08-05.md`](../../HANDOFF_INTEGRACAO_line_Vector_ui_states_2026-08-05.md) | W7 — os estados de UI |
| 2 | [`HANDOFF_line_Vector_tokens_2026-08-06.md`](HANDOFF_line_Vector_tokens_2026-08-06.md) | a reforma de tokens: o estado, a medição, as 5 waves |
| 3 | [`HANDOFF_INTEGRACAO_line_Vector_W4c1_2026-08-06.md`](HANDOFF_INTEGRACAO_line_Vector_W4c1_2026-08-06.md) | a camada numérica |
| 4 | [`HANDOFF_INTEGRACAO_line_Vector_W4c2_2026-08-06.md`](HANDOFF_INTEGRACAO_line_Vector_W4c2_2026-08-06.md) | a escala viva |
| 5 | [`HANDOFF_INTEGRACAO_line_Vector_W4c3_2026-08-06.md`](HANDOFF_INTEGRACAO_line_Vector_W4c3_2026-08-06.md) | a **math** |
| 6 | [`HANDOFF_INTEGRACAO_line_Vector_W4c4_2026-08-06.md`](HANDOFF_INTEGRACAO_line_Vector_W4c4_2026-08-06.md) | os tokens de escala no documento |
| 7 | [`HANDOFF_INTEGRACAO_line_Vector_W4c5_2026-08-07.md`](HANDOFF_INTEGRACAO_line_Vector_W4c5_2026-08-07.md) | **DTCG** — a tabela sai e entra |
| 8 | [`HANDOFF_INTEGRACAO_line_Vector_W7r_2026-08-07.md`](HANDOFF_INTEGRACAO_line_Vector_W7r_2026-08-07.md) | o modo de **preview** |
| 9 | [`HANDOFF_INTEGRACAO_line_Vector_W7c_2026-08-08.md`](HANDOFF_INTEGRACAO_line_Vector_W7c_2026-08-08.md) | a **curva** da transição |
| 10 | [`HANDOFF_INTEGRACAO_line_Vector_text_wrap_2026-08-08.md`](HANDOFF_INTEGRACAO_line_Vector_text_wrap_2026-08-08.md) | W2a — o texto reflui |
| 11 | [`HANDOFF_INTEGRACAO_line_Vector_hierarquia_2026-08-08.md`](HANDOFF_INTEGRACAO_line_Vector_hierarquia_2026-08-08.md) | o menu que não foge do cursor |
| 12 | [`HANDOFF_INTEGRACAO_line_Vector_mola_2026-08-08.md`](HANDOFF_INTEGRACAO_line_Vector_mola_2026-08-08.md) | ⭐ W7m — **a MOLA** e a lei por canal (§7.1) |

O plano-mãe é [`PLANO_UI_UX_padrao_figma.md`](PLANO_UI_UX_padrao_figma.md) — a tabela de waves
dele (§ perto do fim) diz o que cada uma entregou e o que ficou.

---

## 9 — Resumo para quem tem trinta segundos

1. **Nenhum ADR, nenhuma dep externa, contrato congelado intacto** ⇒ esta linha fica **fora** de
   toda disputa de número da janela **exceto uma**.
2. **Essa uma é o `PROJECT_SCHEMA`: 55 → 62, SETE degraus, e o valor é PROVISÓRIO.** Conte
   contra o `main` do dia. E confira o `project_schema_tests.rs` **mesmo que o `project.rs`
   funda limpo** — o modo de falha é mudo.
3. **O contador de componentes é TRÊS** (52→54 no `ph2d-ecs`, 53→55 nos dois espelhos).
4. **Rode `--workspace`, em debug E em release.** No tip ela dá **13 166 / 13 166** (§4.1); se as
   duas flakes de razão (`nesting_clock` · `measure_normals`) aparecerem sob carga, **re-rode-as
   isoladas antes de suspeitar do merge**.
5. **Cinco smokes novos (61–65), todos aprovados; as cenas herdadas têm de continuar iguais.**
