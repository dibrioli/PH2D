# HANDOFF DE INTEGRAÇÃO — `line/Vector` · a máquina de estados do Morph (2026-08-26)

> ⚠️ **Entregável de fecho de linha** (DIRETRIZ §1.5.9). A linha **não integra e não pusha**
> ([`CLAUDE.md §0.7`](../../../CLAUDE.md)) — ela fecha, entrega isto e PARA.
>
> **Smoke do Enio: OK** (`PH2D_BUILD_SMOKE=75`, 2026-08-26, depois de seis reports encadeados).

---

## 1 — Identidade

| | |
|---|---|
| branch | `line/Vector` |
| HEAD | `9a6c901ab` |
| merge-base com `main` | `0f5ce8040` |
| commits da linha | **29** |
| ficheiros tocados | **81** |
| ⚠️ `main` avançou desde a base | **84 commits** (`git rev-list --count HEAD..main`) |

⚠️ **Os 84 commits do `main` são o dado que decide o trabalho do integrador.** A tabela do §3 mede
esta linha contra o `main` **do dia do fecho**; ela é **referência, nunca evidência**
(DIRETRIZ §1.5.9 item 3). **Re-rode `collision-surface.sh` nesta worktree imediatamente antes de
fundir** — a divergência entre as duas leituras é ela própria um achado.

---

## 2 — O que a linha entrega, em uma frase por bloco

**A máquina de estados do Morph** (item **2** da fila do [doc 29](../29_fila_morph_state_machine_e_texture_pattern.md)),
do desenho ao smoke aprovado:

- **um botão faz o conjunto** — o artista escolhe N formas e clica *Make Morph States*; nasce um
  objecto na Hierarquia com as formas por baixo, alinhadas no mesmo ponto, e **só uma aparece**;
- **as setas são virtuais** (decisão do Enio, 25/08) — o grafo é completo por construção e
  **ninguém o desenha**;
- **uma tecla por FORMA**, não por transição — a lista do painel é `n`, não `n(n-1)`;
- **um MODO de pré-visualização** que **toma o teclado** (senão as setas morfam a forma *e* movem o
  desenho);
- **▶ Play · ⊘ Desconectar · botão que abre a lista de eventos · Undo Morph States**;
- **arrastar na Hierarquia É entrar** — a lista de estados **são os filhos**;
- e o conjunto **anima dentro do sistema de States** que já existia.

**Cena de smoke: `PH2D_BUILD_SMOKE=75`** (19 passos impressos no terminal).
**Diagnóstico: `PH2D_MORPH_LOG=1`** — três eventos (clique, reparo, saída-cedo), **nenhum periódico**.

---

## 3 — Superfície de colisão (saída do `collision-surface.sh`, colada)

```
SUPERFÍCIE DE COLISÃO — line/Vector contra main
  merge-base 0f5ce8040   ·   29 commit(s)   ·   81 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                         98   (base: 97)
  ⚠   └ tripla do gate               (98, 13, 14)   (base: (97, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              —   (base: —)
  ⚠ ph2d-render (espelho)                  72   (base: 71)
  ⚠ ph2d-script (espelho)                  72   (base: 71)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0167   próximo livre: 0168
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 1 pacote(s) '+name' novo(s):
      "ph2d-morph-machine"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou
    nenhum arquivo da linha passa do teto
```

### 3.1 — Os TRÊS números que somam entre linhas (⛔ contar, nunca copiar)

| número | esta linha | base | onde |
|---|---|---|---|
| **`PROJECT_SCHEMA`** | **98** | 97 | [`project_schema.rs`](../../../shells/desktop/src/project_schema.rs) + a **escada** ao lado + a **tripla** em [`project_schema_tests.rs`](../../../shells/desktop/src/project_schema_tests.rs) — **três sítios** |
| **registro de componentes** (2 espelhos) | **72** | 71 | `ph2d-render/src/registry.rs` · `ph2d-script/src/registry.rs` |
| **contagem de secções do painel Vector** | **37** | 36 | `crates/ph2d-panel-vector/tests/seam.rs` |

⚠️ **Se outra linha também subiu qualquer um deles, o valor certo é a CONTAGEM, não o de nenhum dos
dois lados** — e a colisão passa **muda** quando as duas escrevem o mesmo literal.

### 3.2 — `PROJECT_SCHEMA` 97 → 98 **SEM degrau de migração**

Decisão do Enio (2026-08-26): *"não há projetos salvos. esse app está em fase inicial de
desenvolvimento, podemos fazer o que quisermos."*

⚠️ **Mas o bump FICA, e a razão é o oposto de cerimónia:** o postcard é **posicional e
não-auto-descritivo**, então **sem** ele um ficheiro v97 seria lido **errado, em silêncio**. Com ele,
o `project_load` **recusa em voz alta**. *O bump transforma um mal-entendido silencioso numa recusa
legível.*

⇒ **o integrador não precisa de escrever degrau nenhum**; precisa de **re-contar o número**.

### 3.3 — Símbolos novos que outra linha pode duplicar

| símbolo | valor | risco |
|---|---|---|
| `VECTOR_SECTION_MORPH_STATES` · `VECTOR_MORPH_STATES_MAKE` · `VECTOR_MORPH_PREVIEW` · `VECTOR_MORPH_DISSOLVE` · `VECTOR_MORPH_SHAPES_LABEL` | `hash_node_id("vector.morph.*")` | ⭐ **baixo** — são **derivados da string**, não literais numéricos; duas linhas só colidem se escolherem a mesma string |
| `morph_shape_play_id` · `_disconnect_id` · `_key_button_id` · `_key_option_id` | famílias derivadas do índice | idem |
| `MAX_MORPH_STATES` | **118** | novo, sem par no `main` — ⚠️ **medido** (o teto é o do painel, não um palpite) |
| `MAX_MORPH_ACTIONS` | **24** | idem |
| `ph2d_ecs::VecMorphMachine` | componente **registado** | é ele que move os dois espelhos para 72 |
| **15 chaves i18n** novas em `crates/ph2d-i18n/src/vector.rs` | `panel.vector.morph.*` | colisão só por chave repetida |
| `.typos.toml` | `"^participante(s)?$"` | entrada nova na família pt-PT (a 5.ª) |

⛔ **Contrato congelado (§4/§6): NENHUM tocado.** `NodeOp`/`OpResolver`/`NodeManifest` e
`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` **intocados** (confirmado pelo script).
**Nenhum ADR criado** ⇒ fora de toda disputa de número.

---

## 4 — Foundational / compartilhado tocado, e por quê

| área | ficheiros | aditivo? |
|---|---|---|
| **crate NOVA** | `crates/ph2d-morph-machine/` (lei pura, sem ECS) | ⭐ nova — **folha**, e é isso que a mantém fora da fila do runtime |
| `ph2d-ecs` | `vec_morph_machine.rs` (componente novo) · `lib.rs` · `scene/registry.rs` + testes | **aditivo** (um componente novo no registo) |
| `ph2d-render` · `ph2d-script` | `registry.rs` (os dois espelhos) | **aditivo** — 71 → 72 |
| `ph2d-editor-core` | `ids/chrome/vector_morph.rs` (novo) · `mod.rs` · `vector.rs` · `vector_sections.rs` | **aditivo** |
| `ph2d-ui-state` | `pose.rs` (campo `morph_shape`) · `transition.rs` (`MorphStep`/`morph_steps`) · `machine.rs` · `lib.rs` | ⚠️ **campo novo no `ObjectPose`** ⇒ é ele que move o `PROJECT_SCHEMA` |
| `ph2d-i18n` | `vector.rs` | aditivo |
| `ph2d-component-desc` | `catalog/vector.rs` | aditivo |
| `ph2d-panel-vector` | secção `paint_morph_states.rs` + `seam_morph_states.rs` | ⭐ **secção PRÓPRIA** — ver §6.1 |
| `shells/desktop` | 25 ficheiros (ver §4.1) | mistura |

### 4.1 — Os pontos da shell que o integrador tem de olhar com cuidado

| ficheiro | o que mudou | por que é delicado |
|---|---|---|
| `render_loop/mod.rs` | 3 blocos: a chamada do `tick` (com `drives(...)`), o braço do despacho dos verbos, e a **reconciliação** tarde no quadro | é o ficheiro mais disputado do repo; os três blocos são **aditivos e distantes entre si** |
| `input_dispatch/keyboard.rs` + `keyboard_modal.rs` (novo) | o modo do Morph **toma o teclado** antes de todo atalho | ⚠️ **ordem load-bearing**: logo depois do retrato dos dispositivos, **antes** de qualquer atalho |
| `preview_drive.rs` | dois `Driven` novos (`MorphPair`, `MorphT`) | aditivo ao ledger |
| `vec_ui_state_edit.rs` → **`vec_ui_state_table.rs`** (novo) | as três operações sobre a tabela saíram para irmão | ⚠️ **é um MOVE**: se outra linha tocou o `vec_ui_state_edit.rs`, o conflito aparece no ficheiro que encolheu |
| `render_loop/ui_state_bridge.rs` | `Cooked { bool_morphs, morph_steps }` — a assinatura do `dispatch` passou a 7 args | ⚠️ **assinatura mudada**, não só acrescentada |
| `project_schema.rs` + `project_schema_tests.rs` | 97 → 98 e a tripla | ⚠️ §3.1 |
| `the_highlight_has_one_source.rs` | a **allowlist de som** cresceu (3 → 5) | número contado; cada entrada documenta o que confirma |

---

## 5 — O que só o `ship.sh` apanha (o gate de integração NÃO roda)

| verificação | estado nesta worktree |
|---|---|
| `cargo fmt --all -- --check` | **limpo** |
| `cargo clippy --all-targets` (crates da linha) | **0 warnings** |
| `typos` (projecto inteiro) | **0** — ⚠️ com **uma entrada nova** no `.typos.toml` (§3.3) |
| `cargo machete` | **limpo** (*"didn't find any unused dependencies"*) |
| `cargo deny` / `audit` (RUSTSEC) | ⚠️ **não rodados** — mas **nenhuma dependência EXTERNA foi acrescentada** (o único `+name` do `Cargo.lock` é a crate interna `ph2d-morph-machine`), então o risco é o do *drift* pré-fork, não desta linha |
| suíte da workspace | **18 923 testes, 0 falhas** (`--cargo-profile ci-test --no-fail-fast`) |

⚠️ **Flakes de recurso encontradas ao longo da jornada** — todas **membros NOMEADOS** da família do
`CLAUDE.md` §5.0, todas em crates que `git diff main` **não toca numa linha**, e todas **3/3 ou 5/5
verdes** com a máquina calma: `the_trusted_len_collect_allocates_once` ·
`a_round_live_offset_costs_like_the_other_joins` · `a_wet_move_costs_…` ·
`the_cost_of_a_gated_stroke_…` · `measure_normals_parallel_speedup` ·
`the_shape_match_is_linear_in_the_mesh` · a família `flip_smooth::…::orcamento` ·
`only_the_lower_row_breathes_…`.

⛔ **E uma medição que vale para toda a jornada:** o **conjunto** de reprovadas **mudou entre quatro
corridas do mesmo binário**, e numa delas a máquina estava a **`load 52`** — com `ps` a mostrar
binários de teste de **OUTRA linha** a 200%. *Uma leitura de relógio nesta workstation mede a máquina
inteira, não a árvore de quem a lê.*

---

## 6 — Ordem, dependências e o que ainda NÃO foi smokado

### 6.1 — Ordem dos commits

Os 29 commits são **sequenciais e cumulativos** (W1 → W11j). ⛔ **Não os reordene nem os escolha a
dedo**: a W7+W8 **reverte** o desenho da W4 (a secção deixou de ser um pedaço da *States* e passou a
ser **própria**), e a W11i **muda o dono** da cura da W11h (do gesto para a derivação). Um
cherry-pick parcial reintroduz um desenho que o Enio já recusou.

### 6.2 — O que o Enio smokou (e aprovou)

Os 19 passos da cena **`=75`**, incluindo os seis reports encadeados de 26/08 e as respectivas
curas. **Último veredito: OK.**

### 6.3 — O que **não** foi smokado, e é o que o integrador deve pedir

| | por quê |
|---|---|
| **abrir um `.ph2dproj` gravado ANTES desta linha** | o `PROJECT_SCHEMA` subiu **sem degrau**: o esperado é uma **recusa legível**, não um crash nem uma leitura errada. ⚠️ É o único ponto do fecho em que o comportamento correcto é o app **dizer não** |
| **um conjunto de Morph States dentro de um projecto REAL** (não a cena de smoke) | toda a jornada correu sobre a `=75` |
| **o modo do Morph junto com o playhead a andar** | os dois relógios existem; nunca foram exercitados juntos |

---

## 7 — O que fica ABERTO (não é trabalho parado — é bloqueio ou decisão do Enio)

| | estado |
|---|---|
| *"funcional no runtime do game"* (o pedido original do item 2) | ⛔ **BLOQUEADO** no `shells/game`/**R1**, adiado por decisão do Enio. É o **mesmo** bloqueio dos contextos do Input Map, e **não** um preço desta feature: a lei já vive numa crate-folha e corre no modo de pré-visualização hoje |
| interromper um morfo **a meio** faz a forma **saltar** em vez de desmorfar | ⚠️ **NOMEADO, não curado** ([§11.3](../32_plano_maquina_de_estados_do_morph.md)): uma pose carrega **uma** forma, e o par vivo `(A, B, t)` não cabe nela. Curá-lo é **modelo novo** — decisão do Enio |
| as **setas** desenhadas no canvas e o arrasto forma→forma | ⛔ **RETIRADOS por decisão do Enio** (25/08) — ⚠️ o código existiu (W3a/W3b) e foi apagado; não reconstruir sem ler o [§5](../32_plano_maquina_de_estados_do_morph.md) |
| **item 3 da fila: Texture pattern** | ⏳ **é o que falta da fila inteira**, e continua **sem plano** ([doc 29 §F2](../29_fila_morph_state_machine_e_texture_pattern.md)) |

---

## 8 — A UMA LINHA do `CLAUDE.md §5` (o integrador escreve; a narrativa fica AQUI)

Na entrada **Vector**, na fila do *Aberto*, o item **(2)** passa de pedido a fechado:

> ✅ **(2) A MÁQUINA DE ESTADOS DO MORPH FECHOU** ([plano 32](docs/Vector%20Module/32_plano_maquina_de_estados_do_morph.md) W1–W11j, [handoff](docs/Vector%20Module/handoffs/HANDOFF_INTEGRACAO_line_Vector_morph_states_2026-08-26.md)) — um botão faz o conjunto, **as setas são virtuais**, **uma tecla por FORMA** (a lista é `n`, não `n(n-1)`), um **modo** que toma o teclado, e o conjunto **anima dentro do sistema de States**. ⚠️ A lista de estados **são os FILHOS**, e daí saem de graça o arrastar-para-dentro, a ocultação e a reparação da animação quando uma forma sai (⊘ · apagar · arrastar para fora). ⛔ *"funcional no runtime do jogo"* segue **bloqueado no `shells/game`/R1**, adiado pelo Enio — o mesmo bloqueio dos contextos do Input Map. Cena **`=75`** · diagnóstico `PH2D_MORPH_LOG=1`.

⚠️ **E o item (3) Texture pattern passa a ser o único aberto da fila.**

---

## 9 — ⚠️ As cinco coisas que uma leitura rápida do diff entende ao CONTRÁRIO

1. **`PROJECT_SCHEMA` subiu sem degrau de migração — e está certo.** Ver §3.2. Quem "consertar"
   acrescentando um degrau vai escrevê-lo contra um formato que nunca existiu em disco.
2. **A secção *Morph States* não é um pedaço da secção *States*.** A W4 pendurou-a lá e o Enio
   reportou (*"contaminou a feature previamente implementada"*); a W7 **restaurou o
   `paint_states.rs` byte-a-byte a partir do `main`**. ⛔ Se o merge as juntar outra vez, o defeito
   volta — e **nenhum dos 12 gates da W4 o via**, porque nenhum olhava para o que era **pintado**.
3. **A ocultação de um membro NÃO é um `Visibility` guardado.** Ela é **derivada** de ser filho de
   um conjunto (`morph_set::is_set_member`, lida pelo `vec_entities::visible_chain`). Reintroduzir a
   escrita faz o arrasto na Hierarquia mover **metade** — a forma entra na lista e continua a
   desenhar-se, ou sai da lista e **desaparece**.
4. **A `Transition::at` SEGURA a forma de partida no meio do voo.** Calá-la ali foi tentado (W11d) e
   **revertido** (W11e): a pose é o escritor de **base** e o passo **refina-a**; calada a base, o
   mundo fica com o valor do quadro anterior sempre que o passo não fala.
5. **A reparação da tabela de States corre TARDE no quadro, de propósito** — depois do despacho dos
   painéis, para entrar na **mesma fotografia** do gesto e custar **um** Ctrl+Z. A correr antes, ela
   chega um quadro atrasada e custa um **segundo** passo de undo.

---

## 10 — ⚠️ As três premissas do plano que a implementação REFUTOU

1. **«a parede é o `VecMorph` ser entre duas formas»** — falso. Quem já interpola N objectos entre
   poses é a `ph2d-ui-state`; a parede era o **catálogo** (quatro papéis fixos, sem estado nomeado).
2. **«o playhead é o modo»** — falso, e custou um report. O playhead **não tranca o teclado do
   editor**: com ele a andar, a mesma tecla morfa a forma **e** faz o que ela faz no editor.
   *Um modo cuja entrada não exclui os outros consumidores não é um modo — é mais um produtor.*
3. **«ordenar os dois motores dentro do quadro basta»** — falso. A transição de UI só ganha nos
   instantes em que **fala**, e ela **cala-se nas pontas**: no repouso e na chegada quem escrevia era
   a máquina de teclas. ⇒ o sistema de States tem **precedência** e a máquina **larga**
   (`morph_machine_drive::drives`).

---

## 11 — ⛔ A lição metodológica que esta linha pagou seis vezes

**Uma fixtura que não contém o fenómeno aprova a cura errada.**

Os seis reports encadeados de 26/08 têm **um** padrão: a suíte verde, o produto vermelho, e a
diferença sempre no que o arnês **não sabia produzir** —

- gates que mediam `capture`/`install` (as duas metades **certas**) enquanto o defeito vivia na
  **composição**, no braço do despacho;
- gates que mediam o **componente guardado** em vez da resposta que o **canvas lê**;
- e, no fim, **quatro waves seguidas** que puseram o conjunto na **raiz**, onde a chave da tabela de
  States e o id do conjunto são o mesmo **por construção** — enquanto o do Enio era **filho de outra
  forma**, com a animação na tabela do **PAI**.

⚠️ **E foi o INSTRUMENTO que fechou a distância, não mais leitura:** quatro hipóteses tinham sido
eliminadas por grep e o produto continuava vermelho. *Quando N leituras eliminam N hipóteses e o
produto continua vermelho, o que falta é um FATO.* O `PH2D_MORPH_LOG` teve de ser corrigido **duas
vezes** antes de servir — imprimia por quadro (*"há milhares de logs"*), e depois nomeava o sintoma
sem nomear a causa; foi o **inventário da cena** que a deu.

⚠️ **Cinco vezes escrevi a guarda certa e nenhum gate a cobria** — registado em
[`feedback_i_write_the_right_guard_and_do_not_gate_it`](../../../project-memory/feedback_i_write_the_right_guard_and_do_not_gate_it.md),
com o padrão refinado pela medição: *o dano vive um passo à frente do que o gate da feature olha —
noutro subsistema, ou no quadro seguinte.*

---

## 12 — Onde ler o mecanismo

- **[plano 32](../32_plano_maquina_de_estados_do_morph.md)** — §1..§16, uma secção por wave, com a
  tabela medida e as provas de mutação. O **§16** é o achado que fechou a jornada.
- **[doc 29](../29_fila_morph_state_machine_e_texture_pattern.md)** — a fila, com o item 2 fechado e
  o 3 como único aberto.
- **[pesquisa 31](../31_pesquisa_maquinas_de_estado.md)** — a base Rive e as duas correcções.
- Memórias novas desta linha: `feedback_a_map_the_tick_clears_must_be_opened_not_looked_up` ·
  `feedback_a_runtime_machine_must_be_seeded_by_the_world_it_drives` ·
  `feedback_a_shared_section_header_is_a_regression_to_whoever_arrived_first` ·
  `feedback_a_mode_whose_entry_excludes_nobody_is_just_another_producer` ·
  `feedback_a_model_change_must_re_ask_what_every_gate_still_measures` ·
  `feedback_i_write_the_right_guard_and_do_not_gate_it`.

---

**Linha `Vector` pronta (HEAD `9a6c901ab`, 29 commits). Aguardo ordem de integração.**
