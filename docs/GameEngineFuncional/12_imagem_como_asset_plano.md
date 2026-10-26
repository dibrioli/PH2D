# 12 — Trocar a imagem de um objecto como na Unity (plano, 26/10)

Report do dono no smoke do J4 (memória `project_image_is_a_project_asset_reference_2026_10_26`): *«não posso
mudar a imagem da sprite do Background Layer — a Godot resolve de forma muito apropriada; somos mais parecidos
com a Unity»*. Vale para Sprite, Background Layer, Animated Sprite, o Player (a folha e as animações por
intervalos), e o conteúdo do Flip, do Vector e do Pixel (linha paralela). Antes do J5.
Antecessor: [11](11_camara_e_parallax_plano.md) · [handoff 11](handoffs/HANDOFF_CONTINUACAO_2026-10-25_11_CAMARA_E_PARALLAX.md).
Lei da partilha: `feedback_the_content_of_an_asset_is_shared_only_which_asset_is_per_object` — **os pixels são
do asset (partilhados); QUAL asset é do objecto**.

## Medido (antes de desenhar)

**Quase tudo existe; o que falta é a fábrica na biblioteca, a porta que REFERENCIA, e o seletor.**

| Peça | Onde | Estado |
|---|---|---|
| Referência durável de uma imagem | `Sprite.source = Atlas{key}` + `atlas_asset_map` (`key → AssetId`, regravado nas MESMAS células ao abrir: `project_assets.rs::materialize_assets`); `SpritePixels(AssetId)` (blake3) para `Individual` | ✅ durável; o `.ph2dproj` grava os pixels UMA vez por imagem (`SavedAsset`, `project_sprite_pixels.rs`) |
| Arte de fábrica | `ph2d-render/src/atlas/fabrica.rs`: chaves `u32::MAX-2..-9`, `FACTORY_ART` (8), `factory_pixels` (SVG rasterizado) | ⛔ **fora da biblioteca**: o índice só lista o que está no `atlas_asset_map` (*«nunca pelo arranque»*, `asset_index_build.rs::texture_of`) ⇒ os montes e o herói não se podem escolher. **É a causa directa do report.** |
| Navegador de assets (miniaturas, busca, filtro de família, arrastar) | `ph2d-panel-asset-browser` (`state.rs`, `paint_thumb.rs`, `event.rs`), índice `ph2d-asset-index` | ✅ |
| Nome único de imagem (o mesmo que o script escreve, `sprite.image`) | `ph2d-app-components/src/library_names.rs` | ✅ (a fábrica não tem nome) |
| Ranhura de textura no Inspector | `sections/render_source.rs:453` `INSP_RENDER_TEXTURE_SLOT`; clique ⇒ `OpenAssetBrowser`; queda de `Image` ⇒ `DropAction::RetextureSprite` ⇒ `asset_drop_apply.rs::drop_retexture` | ⚠️ mostra texto (`Atlas · key 4294967286`), sem miniatura; o clique abre o navegador mas **não escolhe**; a queda **copia os pixels** (`commit_edited_texture`, o funil das FERRAMENTAS de imagem) ⇒ uma textura GPU nova por sprite e o `SpriteGrid`/`UvTransform` da imagem antiga ficam (os montes de fábrica são 3×1 com a célula do meio ⇒ uma imagem nova mostraria só o terço do meio) |
| Arrastar uma imagem para a cena cria o objecto | `drop_spawn_image` | ✅ |
| Folha + animações por intervalos | `SpriteGrid` + `SpriteAnimations` (tags `[from,to]` sobre células), componentes à parte da imagem | ✅ — trocar a imagem não as apaga |
| Player | `object_add.rs::anima`: `HERO_SHEET_KEY` fixo | ⚠️ ninguém escolhe a folha |
| Background Layer | `veste_camada`: ladrilho por `ladrilho_m(art)` (texels da fábrica) | ⚠️ o ladrilho não sabe ler uma imagem do projecto |
| Desfazer | `ProjectState` captura o mundo por diff | ✅ — mexer nos componentes basta |
| Flip | `FlipObjectRef(u64)` = `FlipObjectId` **durável**; `FlipEntityMap` 1:1 (`ph2d-flip-entities/src/entities.rs:20`); a cópia de um objecto Flip **larga** a referência (`instance_docs.rs:61`) | ⛔ conteúdo 1:1 com o objecto, sem partilha |
| Vector | um path por entidade (`VecPathRef` = `VecPathId`); a partilha existe pelos **Componentes** (mestre com peças vetoriais, instância com cópia + sync, `instance_docs_tests.rs:66`) e a troca por outro componente já é um verbo (`AssetCardAction::ReplaceSelection`) | ⚠️ miniatura do componente vetorial vazia (só lê texturas) |
| Pixel (linha paralela) | `PixelSprite(u32)` por objecto, pixels em `project.pixel_sprites` (P `shells/desktop/src/project.rs:186`) | contrato abaixo |

**Oráculo Godot 4.7.2 (MIT, `pacman -Qi godot`)**, corrido por [`ferramentas/textura_godot.gd`](ferramentas/textura_godot.gd)
(`PH2D_GPU=1 bash scripts/ph2d-run.sh godot --headless -s … -- <pasta>`), imagens nossas:

| caso | resultado |
|---|---|
| `Sprite2D` 8×4 frame 13, `scale 2`, *Repeat*; troca a textura por uma de outra largura | `hframes 8 · vframes 4 · frame 13` **mantidos**; rect 32×32 → **48×32** (segue a imagem); `scale 2` e *Repeat* mantidos |
| folha MENOR que o frame | frame 40 **não é preso** |
| `Parallax2D.repeat_size` depois de trocar a textura da filha | **não muda** (300, à mão) |
| gravar a cena com duas sprites da mesma textura | **1** `ext_resource`, 2 `ExtResource(...)`, 0 `sub_resource`: referência, uma vez |
| carregar o recurso 2× | a mesma instância (conteúdo partilhado) |

A Unity (proprietária, referência de produto, não se corre) faz o mesmo no *Sprite Renderer*: o campo guarda a
referência; em *Simple* o tamanho segue a imagem (a escala do objecto fica); em *Tiled* o tamanho desenhado fica
e os ladrilhos seguem o tamanho nativo da imagem; o corte em quadros é da IMAGEM (*Sprite Editor*).

## Desenho

### A porta: «qual conteúdo este objecto mostra»

UMA porta, **sem componente novo e sem degrau de `PROJECT_SCHEMA`**: a referência de cada família já existe e
é durável. O que a porta junta é a pergunta e a escolha:

```text
ContentRef = Image(AssetId)          // Sprite, Background Layer, Animated Sprite, Player (folha)
           | Flip(FlipObjectId)      // 12b
           | Component(StableId)     // Vector (e qualquer peça composta): a troca por outro componente
           | Pixel(u32)              // contrato da line/PixelArt (abaixo)
content_of(world, e) -> Option<ContentRef>
choose(world, e, ContentRef, facts) -> Result
```

- **Imagem = `AssetId`** (blake3, o nome que o `SpritePixels`, o `DragPayload::Image`, o índice e o script já
  usam). Escolher NÃO copia pixels: fábrica ⇒ `Atlas{chave de fábrica}`; importada no atlas ⇒ `Atlas{key}` (a
  mesma célula); só-individual (16 bits, grande) ⇒ textura GPU + `SpritePixels(id)`. Ficheiro antigo = os
  mesmos componentes ⇒ abre igual.
- **A fábrica entra na biblioteca:** os pixels de cada `FACTORY_ART` entram no `AssetDb` à 1.ª pergunta
  (`factory_assets: chave ↔ AssetId`, fora do `atlas_asset_map` ⇒ o save **não** os grava: o programa
  traz-nos); o índice lista-os SEMPRE (usados ou não), com nome traduzido (`asset.factory.*`). `FactoryArt`
  ganha `name` (chave i18n) e `texels_per_m` (o tamanho nativo; hoje implícito por arte: herói 192, chão 256,
  bandeira/moeda 128, fundo 128).

### As leis da troca (pura, `ph2d-app-sprite-anim::image_choice`, gates sem placa)

1. **Escala fica, o tamanho segue a imagem** (Godot medido + Unity *Simple*): `novo = antigo × nativo_novo /
   nativo_antigo`, por eixo. ⚠️ Muda a escolha do `drop_retexture` (*«o tamanho não muda»*): o produto de
   referência do dono decide a lei.
2. **O corte em quadros é da imagem, senão é do objecto:** imagem com grelha própria (fábrica: herói 8×4,
   moeda, bandeira; azulejo ⇒ a célula do meio) ⇒ essa; imagem sem grelha num objecto ANIMADO
   (`SpriteAnimations`) ⇒ a grelha do objecto fica (Godot: `hframes`/`frame` mantidos) e as animações por
   intervalo também; num objecto sem animação ⇒ sem grelha.
3. **Camada que repete (`ScrollRepeat`) = Unity *Tiled*:** o ladrilho segue a regra 1 (escala da camada × o
   nativo da imagem nova); o número de ladrilhos sobe até cobrir pelo menos a largura antiga; `UvTransform`
   [n, 1] e `ScrollRepeat.tile` reescritos (melhor que a Godot, onde o `repeat_size` fica errado à mão);
   `ScrollFactor`, `ZIndexOverride`, *Repeat* intactos.
4. **Sprite com *Repeat* sem `ScrollRepeat`** (blocos do nível): tamanho e repetições ficam (Godot: *region* e
   *repeat* do nó).
5. **Um passo de Desfazer**, e a escolha é a mesma por qualquer porta (Inspector, queda, seletor, script).

### Inspector e seletor (tudo pelo editor)

- A ranhura da secção da imagem passa a mostrar **miniatura + nome** da imagem (o nome único da biblioteca);
  a estratégia (Atlas/Individual) continua na linha de baixo.
- **Clique na ranhura = seletor** (o *object picker* da Unity / *Quick Load* da Godot): o navegador de assets
  abre em **modo escolher** para aquele objecto — só imagens, busca com foco, faixa *«Choose the image for
  Hills»* com ✕; um clique numa imagem escolhe e fecha; `Esc` cancela. ⛔ Não é uma segunda grelha: é o mesmo
  painel com um alvo.
- **Arrastar** uma imagem do navegador para a ranhura = a mesma escolha (a queda deixa de copiar pixels).
- Ids novos no módulo `ids` da crate que os lê; texto `ph2d-i18n`; cores/medidas por tokens.

### 12b — Flip e Vector pela mesma porta (depois do smoke das imagens; o 12a muda a porta que o 12b usa)

- **Flip:** o objecto Flip ganha o campo *Drawing* (miniatura + nome) e o seletor lista os desenhos Flip do
  projecto; escolher re-aponta o `FlipObjectRef`. Pede o `FlipEntityMap` de 1:1 para 1:N (vários objectos
  mostram o mesmo desenho; editar o desenho muda todos) e a cópia a manter a referência em vez de a largar.
  Medir o custo na abertura do 12b.
- **Vector:** a partilha já é o Componente; o campo *Drawing* de uma instância vetorial mostra o componente e o
  seletor lista os componentes ⇒ escolher = o verbo `ReplaceSelection` que existe. Falta a miniatura vetorial do
  cartão (hoje vazia).

### Contrato para a line/PixelArt (não se toca o código dela)

O Pixel pluga por **append**: (1) variante `ContentRef::Pixel(u32)` (o `PixelSprite`); (2) `content_of` devolve-a
para quem tem `PixelSprite`; (3) uma fonte do índice com os sprites Pixel do projecto (nome + miniatura do
quadro 0) na família *Pixel*; (4) `choose` re-aponta o `PixelSprite` (conteúdo partilhado: pintar muda todos os
que o mostram); (5) o campo do Inspector é o MESMO widget (miniatura + nome + seletor). Colisão medida
(`collision-surface.sh`): A e P tocam `render_loop/mod.rs`, `fase_object_add.rs`, `fase_selection_highlight.rs`,
`fase_snapshots_publish.rs`; P e V `fase_bus_inspector.rs` (esta onda acrescenta um braço lá); A e V
`undo_app.rs`. O Vector (V) não colide com a porta (parte de `9ea782138379`).

## Gates

- **Trocar a imagem muda o que se desenha (pixel):** nível 2, Hills ← outra imagem (importada) pela acção do
  Inspector; o pixel da camada passa à cor da imagem nova; CONTROLO sem a troca fica.
- **Sobrevive a salvar/abrir e ao Ctrl+Z:** a referência volta igual depois de gravar e abrir; o Desfazer devolve
  a imagem, o tamanho, a grelha e o ladrilho antigos num passo.
- **A folha nova mantém as animações por intervalo:** Player ← folha importada 8×4: tags, `frame`, animador iguais;
  a fábrica volta pela mesma porta.
- **Repetição/parallax intactos:** a camada com a imagem nova cobre a vista em toda a faixa da câmara do nível 2
  (o gate do plano 11 sobre a camada trocada), `ScrollFactor` igual, `ScrollRepeat.tile` = o ladrilho novo.
- **Ficheiro antigo igual:** os componentes de um projecto gravado antes desta onda saem byte a byte iguais; as
  chaves de fábrica e a ordem de `FACTORY_ART` não mudam; as fábricas não entram no `SavedAsset`.
- **A fábrica está na biblioteca:** as 8 imagens no índice com nome, sem duplicar as importadas.
- **Seletor (seam, `ph2d-ui-testkit`):** clique na ranhura ⇒ navegador em modo escolher só com imagens ⇒ clique
  numa ⇒ a acção de escolha com aquele objecto ⇒ painel fecha; `Esc` cancela sem escolher.

## Smoke (nível 2)

Trocar a imagem dos montes e a folha do Player pelo Inspector (seletor e arrastar), Play, salvar, abrir, Ctrl+Z.
Fotografado antes de mandar.

## Fora desta onda

O *Sprite Editor* (cortar uma imagem importada em quadros no asset, à Unity) — hoje o corte de uma folha nova
continua a ser proposto pelo editor de animações (`propose_after_drop`); J5 (moedas).
