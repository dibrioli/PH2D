# 13 — Isolamento por uso: o plano (26/10)

Antecessor medido: [`13_isolamento_por_uso_medido.md`](13_isolamento_por_uso_medido.md) (os 3 defeitos do dono, a
Unity em batchmode). ⛔ A estratégia Atlas/Individual FICA no objeto, na secção Texture & Atlas (decisão do dono).

## O que o código faz hoje (lido antes de desenhar)

- **O azulejo de fábrica é uma tira de 3 cópias** (`FactoryArt.grid [3,1]`, `tile: true`) e cada OBJECTO escolhe a
  cópia do meio pelo seu `SpriteGrid {3,1,1}` (`ph2d-app-game` `azulejo`, `object_add::veste_camada`). O atlas não
  tem *gutter* (`atlas/mod.rs` §Mipmaps: só o meio texel de `AtlasRegion::uv`); as cópias SÃO o remédio do fio do
  mip. O `AssetDb` guarda a tira ⇒ miniatura com 3 cópias; o Inspector lê `SpriteGrid` ⇒ «H Frames 3».
- **Quatro portas para meter imagem, duas leis:** linha Image, seletor e queda na ranhura passam por
  `image_choice::choose_library_image`; a queda no canvas (`asset_drop_apply::drop_spawn_image`) COPIA os píxeis
  para uma textura `Individual` ao tamanho `px/ppm`; o script (`script_image`, `sprite.image`) sobe uma cópia
  `Individual` e pinta a imagem INTEIRA (sem a célula), no tamanho autorado.
- **O Inspector não sabe o USO do objecto:** H/V Frames + Frame aparecem sempre que não há cartão Animation
  (`render_source.rs` → `sprite_pose::paint_sheet_grid_rows`), também num bloco que repete.

## Desenho

**D1 — O remédio do mip passa para DENTRO da imagem (o *Extrude Edges* da Unity).** A imagem de fábrica de um
azulejo é UM azulejo (`FactoryArt::frames() = [1,1]`, `native_m` = uma célula); só o ATLAS a guarda envolvida pelas
cópias: `TextureAtlas::insert_wrapped(key, célula, cópias)` empacota a tira e regista a região INTERIOR (a célula
do meio) como a região da chave; o `regrow` volta a envolver (a lista `wraps` vive no atlas, porque o `fetch` dá a
célula). O `AssetDb` guarda a célula ⇒ a miniatura mostra um azulejo. Régua: `os_azulejos_repetem_sem_costura`
(as bordas da célula casam) + gate novo `a_regiao_do_azulejo_e_a_celula_do_meio` (região interior, vizinhos =
cópias, também depois de um `regrow`).

**D2 — Ficheiro antigo:** uma sprite `Atlas{chave de azulejo}` com `SpriteGrid == {cópias, meio}` é a forma ANTIGA
de dizer «o azulejo»; `factory_images::forget_extrusion_grids` tira essa grelha ao abrir (e às cenas do jogo deixa
de a pôr). Sem degrau de `PROJECT_SCHEMA`: nenhum campo gravado muda. Gate: um mundo gravado à moda antiga abre e
amostra o MESMO texel.

**D3 — O corte e o tamanho nativo são da IMAGEM** (`image_choice::image_facts`, UMA consulta): a fábrica pela
`FactoryArt` (bandeira 4×1, moeda 6×1, herói 8×4, azulejos 1×1); uma importada pelos píxeis/`ppm` e pelo corte que
os seus utilizadores levam. ⚠️ Decisão: o corte de uma importada NÃO ganha campo gravado — mora nos objectos que a
mostram, e editá-lo num (H/V do Inspector) re-corta TODOS os que mostram a mesma imagem (a Unity medida: re-cortar
no importer muda todos os objectos). Assim o undo e o gravar já o cobrem (`ProjectState` = mundo). Preço: uma
importada que ninguém usa esquece o corte (a Unity guarda-o no `.meta`); a casa dele, se o dono o pedir, é o blob
versionado da biblioteca (`LibraryDoc.catalog_bytes`, sem degrau).

**D4 — O USO do objecto** (`image_choice::image_use`, lei pura): **parada** (sem *Repeat*, sem animações) ·
**repetida** (`TextureRepeat` Enabled/Mirror ou `ScrollRepeat`: bloco, fundo) · **animada** (`SpriteAnimations`).
O host publica-o no `InspectorSpriteInfo` (`image_use`, `image_cells`) e o painel mostra por uso:

| uso | Texture & Atlas mostra |
|---|---|
| parada | Colunas/Linhas DA IMAGEM (re-cortam todos os utilizadores) e Frame só se a imagem tem > 1 célula |
| repetida | nenhuma linha de quadros; imagem com > 1 célula ⇒ AVISO «esta imagem é animada; o bloco repete só o 1.º quadro» + botão **Make it Animated** |
| animada | nada aqui (o cartão Animation mostra a grelha, como hoje) |

A estratégia Atlas/Individual, o Storage e o Reimport ficam como estão.

**D5 — Tornar animado** (`image_choice::make_animated`): tira *Repeat*/`UvTransform`/`ScrollRepeat`, o tamanho
passa a uma célula da imagem × a escala de um ladrilho, põe `SpriteAnimations` com uma animação de todas as células
e o `SpriteAnimator` a tocar em laço. Um passo de undo. A Unity corta em silêncio — aqui o PH2D AVISA.

**D6 — UMA porta:** `image_choice::place_library_image` (nascer uma sprite com a imagem no ponto = uma sprite nova +
`choose_library_image`, escala 1 ⇒ tamanho nativo de UMA célula, a referência do atlas e não uma cópia) serve a
queda no canvas; o script decide pela MESMA lei (`decide`: fonte, tamanho, grelha) e o override do script leva o
resultado — a referência do atlas quando a imagem lá está (sem subir cópia), a célula da imagem.

## Gates (red primeiro, pela porta)

1. `o_bloco_de_fabrica_nao_mostra_quadros` (jogo): Platform High do nível 1 e 2 não tem `SpriteGrid`, uso
   *repetida*, `image_cells == 1`.
2. `largar_platform_no_canvas_e_a_mesma_imagem` (sprite-anim): `Atlas{PLATFORM}`, 1 × 0,5 m, sem cópia de píxeis.
3. `a_bandeira_num_bloco_avisa_e_fica_animada` (sprite-anim): uso *repetida* + 4 células ⇒ conflito; `make_animated`
   ⇒ 1 × 2 m, animação 0..3 a tocar, sem *Repeat*; desfazer num passo.
4. `a_mesma_imagem_pelas_portas_todas`: linha/seletor/ranhura (`choose_library_image`), canvas
   (`place_library_image`) e script (`decide`) dão a mesma fonte, tamanho e grelha numa sprite parada.
5. Gravar/abrir e Ctrl+Z (os de 12 continuam); ficheiro antigo igual (D2); repetição e paralaxe intactas
   (`camara_tests`, `smoke_tests`, os 13 de `image_choice`); `os_azulejos_repetem_sem_costura` +
   `a_regiao_do_azulejo_e_a_celula_do_meio`.
6. Seam do Inspector por uso (`ph2d-panel-inspector`): parada/repetida/animada pintam as linhas da tabela D4; o
   botão Make it Animated emite a acção; a acção chega ao mundo (shell).

## Smoke `PH2D_GAME_SMOKE=4`

Platform High sem «H Frames»; Image ← Flag ⇒ aviso + Make it Animated ⇒ a bandeira ondula; arrastar «Platform» dos
Assets para o canvas ⇒ um azulejo igual ao das plataformas, *Atlas*, 1 × 0,5 m. Fotografado.
