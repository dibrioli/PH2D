# HANDOFF de integração — `line/GameEngineFuncional02`, 09/11 (Outline da imagem · Auto Key dos pontos · F3 corpo físico e gatilho)

Leitor: o agente integrador. Handoff anterior (já no `main`): [OPCAO_B](HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-08_OPCAO_B.md) (ADR-0199). Mecanismo por onda, nos handoffs de continuação linkados em §2 e nos planos [f2_contorno_e_formas_plano.md](../componentes/f2_contorno_e_formas_plano.md) · [f3_corpo_e_gatilho_plano.md](../componentes/f3_corpo_e_gatilho_plano.md). Memória das decisões do dono: [respostas à crítica da física](../../../project-memory/project_physics_critique_answers_2026_11_09.md).

## 1. Identidade

Branch `line/GameEngineFuncional02`, HEAD `394b3fd4ce`. Merge-base com o `main`: `f254e42bf1` (09/11). **119 commits**, 511 ficheiros, +18 603 −1 987 (`git diff --shortstat main...HEAD`; os −19 132 de `ph2d-app-physics` são as cenas movidas por `git mv`, §6). Nada integrado, nenhum push. **Suíte completa e `nextest-impacted` NÃO correram** (regra do dono 23/10: são do integrador). Árvore: só `.log` de fotos por rastrear em `fotos_fisica/` (lixo do `fotografa_cena.sh`, comite ou apague).

## 2. O que entra (por onda; `git log --oneline main..HEAD`)

**(a) 09/11 blocos 1–2** — [BLOCOS_1_2](HANDOFF_CONTINUACAO_2026-11-09_BLOCOS_1_2.md). Corpo de formas todas desligadas FICA no solver (1 kg, `BodyDesc.shape_off`) `0ad295bd30` `af0cbabde4` `4df0d9d9ca` · «Collides with» com 2 formas = UM passo `9bf303f451` · nomes únicos «Collision Shape (n)» + exceção denunciada `6a3c6e5c9a` · «Differ from this layer» em fila `5c0044706a` `9f1644d7f2` `c0a8c499e9` · forma filha sem «Remove Shape» `72e67ca1ad` · aviso na Nav Region `aad7129625` · `PH2D_PHYSICS_SMOKE_OLHO` `9c0eca9c81` · provas pelo caminho real `dcb56671a7`. **Hierarchy:** um clique seleciona sempre (some `WidgetEvent::LongPress`) `416385f894`; a forma neta arrasta-se para o corpo `de8e2cec50` `d86b8ed7db` `7d95f1210c`.

**(b) Outline W0–W7** — [OUTLINE_W2B](HANDOFF_CONTINUACAO_2026-11-09_OUTLINE_W2B.md) + plano. **W0** custo MEDIDO (`MAX_PIECES=256`, `MAX_POINTS=512`, `trace_fitted`; tabela em `crates/ph2d-outline-trace/README.md` §Ceilings) `ba83afa14c`. **W2a** motor: `ShapeDesc::Outline`, compound de convexos, chave blake3 `7b0adbfe1c`. **W2b** `ColliderShape::Outline` + `ColliderOutline` na FILHA, **schema 225→226** `62497c9740`. **W2c** produtor, crate de cola `ph2d-physics-outline` `e9c883aae8` `50058a1bfd` `41f49b94e7`. **W4** cartão Shape «Outline» (Detail, Alpha cutoff, Ignore specks under, «Too detailed»), canais FISICA 48..50 `118335ca78` `1c69a0525b`. **W5** pontos no canvas (arrastar, duplo clique na aresta, Alt+clique; um passo de undo) `90dcbf8abe`. **W6** cena 120 com Rock/Ring/Marble/Table/Frame `f310dd983f` `c5d8ecf30b` `ae316bc4f4`. **W7** seleção/Delete/fusão `070c3a165c`; «Convert to Collision Outline» (`HierRequest::ConvertToCollisionOutline`, crate `ph2d-physics-outline-vec`) `35bf5ac428`. Depois: lei para FORA (Detail = desvio máximo para fora) `51c66ced3b`; cursores (`ph2d-cursor-art`) `26b27cd4ff`; «Merge by distance» `2a4a05b788`; Detail trava a editar pontos `7bfc4ab258`.

**(c) Auto Key dos pontos + regra B** — plano §«Auto Key dos pontos». Canais FISICA 51/52 (i=anel, j=ponto): fase 1 `fb2019c56b` `9596fe04c6`; fase 2 (acrescentar/apagar vale em todas as chaves; LCS em `ph2d_timeline::topologia`) `2d7eb1435a` `983c9864bc`. **Regra B:** com trilha de pontos, mudar o NÚMERO de pontos trava com o Auto Key ligado ou desligado e abre o modal «This shape's points are animated» (`ContextMenuKind::ConfirmDeletePointsAnimation`; «Delete animation» = `PhysicsFieldEdit::DeletePointsAnimation`, um Ctrl+Z) `677f1cf7f5` `6d0d6c3452` `887744a11b`.

**(d) F3 ondas 0–3** — [0–1](HANDOFF_CONTINUACAO_GEF02_F3_ONDAS_0_1_2026-11-09.md) · [2–3](HANDOFF_CONTINUACAO_GEF02_F3_ONDAS_2_3_2026-11-09.md). Onda 0: 53 cenas + `SceneCtx` para a folha `ph2d-physics-smoke-scenes` `9029969369`. Onda 1: censo de sinais («1 listens» / «nobody listens» / «N send»), crates `ph2d-signal-ends`/`-census`, sufixo pintado (`widget/field_suffix.rs`) `007650926b` `d00414503a`. Onda 2: camada de colisão na timeline diz o rótulo (`Especie::CamadaDeColisao`) `c8e004d79d`. Onda 3: aviso da forma esticada ressuscitado (`InspectorPhysicsInfo::estica`) `89460ca606`; a curva da física guarda o número do CARTÃO (Width/Height inteiros, altura da cápsula inteira, VEL_ANGULAR em °/s), dobra «More» `985319e9ac`.

**(e) F3 onda 4 — a tecla no cartão do Trigger** `e4a95848f8` `ffaf6162e3` `394b3fd4ce`. Linha «Key» (`ph2d-panel-inspector/src/sections/action_trigger_tecla.rs`): o botão mostra as teclas de teclado da acção, ou «Set key» / «Create and set key» / «Press a key… (Esc cancels)»; desligado com Action vazio; aviso cinzento «{key} is also used by {action}». Decisões do dono 09/11: a tecla TROCA as de teclado (os comandos ficam), repetida é aceite e avisa, acção por criar cria e espera. Mecanismo: `EditorAction::ListenForTriggerKey{name}` → `pd.create_input_actions: Vec<PedidoDoMapa>` → `chrome::aplica_pedidos_do_mapa` (`ph2d-editor-core/src/screens/hero/chrome/input_map/pedidos.rs`) cria a acção e chama `WidgetStore::listen_for_binding_replacing`; a captura existente (`capture_if_listening` + `input_map/apply.rs`) faz `retain` das `Binding::Key` quando `input_map_listen_replaces`; `action_trigger_edits::le_teclas` preenche `InspectorTriggerRow::tecla: TeclaDaAccao`. `fase_criar_accao_do_mapa.rs` 55→13 linhas (a lei saiu para o editor-core). **Cura do fecho (auditoria, `394b3fd4ce`):** a escuta armada e esquecida trocava em silêncio as teclas da acção com a tecla seguinte (campo de texto, F5, a 1.ª do jogo; o Input Map não é do Ctrl+Z) ⇒ `WidgetStore::desarma_escuta_no_clique` em cada Down pela porta única `shells/desktop/src/input_dispatch.rs::pointer_button`, antes das saídas cedo; o clique que desarmou não re-arma (`take_listen_cancelled_by_down`); o poll do comando (`input_map_drag.rs`) ignora a escuta do cartão; a escuta da JANELA do Input Map não muda. `card_field.rs` 512→426 (contas `eval_number`/`Expr` para `widget/card_field/conta.rs`, 90) por `architecture_widget_loc_cap` (500).

## 3. Superfície de colisão (colada de `collision-surface.sh`, 09/11; REFERÊNCIA, o integrador lê o `main` no ficheiro)

```
SUPERFÍCIE DE COLISÃO — line/GameEngineFuncional02 contra main
  merge-base f254e42bf1   ·   119 commit(s)   ·   511 arquivo(s)
  ⚠ PROJECT_SCHEMA                        226   (base: 225)
  ⚠   └ tripla do gate               (226, 13, 29)   (base: (225, 13, 29))
    VEC_SCENE_SCHEMA 29=29 · FLIP_SCHEMA 13=13 · DOC_VERSION (timeline) 25=25
  REGISTRO: ph2d-ecs 119 (base 119)
  ✗ ph2d-render (espelho = ecs + K)  —   (base: —)
  ✗ ph2d-script (espelho = ecs + K)  —   (base: —)
  CONTRATO CONGELADO: node.rs intocado · tool.rs intocado
  ADR: último no disco 0199, próximo livre 0200; esta linha não cria ADR
  Cargo.lock: 6 pacotes '+name' novos: ph2d-cursor-art · ph2d-physics-outline ·
    ph2d-physics-outline-vec · ph2d-physics-smoke-scenes · ph2d-signal-census · ph2d-signal-ends
  MARCADORES DE CONFLITO: nenhum
  ✗  1030 / 600   shells/desktop/src/main.rs
  ✗ SONDA CEGA em: ph2d-render, ph2d-script
```

Notas: (1) `main.rs` 1030/600 é o aviso do script; o gate real é `file_loc_caps` (`shells/desktop/tests/it/file_loc_caps.rs:37`, `FILE_OVERAGE_OK` main.rs = **1031**) ⇒ ✓. A linha acrescentou 3 linhas (`g.cursores.prepara(...)`; `main` tinha 1027). (2) As duas «sondas cegas» (ph2d-render/ph2d-script) são «—» também na base: bug de caminho do script, não desta linha. (3) Os 6 pacotes novos são internos.

**Contadores como DELTA** (o integrador RECONTA, §5.0): `PROJECT_SCHEMA` **+1** (225→226, `62497c9740`: `ColliderShape::Outline` + componente `ColliderOutline` na filha; escada `shells/desktop/src/project_schema.rs:378`, tripla `(226, 13, 29)` em `project_schema_tests.rs:367`). O `main` está em 225 no merge-base: se outra linha subir antes, `python3 scripts/schema-recount.py <valor>`. Registo `ph2d-physics-ecs` **+1** (47→48, `lib.rs:317`), espelhos `ph2d-app-macros/src/mundo/portas.rs:174` e `ph2d-component-desc/src/catalog/physics.rs:233`; registo `ph2d-ecs` (119) inalterado. Sem migração (CLAUDE.md §0.11). Cena de smoke da física **+1** (121; `CENAS` 120→121). Atlas de fábrica: `ROCK_KEY = u32::MAX - 10` (`ph2d-render/src/atlas/fabrica.rs:76`, 9.ª entrada de `FACTORY_ART`). ADR: nenhum (próximo livre 0200).

**Pontos quentes de linhas vivas.** `ph2d-app-components` (PixelArt) tocado em 9 ficheiros: `action_trigger_inspector.rs` (onda 4: `+tecla: Default::default()`, −1 linha de comentário ⇒ tecto 80 869 intacto), `instantiate.rs`, `instantiate_tests.rs`, `marcas_inspector.rs`, `marcas_inspector_tests.rs`, `nav_inspector.rs`, `nav_inspector_tests.rs`, `nav_smoke.rs`, `smoke_desenho_e_corpo_tests.rs`. `ph2d-panel-timeline` (TimelinePrincipal): **0 ficheiros** (a topologia mora em `ph2d-timeline`). `cards_sprite_folhas` / `cards_anim`: **0 ficheiros**.

## 4. Foundational tocado e por que é aditivo

**Contratos congelados (§6) NÃO tocados.** `git diff --name-only main...HEAD | grep -E "ph2d-nodegraph|ph2d-tool|ph2d-vector|ph2d-imageio"` só acusa `crates/ph2d-tool-painter/{Cargo.toml,tests/it/*}` (falso positivo do padrão `ph2d-tool`: dev-dep `image` 0.25 e dois testes `wet_*_look_probe` vindos da shell; nenhum `src`, nenhum trait `Tool`). `git diff main...HEAD -U0 | grep -E "^[+-][^+-].*(trait (NodeOp|OpResolver|Tool|RasterEditTool|CanvasPaintTool)\b|struct NodeManifest|enum PanelEvent|enum VectorOp|struct (Vertex|Segment|Region)|enum AnimValue|fn sample\(|ColorProfile|MAX_SPIRAL_TURNS|MAX_POLYGON_SIDES|MAX_VERTICES_PER_LLM_GEN)"` → vazio. Gates verdes no fecho: `architecture_contract_surface` 3/3 · `architecture_tool_contract_surface` 4/4 · `architecture_vector_contract_surface` 11/11 · imageio `capped` 6/6.

| Onde | O que | Aditivo porque |
|---|---|---|
| `ph2d-physics` | `world/outline.rs` (`OutlineKey`, `register_outline`, `OutlineCache`); `ShapeDesc::Outline` | cena sem Outline é byte-idêntica |
| `ph2d-physics-ecs` | `ColliderShape::Outline`, `ColliderOutline`, `outline.rs`; canais FISICA **48..52** depois de `ZONA_QUEDA=47` (CONTORNO_DETALHE/ALFA/MANCHAS/PONTO_X/PONTO_Y); `CAMPOS` 41→46 | anexados ao fim |
| `ph2d-ecs` | `Especie::CamadaDeColisao` (a meio do enum, NÃO serializada); `canal::unidade()`: FISICA `RAIO..=CAPSULA` e `DESLOC_X/Y` = Comprimento | ver §5 |
| `ph2d-timeline` | `topologia.rs` (`BTreeMap`) | módulo novo |
| `ph2d-editor-core` | `EditorAction::ListenForTriggerKey`, `HierRequest::ConvertToCollisionOutline`, `ContextMenuKind::ConfirmDeletePointsAnimation`, `widget/field_suffix.rs`, campos `input_map_listen_replaces` / `input_map_listen_cancelled_by_down` no `WidgetStore` | variantes/campos novos |
| consumidores | `ph2d-app-physics`, `-components`, `-macros`, `ph2d-panel-*`, `ph2d-component-desc`, `shells/desktop`, `shells/game-scenes`, 6 crates novas | — |

**Significado gravado de canais mudou SEM subir versão** (permitido, §0.11): FISICA `MEIO_X/Y` metade→inteiro, `CAPSULA` metade→altura inteira, `VEL_ANGULAR` rad/s→°/s (`985319e9ac`). **Determinismo:** nenhum `HashMap`/`HashSet` novo (`OutlineCache`, `Outlines`, `topologia.rs` são `BTreeMap`); blake3 sobre bytes LE; `sort_unstable` com desempate total.

## 5. Símbolos que podem colidir (grep na integração)

- **Campos SEM `Default`** (quem constrói literal precisa deles): `InspectorPhysicsInfo{outline: None, estica: None, collides_pinned: 0}` (literais em `ph2d-app-physics/src/inspector/body.rs:144,231`, `seam_physics.rs:29`, `o_inspector_armado.rs:415`); `InspectorTriggerRow{tecla: Default::default()}`; `BodyDesc{shape_off: false}`.
- **Variantes novas (match exaustivo parte):** `ShapeDesc::Outline{half_x,half_y,key}` · `ColliderShape::Outline{half_x,half_y}` (tag de forma 3) · `Especie::CamadaDeColisao` (`canal_palavra::prende`) · `PhysicsFieldEdit::{DeletePointsAnimation, Outline(OutlineEdit)}` · `EditorAction::ListenForTriggerKey` · `HierRequest::ConvertToCollisionOutline` · `ContextMenuKind::ConfirmDeletePointsAnimation` (+ lista `ALL`).
- **QUEBRAM chamadores:** `WidgetEvent::LongPress` REMOVIDO (`416385f894`); `HierarchyDragState.down_timestamp_ns` removido e `begin_hierarchy_drag` perdeu `timestamp_ns`; `objects_on_layers` devolve `Vec<ObjetoDaCamada>` (muda `ph2d-panel-physics::set_layer_objects`); `estado_da_selecao` devolve `EstadoDaSelecao`; `INSP_PHYS_SHAPE` 3→4 entradas; `ConducaoFisica` +2 campos e assinatura de `substituir` mudou; `SceneSetup`/`SceneCtx` movidos para `ph2d-physics-smoke-scenes` (re-exportados por `ph2d-app-physics`).
- **Nomes:** `ColliderOutline` · `OutlineCache` · `canais_pontos::regra_b` · `PontosNoCanvas` · `listen_for_binding_replacing` · `TeclaDaAccao` · `PedidoDoMapa` · `ROCK_KEY` · cena física 121.

## 6. Catracas de LOC (SOMAM entre linhas; reconta o integrador; cura = MOVER)

- **Shell** (`the_shell_only_shrinks` ✓ 2/2): `TETO_LOC = 189 041` (`architecture_the_shell_only_shrinks.rs:112`) NÃO foi tocado. A linha ENCOLHEU a shell (`9695fef639`: 189 093→188 968; onda 4: fase do mapa 55→13). `main.rs` 1030 ≤ 1031. Se o gate de obsolescência (`CENSO_OBSOLETO_ACIMA_DE`, `:186`) acusar «catraca obsoleta» depois do rebase, a cura é BAIXAR `TETO_LOC` ao medido, nunca subir.
- `ph2d-app-physics` **38 085** (era 55 515: onda 0 moveu 18 056 linhas por `git mv` para `ph2d-physics-smoke-scenes`, nova, tecto **18 572**; cura do tecto anterior `63d5638856` 55 579→55 454 e `e74f1a4c4f` 55 559→55 508). `ph2d-app-components` **80 869/80 869** (intacto). `ph2d-app-macros` 14 630. `card_field.rs` 426 ≤ 500. `nav.rs` 595 ≤ 600. Lista `TETOS_DAS_CRATES_DA_FAMILIA` (linhas 212–240): conflito textual provável.

## 7. O que um merge pode partir

- Qualquer literal de `InspectorPhysicsInfo` / `InspectorTriggerRow` / `BodyDesc` de outra linha (§5) e `match` exaustivo sobre as variantes novas.
- Quem ouvia `WidgetEvent::LongPress` ou usava `down_timestamp_ns`.
- Quem acrescentou cena de smoke em `ph2d-app-physics` (agora é `ph2d-physics-smoke-scenes`) ou lê a pasta antiga; os 2 gates da shell (`handle_scenes_start_paused`, `scenes_that_ask_for_the_ruler_open_the_timeline`) já varrem a nova.
- Retratos: schema 226 abre sem migração; fixtures com `MEIO_X/Y`, `CAPSULA` ou `VEL_ANGULAR` chaveados leem o número novo (cartão), sem aviso.
- Conflito provável: `ph2d-physics-ecs/src/lib.rs` (`reg.len()`), `project_schema*.rs`, `Cargo.lock` (`cargo metadata`, nunca à mão), `ph2d-panel-inspector` (cartões Body/Shape/Trigger), `ph2d-i18n` `canais.rs`/`cartoes.rs`.

## 8. Prova de fecho (medida em 09/11 na linha; suíte completa e nextest-impacted NÃO corridos)

`cargo fmt --check` nos 31 pacotes ✓. Clippy: `PH2D_SUITE=fecho bash scripts/ph2d-run.sh cargo clippy -p <31 pacotes tocados> -p ph2d-spike --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` ✓ (refeito depois da cura em editor-core, panel-inspector, host-desktop, i18n ✓). Gates por nome: `file_loc_caps` 4/4 · `arch_safe_clamp_only` 2/2 · `the_shell_only_shrinks` 2/2 · `architecture_workspace_file_loc_cap` 2/2 · `architecture_panel_loc_cap` 4/4 · `architecture_widget_loc_cap` 1/1 (estava ✗ 512/500 antes da cura) · `architecture_no_dependency_climbs_a_layer` 3/3 · `architecture_cycle_prevention` 4/4 · `architecture_claude_md` 3/3 · contratos como em §4. Testes do item (onda 4): editor-core `o_cartao_do_trigger` 8/8 · `card_field` 4/4; panel-inspector `gatilho` 13/13 · `every_word` 5/5; host-desktop `a_accao_pedida` 3/3; panel-registry-init `o_inspector_armado` 3/3; app-components `action_trigger` 4/4. Ondas anteriores: secções «Prova» dos quatro handoffs de continuação.

Binário do smoke (fecho 09/11, depois de `rm -rf target/*/incremental` — 131 GB devolvidos; 2.ª corrida de `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, zero «Compiling»):

```
    Finished `smoke` profile [optimized] target(s) in 0.22s
```

## 9. Provas de mutação

**Onda 4: 7/7 sangram** — M1 `retain` removido → `a_tecla_do_cartao_troca…` · M2 `if escuta`→false → `o_pedido_cria…` · M3 flag nunca limpa → `esc_no_cartao_cancela…` · M4 `o.id != accao.id` removido → `a_linha_key_le_a_tecla…` · M5 `rows.first()` → `o_botao_key_pede_a_tecla_da_fileira_aberta` · M6 `a_escuta`→false → `a_escuta_o_botao_key_cancela` · M7 registar sempre → `sem_nome_o_botao_key_nao_e_alvo`. **Cura do fecho 3/3** — desarma que não pára → `um_clique_fora_desarma_a_escuta_do_cartao` · `cancelada ||` removido → `o_clique_que_desarmou_nao_rearma` · porta sem a chamada → `a_porta_do_clique_desarma_a_escuta_do_cartao_antes_de_sair_cedo`.

Ondas anteriores: todas sangram (contagens nos handoffs de continuação): blocos 1–2 (5 + 3 + 2), Outline W0 (3) e W2a (4), regra B 7/7 por porta, F3 onda 1 (6 + 6), onda 2 (2), onda 3 (4 + 3). **SEM mutação** (herdado): o ramo `was_off` de `retune_body` e a recusa `shape_off` de `zone_effect` (gates `9208ff756e`/`ade21fa807` entraram depois).

## 10. Smokes (o dono já fez; 1 e 2 são o fecho da onda 4)

Build: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && bash scripts/ph2d-run.sh cargo build --profile smoke -p ph2d-host-desktop`. Prefixo de todos: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && … cargo run --profile smoke -p ph2d-host-desktop`.

1. **Tecla do Trigger (onda 4) — smoke OK do dono 09/11.** `PH2D_TRIGGER_SMOKE=1`. Abre a secção Trigger com a linha Key à vista; «Q is also used by dash»; clique no botão → «Press a key…»; J troca a tecla; Esc cancela; acção por criar (ex. laser) → «Create and set key» → H; conferir em Settings › Input Map. Errado: Key ausente, tecla seguinte engolida depois de clicar fora, comandos do mapa perdidos. Fotos: [`fotos_fisica/onda4/`](fotos_fisica/onda4/) (`01_trigger_key.png`, `02_cartao_trigger_key.png`).
2. **Física 120.** `PH2D_PHYSICS_SMOKE=120` (+ `…_CAMADA_NA_TIMELINE=1`: faixa de camada do Player «Player»→«Props», não números · `…_SELECT=Player` + `…_OLHO="Collision Shape (7)"` o olho · `…_EXCECAO=1` [+ `…_ABA=excecao`] a aba «Differ from this layer»). Fotos [`camada_timeline/`](fotos_fisica/camada_timeline/) · [`olho_09_11/`](fotos_fisica/olho_09_11/). Bola esticada: `PH2D_GAME_SMOKE=4 PH2D_GAME_SMOKE_FOTO=fisica_esticada` (aviso + «Set Scale to 1»). Sinais: `PH2D_PHYSICS_SMOKE=121` (Coin A «1 listens», Coin B «nobody listens», Score «1 sends»; [`sinais/`](fotos_fisica/sinais/)).
3. **Outline** (cena 120; fotos [`outline/`](fotos_fisica/outline/) 01–20): `PH2D_PHYSICS_SMOKE_OUTLINE="<nome do corpo>"` · `…_OUTLINE_PONTOS=1|2` · `…_VETOR=1|2` · `…_OUTLINE_MERGE=<px>` · `…_OUTLINE_DETAIL` / `…_OUTLINE_LADO=<m>` (só fotos). Menu da Hierarquia ▸ «Convert to Collision Outline»; modal da regra B ao apagar um ponto com trilha. Valores exatos em `ph2d-app-physics/src/physics_smoke_ficha.rs:27,35`, `ph2d-physics-scene/src/ficha.rs:384-407`, `ph2d-physics-outline-vec/src/smoke.rs:24`. Não re-corri smoke nenhum ao escrever isto.

## 11. ABERTO

1. «Set Scale to 1» repõe só o 1.º da cadeia com escala desigual (`estica.rs:25-28`): forma (2,1) sob corpo (1,3) pede dois cliques, cada um diz quem. Decisão de produto: repor a cadeia inteira?
2. Suspeitas da auditoria NÃO confirmadas: (a) altura da cápsula convertida com raio possivelmente velho no mesmo quadro quando os dois têm chave (`canais_corpo.rs` `edicao` → `event_physics.rs:198`); (b) «Delete animation» do modal da regra B pode custar 2 passos de Ctrl+Z (`fase_bus_physics_sections.rs:45-49`); (c) janela Input Map aberta e cartão armado na mesma acção: a linha «Bind…» da janela mostra escuta mas a tecla troca; (d) «N listen» pode contar a própria caixa.
3. `OutlineCache` e a tabela de peças só crescem (`ph2d-physics/src/world/outline.rs:76`, `ph2d-physics-ecs/src/outline.rs:125`): memória numa sessão longa a arrastar Detail.
4. O botão antigo «Create this action» coexiste com «Create and set key» (o dono não pediu para tirar).
5. Herdados: `from_control` e botões do painel autorado fora do censo de sinais (uma caixa que OUVE pode dizer «nobody sends»); canais da família JUNTA ainda em radianos; O(pontos²) do traço e `trace()` antes do tecto (`OUTLINE_W2B` §6); ponte de navegação (`nav_custo.rs`) e `palco_fisica` (impressão digital só da forma principal) — ver BLOCOS_1_2 §6; macro que muda o nº de pontos sem modal; painel antigo `PH2D_CARDS=0` sem esmaecer campos travados.
6. Suíte completa + `nextest-impacted`: do integrador. Integração só por ordem do Enio; smoke do dono não é aprovação de integração.
