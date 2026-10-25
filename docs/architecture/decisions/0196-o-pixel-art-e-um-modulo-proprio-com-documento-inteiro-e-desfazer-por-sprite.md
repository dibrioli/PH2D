# ADR-0195 — O Pixel Art é um módulo próprio, com documento inteiro e desfazer por sprite

- **Status:** Accepted — o PRODUTO é decisão do DONO (25/10: módulo NOVO e autocontido, sem Painter; o
  Aseprite é a espinha das etapas); o desenho técnico é delegado.
- **Data:** 2026-10-25
- **Linha:** `line/PixelArt` (onda P0)
- **Amparo:** [estudo do Aseprite](../../PixelArt/01_estudo_aseprite.md) · [plano do módulo](../../PixelArt/02_plano_modulo.md)
  · o MVP web e o contrato JS↔Rust em `docs/Pixel Art/02_spec_engine_e_porte.md` (não versionado).

> ⚠️ **Número RECONTADO** (§5.0): escrito como `0192` em 25/10 (o `main` estava em `0191`); na integração
> de 29/10 o `main` já tinha `0192`..`0194` (BlenderConstraints), e este ADR passou a **`0195`**.

## Contexto

O dono quer pixel art e animação de sprites ao nível do Aseprite, e decidiu que o módulo não depende do
Painter (que está em desenvolvimento). O MVP web provou as leis numéricas (kernels inteiros, linha
AB == BA, um gesto = um passo, save byte-determinístico), mas mapeava o porte para planos sidecar no
`PainterTool` — essa parte está revogada. Faltava decidir: onde vive o documento, como entra no
projecto, como se desfaz, como se desenha, e por onde a entrada chega.

## Decisão

1. **Cinco crates, a seta num só sentido.** `ph2d-pixel-core` (documento e leis, **sem dependências**,
   gate `the_pixel_core_declares_no_dependency` + censo de relógio/SO/hash) ← `ph2d-pixel-io` (vazia na
   P0 salvo `frame_rgba`; `.ase`/folha/GIF/paletas na P9) ← `ph2d-pixel-render` (a vista: imagem
   composta com id estável e amostragem nearest, xadrez, grelha, escada de zoom inteiro) ←
   `ph2d-panel-pixel-tools` (ferramentas, cores, paleta) ← `ph2d-app-pixel` (a família). ⛔ Nenhuma
   delas depende de `ph2d-app-painter` nem do contrato `Tool=12`: **as ferramentas são internas à
   família** (um `enum Tool`), não ferramentas de bitmap da casa.
2. **O documento é inteiro e esparso.** `Sprite` = modo de cor por sprite (**RGBA, Grayscale e Indexed
   desde o 1.º dia** — o MVP era só índice), paleta (DB32 por omissão, a que o Aseprite 1.3.18 abre),
   camadas (a Background do Aseprite: opaca, a borracha pinta a cor de fundo nela), frames com duração,
   cels com caixa própria (fora dela é transparente). Ids crescentes, nunca reciclados; iteração em
   `BTreeMap`.
3. **Toda mutação passa por `apply(sprite, op) -> inverso`.** O primeiro op é `Patch` («põe a caixa do
   cel em `bbox`, guardando o que já estava, e escreve estes bytes em `rect`»): o inverso devolve o cel
   ao byte, inclusive a caixa que o traço alargou. O traço acumula células entre premir e largar
   (`Stroke`, linha canonicalizada desde o último ponto) e vira **um** `Patch` — um gesto = um passo.
4. **O desfazer é POR SPRITE, fora da fila do `ProjectState`.** Cada documento tem `History` (pilhas
   de inversos, tecto **em bytes**: 64 MiB por sprite, o do MVP — o recurso é a RAM; uma contagem é
   multiplicador, ADR-0117). Em Pixel Sprite ▸ Draw o `Ctrl+Z`/`Ctrl+Shift+Z`/`Ctrl+Y` vão a ele; fora
   do Draw, à fila do mundo (criar, mover, apagar o objecto). O documento não entra no `ProjectState`:
   o diff por quadro compararia o blob inteiro a cada quadro com entrada (256 KiB por cel RGBA 256²) e
   um traço contínuo pagaria isso a cada amostra.
5. **O objecto é um marcador; o documento é um blob.** `ph2d_ecs::PixelSprite(u32)` (registado ⇒ viaja
   no `WorldSnapshot`, no undo do mundo e no save) é a chave do documento; `ObjectKind::PixelSprite`
   deriva dele (fora do `DRAWABLE` na P0: blend/máscara de objecto ainda não o lêem). Os documentos
   gravam-se no **último campo** do `ProjectFile`, `pixel_sprites: Vec<u8>` (formato `PH2DPIXEL`
   major/minor com registos etiquetados — o molde `.riv` do `ph2d-macro`: etiqueta desconhecida salta-se
   dentro da MAJOR), só os dos objectos vivos; um blob ilegível recusa o projecto inteiro. Um degrau:
   `PROJECT_SCHEMA` 193 → 194. Duplicar um objecto dá à cópia uma cópia do documento (o mais velho
   por `StableId` guarda o original e o desfazer).
6. **A entrada é uma composição fina na shell.** `input_dispatch/despacho_pixel.rs` encaminha premir/
   largar (antes do Flip), mover (antes dos arrastos de topo), a roda (antes do pan do quadro) e o
   teclado (depois do quadro, salvo campo de texto com foco) a `ph2d_app_pixel::input`, que decide e
   devolve «consumido». Não há porta genérica de ponteiro para modos na casa (o Painter usa downcast
   sobre o `CanvasPaintTool`, congelado); criá-la é outra decisão.
7. **Zoom inteiro.** Em Draw uma célula ocupa um número inteiro de píxeis de ecrã: a roda anda pela
   escada `1…96` px/célula à volta do cursor, e depois de cada zoom/pan a câmera desloca-se para a
   aresta da célula (0,0) cair num píxel inteiro. Entrar em Draw num sprite novo enquadra-o (o maior
   degrau que cabe em 60 % da janela). A roda do RATO anda um degrau por linha; a do TRACKPAD
   (`PixelDelta`, o gesto aos bocados) soma os píxeis e anda um degrau quando eles dão o factor dele
   pela lei da roda do viewport (`Camera2d::wheel_px_for`) — o gesto aproxima como fora do Draw.
   O degrau lê-se no rodapé, nunca num aviso por degrau.
8. **Desenho na P0: a cena de sobreposição do viewport**, por cima dos sprites do mundo, numa imagem
   composta na CPU com `StableImage` (o atlas do Vello reusa-a até ao próximo traço). O clique e a caixa
   do gizmo de objecto perguntam à mesma geometria que a tinta usou (`draw::pick_at`/`half_extents`).

9. **Emenda 25/10 (report do smoke do dono).** O modo chama-se **Draw** (o par D6 é
   `(PixelSprite, Draw)`, como o Flip; nasceu Edit). Em Draw as ferramentas e as duas amostras de cor
   moram na **coluna da esquerda**, a mesma porta das barras do vetor e do pintor (`ToolBars`, marca
   `pixel_rail`: a fila de cima guarda o seletor de modo e o vetor inactivo não as apaga); o vetor
   larga a mão enquanto o objecto activo é um Pixel Sprite, em Draw ou seleccionado em Object
   (`WidgetStore::family_owns_canvas`, lido nos dois sítios que decidem se o vetor está activo). O painel PIXEL da direita fica
   com as cores e a paleta. As amostras abrem o seletor de cor partilhado e a família lê-o de volta.

10. **Emenda 25/10 (onda P3 — camadas).** A camada ganha `opacity`, `blend` (os 19 modos, ids do
    `.ase`), `locked`, `reference`, `kind` (Image · Group{collapsed}) e `parent` (a árvore; a ordem do
    `Vec` é o z entre irmãos). Toda mudança de lista é UM `Op::Layers { layers, cels }` — a lista
    inteira e os cels que mudam com ela —, validado pelas leis da árvore (ids já emitidos, pai é
    grupo, sem ciclo, Background única/primeira/de imagem, nenhum cel órfão nem em grupo); as acções
    do menu Layer são construtores puros em `ph2d_pixel_core::layers`. Formato `PH2DPIXEL` 1.1 (campos
    novos do registo Layer; um 1.0 lê-se com os padrões). **Leis medidas no Aseprite Trial observado**
    (103 ficheiros `.ase` nossos, 6 144 células cada, todos EXACTOS, fixtura `oracle_trial_blend.json`):
    a cor final é `d + trunc((B(d,s) − d) · mul(Sa, opacidade) / 255)`; Dodge/Burn/Divide arredondam a
    divisão; Hue/Saturation/Color/Luminosity (W3C, `f64`) truncam; **o grupo é pass-through e o seu
    modo e opacidade NÃO entram** (o diálogo de propriedades de um grupo no Trial só tem o Nome — o
    painel faz o mesmo); a opacidade do cel segue a lei da camada; o Indexed mistura em RGB e não volta
    à paleta. A composição é na CPU por janela (`compose_window`); a GPU com paridade fica para quando
    a medição da cena a pedir (o plano punha-a aqui). Copy e *Layer via Copy/Cut* pedem selecção, como
    no Trial: até a P6 a selecção é só *Select All* (`Doc::selected_all`).

## Alternativas recusadas

| Alternativa | Porque não |
|---|---|
| Planos sidecar no `PainterTool` (o porte do MVP) | O dono separou os módulos; acoplaria o Pixel ao ciclo de vida de outra linha. |
| Ferramentas sob o contrato `Tool=12` | Contrato congelado das ferramentas de bitmap; o Pixel tem as suas leis (inteiras, por célula). |
| O documento dentro de um componente (como o `Macro`) | Entraria no `ProjectState`: o diff por quadro e o snapshot pagariam o blob inteiro a cada amostra de traço. |
| Um `Sprite` derivado (render pelo passe de sprites do mundo) já na P0 | O marcador `Sprite` faria do objecto uma IMAGEM (`kind_of`), com os modos e ferramentas de imagem a aplicarem-se a ele; fica para a P14, com o tipo próprio. |
| Grupo ISOLADO (compor os filhos sobre transparente e misturar com o modo/opacidade do grupo) | Medido no Trial: 1 663–1 979 de 6 144 células certas; pass-through dá 6 144/6 144 (P3). |
| Opacidade de camada em Indexed como cobertura Bayer 8×8 (a lei do MVP) | O Trial mistura em RGB: 6 144/6 144; voltar à paleta mais próxima acerta 183–189 (P3). |
| `clip` (máscara da camada de baixo, a do MVP) | O Aseprite não a tem (grupos); fica fora (P3). |
| Fila única do editor com um passo por traço | Possível por diff, mas o custo do diff e a semântica do Aseprite (o desfazer é do sprite) apontam para fila própria. |

## Consequências

- O `Ctrl+Z` muda de dono com o modo: em Draw é do sprite. É o idioma do Aseprite (desfazer por
  documento) e do Motion/Timeline da casa (filas próprias).
- **Abertos, por onda (plano §5):** a ordem de desenho do mundo (luz, camadas, câmera do jogo), o
  clique por pixel opaco e a ligação às tags/slices/normal map são a P14; a composição na GPU (cel →
  textura, LUT 256×1, 19 modos num shader) chega com as camadas (P3), com paridade bit-a-bit contra a
  composição CPU deste núcleo; a conversão RGB→Gray do núcleo (Rec. 601) é provisória até à P2, contra
  o oráculo.
- O tecto `MAX_SIDE = 65535` é o do formato `.ase` (um `WORD`), não da memória.
