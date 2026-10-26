# Handoff de continuação — `line/GameEngineFuncional`, plano 13 (isolamento por uso) (26/10)

**Próxima janela:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` + `pwd` +
`git branch --show-current` ANTES de ler. Base (merge-base com o main) `c3662cd8e4`; a onda começou em
`e2bf44f6eb`. **Não integrada, não empurrada.** Antecessor:
[`HANDOFF_CONTINUACAO_2026-10-26_12_IMAGEM_COMO_ASSET.md`](HANDOFF_CONTINUACAO_2026-10-26_12_IMAGEM_COMO_ASSET.md).
Plano: [`13_isolamento_por_uso_plano.md`](../13_isolamento_por_uso_plano.md) (o §«Feito (26/10)» resume) ·
medido: [`13_isolamento_por_uso_medido.md`](../13_isolamento_por_uso_medido.md) (a Unity medida, o modelo Godot e os
custos).

## Estado

**Smoke do dono: POR FAZER.** Os 3 defeitos que o dono apontou no smoke 4: (a) o bloco de fábrica (Platform High)
mostrava «H Frames 3» — as 3 cópias do azulejo eram quadros; (b) largar «Platform» dos Assets no canvas dava outra
coisa (Individual, píxeis copiados) em vez da mesma imagem dos blocos; (c) uma imagem animada (Flag, 4 quadros)
num objecto que a repete mostrava só o 1.º quadro, sem aviso. Decisão do dono (memória
`project_isolation_by_use_and_atlas_strategy_stays_on_object_2026_10_26`): o Inspector mostra o que cada USO da
imagem precisa; a estratégia Atlas/Individual fica no objecto; modelo Godot adiado.
Fotos: [`fotos_13/`](fotos_13/) — `nivel_4_bloco.png` (Platform High, Texture & Atlas aberta: sem H/V Frames,
Source 256×128), `nivel_4_bandeira.png` (Image ← Flag: aviso «This image has 4 frames, but this object repeats it —
only the first one shows.» + **Make it Animated**; o bloco mostra o 1.º quadro 3×, como a Unity *Tiled*),
`nivel_4_animada.png` (bandeira de pé 1×2 m = 100×200 px, cartão «Animation (1)»), `nivel_4_largada.png`
(«Platform» largada a [6,3] m: Atlas, 256×128, 100×50 px, nome «Platform», «Frames of the image» + H/V Frames 1).

| sha | O quê |
|---|---|
| `e2bf44f6eb` | docs: isolamento por uso — os 3 defeitos, a Unity medida, a decisão da estratégia (início) |
| `f0ddde51ba` | test: os 3 defeitos em vermelho, pela porta; o plano |
| `44f12e22e5` | fix: o azulejo de fábrica é UMA imagem — as cópias passam para dentro do atlas |
| `7842d8e721` | feat: o Inspector por USO, Make it Animated, o corte é da imagem e o script pela mesma lei |
| `289e20993f` | feat: as fotos do smoke 4 (bloco, bandeira, animada, largada) e a arrumação da onda |
| (este) | docs: handoff de continuação + o plano com o feito |

## O que entrou

- **`ph2d-render`:** `TextureAtlas::insert_wrapped(key, w, h, rgba, copies)` empacota a imagem ENVOLVIDA por cópias
  de si e responde `region()/region_uv()/region_px()` com a cópia do MEIO (campo
  `wraps: BTreeMap<u32, ([u32;2], AtlasRegion)>`); `insert` simples e `remove` limpam o wrap; `regrow_inplace`
  re-envolve quando o `fetch` devolve a imagem única (a biblioteca guarda a imagem, não as cópias).
  `pub fn wrap_image(rgba, size, copies)`. `FactoryArt::{image_px, frames}` (azulejo = 1×1 quadro, imagem = uma
  célula); `factory_image(art)` = a IMAGEM (uma célula num azulejo); `factory_pixels(art)` = a tira do atlas,
  byte-idêntica a antes (`HASH_DAS_ARTES_ANTIGAS` verde). `insert_reserved_tiles` usa `insert_wrapped` nos azulejos.
- **`ph2d-app-components`:** `factory_images::register` guarda a imagem (um azulejo) no `AssetDb` ⇒ a miniatura
  mostra um azulejo (**fecha** o ABERTO da onda 12 «miniatura com 3 cópias»). `factory_images::forget_extrusion_grids(world)`
  tira o `SpriteGrid {cópias, meio}` legado dos sprites de azulejo de fábrica, chamado em
  `shells/desktop/src/project_load.rs` depois de `apply_project` (ficheiro antigo abre com o mesmo texel; sem
  degrau de `PROJECT_SCHEMA`, nenhum campo gravado mudou). `script_image`: `ImageOverride.law: Option<OverrideLaw
  {atlas_key, size, grid}>`, `ImageOverride::ready()`, `type Decide`; `upload(renderer, db, sim, decide)` pergunta
  primeiro à lei; `cut()` usa `law.grid`; `ImageOverride` perdeu `Eq` (f32).
- **`ph2d-app-game`:** as cenas (azulejo, `object_add::veste_camada`) já não inserem a grelha de cópias; smoke
  `Foto += Bloco|Bandeira|Animada|Largada`; `PlanoDaFoto` é agora struct `{foto, escolhas, anima, larga, alvo}`;
  `plano_da_foto(sim, ids, fabrica)`; `ONDE_LARGA` [6,3]; `CENTRO_DAS_FOTOS_13` [7,2]; `Foto::do_plano_13`.
- **`ph2d-app-sprite-anim`:** `image_choice`: `image_facts` pub (A consulta), `factory_facts` usa `frames()` (azulejo
  não tem corte), `cut_of_users(world,id,atlas)`, `library_facts(world,…)` (corte dos utilizadores para a importada,
  nunca para a de fábrica), `place_library_image(sim, acquire, db, atlas, ppm, at, base, id)` = sprite nova
  (`spawn_sprite`, azulejo branco de reserva) + `choose_library_image`, com o nome da imagem da biblioteca;
  `override_law(world,e,id,db,atlas,ppm)` = a mesma lei sobre cópia de rascunho (objecto que repete guarda o tamanho
  autorado). Módulo novo **`image_use`**: `ImageUse{Still,Repeated,Animated}`, `image_use()`,
  `ImageConflict::AnimatedImageRepeated{cells}`, `cells()`, `conflict()`, `make_animated()` (tira
  `TextureRepeat/UvTransform/ScrollRepeat`; tamanho = uma célula × a LARGURA de um azulejo antigo, a imagem não
  deforma; `SpriteGrid` quadro 0; `SpriteAnimations` com tag `LOOP_TAG="loop"` 0..n-1; `SpriteAnimator` autoplay),
  `inspector_use()`, `make_shown_image_animated()`, `recut_users(world,e)` (importador Unity: recortar um recorta
  todos os sprites com a mesma chave de atlas ou o mesmo `SpritePixels`). `sheet.rs` descodifica a imagem de
  fábrica (não a tira). Deps novas: `ph2d-core`, `ph2d-image-import`; dev-dep `ph2d-app-game`.
- **`ph2d-editor-core`:** enum `InspectorImageUse`; `InspectorSpriteInfo += image_use, image_cells`;
  `EditorAction::MakeImageAnimated{entity_bits}`.
- **`ph2d-panel-inspector`:** `sections/image_use_rows.rs` — por uso: *Still* = «Frames of the image — every
  object that shows it uses the same.» + H/V Frames + Frame só se > 1 célula; *Repeated* = nada, ou aviso + botão
  se `image_cells>1`; *Animated* / cartão de animação = nada. Id `INSP_IMAGE_MAKE_ANIMATED` (`ids/menus.rs`),
  registado em `populate.rs` (laço com os botões de formato e reimport; `populate.rs` a 595 LOC, tecto 600), braço
  do evento só com `Repeated && cells>1`.
- **i18n novas:** `panel.inspector.image.{animated_in_block, make_animated, cut_of_the_image}`,
  `shell.asset_drop_apply.{now_animated, could_not_animate}`.
- **Shell:** `asset_drop_apply.rs` `drop_spawn_image` → `image_choice::place_library_image` (a cópia de píxeis para
  Individual acabou); `App::make_image_animated`; `troca_de_imagem_do_smoke` executa o plano (`choose_image`,
  `make_image_animated`, `drop_spawn_image`) e enquadra a câmara + abre Texture & Atlas nas fotos da onda 13;
  encanamento `pd.make_image_animated` (`fase_bus_drain_out`, braço em `fase_bus_inspector`, `fase_hero_commits`,
  `fase_recipe_and_asset_verbs`); `snapshots_inspector_sprite` enche `image_use/image_cells`;
  `inspector_commits_sprite` chama `recut_users` após um commit de H/V Frames; `fase_extract_inputs` passa
  `override_law` ao `script_image::upload`; `sim_extract_emit` desenha o override de script pela lei (uv do atlas,
  tamanho, célula); `main.rs` larga o re-export `MIN_SPRITE_SIZE`. Delta da shell vs `e2bf44f6eb`: +115/−33 = +82
  LOC líquidas (`the_shell_only_shrinks` verde, dentro da folga).
- **Decisões que a medição/desenho fixaram:**
  - **D3** — o corte de uma imagem IMPORTADA não tem campo gravado: vive nos objectos que a mostram e recortar um
    recorta todos (undo/save de graça pelo `ProjectState`). Preço: imagem importada que ninguém usa esquece o corte
    (a Unity guarda-o no `.meta`). Casa futura, se o dono pedir: o blob versionado `LibraryDoc.catalog_bytes` (sem
    degrau de `PROJECT_SCHEMA`); um campo novo em `LibraryDoc` partiria os tipos de migração congelados
    (`ProjectStateV128` referencia o `LibraryDoc` vivo).
  - **Make it Animated** dimensiona pela LARGURA de um azulejo antigo (a imagem não se deforma).
  - O override de script num objecto que REPETE mantém tamanho e UV autorados (o azulejo não viaja no override).

## Superfície que um merge pode partir

- `TextureAtlas::region()` devolve a região INTERIOR para chaves envolvidas: quem lia `region().w` de um azulejo
  de fábrica passa a ler 256, não 768. Métodos novos de `FactoryArt` + exports `factory_image`/`wrap_image`.
- A imagem de fábrica no `AssetDb` mudou (os `AssetId` dos azulejos mudam; nunca se gravam no projecto).
- `InspectorSpriteInfo` ganha 2 campos: **todo literal de struct em fixtura tem de os somar** (12 fixturas
  corrigidas aqui, incl. `crates/ph2d-panel-registry-init/tests/it/o_inspector_armado.rs`) — linha que acrescente
  fixtura nova parte na compilação.
- `EditorAction` ganha a variante `MakeImageAnimated` (`match` exaustivo alheio parte).
- `ImageOverride`: campo novo, sem `Eq`. `script_image::upload` ganha o argumento `decide`.
- `PlanoDaFoto` passou de tupla a struct; `Foto` ganha 4 variantes.
- i18n: chaves novas acima. `populate.rs` do Inspector: laço de registo de botões reordenado (a 595/600 LOC —
  outra linha que lá some botões estoura o tecto).
- Ficheiros da shell partilhados tocados: `asset_drop_apply.rs`, `fase_bus_inspector.rs`, `fase_bus_drain_out.rs`,
  `fase_hero_commits.rs`, `fase_recipe_and_asset_verbs.rs`, `snapshots_inspector_sprite`, `fase_extract_inputs`,
  `sim_extract_emit`, `project_load.rs`, `main.rs` (`MIN_SPRITE_SIZE`). Conferir com
  `collision-surface.sh` pelo caminho absoluto, contra o merge-base `c3662cd8e4`.
- Nenhum degrau de `PROJECT_SCHEMA`, nenhum componente novo, nenhum campo gravado novo.

## Prova de fecho (DELTA)

Gates novos, verdes: sprite-anim `image_use::gate_tests` — `o_bloco_de_fabrica_e_um_azulejo_sem_quadros` (a),
`largar_platform_no_canvas_e_a_mesma_imagem` (b, + nome «Platform»), `a_bandeira_num_bloco_avisa_e_fica_animada`
(c; desfazer num passo, gravar/abrir), `a_mesma_imagem_pelas_portas_todas` (linha/seletor/ranhura, canvas,
script; Platform, Flag, Hills, importada), `um_bloco_gravado_antes_abre_com_o_mesmo_desenho` (ficheiro antigo),
`re_cortar_um_re_corta_todos_os_que_mostram_a_imagem`; render `a_regiao_do_azulejo_e_a_celula_do_meio` (placa;
mutação do desvio do meio +1 sangra ⇒ o teste corre mesmo na placa) e
`a_imagem_de_um_azulejo_e_uma_celula_e_o_atlas_a_envolve`; `os_azulejos_repetem_sem_costura` e
`as_artes_antigas_sao_as_de_antes` continuam verdes; panel-inspector `seam_image_use` (2:
`cada_uso_pinta_so_os_seus_campos`, `make_it_animated_pede_a_accao_so_onde_o_aviso_existe`, com controlo).
**Red-first:** `f0ddde51ba` mostrou os três vermelhos pelas razões certas (imagem da plataforma [3,1]; a queda no
canvas tentava copiar píxeis; nenhum conflito reportado).

Fixturas da onda 12 actualizadas (`image_choice_tests`, `image_choice_gate_tests` com camada sem a grelha de
cópias), app-game `smoke_tests`/`camara_tests` (sem grelha nos azulejos). Corridas POR NOME, verdes: sprite-anim
`image_` (19), render `fabrica` (6)/`regrow` (4)/`região` (1), app-game `nivel` (7)/`camara` (4), app-components
`script_image` (4)/`factory_images` (1), panel-inspector `seam_` (219), `inspector_regression` (46),
`every_painted` (1), `the_sheet_grid` (2), registry-init `o_inspector_armado` (3), host-desktop
`flip_edits_set_the_flags`, editor-core `the_shell_only_shrinks`, `panel_wiring_parity` (2),
`architecture_panel_loc_cap` (4 — estava vermelho a 605 LOC, curado por CORTE, sem isenção), tectos de LOC de
workspace/widget. clippy `-D warnings --all-targets` das 9 crates tocadas limpo; `cargo fmt` limpo.

**Vermelho pré-existente, inalterado:** registry-init `a_carga_de_comandos_de_um_painel_so_encolhe` (inspector 92
contra 89, vem da onda 07; o botão novo não muda a contagem) — decide-se no fecho da linha.

**Suíte:** `PH2D_SUITE=onda nextest-impacted` arrancou (cadência ≥ 10 commits) e foi PARADA de propósito: regra do
dono 23/10 — suíte completa/de impacto só antes da integração (`feedback_full_suite_only_before_integration`); o
integrador corre-a. Binário do smoke: 2.º `cargo build -p ph2d-host-desktop --profile smoke` = 0 Compiling,
«Finished `smoke` profile [optimized] target(s) in 0.23s».

## Smoke (para o Enio)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=4 ./target/smoke/ph2d-host-desktop`
2. Na Hierarchy clique **Platform High**; no Inspector abra **Texture & Atlas**: não há «H Frames»/«V Frames»; a
   linha Image mostra um só bloco «Platform».
3. Clique na caixa **Image** e escolha **Flag**: o bloco mostra a bandeira cortada e, em Texture & Atlas, aparece
   o aviso «This image has 4 frames…» e o botão **Make it Animated**.
4. Clique **Make it Animated**: o bloco vira uma bandeira de pé (1 × 2 m) e aparece o cartão **Animation**;
   **Run ▸ Play in a Window**: a bandeira ondula. **Ctrl+Z** volta ao bloco.
5. Abra a janela **Assets** (Window ▸ Assets) e arraste **Platform** para o canvas: nasce um objecto «Platform»
   de um bloco, igual aos das plataformas (Atlas, 1 × 0,5 m).
6. **File ▸ Save**, feche, abra: tudo igual.
7. Deu errado se: «H Frames 3» aparece em Platform High; o botão não aparece com a bandeira; a bandeira não anima
   no Play; o Platform largado tem outra aparência, «Individual» ou 3 blocos; depois de abrir o ficheiro algo mudou.

## ABERTO

- Script `sprite.image` num objecto que repete (fundo/bloco) mantém tamanho e UV autorados.
- Imagem importada sem uso esquece o corte (D3; casa futura = `catalog_bytes`).
- «Storage Atlas · key 4294967288» mostra a chave crua (antigo).
- `Esc` não cancela o seletor (herdado).
- Da onda 12, ainda de pé: **12b Flip/Vector pela mesma porta** · a GPU de uma imagem só-individual escolhida não
  se liberta · *Sprite Editor* (cortar no asset) fora · o condutor do Play (`PH2D_GAME_SMOKE_DRIVE`) não arranca
  na cena 4 · tecto do inspector 92 vs 89 para o fecho da linha (medir em `d482366656`, subir com ⬆️ e motivo
  «capacidade nova» ou declarar composto) · heranças do handoff 11 (ver o handoff 12) · suíte de impacto
  (integrador). Fechado aqui: «miniatura com 3 cópias».
- Modelo Godot (imagem sem estratégia no objecto): adiado pelo dono; custos no doc 13 medido.

## PRÓXIMO

O smoke do dono decide. Depois: **12b** (Flip/Vector pela mesma porta) ou **J5 moedas**
([plano 03](../03_plano_jogo_de_plataforma.md)), conforme o dono.
