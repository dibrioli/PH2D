# HANDOFF de integração — `line/GameEngineFuncional02`, 08/11 (física em cartões: Physics Body, Static Body, Collision Shape, Collision Layers)

Leitor: o agente integrador. Branch `line/GameEngineFuncional02`, HEAD `04a158179c` (antes deste commit de docs), **104 commits** sobre a base `652b506b58` = merge-base = ponta do `main` (`git rev-parse main` = `652b506b58`, conferido 08/11: o `main` NÃO andou; `--ff-only` serve sem rebase). `git diff --shortstat 652b506b58..HEAD`: 372 files, +17106 −1885. Nada integrado, nenhum push. **Suíte completa e `nextest-impacted` NÃO correram** (regra do dono, 23/10: só o integrador). Mecanismo de cada onda: [continuação 07/11](HANDOFF_CONTINUACAO_2026-11-07_FISICA.md) (ondas 4 a 9g) e a ficha [`f1_physics_static_shape.md`](../componentes/f1_physics_static_shape.md) (decisões do dono, ondas 6–9g, «Fecho da linha (07–08/11)»). Aqui só o que a integração precisa.

⚠️ **Linha irmã `line/GameEngineFuncional`** (Sprite + Animated Sprite, [handoff 07/11](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-07_ANIMATED_SPRITE.md)) também está fora do `main` e parte da mesma base. Ordem e partilha: §8.

## 1. O que entra

Ondas 4–9g e o fecho 07–08/11: cartões Physics Body / Static Body / Collision Shape (forma com alças no canvas, Esc a meio com Auto Key); cartão **Collision Layers**; **32 camadas com nome** (schema 218/219, §3); **aba central «Collision Layers»** (`ph2d-panel-physics::aba`, um `DocumentSurface`: matriz, camada aberta, objetos, Rename, «+ Layer», «Remove» com destino); canvas com contorno na cor da camada, tipo do corpo pelo traço e, no fecho, **sombra escura sob os contornos** (`tinta_canvas::sombra_do_contorno`; decisão do dono 07/11, com a regra «não perder poder de configuração»). Fecho: destino do Remove em lista suspensa (`PHYSICS_LAYER_MOVE_PICK`, ✓ na escolhida, «Move them to» ≥ 4,5:1), duplo clique no Layer conduzido no app, nome da camada no Ray Sensor (clássica: ao lado do número ou linha própria; cartão: «Layer · nome»), face clássica `PH2D_CARDS=0` com a lista suspensa.

## 2. Superfície de colisão (foundational) — por que é aditiva

Saída colada de `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02/scripts/collision-surface.sh` (08/11), resumida: `PROJECT_SCHEMA 219 (base 217)`, tripla `(219, 13, 29)` (base `(217, 13, 29)`); `VEC_SCENE_SCHEMA 29`, `FLIP_SCHEMA 13`, `DOC_VERSION 25` iguais à base; `ph2d-ecs` 120 = base 120 (**DELTA 0** no registo; as sondas dos espelhos `ph2d-render`/`ph2d-script` acusam «cega» no script, que não acha a const; não é desta linha); `node.rs` e `tool.rs` intocados; nenhum ADR criado (próximo livre 0199); nenhum marcador de conflito; `Cargo.lock`: 1 pacote novo, `ph2d-shape-handles`. Tecto de LOC: `shells/desktop/src/main.rs` 1023/600 — **pré-existente, a linha tocou 1 linha** (isenção da shell).

| Onde | O que | Aditivo porque |
|---|---|---|
| **`ph2d-editor-core`** (22 ficheiros, +757 −111) | (1) `interaction/dispatch/focus.rs`: o braço `Dropdown` de `apply_click` passa a **EMITIR `WidgetEvent::DoubleClick(id)`** no 2.º clique. (2) `dispatch::focus_text_field_selected`: porta nova, foco com tudo selecionado (renomear). (3) `property_row::paint_fields_row_com_fim` (porta nova; `paint_fields_row_inner` ganhou `fim_w`, **0 nos chamadores antigos**). (4) gate `architecture_every_editable_field_is_a_channel`: sai a isenção `POR_FUNCAO` `paint_collision_rows` (os laços deixaram de existir), entra a classe `DIALOGO_SEM_COLUNA` (o destino do Remove) com censo de obsolescência. `action_bus*.rs` (variantes `LayerRename`/`LayerShow`/`LayerPairToggle`/`LayerMatrixOpen`/`LayerRemove`), `dispatch/key.rs` (`esc_fecha`), `scroll.rs`, `roda_lateral.rs`, `nome_curto.rs`, `paint_contorno_camada.rs`, `cards_linhas_triangulo.rs`, `widget/modal.rs` (`sem_rodape`) | (1) **muda comportamento**: antes nenhum evento, agora um `DoubleClick`. Varredura por CONSUMIDOR (auditoria): nenhum painel reage mal (os que convertem duplo em clique não tratam ids de listas; os outros decodificam por tabela própria). (2)–(4): funções/variantes novas |
| `ph2d-physics-ecs` (+1031) · `ph2d-physics` (`world/layers.rs`) | `layer_names.rs` (nomes, mostradas, blob), `settings.rs`: `PhysicsSettings.layer_matrix` `[u8; 8]` → `[u32; 32]` **no meio do fio**; `player_input.rs`; `body_law.rs` | leitura dos `<= 218` por `LayerRowsV218` (§3) |
| `ph2d-panel-physics` (22) · `ph2d-app-physics` (23) · `ph2d-panel-inspector` (16) · `ph2d-shape-handles` (**crate NOVA**, 6) · `ph2d-physics-scene` (5) · `ph2d-panel-registry-init` | a família da física; `ph2d-physics-scene/src/ficha.rs` = corpos da cena 120 (porque `ph2d-app-physics` não tinha folga) | consumidores / folha nova |
| `ph2d-i18n` (5) · `ph2d-tokens` (7) · `docs/design/tokens.json` | chaves `panel.inspector.ray.layer_named`, `card.glyph.l` e o resto de `cartoes.rs` (+135); `camadas_de_colisao.rs`, `skin_body.rs` (paleta das camadas) | chaves/tokens novos: **colidem só se outra linha usar o mesmo nome** (§3) |
| `ph2d-tool-registry` · `ph2d-timeline` (`TimelineHistory::desiste`, +6) · `ph2d-ecs` (`canal_mundo.rs`) · `ph2d-app-game`/`-components`/`-sprite-anim` | tabelas de ids, método novo, folgas de catraca | linhas de tabela / método novo |
| `shells/desktop` (38) · `shells/game-scenes` (2) | `init.rs` (instala `AbaDasCamadas`), schema, `physics/*` e `bone_undo_probe.rs` (corte de comentário), cena 120; `plataforma_alargada.rs` (gate novo) | composição |

`Cargo.lock` +26: o pacote `ph2d-shape-handles` e arestas internas. Conflito de lock = `cargo metadata`, nunca à mão. `git diff --stat 652b506b58..HEAD -- scripts .config .github .claude` → vazio (08/11).

## 3. Contratos e schema (corrido em 08/11)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02
git diff --stat 652b506b58..HEAD -- crates/ph2d-editor-core/src/tool.rs crates/ph2d-nodegraph/src crates/ph2d-imageio/src/color.rs crates/ph2d-vector-doc/src crates/ph2d-vector-traits/src   → (vazio)
```

**§6 intocado** (NodeOp/OpResolver/NodeManifest, Tool/RasterEditTool/CanvasPaintTool/PanelEvent, Vector data-model, imageio). `ph2d-tool-registry` mudou (`lib.rs`, `node_id.rs`), mas não é a superfície gateada; `architecture_tool_contract_surface` verde.

**`PROJECT_SCHEMA` 217 → 219: DOIS degraus, ambos desta linha** (`shells/desktop/src/project_schema.rs`, commits `e0ccb00302` · `eb5cd37f7b` · `9644b6908f`):
- **218** (onda 2): os nomes das camadas, blob novo `PH2DLAYN` no fim do `ProjectFile`. Const `PROJECT_SCHEMA_SEM_NOMES_DE_CAMADA = 217`.
- **219** (onda 5): `layer_matrix` de `[u8; 8]` a `[u32; 32]` no meio do fio; `LayerNames` v2 (32 nomes + mostradas). Const `PROJECT_SCHEMA_MATRIZ_DE_OITO = 218`. Os `<= 218` leem-se sob `LayerRowsV218` (`LayerMatrix::rows_from_legacy`) e mostram as oito. Gate `project_layer_names_tests`.
- ⚠️ A [continuação 07/11](HANDOFF_CONTINUACAO_2026-11-07_FISICA.md) diz «218»: é anterior à onda 5. **O valor no código é 219.**
- **O integrador RECONTA**: o degrau é DELTA **+2** sobre o `main` do dia (aqui `main` = 217). Se o `main` andar, `python3 scripts/schema-recount.py <valor que o degrau passa a ter no main>` e as constantes `..._SEM_NOMES_DE_CAMADA` / `..._MATRIZ_DE_OITO` seguem. A escada e a tripla moram em ficheiros irmãos (`project_schema*.rs`, `project_schema_tests.rs`, três sítios). Os degraus do 3D antigos foram arquivados em `project_schema_history_v99.rs` (−183).

**Símbolos que podem colidir (grep na integração):** ids `physics.layers.move_pick` (`PHYSICS_LAYER_MOVE_PICK`), `physics.matrix.open`; variantes `EditorAction::LayerRename|LayerShow|LayerPairToggle|LayerMatrixOpen|LayerRemove`; chaves i18n `panel.inspector.ray.layer_named` = `"Layer · {name}"`, `card.glyph.l` = `"L"`; cenas de smoke `PH2D_PHYSICS_SMOKE=120` e `PH2D_PHYSICS_SMOKE_ABA`/`_SELECT`/`_CLIQUE` (pré-existentes na linha) (passos novos `layer`, `duplo:layer`, `escreve:<t>`, `ray`, `fisica`); tokens em `ph2d-tokens/src/camadas_de_colisao.rs`.

## 4. Catracas e pisos MOVIDOS com medição (o integrador RECONTA, soma entre linhas)

Causa comum: a matriz saiu do painel para a aba (ondas 8/9a).

| Catraca | De → para | Medido | Nota |
|---|---|---|---|
| `POPULACAO_RITMO` physics | 144/117/117 → **116/51/65** | 122/54/68 (base 152/124/124) | só encolhe |
| `COLUNA_UNICA_PISO` | 3 → **2** | — | o 3.º era o physics (8 linhas da matriz em x=36 no merge-base) |
| `ALTURA_DE_ABERTURA` physics | 1064 → **1037** | 1037 | catraca que só encolhe |
| shell (`the_shell_only_shrinks`) | **189 041 de 189 041, folga 0** | — | paga por corte de comentário; a próxima costura MOVE |
| `ph2d-app-physics` | 55 494 de 55 515 | — | folga 21 |

Se a linha irmã também gastou folga da shell, **a soma estoura**: cura = MOVER, nunca subir.

## 5. Medido e provado (gate do fecho, da linha, 07–08/11)

- fmt ✓; `clippy -D warnings --all-targets` ✓ nas 19 crates tocadas.
- Crates inteiras verdes (físicas, inspector, registry-init, physics-ecs, app-physics lib 500, shape-handles, physics-scene, editor-core `interaction::dispatch` 189) e os gates por nome (shell_only_shrinks, workspace_file_loc_cap, no_magic_numeric, wiring parity, canais, panel_loc_cap, no_dependency_climbs, row_pitch, tool_contract_surface, arch_safe_clamp_only, dropdown door ×2, claude_md budget; shell `file_loc_caps`/`project_schema`/`layer_names`/`esc_a_meio`; game-scenes `a_plataforma_alargada`): verdes. Contagens por crate: ficha, «Fecho da linha».
- **Flake de carga:** `ph2d-physics-ecs measure_player_budget::the_cost_of_a_player_is_linear_in_their_number` (13,14× vs ~7× a load 10–12; sozinho 3× verde). [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md).
- **Mutações do fecho (todas sangraram):** dropdown sem `push` do DoubleClick · `focus_text_field_selected` sem seleção · guarda `cards::active()` do duplo clique clássico · `REMOVENDO.set(None)` em `abre`.
- **Conduzido no app** (ponteiro real, `PH2D_PHYSICS_SMOKE_CLIQUE`): Crate, janela 2700, `duplo:layer,escreve:Boxes,desfaz` → `[fila-mao]` nomes Props → Boxes → Props (um passo).
- Fotos: [`fotos_fisica/fecho/`](fotos_fisica/fecho/) (canvas_sombra_zoom, aba_remove_*, aba_barra_horizontal, insp_duplo_*, classica_layer, ray_layer_nome*).

### 5.1 Emenda depois do smoke (dono 08/11)

*«Duplo clique em layer vamos retirar pois já temos o botão.»* · *«Vamos passar os botões Rename layer e Open in tab para acima de collides with e abaixo de layers.»*

- **O que mudou:** sai o braço `DoubleClick(INSP_PHYS_LAYER_PICK)` de `ph2d-panel-inspector/src/event_physics.rs` (renomear só por «Rename layers…», que abre com `focus_text_field_selected`); `sections/physics_colide.rs::camada` pinta Layer → [«Rename layers…» · «Open in tab…»] → «Collides with» → frase. Forma solta: Layer + «Rename layers…» no mesmo sítio, sem fila. Ids, edições, undo, Auto Key, resumo dobrado: iguais. `editor-core` intocado (o `DoubleClick` do `Dropdown` fica, sem consumidor no chip); duplo clique na aba central intocado. A mutação «guarda `cards::active()`» acima deixa de existir (o braço saiu).
- **Testes:** novo `o_duplo_clique_no_chip_da_layer_nao_renomeia_so_o_botao` (substitui os dois do duplo clique); `os_quadros_quebram_pelo_nome_sem_se_tocar` e `a_forma_solta_nao_mostra_os_quadros_mas_mantem_a_camada_e_o_rename` provam a ordem nova. Mutação: devolver o braço antigo → vermelho; restaurado (`git diff` conferido). Por nome verdes: inspector `o_cartao_das_camadas` 11, `os_quadros_da_colisao` 9, `o_cartao_da_fisica` 19, `seam_physics` 43, `a_camada_pelo_nome` 2, `o_modo_rename` 1, `every_word` 5; registry-init `quantas_entradas_tem_cada_painel` 15, `lei_da_compactacao` 9, `onde_comeca_o_valor` 1, `nenhum_rotulo_do_app_pinta_nada` 13, `nenhum_nome_por_cima_do_controlo` 2, `desfazer_nao_pula` 11; editor-core canais 6, wiring parity 2, `no_magic_numeric` 3, `workspace_file_loc_cap` 2, `architecture_panel_loc_cap` 4. Fecho `cargo test -p ph2d-panel-inspector -p ph2d-panel-registry-init --profile smoke`: **1148 passaram, 0 falharam, 40 ignorados**; fmt ✓; clippy `-D warnings --all-targets` ✓. Nenhuma catraca mudou.
- **Foto:** [`insp_botoes_acima_recorte`](fotos_fisica/fecho/insp_botoes_acima_recorte.png) (inteira: `insp_botoes_acima.png`; Crate, janela 1930 × 2700). `insp_duplo_*` ficam como registo do que o dono recusou.

### 5.2 Emendas 2 e 3 depois do smoke (dono 08/11)

*«a aparência aqui poderia ser melhor»* · *«use os retângulos coloridos ao lado do nome no dropdown»* · *«Ray Sensor não deveria ter os mesmos campos????»*

- **Remove (aba):** uma linha só (rótulo, chip, botões `FIELD_H`), opções com o quadradinho da camada (a função da lista «Layers»), sem contorno por linha. Só `ph2d-panel-physics/src/aba/`.
- **Ray Sensor:** Layer = a lista suspensa do collider (o chip É `INSP_RAY_LAYER`: mesmo canal do losango) + «Sees» (linha da matriz do mundo). Ids novos `INSP_RAY_LAYER_OPTS`, `INSP_RAY_SEES`, `INSP_RAY_SEES_ADD`; i18n `panel.inspector.ray.sees_*`; sai `layer_named`/`card.glyph.l`. `marcas_mundo` (app-physics): os quadros «Sees» partilham o losango da matriz.
- **Foundational:** APAGADA a porta `paint_fields_row_com_fim` do `ph2d-editor-core` (`property_row.rs` volta ao byte de antes de `03d9ea5f17`; `property_row/casas.rs` sem o `fim_w`); o gate `architecture_every_editable_field_is_a_channel` ganhou as isenções `fila_da_matriz`/`chip` (nenhuma catraca de dívida subiu). Nenhuma catraca movida.
- Detalhe, testes, mutações e fotos: ficha, «Fecho da linha» → «Emendas 2 e 3».

## 6. Auditoria (duas lentes) do fecho: achado · cura

| Achado | Cura |
|---|---|
| duplo clique na face clássica abria renomear invisível | só no cartão (guarda `cards::active()`) |
| a face clássica guardou o alcance 0–7 dos chips antigos | alcança 0–7 sempre e as visíveis 8–31 |
| o Remove podia remover a camada para ela mesma ao abrir a aba noutra camada | a pergunta cai em `abre`/`fecha`; «Move and remove» recusa a 0 e a própria |
| sem marca na escolhida | ✓ na linha escolhida |
| renomear somava ao nome («PropsBoxes») | `focus_text_field_selected` |


## 7. NÃO provado / ABERTO

**Não provado:** o rótulo «Layer · nome» do Ray no CARTÃO só por foto (o arnês não abre o corpo desse cartão) · cursor no app · teclas ←/→/↑ e Play na cena 120 · o resto: continuação 07/11 («Não provado», fim da 9b) · `suite-completa` e `nextest-impacted` (integrador).

**Aberto, para DEPOIS da integração:**
1. **Polígono pelo contorno da imagem** — decisão do dono (07/11): item novo, com proposta; mexe no motor e no formato salvo.
2. Das ondas 8 e 9e–9g: faixa da linha pairada sobre as células, linhas da lista sem vão, a referência é a maquete v2 e não o editor atual da irmã (continuação, «NÃO feito / aberto»).
3. Smoke do dono: onda 5, 6, 7 e este fecho (§9). **Passo 10 do smoke de 07/11** (rótulo, decisão dele) por repetir.

## 8. Riscos de integração

- **Ordem com `line/GameEngineFuncional` (irmã).** Partilha: (a) `CLAUDE.md` §5.1, entrada «Game Engine» (mesma linha; os dois links ficam); (b) catraca da shell e `Cargo.lock`; (c) folgas de `ph2d-app-game`/`-sprite-anim`/`-components`; (d) `ph2d-editor-core` (ela `doc_surface.rs`/`surface_input.rs`, esta `dispatch/*`, `modal.rs`); (e) a irmã NÃO mexeu em `project_schema*.rs`. Registo de componentes: ela +2, esta 0. Sem código partilhado entre o editor de animações dela e a nossa aba: «Para o integrador» das ondas 8 e 9a na [continuação](HANDOFF_CONTINUACAO_2026-11-07_FISICA.md).
- **Primário:** `git -C /home/enio/Documentos/Projetos/PH2D status --short` ANTES; nunca descartar `M`/`??` alheio.
- **`CLAUDE.md` §5:** o §5 inteiro tem tecto; reconte após fundir as duas linhas (`architecture_claude_md_cabe_no_orcamento`).
- `.config/testes-lentos.tsv` e `Cargo.lock`: regenerar. Existem: `scripts/collision-surface.sh` (caminho ABSOLUTO), `censos-da-arvore-combinada.sh`, `recontar.py`, `schema-recount.py <n>`, `integrar-rodada.sh`.
- **O que só o `ship.sh` pega:** `cargo machete` NÃO foi corrido neste fecho (a crate nova `ph2d-shape-handles` e as deps de `ph2d-physics-ecs`/`ph2d-panel-physics` merecem-no). `cfg(target_os` novo: 0 (`git diff -U0 652b506b58..HEAD -- '*.rs' | grep -c '^+.*cfg(target_os'`).

## 9. SMOKE do dono (depois da integração; antes, na árvore da linha)

⚠️ Binário: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && cargo build --profile smoke -p ph2d-host-desktop` (depois da integração, o mesmo no `main`).

1. **Sombra nos contornos.** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && PH2D_PHYSICS_SMOKE=120 ./target/smoke/ph2d-host-desktop`. Na cena, o contorno colorido de cada caixa tem uma sombra escura por baixo; o Crate laranja já se lê sobre a arte laranja. Errado: contorno sem sombra, ou sombra que cobre a cor.
2. **Renomear a camada pelo botão.** Clique no **Crate** na lista da esquerda; no Inspector, cartão **Collision Layers**: logo abaixo da **Layer** estão **Rename layers…** e **Open in tab…**, e só depois **Collides with**. Clique em **Rename layers…**, escreva `Boxes`, Enter, depois **Done**. O nome muda para «Boxes» (não «PropsBoxes»). **Ctrl+Z** volta a «Props» num só passo. Um duplo clique na Layer só abre/fecha a lista, nunca o renomear. Errado: botões abaixo das caixinhas, o nome soma ao anterior, o duplo clique abre o renomear, ou o Ctrl+Z precisa de mais de um passo.
3. **Remove.** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && PH2D_PHYSICS_SMOKE=120 PH2D_PHYSICS_SMOKE_ABA=remove_lista ./target/smoke/ph2d-host-desktop` (com `=remove` a pergunta fica fechada). Abre a aba **Collision Layers** com a pergunta do **Remove**: «Move them to» é uma lista que abre ao clicar, com ✓ na escolhida; o texto «Move them to» lê-se bem. **Move and remove** não deixa mover para a camada 0 nem para a própria. Errado: lista sempre aberta, sem ✓, ou aceita a própria camada.
4. **Face clássica.** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && PH2D_CARDS=0 PH2D_PHYSICS_SMOKE=120 ./target/smoke/ph2d-host-desktop`. Selecione o Crate: a **Layer** é uma lista com nomes (0–7 e as visíveis até 31); com o Inspector estreito o nome fica por cima do chip. Errado: só números, ou nome cortado.
5. **Ray Sensor.** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && PH2D_RAY_SMOKE=1 ./target/smoke/ph2d-host-desktop`. Selecione o Ray Sensor: **Layer** é uma lista com os nomes das camadas (como a do collider); debaixo, **Sees** com um quadro por camada e «+ Add layer». Escolha outra camada na lista: o contorno do campo muda de cor; clique num quadro de «Sees»: liga/desliga o que o raio vê (o mesmo que o «Collides with» do collider). Errado: um campo de número, ou o quadro não muda nada.
6. Diga o que deu errado em cada passo.



Larguras próprias para a aba (segue as do editor de animações, onda 9g) · tabela de camadas com 8 fixas (o dono quis 32, schema 219) · cura do contorno do Crate por espessura (é a cor contra a arte; a cura é a sombra) · harness de gesto dentro da `App` em `cargo test` (a `App` segura uma surface de janela; trabalho grande, não feito).
## ⛔ Recusas MEDIDAS (na ficha)

Larguras próprias para a aba (segue o editor de animações, 9g) · 8 camadas fixas (o dono quis 32) · cura do contorno do Crate por espessura (é a cor contra a arte; a cura é a sombra) · harness de gesto dentro da `App` em `cargo test`.

## Binário do smoke (já compilado, 08/11, depois das emendas 2 e 3, `429662bb16`)

`rm -rf target/debug/incremental target/smoke/incremental`. Depois, `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` duas vezes; a 2.ª, sem nenhum «Compiling»:

```
    Finished `smoke` profile [optimized] target(s) in 0.21s
```
