# Handoff de continuação — `line/GameEngineFuncional`, plano 12 (imagem como asset de projecto) (26/10)

**Próxima janela:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` + `pwd` +
`git branch --show-current` ANTES de ler. Base (merge-base com o main) `c3662cd8e4`; a onda começou em
`6ef753e124`. **Não integrada, não empurrada.** Antecessor:
[`HANDOFF_CONTINUACAO_2026-10-25_11_CAMARA_E_PARALLAX.md`](HANDOFF_CONTINUACAO_2026-10-25_11_CAMARA_E_PARALLAX.md).
Plano: [`12_imagem_como_asset_plano.md`](../12_imagem_como_asset_plano.md) (o §«Feito (26/10)» tem os números).

## Estado

**Smoke do dono: POR FAZER.** O report que decidiu a onda: o dono, no smoke do J4 (26/10), não conseguia mudar
a imagem do Background Layer; padrão Unity (memória `project_image_is_a_project_asset_reference_2026_10_26`).
Fotos: [`fotos_12/`](fotos_12/) — `nivel_4_inicio.png` (Player com Image «Hero»), `nivel_4_seletor.png`,
`nivel_4_trocado.png`.

| sha | O quê |
|---|---|
| `edae891143` | feat: trocar a imagem de um objecto como na Unity |
| `1cd765fe6d` | fix: os vermelhos da verificação por nome — a foto do smoke sai da shell |
| `782479a6f9` | test: a escala fica com a imagem nova (a mutação 3 sobrevivia) |
| `4bff90d680` | chore: clippy da onda (tipos nomeados, parâmetro sem uso) |

## O que entrou

- **MEDIDO antes de codar:** a referência durável JÁ existia (`Sprite.source` `Atlas{key}` + `atlas_asset_map`
  regravado nas mesmas células; `SpritePixels(AssetId)` blake3); o navegador de assets (miniaturas, busca,
  arrastar) e a ranhura de textura do Inspector também. **Causa do report:** a arte de fábrica (chaves
  `u32::MAX-2..-9`) não estava no `atlas_asset_map` ⇒ fora da biblioteca; a ranhura estava no fundo da secção
  «Texture & Atlas» (fechada) com o texto «Atlas · key …»; a queda copiava píxeis (`commit_edited_texture`) e
  deixava `SpriteGrid`/`UvTransform` antigos. Flip = `FlipObjectRef` (`FlipObjectId` durável) 1:1
  (`FlipEntityMap`); Vector = um path por entidade, partilha só por Componentes.
- **Oráculo Godot 4.7.2 (MIT)** (`ferramentas/textura_godot.gd`; `PH2D_GPU=1 bash scripts/ph2d-run.sh godot
  --headless -s … -- <pasta>`): trocar a textura mantém `hframes` 8/`vframes` 4/`frame` 13, `scale` 2 e `repeat`;
  `rect` 32×32→48×32 (segue a imagem); `frame` 40 não é preso numa folha menor; `Parallax2D.repeat_size` NÃO muda
  (300); o `.tscn` grava 1 `ext_resource` para 2 sprites (referência); carregar 2× = a mesma instância.
- **`ph2d-render`:** `FactoryArt` + `name` (i18n `asset.factory.*`) + `texels_per_m` (herói 192; chão, plataforma,
  terra, moeda 256; bandeira 128; nuvens/montes 128) + `native_m()`.
- **`ph2d-app-components`:** `factory_images.rs` (registo no `AssetDb` + mapa no `init.rs` da shell;
  `is_factory_key`; `saved_by_project` usado pelo `collect_assets` ⇒ o save não grava a fábrica; `names`);
  `asset_index_build` lista TODA imagem do mapa mesmo sem uso; `library_names` nomeia a fábrica; `texture_of`
  pub; `image_card` (nome + miniatura com memo); `inspector_strategy` nunca devolve a célula de fábrica.
- **`ph2d-app-sprite-anim` `image_choice.rs`:** `choose_image` (lei pura: 1 a escala fica; 2 corte da imagem,
  senão do objecto animado, senão nenhum; 3 camada com `ScrollRepeat` = Unity *Tiled*: o ladrilho segue a
  imagem, ⌈largura antiga/ladrilho⌉ ladrilhos, `UvTransform` [n,·] e `ScrollRepeat.tile` reescritos; 4 sprite
  com `Repeat` sem `ScrollRepeat` guarda tamanho e repetições; tira `SpriteRegion`/`SpriteSheetRef`;
  `SpritePixels` só no Individual); `choose_library_image` (a porta: célula do atlas que já tem a imagem,
  senão textura própria via closure `Acquire`); `bring_into_library` (Import à Unity, sem objecto).
- **`ph2d-editor-core`:** `EditorAction::OpenAssetBrowser` → `OpenImagePicker{entity_bits}` +
  `ChooseImage{entity_bits, asset}`.
- **`ph2d-panel-asset-browser`:** modo escolher (`paint_choose.rs`, `choose_image_for`, faixa «Choose the image
  for “X”» com ✕ `ASSET_CHOOSE_CANCEL`, só imagens, clique escolhe e fecha, fechar o painel desiste).
- **`ph2d-panel-inspector`:** linha «Image» logo abaixo de «Visible» (`sections/image_row.rs`, `slot_image.rs`
  — miniatura + nome; `INSP_RENDER_TEXTURE_SLOT` mudou-se para lá; «Storage» em Texture & Atlas volta a ser só o
  facto).
- **Shell:** `drop_retexture` → `App::choose_image` (`asset_drop_apply.rs`); braços `OpenImagePicker`/
  `ChooseImage` em `fase_bus_inspector`; `abre_o_seletor` em `fase_recipe_and_asset_verbs`; `publish_slot_image`
  em `snapshots_inspector`; `troca_de_imagem_do_smoke`. ~+94 LOC líquidas.
- **`ph2d-app-game`:** `CENAS` 3 → 4. Cena 4 = nível 2 + `heroi_azul.png` (800×400, 8×4 de 100 px) e
  `montes_por_do_sol.png` (800×600) trazidos à biblioteca (`docs/GameEngineFuncional/arte`, gerados por
  `ferramentas/arte_trocar.py` a partir dos SVG de fábrica). `PH2D_GAME_SMOKE_FOTO=seletor|trocado` conduz a foto
  (`plano_da_foto`).
- **Sem componente novo, sem campo gravado novo, SEM degrau de `PROJECT_SCHEMA`.** Ficheiro antigo abre igual
  (chaves de fábrica e ordem de `FACTORY_ART` iguais).

## Superfície que um merge pode partir

- `FACTORY_ART`: campos novos (`name`, `texels_per_m`).
- `EditorAction`: `OpenAssetBrowser` SOME (rename) — outra linha que o use quebra.
- `CENAS` do jogo = 4 (número que soma entre linhas: contar).
- i18n chaves novas: `asset.factory.*`, `panel.asset_browser.pick.choose_for`, `panel.inspector.image(.none)`,
  `app.sprite_anim.image_choice.*`.
- O `atlas_asset_map` passa a ter as 8 chaves de fábrica desde o arranque (quem assumia mapa vazio = «nada
  importado» parte); o índice lista imagens sem uso (o texto «No assets yet» deixa de aparecer).
- Ficheiros da shell partilhados: `fase_snapshots_publish.rs` (esta onda tocou; `line/PixelArt` também, como
  `render_loop/mod.rs`, `fase_object_add.rs`, `fase_selection_highlight.rs`) e `fase_bus_inspector.rs` (+2
  braços; P e V também tocam). `undo_app.rs`: A e V.
- **Contrato para o Pixel** (plano §Contrato): `ContentRef::Pixel(u32)` por append, `content_of`, fonte do
  índice, `choose` re-aponta `PixelSprite`, o MESMO widget da linha Image.

## Prova de fecho (DELTA)

Gates novos (verdes): sprite-anim `image_choice` (13: 7 leis + `trocar_a_imagem_muda_o_que_se_desenha` [texel CPU
da imagem referenciada; o pixel na placa provado pela foto `nivel_4_trocado`], `a_troca_sobrevive_a_gravar_e_abrir`,
`desfazer_devolve_…_num_passo` [instantâneo do mundo = `ProjectState`], `o_ladrilho_de_fabrica_e_o_de_antes`
[bit a bit], `a_fabrica_entra_na_biblioteca_e_fica_fora_do_ficheiro`, `a_folha_nova_do_player_mantem_as_animacoes`,
`uma_sprite_ampliada_continua_ampliada`…); app-components
`factory_images::a_biblioteca_oferece_a_fabrica_pelo_nome_mesmo_sem_uso` (com controlo); asset-browser
`seam_choose` (2); inspector `seam_texture_slot` (4, actualizado). **Mutação 12/12 sangra** (a 3 sobrevivia e
ganhou o gate da escala). `the_shell_only_shrinks` verde (dentro da folga); `fn_loc_caps` verde
(`fase_snapshots_publish` voltou a ≤ 200 — estava a 207 desde a onda 07). clippy `-D warnings --all-targets` das
9 crates tocadas limpo; `cargo fmt` limpo.

**Suíte de impacto** (`nextest-impacted`, `PH2D_SUITE=onda`): 3 vermelhos FORA da onda:

- (a) `ph2d-panel-registry-init` `a_carga_de_comandos_de_um_painel_so_encolhe`: inspector 92 contra tecto 89
  (`crates/ph2d-panel-registry-init/tests/it/quantas_entradas_tem_cada_painel.rs` ≈ l.1102). NÃO é da onda 12 (a
  ranhura Image não conta, é só hit); vem da onda 07 (cartão Animation: `insp_anim_open_editor`,
  `insp_anim_play_row_00` + 1 por identificar); o tecto foi escrito no main em `d482366656` (24/10). **Medir em
  `d482366656` e decidir** (subir com ⬆️ e motivo «capacidade nova», ou declarar composto) no fecho da linha.
- (b) `ph2d-tool-painter` `pular_a_pincelada_e_esperar_nao_cumpre_nenhuma`: TIMEOUT 600 s = carga (3/3 verde
  isolado, ~75 s, loadavg 128–232).
- (c) `ph2d-host-code-scenes` `a_cena_4_conduzida_passo_a_passo`: não reproduz (3/3 verde isolado e com a binária
  inteira).

## Smoke (para o Enio)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=4 ./target/smoke/ph2d-host-desktop`
2. Na Hierarchy clique **Hills**; no Inspector, por baixo de **Visible**, a linha **Image** mostra a miniatura e
   o nome «Hills».
3. Clique nessa caixa: abre a janela **Assets** só com imagens e a faixa «Choose the image for “Hills”».
   Escreva «montes» na busca e clique **montes_por_do_sol**: a janela fecha e os montes ficam cor de laranja,
   repetidos de ponta a ponta.
4. **Ctrl+Z** volta aos montes azuis; **Ctrl+Shift+Z** (ou Redo) devolve.
5. Clique **Player**, na linha Image clique e escolha **heroi_azul** (ou arraste o cartão da janela Assets para a
   caixa Image): o boneco fica azul, do mesmo tamanho. **Run ▸ Play in a Window**: corre e salta com as
   animações de sempre.
6. **File ▸ Save**, feche, abra o ficheiro: as duas imagens continuam.
7. Deu errado se: a caixa Image não aparece; a janela mostra prefabs; os montes ficam com um buraco ou um só
   monte esticado; o boneco muda de tamanho ou fica parado sem animar; depois de abrir o ficheiro volta a
   imagem antiga.

## ABERTO

- **12b Flip** (`FlipEntityMap` 1:N, cópia mantém a referência, campo Drawing) **e Vector** (campo Drawing =
  componente + `ReplaceSelection`; miniatura vetorial vazia) — plano §12b.
- A miniatura de um azulejo de fábrica mostra as 3 cópias do atlas (a imagem guardada é a tira 3×1).
- A GPU de uma imagem só-individual escolhida não se liberta (como antes: o undo precisa dela).
- O *Sprite Editor* (cortar a imagem em quadros no asset) fora da onda.
- O condutor do Play (`PH2D_GAME_SMOKE_DRIVE`) não arranca na cena 4 (foto do jogo não tirada).
- `Esc` não cancela o seletor (só o ✕ / fechar o painel).
- Suíte de impacto: o vermelho (a) acima (tecto do inspector 89 vs 92) decide-se no fecho da linha.
- Herdado (handoff 11): `AnimRules` sem nome de script nem lei em `property_laws::LEIS` · barra de largada das
  regras sem seam · «This frame» `1.330x` vs `1.33x` · R1 vista dupla ao vivo · View ▸ Focus mode morto ·
  `ph2d.disconnect` · a faixa do editor não rola ao arrastar junto à borda · corpo sem troço recto (bola) a
  agachar abaixo do piso pode encostar e travar (sem gate) · suíte de impacto (integrador).
- Herdado (menor): o desenho é mais largo que a cápsula (±0,27 de pé, ±0,30 agachado, raio 0,2) — encostado à
  parede o capacete pinta uns cm por cima dela.
- Herdado: regra derivada por NOME de convenção — renomear a tag `crouch` tira a regra derivada (a 1.ª edição do
  cartão grava as regras e daí o rename leva-as).
- Herdado: o editor (fora do Play) mostra as camadas na pose autorada (a paralaxe só anda com a câmara do jogo)
  · a Background Layer escalada pelo Transform desalinha a repetição (o ladrilho é em metros do mundo) ·
  `CameraLimits` do nível 1 continua a max x 18 (não medido).

## PRÓXIMO

O smoke do dono decide. Depois: **12b** (Flip/Vector pela mesma porta) ou **J5 moedas**
([plano 03](../03_plano_jogo_de_plataforma.md): `Area` + sinal «entrou», `Audio Player`, HUD `Label`, destruir
objecto; medir primeiro o que existe), conforme o dono.
