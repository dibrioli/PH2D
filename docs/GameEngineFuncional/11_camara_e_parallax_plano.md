# 11 — A câmara presa ao nível e o fundo em parallax (J4, plano, 25/10)

Pedido (J4 do [plano 03](03_plano_jogo_de_plataforma.md)): a câmara do jogo segue o jogador **presa aos
limites do nível**, e o fundo anda em **parallax** (o céu e uma ou duas camadas de cenário ao longe, mais
devagar que o primeiro plano). Tudo pelo editor; arte no estilo do [plano 06](06_arte_vetorial_plano.md).
Antecessor: [10](10_arte_de_agachar_plano.md) · [handoff 10](handoffs/HANDOFF_CONTINUACAO_2026-10-25_10_ARTE_DE_AGACHAR.md).

## Medido (antes de desenhar)

**A câmara JÁ faz o J4 inteiro, e o nível 2 já a usa.** Nada disto se reconstrói:

| Peça | Onde | O nível 2 |
|---|---|---|
| `GameCamera` (altura da vista, `offset`, prioridade, dolly) | `ph2d-ecs/src/camera_2d.rs:88` | `height_world` 8 |
| `CameraFollow` (alvo por nome, amortecer, zona morta, antecipação, `offset`) | `camera_2d.rs:130` | segue «Player», zona morta 0,25, `offset [0, 1]` |
| `CameraLimits` (min/max; nível menor que a vista ⇒ centro da caixa) | `camera_2d.rs:169`, `clamp_axis_to_limits` | `[-13, -1]`–`[18, 9]` |
| Lei do quadro: zona morta → amortecer → limites, 1× por quadro | `ph2d-app-components/src/camera_2d.rs:146` | — |
| A vista é tomada no Play (janela e editor) | `janela_do_jogo.rs:47` liga `game_camera_preview`; `fase_game_camera.rs:100` | — |
| Objecto «Camera» no Add; Follow/Limits pelo Inspector | `object_add.rs:13`, `camera_inspector.rs` | — |
| Parallax: `ScrollFactor{k}`, `ScrollRepeat{tile}`, `ScrollLimits`, `ScrollMotion` + ponte + secção do Inspector | `ph2d-ecs/src/scroll_factor.rs`, `parallax_bridge.rs:84`, `parallax_inspector.rs`; fase `fase_paralaxe.rs` (depois da câmara, antes do extract) | **nenhuma camada** |

- **Os dois `offset` não são o mesmo.** O do `CameraFollow` soma-se à MIRA (antes dos limites ⇒ preso); o do
  `GameCamera` soma-se à VISTA depois dos limites (como o abanão). **Oráculo Godot 4.7.2 (MIT)**, corrido por
  [`ferramentas/camara_godot.gd`](ferramentas/camara_godot.gd) (`godot --headless`): encostada a um limite,
  o `offset` da `Camera2D` PASSA o limite (em cima: vista a −1100 com limite −1000; à direita 2100 contra
  2000); nível menor que a vista ⇒ ignora os limites; `Parallax2D.scroll_scale 0,5` ⇒ a camada anda metade
  da câmara. ⇒ o PH2D já reproduz o alvo; **a lei não muda**. O nível 2 usa o `offset` do Follow ⇒ a vista
  fica dentro dos limites.
- **O céu do nível 2** é um `Sprite` (quadrado branco tingido `#C2E3E8`, 40 × 16 m em `[2,5, 4]`,
  `ZIndexOverride(-1)`) sem parallax. ⚠️ `#C2E3E8` é a cor dos MONTES no fundo do Kenney, não a do céu.
- **Referência de conteúdo:** `Tilemap/tilemap-backgrounds_packed.png` do mesmo zip (sha256 `d01a196d…`, CC0):
  fundo azul = céu `#DFF6F5` · faixa de nuvens `#FFFFFF` · montes `#C2E3E8` com árvores-silhueta. Uma imagem
  só no pacote; aqui sai em três planos.
- **A repetição só ENVOLVE a posição** (`ScrollRepeat` ⇒ um inteiro de ladrilhos relativo à vista): quem
  desenha as cópias é a própria sprite, mais larga que a vista + 1 ladrilho, com `UvTransform` + *Repeat*
  (o que os blocos do nível já fazem, com a cópia do MEIO contra o fio do mip).
- **Faixa da câmara no nível 2** (vista 8 m, 16:9 ⇒ meia-largura 7,11): centro x ∈ `[-5,89, 10,89]`,
  y ∈ `[3, 5]`.
- Atlas 8192 px; chaves de fábrica livres: `u32::MAX-8`, `-9`.

## Desenho

1. **Arte** (`arte_svg.py`, estilo do plano 06, paleta do fundo do Kenney): `nuvens.svg` (8 × 2 m, faixa de
   nuvens brancas com UM tom mais escuro por baixo, repete na horizontal) e `montes.svg` (8 × 4 m, montes
   `#C2E3E8` com o tom escuro e duas árvores-silhueta; o corpo desce cheio até à borda de baixo para nunca
   mostrar o céu por baixo). `FactoryArt` `CLOUDS_TILE` (`u32::MAX-8`) e `HILLS_TILE` (`u32::MAX-9`), azulejos
   3 × 1 a 128 px/m, **no FIM** de `FACTORY_ART` (6 → 8).
2. **Objecto novo no Add ▸ Game: «Background Layer»** (append em `ph2d-app-game::object_add::ENTRIES`, 5 → 6):
   os montes de fábrica numa sprite de 4 ladrilhos com *Repeat*, `ScrollFactor k [0,4, 0,4]`,
   `ScrollRepeat` com o ladrilho, `ZIndexOverride(-1)`. Dica ilustrada: três faixas (céu, montes, chão) com
   setas de comprimento diferente. O resto (a fracção, a imagem, a ordem) edita-se no Inspector que já existe.
3. **Nível 2:** o céu passa a ser uma Background Layer `k [0, 0]` (parada no ecrã) na cor do céu do Kenney
   `#DFF6F5`, `ZIndexOverride(-3)`; **Clouds** `k [0,15, 0,15]` (−2) e **Hills** `k [0,4, 0,4]` (−1), cada uma
   autorada para cair no sítio na vista do início (`autorada = no_ecrã − centro₀·(1−k)`). A câmara e os
   blocos não mudam.

Sem campo gravado novo, sem componente novo ⇒ **sem degrau de `PROJECT_SCHEMA`**.

## Gates

- **A câmara não mostra fora dos limites (pixel):** o nível 2 com o jogador levado a cada ponta (parede da
  esquerda, fim da direita, plataforma alta, fundo do buraco) e a câmara a apanhá-lo; para as janelas
  16:9, 4:3 e 21:9, todo pixel da janela mapeia para dentro de `CameraLimits` (±½ px). CONTROLO: sem
  `CameraLimits`, o mesmo percurso mostra fora.
- **O parallax anda a fracção autorada:** duas vistas com Δc; cada camada anda `(1 − k)·Δc` no mundo
  (= `k·Δc` no ecrã); o chão anda 0. Em toda a faixa da câmara, as nuvens e os montes cobrem a largura da
  vista (sem buraco à esquerda/direita) e os montes não deixam o céu ver-se entre eles e o chão.
- **Ficheiro antigo igual:** os texels das 6 artes antigas têm o hash medido ANTES desta onda; as chaves e a
  ordem delas em `FACTORY_ART` não mudam.
- Add: a Background Layer nasce com os componentes acima; a dica tem desenho; i18n das chaves novas.

## Smoke

`PH2D_GAME_SMOKE=2`: Play; correr para a direita e saltar para a plataforma alta: o céu fica parado, as
nuvens mal andam, os montes andam devagar, o chão passa à velocidade do boneco; nas duas pontas do nível a
câmara pára e não mostra nada de fora. Fotografado antes de mandar.

## Fora desta onda

`ScrollMotion` (as nuvens a derivar sozinhas) existe e fica para o artista; parallax vertical diferente do
horizontal; a lei do `offset` da `GameCamera` (igual à da Godot).
