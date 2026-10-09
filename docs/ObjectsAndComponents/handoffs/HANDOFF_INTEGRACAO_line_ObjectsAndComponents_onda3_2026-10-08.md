# HANDOFF DE INTEGRAÇÃO — `line/ObjectsAndComponents`, onda 3 (08/10): o Motion como objecto + busca por intenção + filtro da Hierarquia

> Leitor: o agente integrador (e a próxima janela desta linha). Plano e leis:
> [`01_objeto_e_componente.md`](../01_objeto_e_componente.md). Ondas anteriores:
> [onda 1](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_2026-10-08.md) ·
> [onda 2](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_onda2_2026-10-08.md). **As TRÊS ondas vão
> juntas na mesma branch** — integre-as num só passo. ⚠️ A onda 3 toca ficheiros da
> `line/motion-value` (viva): §4, secção «Colisão com a motion-value».

## 1. Identidade

- Branch `line/ObjectsAndComponents` · worktree `Worktrees/line-ObjectsAndComponents` · base (merge-base)
  `2a925859d` (= o `main` de 08/10; o `main` não andou, sem rebase). Ondas 1 e 2 já na branch (22 commits).
- Commits da onda 3, por cima: F6 `87d332a8a`, `61fc08972` · F5 + curas `169b4d233` (dados + fixturas),
  `d5a211755`, `776c3a844`, `673535ac6`, `71c864e57`, `ddca9b62f`, `3db8a529c`, `1fb4ef953`,
  `d70fcfc6c` (curas do portão) + os commits deste handoff.
- Diff da onda 3 (`3005b1192..HEAD`): 103 ficheiros, `+3998 −155`. **Shell: `+61 −27`**;
  `the_shell_only_shrinks` verde.

## 2. O que entrou

Escolhas do DONO (08/10): **F5.1** o resultado do Motion é SEMPRE visível (Play incluído; some pelo
olho) · **F5.2** a forma copiada vira FILHA do objecto Motion e não se desenha sozinha · **F5.3**
mover/girar/escalar o objecto leva o que ele gera («vai junto», como no Cavalry) · **F6.1** busca por
intenção só em inglês · **F6.2** filtro = botão ▾ ao lado da busca, só componentes presentes, com contagem.

### F5 — o Motion é um objecto (Object · Edit, nós com dono)

- **Dados** (`ph2d-motion-doc`): `MotionDoc.owners: BTreeMap<NodeId,u64>` + `backdrop_owners`; secção
  `[owner]` (`o <node> <sid>`, `ob <backdrop> <sid>`) escrita POR ÚLTIMO e só se não-vazia ⇒ documento
  antigo grava byte a byte o mesmo (fixturas `crates/ph2d-motion-doc/tests/fixtures/*_antes_do_dono.txt`,
  gravadas em `169b4d233` ANTES da mudança; README lá). API: `copy_owner`, `remove_owner`, `rehome`,
  `adopt_unowned`. O texto do MotionDoc não tem versão própria; **`PROJECT_SCHEMA` NÃO sobe (187)**.
- **Mundo:** `ph2d_ecs::MotionGenerator` (componente REGISTADO) · `MotionTemplate` DERIVADA (não registada)
  + `assign_motion_templates` · `ObjectKind::Motion` (`ALL` 6→7; fora do `DRAWABLE`) + descritor
  intrínseco «Motion Generator» (família Animation) · `SubtreeCopies` com `CopyReader{Timeline,Motion}`.
- **`ph2d-app-motion`** (`src/motion_objeto*.rs`, `motion_object_add.rs`): upkeep por quadro —
  migração (nó sem dono, ou dono que nunca viveu ⇒ objecto em edição ou nasce «Motion»); dono que
  estava vivo e morreu ⇒ os nós saem num passo do undo do Motion, nunca com o relógio a andar; cópia do
  editor ⇒ `copy_owner`; o dono em edição segue a selecção e o pai de um molde; moldes ligados no
  momento da ligação/migração (o 1.º quadro só semeia). `view::so_do_dono` antes do fold. `modo::Family`
  (Object·Edit; wants com a ferramenta na mão; ⛔ SEM conteúdo na Hierarquia — dono no smoke de 09/10:
  «nodes apenas nos grafos»; o «Nodes (n)» que nasceu nesta onda saiu, com teste a trancá-lo).
  Add ▸ Motion no grupo 2D (i18n `object_add.motion`). `MotionState.objeto: ObjetoState`.
- **Origem do dono (a pose do objecto desloca o que o grafo gera):**
  CPU — `MotionCookPump.faixas: Vec<FaixaDoSink>` + `composicoes` (`ph2d-eval-motion`);
  `origem::aplica_cpu` corre ANTES do LOD, sobre a cópia local (recozinhar o tick avançaria a
  simulação; identidade = zero alocação). Placa — `GpuInstances/GpuFormas.faixas`,
  `ph2d_gpu_cook::instances::dono::PasseDoDono` + `GpuCook::aplica_donos` (sprites 47 palavras base 12;
  formas 16 base 4; a placa reescreve a cada quadro ⇒ nunca duas vezes).
- **Gizmos:** `Camera2d.model: [f32;6]` (identidade por omissão; `world_to_screen`/`screen_to_world`/
  `affine`/`view_proj`) + `motion_objeto::{afim_em_edicao, camera_do_dono, pai_em_edicao}`; warp, pivô,
  colisor e pontos usam a câmara do dono; o campo usa `parent_world` = pose do dono.
  `ph2d_node_source_object::OBJECT_PARAM` passou a `pub`.
- **Espaço:** o grafo pensa no espaço do OBJECTO; entradas de mundo (cursor, colisores) continuam em
  mundo (o *layer space* do Cavalry); com o objecto na origem coincidem (ABERTO a).

### F6.1 — busca por intenção (Add Component)

- Tabela `crates/ph2d-i18n/src/component_keywords.rs` (59 componentes oferecíveis); chave derivada
  `ComponentDesc::keywords_key` = `component.<snake>.keywords`. `PaletteItem.keywords: &'static str`
  (14 construtores; `""` nos outros consumidores).
- `item_matches(query, &PaletteItem)` = rótulo por substring OU toda palavra da busca em rótulo+palavras.
  `top_match` em 3 degraus: nome > palavra inteira > fragmento («jump» achava o Nav Link «jump link»).
- A busca saiu para `widget/command_palette/busca.rs` (tecto 500 LOC). Censo do catálogo declara as duas
  tabelas; chave derivada de componente vivo conta como usada. «jump»/«fall» ⇒ Physics Body (o Platform
  Player não é oferecido pelo `+`).

### F6.2 — filtro da Hierarquia por componente

- `ComponentSet(u128)` em `ph2d-component-desc` (bit i = i-ésimo autorado; bit 127 = Animação) ·
  `HierarchyEntity.has` · `RowComponents::set_of` (sonda 1× por archetype) · `declare_in_row(timeline,
  bits, &mut HierarchyEntity)` marca o bit da Animação · `HierarchyState.component_filter: Option<u8>`.
- `ph2d-panel-hierarchy/src/filter.rs` + `ids/filter.rs`: `HIER_FILTER_DD`, `hier_filter_opt(i)` derivado
  nos 58 bits baixos (os altos são companheiros que o despacho lê), `HIER_FILTER_ROWS = 64`;
  `compute_match_filter(..., filter: Option<u8>)`. i18n `panel.hierarchy.filter.{all,option,animation}`.

## 3. Versões e contadores (DELTA)

- `PROJECT_SCHEMA` **187, sem diff**. `DOC_VERSION` da timeline **19, sem diff**. MotionDoc: formato de
  texto SEM versão; secção `[owner]` só quando não-vazia.
- Registo de componentes (**CONTE o delta**, não escolha): `ph2d-ecs` **105 → 106**
  (`MotionGenerator`); espelhos `ph2d-render` e `ph2d-script` **106 → 107**. Recontar com os gates
  da árvore combinada.
- `ObjectKind::ALL` **6 → 7** (Motion). Catálogo de componentes oferecíveis com palavras: 59.
- Contratos congelados: **nenhum tocado**.

## 4. Superfície de colisão — o que um merge pode partir

⚠️ Assinaturas que mudaram (outra linha que as use reprova a compilação da árvore combinada):

| sítio | mudança |
|---|---|
| `Camera2d` (`ph2d-render`) | campo NOVO `model: [f32;6]` — construtor literal sem `..Default` reprova (havia 1, curado) |
| `ObjectKind` | variante NOVA `Motion` — `match` exaustivo noutra linha reprova |
| `ph2d-ecs` `SubtreeCopies::take_copies(world, reader)` | ganha `CopyReader{Timeline,Motion}` (antes drenava: dois consumidores precisam de leitores) |
| `ph2d-panel-timeline` `declare_in_row` | recebe `(timeline, bits, &mut HierarchyEntity)` (antes components+content) |
| `item_matches` / `PaletteItem` | `PaletteItem.keywords` novo (todo literal à mão precisa dele); `item_matches(query, &PaletteItem)` |
| `compute_match_filter` | ganha `filter: Option<u8>` |
| `ph2d_node_source_object::OBJECT_PARAM` | passou a `pub` |
| `MotionCookPump` | campos `faixas`, `composicoes`; `GpuInstances/GpuFormas.faixas` |
| `MotionState` | campo `objeto: ObjetoState` |
| `paint_hierarchy_body`/`paint_hierarchy_head` | internas, mudaram |
| `GizmoDragState` | intacto |

- **Colisão com a `line/motion-value` (viva)** — ficheiros dela que a onda 3 tocou, todos ganchos pequenos:
  `Cargo.lock` +1 (aresta `ph2d-app-motion → ph2d-component-desc`; **regenere, nunca à mão**) ·
  `ph2d-app-motion/src/lib.rs` +2 · `motion_bridge_quadro.rs` +4 (`aplica_cpu` antes do LOD;
  `aplica_gpu` antes do `return` do `cook_gpu`) · `motion_bridge.rs` +3 −2 (gate
  `!motion_active && !motion.objeto.desenha()`; `sinks_visiveis`) · `motion_state.rs` +3 (campo
  `objeto`; +1 linha em `motion_state_verbos.rs`) · `shells/desktop/src/render_loop/fase_motion_gizmos.rs`
  +1 −1 (câmara do dono no retrato do colisor) · `present.rs` +6 −2 (`motion_draws`).
- **Ids/i18n novos:** `HIER_FILTER_DD`, `hier_filter_opt(i)`;
  `object_add.motion`, `panel.hierarchy.filter.{all,option,animation}`, `component.<snake>.keywords` (59).
- **Shell (composição):** `+61 −27`; a catraca soma entre linhas (onda 1 +35, onda 2 +194).

## 5. Portão de fecho (batched, sobre o diff acumulado)

- 1.ª corrida (sobre `61fc..1fb4`): `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` ✓ ·
  censos `--workspace` **82/82** ✓ · `machete` ✓ · `check-standalone-optional` ✓ ·
  `check-workflow-packages` ✓ · `nextest-impacted` **16882/16884** — 2 reais: `command_palette.rs`
  516>500 LOC e `fase_vector_overlays` 201>200 — curados em `d70fcfc6c` · clippy 3
  (`too_many_arguments`, `needless_borrow`, `type_complexity`) curados · `fmt` 48 ficheiros curado.
- Depois das curas: `clippy --workspace --all-targets -D warnings` ✓ · `fmt` ✓ · os dois tectos ✓.
- 2.ª corrida, sobre o diff final (`d70fcfc6c`): `nextest-impacted` **16886/16886** (9915 ignorados), 101,8 s.

## 6. Provas de mutação (18/18 SANGRARAM; 4 controlos cada — verde antes, âncora casou 1×, compila, restaurado com `checkout` + `touch`)

| # | mutação | veredito |
|---|---|---|
| M1 | `intent_words` | SANGROU |
| M2 | degrau «palavra inteira» | SANGROU |
| M3 | `filter_hit` | SANGROU |
| M4 | `set_of` por archetype | SANGROU |
| M5 | leitor de cópias | SANGROU |
| M6 | `adopt_unowned` na migração | SANGROU |
| M7 | guarda do Play | SANGROU |
| M8 | `copy_owner` | SANGROU |
| M9 | cópia local da origem | SANGROU |
| M10 | `MotionTemplate` no `off_canvas` | SANGROU |
| M11 | a raiz não é molde | SANGROU |
| M12 | `so_do_dono` | SANGROU |
| M13 | `unmodel_point` | SANGROU |
| M14 | WGSL `t` (na placa, `PH2D_GPU=1`, `--ignored`) | SANGROU |
| M15 | `field_view` com a afim | SANGROU |
| M16 | `adopt_new` com dono | SANGROU |
| M17 | arestas do `copy_owner` | SANGROU |
| M18 | semear moldes | SANGROU |

## 7. Testes novos

- `ph2d-motion-doc` `owner_tests` (10, incl. as 2 fixturas byte a byte).
- `ph2d-app-motion`: `motion_objeto_tests` (11) · `motion_objeto_modo` (2) · `field_gizmo_tests` (+1) ·
  `motion_bridge_subgraph_tests` (+2) · `motion_object_add` (1).
- `ph2d-gpu-cook` `dono` (1, `#[ignore]` placa — verde aqui) · `ph2d-render` `camera` (+1).
- `ph2d-ecs` `subtree_copies` (1), `motion_generator` (1) · `ph2d-entity-visibility` `off_canvas` (+1).
- `ph2d-app-components` `component_palette_intent_tests` (3), `row_components` (1).
- `ph2d-editor-core` `command_palette` (+3) · `ph2d-panel-hierarchy` `the_component_filter` (3, cliques
  reais) · `ph2d-component-desc` `component_set` (2).

## 8. Premissas que caíram

1. «As formas publicadas como filhos» — o resultado é um fluxo de instâncias; os filhos são os MOLDES.
2. «O Motion já é coisa da cena» — só desenhava com a ferramenta na mão.
3. «N documentos» — UM, com dono por nó (258 ficheiros contam com o tipo).
4. «`take_copies` drena» — dois consumidores precisam de leitores.
5. «A origem num ponto comum» — são ~25 sítios, 3 caminhos de desenho + gizmos.
6. «Pular ⇒ Jogador plataforma» — o Platform Player não é oferecido; é o Physics Body.
7. As formas do Motion (passe vectorial e faixas em `fase_hero_scene`) também estavam presas a
   `motion_tool_active` — a foto do smoke apanhou.

## 9. Smokes (binário `smoke` compilado na worktree)

```bash
cd ~/Documentos/Projetos/PH2D/Worktrees/line-ObjectsAndComponents
PH2D_MOTION_OBJETO_SMOKE=1 ./target/smoke/ph2d-host-desktop   # Object, ferramenta desligada
PH2D_MOTION_OBJETO_SMOKE=2 ./target/smoke/ph2d-host-desktop   # Swarm em Edit
PH2D_VIDA_SMOKE=1 ./target/smoke/ph2d-host-desktop            # F6: ▾ ao lado da busca; Add Component
```

Fotografados com `fotografa_cena.sh`:

- `=1`: duas simulações — **Pile** (formas a cair em duas taças, cena `=114`) e **Swarm** (cópias da
  Ball puxadas por íman/ruído/arrasto, objecto em (400,150) px rodado ~20°). Hierarquia «Scene ▸ Pile»,
  «Swarm ▸ Ball» (o molde; sem linhas de nós). Errado se o Swarm não acompanhar a pose do objecto, ou
  se a Pile sumir com a ferramenta desligada.
- `=2`: Swarm em Edit — o editor de nós mostra SÓ os 8 nós dele. Errado se aparecerem os da Pile.
- F6 em `PH2D_VIDA_SMOKE=1`: o ▾ «All» ao lado da busca (lista só os componentes presentes, com
  contagem); Add Component com «fall» e «jump» ⇒ Physics Body, «play sound» ⇒ componente de som.
- As legendas da demo `=114` não seguem a origem do dono ⇒ a Pile fica na origem no smoke (ABERTO c).

## 10. O que fica ABERTO

- (a) **Entradas de mundo** (cursor, colisores da cena, câmara) não são levadas ao espaço do objecto:
  com o objecto deslocado, um grafo que siga o cursor fica deslocado (é o *layer space* do Cavalry).
  Cura candidata: opção «world space» por nó.
- (b) Gizmo de campo com escala NÃO uniforme do objecto e campo rodado: a caixa mostrada é aproximada
  (o `GizmoView` não exprime cisalhamento).
- (c) Legendas/overlays próprios das demos (`motion_demo_legend`) não seguem a origem.
- (d) Undo de apagar um objecto Motion = DOIS Ctrl+Z (o global devolve o objecto, o do Motion devolve
  os nós) — a lei da Animação.
- (e) O template linking só liga `source.object` (os `path` de `motion.path`/spline-wrap são guias, não
  moldes).
- (f) **MAX do filtro ▾:** ≤ 64 linhas (catálogo 59 + Animação + All); numa cena com > ~30 tipos o
  popover fica maior que o ecrã (clampa ao viewport; linhas fora não se registam).
- (g) Carry-over da onda 2: nome «andar 2», undo em dois passos, a animação própria não toca sozinha no
  Play (handoff da onda 2, §10).
- **Próxima onda: decisão do DONO** (nenhuma está planeada); o integrador não a abre.

## 11. Fecho

- `rm -rf target/*/incremental` e build `smoke` (2×), 2.ª saída:
  `rm -rf target/*/incremental` feito; `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida:

```text
    Finished `smoke` profile [optimized] target(s) in 0.19s
```

- `bash scripts/agent-loop-profile.sh`:

```text
  ✗ paralelismo de ferramenta              1.14/passo   alvo: >= 1,5  (10% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                212   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                484 : 294   alvo: <= 1,0  razao 1.6x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  37%   alvo: >= 80%  (982 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         411 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```
- `bash scripts/agent-loop-profile.sh` corre no fim.
- Integrar não é aprovar: smoke é do Enio; integração e ship só por ordem dele.
