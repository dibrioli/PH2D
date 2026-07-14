# HANDOFF DE INTEGRAÇÃO — `line/motion-value` (2026-07-13)

> ⚠️ **HISTÓRICO — CONSUMIDO PELO INTEGRADOR.** A linha INTEGROU em 2026-07-13 (`main` `4d203d48`).
> Este era o handoff PRO INTEGRADOR; o handoff vivo da linha é
> [`HANDOFF_line_motion_value_continuacao_2026-07-14.md`](HANDOFF_line_motion_value_continuacao_2026-07-14.md).
> O registro da integração está em [`REGISTRO_integracao_jornada_2026-07-13.md`](REGISTRO_integracao_jornada_2026-07-13.md).

> **Para o agente integrador.** A linha está **FECHADA e PARADA**: não integrei, não pushei, não
> rodei `ship.sh` (DIRETRIZ §1.5.9 · CLAUDE.md §0.7). O Enio **smoke-testou todas as 7 fatias e
> aprovou**. Gate em lote verde **por exit code** (contrato congelado · caps de LOC · arch-gates ·
> suíte das crates tocadas · clippy `-D warnings` · typos).
>
> **Todo número aqui foi VERIFICADO com comando**, não escrito de memória. E leia a **§12** antes de
> confiar nos meus documentos: eu cometi um erro de fato nesta jornada e o corrigi por escrito.

---

## §0 — Runbook (a ordem exata)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value

# 1. rebase — o `main` ANDOU 3 commits (só memória; §2)
git -C /home/enio/Documentos/Projetos/PH2D fetch origin
git rebase main
#    conflito ESPERADO: project-memory/MEMORY.md   -> UNIÃO das linhas, nunca escolha um lado
#    conflito POSSÍVEL: ph2d-node-registry-init/src/lib.rs (GERADO)
#                       -> NUNCA resolva à mão:  cargo run -p ph2d-node-sync

# 2. gate da ÁRVORE COMBINADA (ADR-0107)
bash scripts/foundational-integrate.sh

# 3. o gate que a LINHA NÃO roda — é aqui que os latentes aparecem (conte 2 a 4 iterações)
./scripts/ship.sh        # NÃO use pipe: o `| grep` mascara o exit code (§8)

# 4. a frase do CLAUDE.md §5 — está pronta pra colar na §9
# 5. push + babysit CI   ← SÓ com ordem EXPLÍCITA do Enio
```

---

## §1 — Coordenadas (verificadas)

| | |
|---|---|
| **Branch** | `line/motion-value` |
| **HEAD** | `ecb8cee3` (+ o commit deste handoff) |
| **Base** | `4cd8ef13` (= `main` no início da jornada) |
| **Commits** | **30** |
| **Diff** | **102 arquivos**, +11 306 / −993 |
| **`main` desde a base** | **+3 commits** — **só `project-memory/`** (§2) |
| **Contrato congelado** | **VERDE, provado:** `architecture_contract_surface` **3/3** (`NodeManifest`=8 · `NodeOp`=2 · `OpResolver`=1) |
| **Crates-nó registradas** | **88** (eram 86) |

**As 7 fatias** (todas smoke-aprovadas pelo Enio):

| # | Fatia | Nota-ADR |
|---|---|---|
| 1 | **Subgrafos** — nesting é uma *dobra da VISTA* | [doc 57](Motion%20Nodes/57_subgrafos_nota_adr.md) |
| 2 | **Params dirigidos por fio** — `value.lfo` → `force.wind.strength` | [doc 58](Motion%20Nodes/58_params_dirigidos_nota_adr.md) |
| 3 | **Busca no add-menu** — fuzzy, ranqueada, nos DOIS nomes | [doc 59](Motion%20Nodes/59_busca_no_add_menu_nota_adr.md) |
| 4 | **O clique que deslizava** — o bug que o menu tinha desde sempre | [doc 59 §5](Motion%20Nodes/59_busca_no_add_menu_nota_adr.md) |
| 5 | **Poisson-disc + Bóia** — 2 nós novos; a neve cai no mar | [doc 60](Motion%20Nodes/60_poisson_e_buoyancy_nota_adr.md) |
| 6 | **Nomes no grafo (F2)** — card, grupo e backdrop | [doc 61](Motion%20Nodes/61_nomes_no_grafo_nota_adr.md) |
| 7 | **Paleta do backdrop (R-click)** | [doc 62](Motion%20Nodes/62_paleta_do_backdrop_nota_adr.md) |

---

## §2 — ⚠️ O `main` andou: a colisão é só de MEMÓRIA (verificado)

Os 3 commits novos (`84d96a66`, `afc858a7`, `b1437eeb`) tocam **exclusivamente** `project-memory/`.
**Zero colisão de código.**

| Arquivo | Situação |
|---|---|
| `project-memory/MEMORY.md` | **Os dois lados APENDARAM linhas e NENHUM removeu nenhuma** (conferido: 0 remoções de cada lado). O merge é **UNIÃO** — **funda as duas listas**, não escolha um lado ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). |
| `feedback_a_click_is_a_press_that_drifted.md` | **Idêntico byte-a-byte** nos dois lados (`git diff` entre eles = vazio). Add/add trivial. |
| 4 memórias **só do `main`** | `feedback_absence_gate_needs_a_presence_sibling` · `feedback_a_mutation_that_survives_may_mean_a_missing_gate` · `feedback_geometry_over_mixed_units_needs_the_consumers_conversion` · `feedback_inherited_affordance_must_be_rederived` — **preserve todas** |
| 1 memória **só desta linha** | `feedback_frozen_contract_can_pick_the_architecture.md` |
| `feedback_stale_comment_and_dead_code_lie.md` | editado **só por mim** — contém a **correção** da §12 |

---

## §3 — As 3 decisões de arquitetura (leia ANTES de resolver qualquer conflito)

**1. O `Graph` NUNCA aninha (doc 57).** O documento ganha *pertencimento* (nó → subgrafo) e o
**editor dobra**. Foi o **contrato congelado** que forçou o desenho: `NodeManifest.inputs` é
`&'static` (porta dinâmica é impossível) e `NodeOp::eval` não recebe `Cook`/`OpResolver` (um nó não
consegue cozinhar um sub-grafo). É o *collapsed graph* da Unreal.
➜ **O cook é BYTE-IDÊNTICO com e sem grupo.** Gate: `grouping_never_changes_the_cook`. **Se ele ficar
vermelho depois do teu merge, alguém quebrou o DESENHO — não o teste.**

**2. Um param dirigido é uma ARESTA que o manifesto não conhece (doc 58).** Não é porta dinâmica: é
estado de **documento**, no mesmo canal do text-param (doc 32). O `would_cycle` **anda pelo fio de
param** e o `remove_node` limpa os **dois lados**.
➜ **Um merge que perca o campo `param_sources` do `Fingerprint` do cook COMPILA e devolve número
velho pra sempre.** O gate que pega:
`re_pointing_a_param_to_another_port_of_the_same_driver_recomputes`.

**3. O label de um nó é um MAPA PARALELO no `Graph`, não um campo do `NodeInstance` (doc 61).** Mesma
razão de sempre: append-only não conflita; campo novo na struct toca **todo** sítio de construção do
repo ([[feedback_foundational_editable_design_for_isolation]]).

---

## §4 — ⚠️ Foundational tocado (a superfície de colisão, verificada)

| Arquivo | O que mudou | Risco |
|---|---|---|
| **`ph2d-nodegraph/src/graph.rs`** | campos novos **`param_sources`** (doc 58) e **`node_labels`** (doc 61) + 7 acessores; **`would_cycle` e `remove_node` ganharam corpo** | **MÉDIO** — é o `Graph`, a foundational mais quente. Aditivo, mas 2 fns existentes cresceram: resolva **pelos estágios do índice** ([[feedback_resolve_conflicts_from_index_stages_not_markers]]) e confira que o walk do ciclo ainda vê os **dois** tipos de dependência |
| `ph2d-nodegraph/src/graph_tests.rs` | **NOVO** — os testes saíram do `graph.rs` (782 LOC vs **cap 700**). **Split, nunca allowlist** | Baixo |
| **`ph2d-nodegraph/src/cook.rs`** | `EvalCtx.driven` · `param()` resolve **fio > override > default** · `cook_node` resolve as fontes na mesma recursão · **+1 campo no `Fingerprint`** | **MÉDIO** — §3.2 |
| `ph2d-nodegraph/src/format.rs` | records **`d`** (v3) e **`t`** (v4), aditivos; o header sobe **só quando o recurso é usado** (senão **byte-idêntico**) | Baixo |
| `ph2d-nodegraph/src/param_source.rs` | **NOVO** (módulo isolado) | — |
| `ph2d-nodegraph/src/attr.rs` | **`VALUE_COLUMN`** (a coluna `"v"`, que era const privado em ~30 crates-nó) | Baixo |
| **`ph2d-motion-doc/src/lib.rs` + `subgraph.rs` (NOVO)** | seção **`[subgraph]`** no formato (aditiva, parseada da cauda) + `Subgraph{id,parent,x,y,title}` | Médio |
| **`ph2d-editor-core/.../types.rs`** | **+3 variantes no FIM** do `enum GraphKey`: `Group`, `Ungroup`, **`Rename`** | Baixo (append-only) — mas **conte, não escolha** |
| `ph2d-editor-core/.../dispatch/keymap.rs` | **`KEY_KEY_G = 0x47`** · **`KEY_F2 = 0xF705`** | Baixo |
| **`ph2d-editor-core/.../dispatch/key.rs`** | **`graph_key_for` virou `pub`, ganhou o parâmetro `alt`**, e é agora **o único mapa** dos verbos do grafo (o shell é o 2º leitor) | **MÉDIO** — se outra linha mexeu no `dispatch_key`, o merge textual pode passar e **só o `check` pegar** |
| **`ph2d-editor-core/.../state/{mod,store_core,store_hierarchy}.rs` + `screens/hero.rs` + `ph2d-panel-timeline/src/marker_rename.rs`** | **`cancel_on_escape` virou FLAG por widget** (`mark_cancel_on_escape`); a lista hardcoded `id == HIER_RENAME_INPUT \|\| id == TIMELINE_MARKER_RENAME_INPUT` **morreu** e os 2 ids agora se marcam | **MÉDIO** — se outra linha adicionou um **3º id** àquela expressão, ele **tem** que virar `mark_cancel_on_escape`, **senão o Esc dele para de cancelar em silêncio** |
| **`ph2d-editor-core/tests/hr12_widgets_a11y.rs`** | **+entrada** na allowlist `PANEL_A11Y_DELEGATE_OK` (`paint_menu.rs`) | **ALTO se outra linha também adicionou** — **funda a união** |
| **`.typos.toml`** | **+3 palavras**: `frase`, `organizacional`, `HDA` | **ALTO se outra linha também adicionou** — **chave duplicada MATA o TOML no parse e o typos nem escaneia** ([[feedback_duplicate_allowlist_key_kills_the_gate_at_parse]]) |
| **`ph2d-ui-testkit`** | **deps novas: `ph2d-host` + `bumpalo`**; `paint_with_layout::<P>()` e `dispatch_pointer_event()` (aditivos) | Baixo |
| `ph2d-node-registry-init` | **REGENERADO** (88 crates-nó) | **Conflito ESPERADO no rebase → `cargo run -p ph2d-node-sync`, NUNCA à mão** |
| `ph2d-panel-motion-params/src/snapshot.rs` | **`ScalarRow.driven`** (campo novo — **toda construção precisa dele**) | Médio |
| `ph2d-editor-core/src/widget/mod.rs` | `format_number` exportado (já existia) | Baixo |

---

## §5 — Ids / consts / variants novos (para detectar colisão)

| Símbolo | Valor / posição | Onde |
|---|---|---|
| `KEY_KEY_G` | **`0x47`** | editor-core keymap |
| `KEY_F2` | **`0xF705`** | editor-core keymap |
| `GraphKey::{Group, Ungroup, Rename}` | **apendados no fim** | editor-core types |
| header do formato de grafo | **`v3`** (param dirigido) · **`v4`** (label) — **só quando usados** | nodegraph format |
| records novos | **`d <id> <param> <src> <port>`** · **`t <id> <label…>`** | nodegraph format |
| seção nova | **`[subgraph]`** | motion-doc |
| `attr::VALUE_COLUMN` | `"v"` | nodegraph |
| `SUBGRAPH_VIEW_TAG` | `0x8000_0000` | panel-motion-graph |
| `GraphIntent` | **+`Rename{target,name}`** · `DriveParam` · `GroupSelection` · `Ungroup` · `EnterSubgraph` · `GoToLevel` · `SpliceReroute` · `SetProbe` · `CutWires` · `MoveWireEnd` | panel-motion-graph |
| `RenameTarget::{Node,Subgraph,Backdrop}` | **enum público novo** | panel-motion-graph |
| `MenuBody::BackdropTints{backdrop,current}` | variant novo | panel-motion-graph |
| typos allowlist | `frase` · `organizacional` · `HDA` | `.typos.toml` |

---

## §6 — Assinaturas alteradas e REMOÇÕES (o `check --workspace` pega; o `merge-tree` **não**)

**REMOVIDO:**
- **`GraphIntent::SetBackdropTitle`** — era **encanamento morto** (handler sem emissor; §12). Virou
  `GraphIntent::Rename{target,name}`. ⚠️ **Se outra linha passou a emiti-lo, ela vira `Rename`.**

**Assinaturas:**
- **`graph_key_for(keycode, cmd, alt)`** — **+1 parâmetro** (editor-core, `pub`).
- **`ScalarRow`** ganhou o campo **`driven`** (panel-motion-params).
- **Internas do painel do grafo** (não cruzam a crate, mas quebram merge textual):
  `geom::{menu_list, menu_row, menu_max_scroll, menu_track, menu_thumb, menu_scroll_at}` **ganharam
  `&Menu`**; `MenuRow.category` → **`.dot: ColorToken` + `.selected: bool`**.

> [[feedback_clean_text_merge_can_be_semantically_broken]] — **um merge limpo no texto pode não
> compilar. Só o `cargo check --workspace` cruza os dois lados.**

---

## §7 — Crates e arquivos novos

**2 drop-crates novas** (dependem só de `ph2d-nodegraph` + `ph2d-node-registry`; leaves copiados):
**`ph2d-node-motion-distribute-poisson`** · **`ph2d-node-force-buoyancy`**.

**Arquivos novos que importam:**
- `ph2d-nodegraph/src/{param_source, graph_tests, cook_param_source_tests}.rs`
- `ph2d-motion-doc/src/{subgraph, subgraph_tests}.rs`
- `ph2d-panel-motion-graph/src/{rename, menu_search, snapshot_menu, snapshot_intent, snapshot_drop, interact_key, interact_subgraph, paint_menu, paint_breadcrumb}.rs`
- **3 gates de integração que PINTAM e DESPACHAM** (`ph2d-panel-motion-graph/tests/`):
  `the_add_menu_actually_adds_a_node` · `f2_actually_renames_the_thing` ·
  `the_backdrop_palette_actually_tints_it` — **todos deslizam 1px** entre o press e o release
  ([[feedback_a_click_is_a_press_that_drifted]]).
- `shells/desktop/src/render_loop/{motion_bridge_fold, motion_bridge_intents, motion_bridge_subgraph, motion_bridge_rename_tests, …}.rs`

---

## §8 — O que **só** o `ship.sh` pega (conte 2–4 iterações)

O gate da linha **não** roda `fmt` do workspace, `clippy --all-targets` do workspace, `machete`,
`deny`, `audit` nem `nextest --cargo-profile ci-test`
([[project_integrator_ship_catches_latents_budget_iterations]]). Latentes prováveis:

- **`cargo machete`** — a `ph2d-ui-testkit` ganhou `ph2d-host` e `bumpalo`.
- **`cargo fmt --check`** — use o **rustfmt PINADO**: `rustup run 1.95 rustfmt --edition 2024 …`. O
  `rustfmt` avulso **quebra** no `cook.rs` (*"let chains are only allowed in Rust 2024"*)
  ([[feedback_ci_direct_lint_gates_and_fmt_skew]]).
- **`typos`** — o risco de **chave duplicada** da §4.
- **`clippy --all-targets` do workspace** — crates que eu não toquei podem acordar.

> ⚠️ **[[feedback_pipe_masks_script_exit_code]]** — `./scripts/ship.sh | grep …` faz o `$?` virar o
> do `grep`, e você lê 0 numa run vermelha. **Meça pelo exit code.** (Eu caí nessa nesta jornada.)

---

## §9 — A frase do `CLAUDE.md` §5 (pronta pra colar)

**Não toquei o `CLAUDE.md` de propósito** — é a maior superfície de colisão do repo e **toda** linha
aberta encosta nela. Acrescente à entrada **Motion Nodes**:

> **Linha `line/motion-value` (2ª jornada, 2026-07-13):** **Subgrafos** ([doc 57](docs/Motion%20Nodes/57_subgrafos_nota_adr.md)) — nesting é **dobra da VISTA**: o `Graph` segue **PLANO** e o cook é **byte-idêntico** com/sem grupo (o contrato congelado forçou o desenho — `NodeManifest.inputs` é `&'static`). Ctrl+G agrupa · Ctrl+Alt+G desagrupa (os nós saem **selecionados**) · duplo-clique entra · breadcrumb sai; a interface do card são **as arestas que CRUZAM** (derivada, nunca declarada) e os vizinhos de fora viram **ghosts** read-only. **Params dirigidos por fio** ([doc 58](docs/Motion%20Nodes/58_params_dirigidos_nota_adr.md)): solte a saída de um nó no **CORPO** de outro e escolha o param — **o fio É a promoção** (não existe estado "promovido"). Um param dirigido é uma **aresta que o manifesto não conhece** (vive no `Graph`, como o text-param), então os **88 nós ficaram dirigíveis sem UMA linha de mudança em nenhum deles** (`EvalCtx::param` resolve **fio > override > default**); o `would_cycle` anda pelo fio de param e o `Fingerprint` do cook ganhou a **fiação** (re-apontar pra outra porta do MESMO nó tem a mesma revisão e valor diferente). **Busca no add-menu** ([doc 59](docs/Motion%20Nodes/59_busca_no_add_menu_nota_adr.md)): fuzzy, **ranqueada**, casando nos DOIS nomes (rótulo + canônico → o **domínio vira query**). **+2 nós** ([doc 60](docs/Motion%20Nodes/60_poisson_e_buoyancy_nota_adr.md)): **`motion.distribute_poisson`** (Bridson 2007 — **o raio é o knob, a contagem é a resposta**; complementa o `motion.scatter`, que é Mitchell e nomeia a *contagem*) e **`force.buoyancy`** (Arquimedes + **onda viajante**; o empuxo é normal à **superfície**, então o flutuante cavalga a marola). **A demo de boot virou "a neve cai no MAR"** (queda → splash que **bate no leito** → boia) e seus 19 cards **têm nome**. **Nomes no grafo** ([doc 61](docs/Motion%20Nodes/61_nomes_no_grafo_nota_adr.md)): **F2** renomeia card/grupo/backdrop — o label do **NÓ** é canal novo (mapa paralelo no `Graph`, record `t`, header **`v4` só quando usado**); o backdrop/grupo **já** podiam ser nomeados pelo painel de params (doc 61 §2 — correção). **Paleta do backdrop** ([doc 62](docs/Motion%20Nodes/62_paleta_do_backdrop_nota_adr.md)): **R-click no cabeçalho** → 8 tons; a busca do popup virou **exclusiva da biblioteca** (o menu de portas vinha pintando uma caixa inerte **que roubava o teclado**). **Gotcha permanente:** todo gate de clique deste painel **desliza 1px** entre press e release — [[feedback_a_click_is_a_press_that_drifted]].

---

## §10 — Smoke (o Enio já aprovou; isto é o re-smoke pós-integração)

```bash
cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop
```

1. **Boot:** a neve cai, **atravessa a água**, **bate no leito** e **boia** na marola. Os cards **têm
   nome** ("The Sea", "Birth Sites", "The Kill Disc").
2. **`A`** → digite `lfo` → clique na linha **deslizando** o mouse (a mão sempre desliza). O nó entra.
3. Puxe o fio do LFO pro **corpo** de um `force.wind` → escolha `Strength` → **o nó ganha um socket**,
   o param oscila, e a linha dele vira **read-only** no painel de params mostrando o número vivo.
4. **Ctrl+G** agrupa · duplo-clique entra · breadcrumb sai · **Ctrl+Alt+G** desagrupa (**os nós saem
   selecionados**).
5. **F2** num card → renomeie → Enter (**Esc** mantém o antigo). Depois de fechar, **`A` volta a
   abrir o menu** (o teclado voltou pro grafo).
6. Chip **Backdrop** → **R-click no cabeçalho dele** → os 8 tons.

---

## §11 — Detalhe de merge que não coube nas tabelas

- **Subgrafos (doc 57):** o `card_ports` do subgrafo **precisa enxergar o fio de param** (doc 58) —
  sem isso, agrupar um nó cujo param é dirigido de fora **faz o fio sumir da tela**, e a tela passa a
  mentir sobre o que a cena computa.
- **Params dirigidos (doc 58):** **3 lugares desligam um fio** (Disconnect, faca, ponta arrastada) e
  **todos passam por UM funil** (`subgraph::unplug`). Um 4º caminho **tem** que passar por ele.
- **Busca (doc 59):** `menu_rows`/`menu_matches` são a **única** fonte das linhas (pintura + hit +
  geometria). Um 2º enumerador é como uma linha passa a significar uma coisa na tela e outra debaixo
  do cursor — **este popup já teve exatamente esse bug**.
- **Poisson + Bóia (doc 60):** o gate do chão do shell tinha um **número mágico duplicando a
  constante da demo** (`-2.0 + 2.4` à mão) e teria ficado **verde apontando pra água vazia**. As
  constantes da demo agora são `pub(crate)` e **o gate lê a constante**.

---

## §12 — ⚠️ Um erro MEU nesta jornada (não confie nos meus docs sem conferir)

Eu afirmei — **num commit e num ADR** — que o rename/cor do **backdrop** e o nome do **grupo** "nunca
tinham sido construídos", porque greppei `GraphIntent::SetBackdropTitle`/`SetBackdropColor` e não
achei emissor.

**Era falso.** O **painel de params** já tinha, o tempo todo, as linhas **Title**/**Color** do
backdrop e a linha **Name** do grupo — **por um canal de intent diferente**
(`motion_bridge_backdrops::params_snapshot` → `apply_param_intent`). Os dois `GraphIntent` eram
**duplicatas mortas**, não a ausência da capacidade. Descobri **rodando o seam** e imprimindo as rows.

- **Corrigido por escrito:** [doc 61 §2](Motion%20Nodes/61_nomes_no_grafo_nota_adr.md) ·
  [doc 62 §1](Motion%20Nodes/62_paleta_do_backdrop_nota_adr.md) · memória
  [[feedback_stale_comment_and_dead_code_lie]] (3º caso).
- **A mensagem do commit `5b80e36d` carrega a afirmação errada** — não se reescreve histórico; os
  docs corrigem.
- **O que o F2 REALMENTE trouxe:** o **label do NÓ** (isso sim não existia em canal nenhum) **+ o
  gesto inline**.

**A lição:** *cace a **CAPACIDADE**, não o símbolo.* **"Quem emite X?"** e **"o usuário consegue
fazer X?"** são perguntas diferentes, e só a segunda importa — e a resposta se dá **executando**,
nunca por grep. Um sistema com N caminhos pra mesma ação tem **N−1 candidatos a PARECEREM mortos**.

---

## §13 — Fila restante (para quem pegar a linha depois)

**A fila de fan-out ACABOU** — o catálogo do plano está construído (88 nós) e o editor F2 está
completo (busca, grupo, faca, probe, backdrops, duplicate, rename). **O que sobra é decisão do
Enio**, não escolha do implementador:

1. **FX de PASSE no compositor HDR** (glow/bloom/blur/vignette/levels/hue-shift) — **cross-module**
   com `ph2d-painter-effects`. **É arquitetura, não fan-out: PARE e reporte.**
2. **`motion.path`** — o plano dizia *"integra `vector.*`"*, mas aquele sistema foi **RETIRADO**
   (ADR-0108) e a geometria mora em `ph2d-vec-scene`, que **o cook não alcança** (um nó só recebe
   params/inputs/playhead). Exige um **canal novo shell→cook**. **Decisão do Enio.**
3. **W4.T4 — dock da timeline** (`motion_timeline_slot`) — **encosta na linha `anim`**.
4. **`motion.delay`** — fan-out normal, mas **marginal**: o `motion.time_remap` já atrasa uma
   sub-árvore **Pure** de graça, o `trail` já ecoa e o `slit_scan` já defasa. Só se justifica pra
   atrasar **o que não é função de `t`** — uma simulação.

### Gaps conhecidos (nomeados, não escondidos)

- **Backdrop órfão:** um backdrop cujos nós foram agrupados fica na raiz emoldurando nada. Decisão
  consciente (um backdrop não *possui* nada), mas dá pra questionar.
- **Reuso/instanciação** (datablock do Blender / HDA): **fora por desenho** (doc 57 §7) — exigiria
  mexer no contrato congelado.
- **O menu do card não oferece input já alimentado por DENTRO** (doc 57 §6.1) — decisão, não
  esquecimento.
- **`SetBackdropTitle` não voltou:** o F2 é o gesto de nome; o painel de params segue sendo o outro
  caminho pro título.
