# F42 — um ícone e uma cor por TIPO de objecto na Hierarquia (dono, 01/11)

> *«Agora quero que cada tipo de objeto na hierarchy tenha um ícone único, bem representativo de sua
> classe e com cores diferentes dos outros tipos.»*

Código: [`hier_types.rs`](../../crates/ph2d-editor-core/src/hier_types.rs) (o vocabulário) ·
[`row_components.rs`](../../crates/ph2d-app-components/src/row_components.rs) (`object_type`, a sonda
do mundo) · [`row.rs`](../../crates/ph2d-panel-hierarchy/src/row.rs) (`icon_color`) ·
[`type_colour.rs`](../../crates/ph2d-tokens/src/type_colour.rs) (temas modernos) · cena
[`smoke_tipos.rs`](../../crates/ph2d-app-components/src/smoke_tipos.rs) (`PH2D_OBJECT_MODE_SMOKE=42`).

## 1 — Os tipos, CONTADOS (não escolhidos)

Fonte: os 8 `ObjectKind` (marcador no mundo) + os componentes que DEFINEM um objecto no menu Add
(`fase_object_add::FAMILIES`) e no registo. Precedência = ordem de `ObjectType::ALL`: o Player ganha a
todos; depois o `ObjectKind`; num vazio, o 1.º componente definidor.

| Tipo | Marcador | Glifo | Token | Escuro (forge) | Claro (sunstone/blueprint) |
|---|---|---|---|---|---|
| Empty | — | `cube` | `obj-empty` | `oklch(0.760 0.010 260)` | `oklch(0.437 0.011 260)` |
| Image | `Sprite` | `sprite` | `obj-image` | `0.740 0.140 245` | `0.422 0.107 245` |
| Vector | `VecPathRef` | `vector` | `obj-vector` | `0.720 0.162 295` | `0.406 0.186 295` |
| Flip | `FlipObjectRef` | `flip` | `obj-flip` | `0.680 0.190 25` | `0.375 0.154 25` |
| Skeleton | `Skeleton` | `bone` | `obj-skeleton` | `0.900 0.050 80` | `0.455 0.050 80` |
| Motion | `MotionGenerator` | `motion-nodes` | `obj-motion` | `0.780 0.130 185` | `0.452 0.079 185` |
| Pixel Sprite | `PixelSprite` | `pixel-sprite` ★ | `obj-pixel-sprite` | `0.700 0.120 215` | `0.391 0.070 215` |
| Player | `PlatformPlayer` | `player` ★ | `obj-player` | `0.850 0.190 145` | `0.491 0.154 145` |
| Camera | `GameCamera` | `camera` | `obj-camera` | `0.760 0.160 55` | `0.437 0.110 55` |
| Light | `Luz` | `light` | `obj-light` | `0.920 0.168 100` | `0.505 0.105 100` |
| Macro | `Macro` | `macro` | `obj-macro` | `0.680 0.200 322` | `0.375 0.180 322` |
| Particles | `ParticleEmitter` | `particle` | `obj-particles` | `0.900 0.080 205` | `0.495 0.084 228` |
| Sound | `AudioSource2D` | `audio` | `obj-sound` | `0.660 0.150 270` | `0.360 0.172 270` |
| HUD | `UiCanvas` | `hud` ★ | `obj-hud` | `0.860 0.076 30` | `0.500 0.087 30` |
| Joint | `PhysicsJoint` | `joint` ★ | `obj-joint` | `0.700 0.110 75` | `0.391 0.082 75` |
| Body | `RigidBody` (sem arte) | `body` ★ | `obj-body` | `0.660 0.148 160` | `0.360 0.082 160` |
| Collider | `Collider` (sem arte) | `collider` | `obj-collider` | `0.800 0.170 125` | `0.468 0.117 125` |

★ = glifo novo, desenho original (traço 1,5, grelha 24).

Leis:
- **Glifo exclusivo por tipo** ⇒ a cor sai do glifo (`ObjectType::of_icon`); nenhum campo novo na
  linha. Gate `cada_tipo_tem_icone_unico`.
- **Cor só no papel Object**; a Cena e o conteúdo ficam como eram. Seleccionado, o ícone mantém a cor
  do tipo (identidade); escondido, `text-disabled`. Gate `o_icone_do_objecto_pinta_a_cor_do_tipo`.
- **Componente com arte não muda o tipo**: um Sprite com corpo é Image (o corpo é o ícone de
  componente à direita). Body/Collider/Camera… só num vazio.
- **A imagem pintada é Image** (dono 06/11, revoga o tipo Painted): dois objectos, Image e Pixel Sprite; o ícone do Painter é da ferramenta. Gate `a_imagem_pintada_e_uma_imagem_e_o_pixel_sprite_e_outro`.
- De passagem: um vazio com `ScrollFactor` (família Camera no catálogo) lia-se como câmera; agora só o
  `GameCamera` faz câmera.

## 2 — Medição da paleta

Contraste WCAG contra `bg-0/1/2`, distância ΔE OKLab mínima entre os 18 (gate
`cada_tipo_tem_cor_propria_e_legivel`: ≥ 4,5 nos 4 clássicos, ≥ 3 nos modernos, ΔE ≥ 0,05):

| Tema | contraste mín. | ΔE mín. | `text-3` mín. (o cinza de antes) |
|---|---|---|---|
| Forge | 5,89 | 0,071 | 4,71 |
| Workshop | 5,83 | 0,071 | 4,75 |
| Sunstone | 4,86 | 0,062 | 3,90 |
| Blueprint | 4,68 | 0,062 | 3,73 |
| Dark | 4,86 | 0,071 | 4,30 |
| Gray | 3,64 | 0,071 | 3,73 |
| Light | 4,28 | 0,062 | 3,17 |
| Oled | 6,59 | 0,071 | 4,43 |
| PlumberRed | 3,01 (empurrada) | 0,056 | 2,37 |
| PlumberGreen | 3,00 (empurrada) | 0,055 | 2,50 |
| Sunset | 3,88 | 0,071 | 3,73 |
| Candy | 3,45 | 0,062 | 3,04 |

## ⛔ Recusas MEDIDAS

| Recusa | Porquê (medido) |
|---|---|
| Emprestar `obj-*` tal-e-qual nos modernos | Plumber: 2,09/1,94 sobre o fundo, abaixo do `text-3` (2,37) ⇒ `legible_on` empurra L até 3:1 só onde falta |
| Gate de 4,5 em todos os temas | o Gray (fundo médio) dá 3,64 mesmo ao `text-3` 3,73; 4,5 é critério de TEXTO, o ícone é gráfico (SC 1.4.11 = 3:1) |
| Painted = «tem camadas no Hierarchy» | o Painter desmonta a tela ao sair do Paint ⇒ o ícone trocava de cor ao entrar/sair; a lista certa é `document_entities()` (a do save) |
| Detectar câmera/luz pela FAMÍLIA do catálogo | a família Camera inclui `ScrollFactor` (fundos de paralaxe) ⇒ um vazio com paralaxe virava câmera |
