# Pixel Art — o módulo da PH2D

Módulo **novo e autocontido** de pixel art e animação de sprites (decisão do dono, 25/10): não
depende do Painter. A referência de etapas é o **Aseprite**; as leis numéricas vêm do MVP web em
[`../Pixel Art/`](../Pixel%20Art/).

| Doc | O quê |
|---|---|
| [01_estudo_aseprite.md](01_estudo_aseprite.md) | O Aseprite 1.3.18.6 mapeado: menus, ferramentas, inks, paleta, export, o que o Trial deixa fazer |
| [02_plano_modulo.md](02_plano_modulo.md) | Arquitetura (5 crates), leis herdadas, oráculos e as ondas P0–P14 |
| [aseprite/](aseprite/) | Fotos do Trial e a ajuda do CLI |

Estado: **P0 + P0b + P1 na `line/PixelArt`** — [ADR-0196](../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md); o objecto Pixel Sprite (Draw), 14 ferramentas (lápis, spray, balde, linha, curva, retângulo, elipse, contorno, polígono…), pixel-perfect, simetria, Tiled Mode, zoom inteiro, desfazer por sprite, gravar no projecto. Smoke `PH2D_PIXEL_SMOKE=1|2`. Último: [handoff P1](handoffs/HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P1.md).

## As cenas de smoke (`PH2D_PIXEL_SMOKE=n`)

- 1 — 32 × 32 em Draw, uma cara em QUATRO traços (quatro passos do desfazer).
- 2 — P1: 64 × 48, uma casa SÓ com as ferramentas de forma (cada forma um passo); as paredes FECHAM.
- 3 — P2: 64 × 48 INDEXED com a DB32, a bola sombreada pela rampa 4‥7 e a tinta Shading na mão.
- 4 — P2: 96 × 64 RGBA em degradês sobre a DB32 — o Color Mode e os dithers que SE VÊEM.
- 5 — a cena 4 com o diálogo Color Mode ABERTO (Indexed + Floyd–Steinberg).
- 6 — a cena 4 já em Indexed com Ordered 4×4 (um passo do desfazer).
- 7 — P3, as camadas (`smoke_camadas`): 19 modos, opacidade, escondida, trancada, grupo.
- 8 — P4, os frames (`smoke_quadros`): 16 frames, 4 tags, cel a 50 %, papel vegetal.
- 9 — P6, selecção e transformação (`smoke_selecao`).
- 10 — P6b, as correcções do smoke da P6 (`smoke_formas`): formas do Vector em células.
- 11 — P6c, as correcções do smoke da P6b (`smoke_ferramentas`).
- 12 — P6d, a forma VIVA (`smoke_viva`): estrela girada com o gizmo, à espera do `Enter`.
- 13 — P6e, o catálogo do Vector (`smoke_catalogo`) e uma engrenagem viva.
- 14 — P6f, a forma viva até ao fim (`smoke_ajustes`).
- 15 — P6g, as pontas soltas (`smoke_pontas`).
- 16 — P6h–P6j, o intervalo da linha do tempo (`smoke_intervalo`).
- 17 — P5, tags dentro de tags, o tocar e a Preview (`smoke_tocar`).
- 18 — P7b, os pincéis e a dinâmica (`smoke_pincel`).
- 19 — P8, gradiente, texto, filtros e ajustes (`smoke_efeitos`).
- 20 — P9b, Export e Import, sobre os 16 quadros da cena 8 (`smoke_exporta`).
- 21 — P10b, uma camada de tilemap já desenhada, com a janela Tilemap aberta (`smoke_tiles`).
- 22 — P10c, ilhas com praia sobre água: as 47 formas desenhadas por regra (`exemplo::terreno_ilhas`), com o pincel de terreno na mão (`smoke_tiles`).
- 23 — GEF 23, o cartão do Pixel Sprite: os degradês 96 × 64 seleccionados, em Object.
- 24 — P11, Slices: 72 × 48, 6 quadros, um botão (`button`, 9-slice) e um boneco a andar (`hero`, pivot nos pés, uma key por quadro), a Slice Tool na mão e a janela SLICES aberta com `hero` selecionada (`smoke_slices`).
- 25 — P12, a sessão: boneco 32 × 32 (Color, Shadow, Outline + `Layer 4`), 4 quadros a andar, sete passos na Undo History (Pencil, Line, Paint Bucket, New Layer, Pencil, Flip Horizontal, Hue/Saturation), `Fill` em `Shift+F` num ficheiro de teclas da cena (temp, nunca o do dono), grade 8 × 8 rosa, onion 1 + 1 com tinta, user data na Color (`base colors`, `hp = 10`); abertas UNDO HISTORY e KEYBOARD SHORTCUTS (`ph2d_pixel_cenas::historia`). Fotos `handoffs/P12_smoke_*.png`.
- 26 — P14 §8, «O herói na cena»: a `heroi_p14` (com a camada Normal Map) como Pixel Sprite com Rigid Body dinâmico e a forma da slice `body` (Add Collider), sobre o «Floor» (estático, forma da slice `ground`), a «Light» à esquerda, a «Wall» à direita; herói e parede com Blocks Light; a regra do cartão Animation «Key held `move_right` → run» (→ ou D). Sonda `shells/pixel-scenes/tests/it/cena26.rs` (`ph2d_pixel_cenas::heroi`). Fotos `handoffs/smoke_P14_cena26_*.png`.
