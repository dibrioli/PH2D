# HANDOFF DE INTEGRAÇÃO — `line/components` (2026-09-02)

> **Entregável de fecho** (DIRETRIZ §1.5.9). ⛔ A linha **não integra e não faz ship** — ela fecha,
> entrega isto e espera ordem explícita do Enio (`CLAUDE.md` §0.7).

---

## 1. Identidade

| | |
|---|---|
| branch | `line/components` |
| HEAD | `b17c2ae1a` |
| merge-base com `main` | `066b4f92e` |
| commits | **77** |
| ficheiros | **290** (`+22 364` / `−4 758`) |

⚠️ **A linha está ABERTA e continua** — o Enio pediu este handoff **antes** da próxima wave
(*Aplicar ao mestre interno*, o modelo da Unity). ⇒ *este documento descreve o estado de hoje, e o
HEAD muda*. O integrador re-lê o HEAD e **re-roda a `collision-surface.sh`** (§1.5.3), sempre.

---

## 2. Foundational / partilhado tocado, e porquê

| onde | o quê | aditivo? |
|---|---|---|
| **`crates/ph2d-asset-index/`** (crate NOVA) | o índice de assets — ADR-0165, F6 | ✅ folha nova |
| **`crates/ph2d-panel-asset-browser/`** (crate NOVA) | o painel — F7 | ✅ folha nova |
| `ph2d-editor-core/src/ids/` | **31 `const` novos** (ver §3) | ✅ aditivo |
| `ph2d-editor-core/src/action_bus*.rs` | ⛔ **NÃO aditivo — ver §3.1** | ⛔ |
| `ph2d-editor-core/src/widget/scrollbar*.rs` | os ids saíram para `scrollbar_ids.rs` (tecto de LOC) | ⚠️ move |
| `ph2d-editor-core/src/screens/hero/` | `canvas_backdrop` (porta nova) · `variant_axes` · `menu_rows` · `pre_populate` | ✅ aditivo |
| `ph2d-render/` | `band_blit.rs` · `world_rt.rs` · shader novo — ADR-0154 Fase 2 (z-order) | ✅ aditivo |
| `ph2d-vec-scene` · `ph2d-ecs` | **só benches novos** (`measure_scene_clone` · `measure_restore`) | ✅ aditivo |
| `shells/desktop/` | o grosso da linha; `Cargo.toml` ganha `serde` com a feature **`rc`** | ⚠️ |

⛔ **Contratos congelados (§6): INTOCADOS** — a sonda confirma (`node.rs`, `tool.rs`).
⛔ **ADR: esta linha não cria nenhum** ⇒ fora de toda disputa de número.

---

## 3. Símbolos que podem COLIDIR — a saída da sonda, colada

⚠️ **Referência, nunca evidência** (§1.5.9 item 3): a tabela mede a linha contra o `main` **de
hoje**. Se outra linha fundir no meio, todo número da coluna «base» muda e **este documento não
reclama**. O integrador re-roda.

```text
SUPERFÍCIE DE COLISÃO — line/components contra main
  merge-base 066b4f92e   ·   77 commit(s)   ·   207 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        105   (base: 103)
  ⚠   └ tripla do gate               (105, 13, 17)   (base: (103, 13, 17))
    VEC_SCENE_SCHEMA                       17   (base: 17)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              —   (base: —)
    ph2d-render (espelho)                  79   (base: 79)
    ph2d-script (espelho)                  79   (base: 79)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0168   próximo livre: 0169
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 2 pacote(s) '+name' novo(s):
      "ph2d-asset-index"
      "ph2d-panel-asset-browser"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
      491 / 700   crates/ph2d-editor-core/src/action_bus.rs  (tem marcador/allowlist — confira o valor congelado)
     1826 / 600   shells/desktop/src/app_state.rs  (tem marcador/allowlist — confira o valor congelado)
     6695 / 600   shells/desktop/src/input_dispatch.rs  (tem marcador/allowlist — confira o valor congelado)
     1449 / 600   shells/desktop/src/main.rs  (tem marcador/allowlist — confira o valor congelado)
    12031 / 600   shells/desktop/src/render_loop/mod.rs  (tem marcador/allowlist — confira o valor congelado)
     1130 / 600   shells/desktop/src/render_loop/sim_extract.rs  (tem marcador/allowlist — confira o valor congelado)
     1350 / 600   shells/desktop/src/render_loop/snapshots.rs  (tem marcador/allowlist — confira o valor congelado)
    nenhum arquivo da linha passa do teto
───────────────────────────────────────────────────────────────────────────────
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.
```

### 3.1 ⛔⛔ O que a sonda NÃO vê, e é o maior risco desta linha

**`EditorAction` foi PARTIDO.** As 33 variantes da Hierarquia saíram para um enum próprio
(`action_bus_hier.rs::HierRequest`) e o barramento passou a ter **uma** variante
`EditorAction::Hierarchy(HierRequest)` — **92 sítios de chamada reescritos**.

⚠️ **Qualquer linha que tenha acrescentado uma variante a `EditorAction` conflita textualmente**, e
o conflito é **fácil de resolver mal**: a variante nova pertence ao enum de fora (é do barramento) ou
ao `HierRequest` (é da Hierarquia)? *A pergunta é o SUJEITO dela* — se carrega uma `row` da
Hierarquia, vai para dentro; senão fica fora.
⚠️ **A razão do corte foi MEDIDA:** o `action_bus.rs` estava no tecto de LOC, e acrescentar **uma**
variante custava **+78 linhas** (o `rustfmt` re-flui as 37 variantes-struct). O ficheiro caiu de
676 para 491.

**Variantes NOVAS no `EditorAction`** (fora da Hierarquia): `OpenAssetBrowser` · `AssetCardVerb {…}`
· `AssetInstantiate {…}` · `InspectorSwapVariant {…}` · `InspectorClearUnusedOverrides {…}` ·
`InspectorAddComponentRequested {…}` · `CatalogVerb {…}` · `SetPresentMode {…}` · `Reimport {…}`.

### 3.2 Os 31 `NodeId`/`const` novos

`ASSET_PANEL` · `ASSET_DRAG_HANDLE` · `ASSET_RESIZE_HANDLE_BL` · `ASSET_CLOSE` · `ASSET_SEARCH` ·
`ASSET_RELATED_CLEAR` · `ASSET_SIZE` · `ASSET_SORT_MODES` · `ASSET_SORT` · `ASSET_KIND_FILTERS` ·
`ASSET_KIND` · `MAX_ASSET_CELLS` · `ASSET_CATALOG_TOGGLE` · `ASSET_CATALOG_NEW` ·
`ASSET_CATALOG_ALL` · `ASSET_CATALOG_UNASSIGNED` · `ASSET_CATALOG_COL` · `ASSET_CATALOG_RENAME` ·
`MAX_CATALOG_ROWS` · `INSP_RENDER_TEXTURE_SLOT` · `MAX_INSTANCE_AXES` · `MAX_INSTANCE_AXIS_VALUES` ·
`INSP_INSTANCE_AXIS_OPTION` · `CTX_MENU_ASSET_INSTANTIATE` · `CTX_MENU_ASSET_SELECT_USERS` ·
`CTX_MENU_ASSET_USES` · `CTX_MENU_ASSET_USED_BY` · `CTX_MENU_ASSET_REMOVE` ·
`CTX_MENU_HIER_REMOVE_FROM_LIBRARY` · `CTX_MENU_CATALOG_RENAME` · `CTX_MENU_CATALOG_DELETE`.

⚠️ Todos são `hash_node_id("…")` de uma string própria ⇒ **colisão de VALOR é improvável**; o que
colide é a **linha do ficheiro** e a lista do `pre_populate` (11 linhas novas lá).

### 3.3 ⚠️⚠️ `PROJECT_SCHEMA` — **DOIS degraus**, `103 → 105`

É o número que **soma entre linhas e nunca se escolhe** (`CLAUDE.md` §5.0). Os dois degraus são:
**104** (`StableId`/`SiblingOrder`/snapshot v2 — a 1.ª migração do repo) e **105** (a taxonomia de
catálogos). ⚠️ A **tripla do gate** acompanha: `(105, 13, 17)`.
⛔ **Se outra linha também subiu o schema, o valor certo não está em nenhum dos dois lados** — conta-se,
e os degraus da escada re-numeram-se **nos TRÊS sítios**.

---

## 4. Contratos congelados encostados

**NENHUM.** Confirmado pela sonda.

---

## 5. O que só o `ship.sh` apanha (o gate de integração não roda)

- **`serde` com a feature `rc`** no `shells/desktop/Cargo.toml` — dependência **já existente** com
  feature nova ⇒ o `machete` não reclama, mas o `deny`/`audit` vê o grafo mudado.
- **Duas crates novas** no `Cargo.lock` (`ph2d-asset-index`, `ph2d-panel-asset-browser`).
- `fmt`/`typos` de ficheiros pré-fork que a linha tocou de raspão.

---

## 6. Ordem, dependências e o que FALTA smokar

**Ordem:** os 77 commits são lineares e cada um compila; não há dependência fora da ordem do log.

**Smokado pelo Enio, com veredito ✅:** o navegador (abrir, buscar, ordenar, tamanho) · catálogos
(criar, renomear, apagar, arrastar para dentro) · arrastar para a tela e para a ranhura de textura ·
o retrato de um Prefab (4 rondas de report até ficar fiel) · o fundo do cartão a seguir o canvas ·
as duas perguntas de relação (*o que usa* / *o que é usado*).

⏳ **NÃO smokado** — o integrador tem de o pôr na lista do Enio:
1. **O undo com a cena partilhada** (`Arc<VecScene>`, commit `d03fa6903`) — desfazer e refazer numa
   sessão com desenho vetorial. ⚠️ É a mudança mais recente e a que toca o **formato do projeto**
   (há gate a provar que os bytes não mudam, mas o gate não abre um `.ph2dproj` antigo).
2. **Abrir um `.ph2dproj` gravado ANTES de 24/08** — tem de dizer *«Project migrated from format 95
   to 105»*. ⚠️ Um **v97/v98 é RECUSADO** de propósito.
3. O `physics_ecs_c9` na matriz 3-OS — **só o CI o mede** (a lane de mestre+instância entrou na F4.7).

---

## 7. Higiene de fecho

- [x] **Gate batched** sobre o diff acumulado: `nextest` da workspace **13 033/13 033** verdes na
      última corrida limpa; clippy `--all-targets` limpo nas crates tocadas.
      ⚠️ Corridas posteriores tiveram vermelhos que são **membros nomeados da família de flakes de
      carga** do §5.0 (`a_round_live_offset_costs_like_the_other_joins` ·
      `a_wet_move_costs_what_the_footprint_costs…` · a família `flip_smooth::…::orcamento` ·
      `the_cost_of_depth_is_linear_not_explosive` · `only_the_lower_row_breathes…`) — **todas
      re-corridas sozinhas e verdes**, com a máquina a `load 25`–`82`.
- [ ] `rm -rf target/*/incremental` — ⏳ **não feito, e é deliberado**: a linha **continua aberta**
      (a wave do *Aplicar ao mestre interno* começa a seguir), e o `incremental/` é o que faz o
      `cargo check -p` voar durante a jornada. *Reclamar no FIM, nunca desligar no COMEÇO.*

---

## 8. A narrativa vai AQUI; o `CLAUDE.md` §5 recebe UMA linha

O que esta linha entregou, por assunto, está nos handoffs e planos já escritos:

- [`07_plano_do_navegador_de_assets.md`](../07_plano_do_navegador_de_assets.md) — o navegador
  (etapas A–D), **§12** o fundo do cartão, **§13** as duas perguntas de relação, **§14** as quatro
  perguntas sobre um controlo.
- [`05_plano_de_implementacao.md`](../05_plano_de_implementacao.md) — o placar por fase, **§F8.0** o
  estudo que refutou metade daquela fase, **§F5.4** a investigação parada com a receita da fixtura e
  o prior art (Unity tem, Godot não).
- [`06_plano_variacoes_sem_chaves.md`](../06_plano_variacoes_sem_chaves.md) — ⏸️ **ADIADO por ordem
  do Enio**, com as **duas recusas medidas**. ⛔ O código saiu inteiro do fonte; uma 3.ª tentativa
  começa perguntando ao dono *o que ficou pior*.
- [`22_auditoria_das_instancias_2026-08-27.md`](../22_auditoria_das_instancias_2026-08-27.md).

### ⚠️ As leis que esta linha pagou (e que valem fora dela)

1. **Uma porta que o vizinho não chama ainda não é uma porta.** A mesma pergunta de duas formas
   («que textura usa esta peça?») foi respondida com a metade errada **quatro vezes**, a última a
   **quinze linhas** da porta que a responde certo. A cura foi o **censo**
   (`the_index_asks_the_texture_door`), não a linha.
2. **VIVO, ALCANÇÁVEL e NO SÍTIO CERTO são três perguntas** — este repo tinha instrumento para uma e
   meia. O `every_asset_browser_control_answers` cobre as quatro.
3. **Um censo que presume o DESTINO de um efeito acusa o vivo** — e a mensagem dele manda construir
   a doença que ele existe para prevenir.
4. **O tamanho serializado é cego à partilha por construção.** Mediu-se `189 MB` de residência com a
   régua que o doc do próprio tipo diz não ver `Arc`.
5. **Um `clamp` antes de um teste de intervalo apaga o teste.**
