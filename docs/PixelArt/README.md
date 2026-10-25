# Pixel Art — o módulo da PH2D

Módulo **novo e autocontido** de pixel art e animação de sprites (decisão do dono, 25/10): não
depende do Painter. A referência de etapas é o **Aseprite**; as leis numéricas vêm do MVP web em
[`../Pixel Art/`](../Pixel%20Art/).

| Doc | O quê |
|---|---|
| [01_estudo_aseprite.md](01_estudo_aseprite.md) | O Aseprite 1.3.18.6 mapeado: menus, ferramentas, inks, paleta, export, o que o Trial deixa fazer |
| [02_plano_modulo.md](02_plano_modulo.md) | Arquitetura (5 crates), leis herdadas, oráculos e as ondas P0–P14 |
| [aseprite/](aseprite/) | Fotos do Trial e a ajuda do CLI |

Estado: **P0 (Fundação) na `line/PixelArt`** — [ADR-0196](../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md); o objecto Pixel Sprite, lápis/borracha/conta-gotas/mão, zoom inteiro, desfazer por sprite, gravar no projecto. Smoke `PH2D_PIXEL_SMOKE=1`.
