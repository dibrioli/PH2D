# HANDOFF DE INTEGRAÇÃO — `line/ObjectsAndComponents`, onda 1 (08/10): a Hierarchy nova

> Leitor: o agente integrador (e a próxima janela desta linha). Plano e leis:
> [`01_objeto_e_componente.md`](../01_objeto_e_componente.md) — o §11 diz o que entrou e as
> premissas que a medição derrubou.

## 1. Identidade

- Branch `line/ObjectsAndComponents` · worktree `Worktrees/line-ObjectsAndComponents` · base `2a925859d`
  (= o `main` de 08/10 de manhã; a linha não precisou de rebase).
- Commits de código: `574409ff2` (F1–F3) · `b8783b538` (F0) · `92d5ad232` (curas do gate) + o commit
  deste handoff. Os três antes deles são o plano (`9046f843b`, `c9e061d9e`, `adc8d861f`).
- Diff: 67 arquivos, `+2005 −99`. **Shell: `+37 −2` (líquido +35)**, só composição.

## 2. O que entrou (F0–F3 do doc 01 §8)

- **F1, raiz Cena**: a 1.ª linha da Hierarquia, SINTÉTICA (`RowRole::Scene`, id
  `ph2d_panel_hierarchy::ids::HIER_SCENE_ROW`). Seleccionada quando nenhum objecto está; o clique
  limpa a selecção (o cadeado do modo já permite «limpar»). Os componentes dela são **Mundo físico** (o ícone abre o
  painel `physics`) e **Animação da cena** (abre a `timeline`). Todos os objectos descem um nível.
- **F2, ícones de componente**: um por FAMÍLIA presente (`hier_rows::component_icon`), lidos pelo
  archetype (`ph2d_app_components::row_components`, sem serializar). O clique selecciona o objecto,
  abre o Inspector e pede o cartão (`WidgetStore::request_inspector_reveal`); o Inspector
  (`ph2d-panel-inspector/src/reveal.rs`) abre o 1.º cartão da família e, no quadro seguinte, rola-o
  até à margem do 1.º cartão. A família de um cartão vem do CATÁLOGO pelo componente que ele edita
  (`SECTION_COMPONENT`, 46 = `LIVE_SECTIONS`, gate de cobertura nos dois sentidos).
- **F3, conteúdo**: `ModeFamily::content` / `open_content` (default vazio). Painter = as camadas de
  cada imagem com documento (ligado + `doc_cache`); Flip = as camadas de cada desenho. Recolhido por
  omissão; clicar numa peça = `HierRequest::OpenContent` → `ModeRequest::OpenContent` → o `drive`
  selecciona o dono, entra no modo do grupo e entrega a peça (Painter: pendente até ao `bind_document`).
- **F0, vocabulário**: «Prefab» nos textos do MOLDE (`shell.rs`, `chrome_panes.rs`), logs dos
  smokes de instância, o fallback do nome do dono (`asset.kind.component`), o passo do tutorial 07.

## 3. Foundational / partilhado tocado — e porque é aditivo

| sítio | o quê |
|---|---|
| `ph2d-editor-core/src/hier_rows.rs` (**novo, folha**) | `RowRole`, `ContentGroup`/`ContentItem`/`ContentMap`, `component_icon`, `row_categories` |
| `…/screens/hero/fixture.rs` | `HierarchyEntity` + `role`, `components`, `content` (9 literais actualizados) |
| `…/screens/hero/mode_drive.rs` | 2 métodos novos com default no `ModeFamily`; o `drive` atende `OpenContent` e publica o conteúdo |
| `…/screens/hero/state.rs` | `GizmoStateGroup.content` + `content_of` |
| `…/object_mode.rs` | variante `ModeRequest::OpenContent { entity, mode, key }` (+ braço no `resolve`) |
| `…/action_bus_hier.rs` | variante `HierRequest::OpenContent { owner, mode, key }` |
| `…/interaction/state/hier_rows_ops.rs` (**novo**) + 1 campo `hier_rows` no `WidgetStore` | grupos abertos + `InspectorReveal` |
| `…/screens/hero/panel_ids.rs` | `PANEL_PHYSICS = "physics"` (+ no `canonical_panel_id`) |
| `…/icons.rs` + `docs/design/icons/film.svg` | `IconId::Film` (Lucide `film`), entre `File` e `FitView` |
| `ph2d-i18n/src/hier_rows.rs` (**novo**) | `hier_rows.scene` · `hier_rows.content.layers` · `hier_rows.content_gone` |
| `ph2d-tool-painter/src/tool/outline.rs` (**novo**) + 1 campo `pending_layer` | `layer_outline_of`, `layered_entities`, `open_layer_of` |

## 4. Ids / variantes / consts novos (colisão!)

- `IconId::Film` — ⚠️ **uma variante no MEIO da enum alfabética**: duas linhas que acrescentem
  ícones colidem no texto do enum e do array `ALL`; o gate `enum_order_matches_svgs` decide.
- `HIER_SCENE_ROW = hash_node_id("hier_scene_row") & ((1<<58)−1)`; `hier_content_row`,
  `hier_component_icon` (SplitMix64 nos 58 bits de baixo — os bits 58–62 são dos companheiros).
- `HierRequest::OpenContent`, `ModeRequest::OpenContent` — `match` exaustivo noutra linha sobre
  qualquer dos dois reprova a compilação da árvore combinada (aqui só havia o `resolve`).

## 5. Contratos congelados

Nenhum tocado (`collision-surface.sh`: `node.rs` e `tool.rs` intocados). `PROJECT_SCHEMA` **não sobe**
(187 = base): nada novo se grava.

## 6. Portão de fecho (batched, sobre `2a925859d..HEAD`)

- 1.ª corrida: `nextest-impacted` **16 762/16 770** — as 8 curadas em `92d5ad232` (ciclo
  `object_mode ↔ hier_rows`; `apply_event` em 203/200; fixtura armada com uma «Scene Root» que a raiz
  nova duplicava; o texto «Follows the prefab» num teste da shell). `check --workspace --all-targets`
  com `-D warnings` ✓ · `clippy --all-targets -D warnings` ✓ (0 lints).
- 2.ª corrida, depois das curas: ver §11.
- Testes novos: `the_new_rows_of_the_hierarchy` (4, CLIQUES REAIS pelo `click_at`: ícone → selecção +
  pedido; ícone da Cena → painel `physics`; conteúdo recolhido → peça → `OpenContent`; a Cena limpa
  e não é linha de objecto) · `o_icone_da_hierarquia_traz_o_cartao` (2: rola e abre; outro objecto
  espera, família sem cartão morre sem rolar) · `rows`/`reveal`/`hier_rows` unitários.
- Medido com foto (`fotografa_cena.sh`): `PH2D_VIDA_SMOKE=1` (a Cena com ⬢🎞; ⚡ nas fábricas; ⬢⚡ no
  herói; o «Ground» sem física não tem ícone) e `PH2D_COMPOSITE_SMOKE=1` (`Composite ▸ Layers (1)`).

## 7. O que só o `ship.sh` / a árvore combinada pegam

- `the_shell_only_shrinks` soma entre linhas: esta põe **+35**.
- O `Cargo.lock` ganhou a aresta interna `ph2d-panel-hierarchy → ph2d-component-desc` (regenerado
  pelo cargo, nenhum pacote externo).
- Outra linha que construa `HierarchyEntity { … }` à mão reprova por campo em falta — a cura é
  acrescentar `role: Default::default(), components: Vec::new(), content: Vec::new()`.

## 8. Premissas do briefing que a medição derrubou

1. «O Mixer (`SceneAudio`) passa a componente da Cena» — **não há mixer**: o `SceneAudio` é o livro
   das vozes vivas, não é dado do artista.
2. «`PROJECT_SCHEMA` sobe» — não sobe; a Cena é linha e o mundo físico continua em `ProjectFile.physics`.
3. «A Cena é um objecto» — como ENTIDADE ela poria a gravidade no undo do canvas (o
   `PhysicsSettings` fica fora dele de propósito, `project.rs:52`) ⇒ linha sintética.
4. «Ponto de extensão como o `ObjectTypeDecl`» — ele não existe no código; o `ModeFamily` já é a
   lista composta das famílias com modo.
5. «Ossos como sub-linhas» — já são (entidades filhas). «Quadros do Flip» → camadas.

## 9. Smokes (binário `smoke` compilado na worktree)

```bash
cd ~/Documentos/Projetos/PH2D/Worktrees/line-ObjectsAndComponents
PH2D_VIDA_SMOKE=1 ./target/smoke/ph2d-host-desktop       # a Cena no topo; ícones nas linhas; clique num ícone → o cartão
PH2D_COMPOSITE_SMOKE=1 ./target/smoke/ph2d-host-desktop  # Composite ▸ Layers (1); abrir e clicar na camada → Paint
```

## 10. O que fica ABERTO

- **Onda 2 = F4** (Animação por objecto: componente + a Timeline segue a selecção; a global vira a
  Animação da Cena) — a maior; documento por `StableId`. Depois F5 (Motion objecto) + F6 (busca por
  intenção + filtro).
- Duas premissas ficam para o dono quando ele smokar: o custo de um nível de recuo (12 px) em todas
  as linhas; e o ícone de cada família (o de `Physics` é o do painel).
- O pedido ao Inspector casa pelo NOME do objecto: duas linhas com o mesmo nome e uma delas já
  seleccionada fazem o cartão rolar no quadro antes da troca (inofensivo — a rolagem fica).
- Uma camada pedida a uma imagem cujo Paint foi RECUSADO fica pendente no Painter até à próxima
  ligação dessa imagem (`pending_layer`); a seguinte substitui-a.

## 11. Fecho

- 2.ª corrida do portão, sobre o diff inteiro depois das curas: `nextest-impacted` **16 770/16 770** ·
  `clippy --workspace --all-targets -D warnings` ✓ · `fmt --check` ✓ · `cargo machete` ✓ ·
  `check-standalone-optional.sh` ✓ · `check-workflow-packages.sh` ✓. Os censos de rótulo
  (`nenhum_rotulo_do_app_pinta_nada`, `onde_comeca_o_valor`) só valem com `--workspace` (as features
  unificadas): 25/25 assim.
- `rm -rf target/*/incremental` feito; o build `smoke`, 2.ª corrida:

```text
    Finished `smoke` profile [optimized] target(s) in 0.20s
```

- `bash scripts/agent-loop-profile.sh` (20 sessões):

```text
  ✗ paralelismo de ferramenta              1.14/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                232   alvo: <= 800
  ✗ cargo test : cargo check                450 : 242   alvo: <= 1,0  razao 1.9x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%
  ✗ contexto relido por passo (media)         487 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               62 mil   alvo: <= 80 mil
```
