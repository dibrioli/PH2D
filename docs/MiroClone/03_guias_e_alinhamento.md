# 03 — Guias e alinhamento de objectos rodados (W5b)

> **Pedido do dono (08/10):** *«investigar e implementar melhor sistema de alinhamento e guias para
> objetos inclinados, considerando a possibilidade alinhar a rotação e criar linhas na rotação das formas.
> Deve estudar como os melhores apps fazem. O alinhamento de formas e frames inclinados não está bom, o
> alinhamento de frames nem sei se existe. Frames com formas dentro de si só o frame gera guias e alinha
> quando o frame é arrastado ou rotacionado, não buscando alinhamento entre formas dentro de frames.»*
>
> Briefing: [`handoffs/HANDOFF_CONTINUACAO_W5b_2026-10-08.md`](handoffs/HANDOFF_CONTINUACAO_W5b_2026-10-08.md).
> Código: `crates/ph2d-board-edit/src/guides.rs` (quem se move, contra quê, em que eixos) e `snap.rs` (a lei
> de uma caixa: bordas, cantos, centro, espaçamento igual). Testes: `guides_tests.rs`.

## §0 — Veredito

Nenhum dos oito apps estudados faz o que o dono pede: **nenhum** desenha guias na direcção de uma forma
rodada e **nenhum** encaixa a rotação no ângulo de outro objecto (documentação oficial de Miro, Figma,
Penpot, Inkscape, Affinity, Illustrator e Godot; e os dois que se correm, Excalidraw e tldraw, medidos:
zero linhas oblíquas em 142 + 8 casos, ângulo final = ângulo do ponteiro). O que os dois oráculos fazem
— e fazem IGUAL — é o chão: um rodado alinha pelos **quatro cantos rodados e o centro** (não pela caixa
envolvente), a tolerância é **8 px de ecrã** em qualquer zoom, há **espaçamento igual** na fila, o frame é
alvo e arrastado alinha **a caixa dele**, `Shift` roda em passos de 15°.

O PH2D fica com esse chão **e** com o que o dono pediu por cima: os eixos PRÓPRIOS de um rodado
(guias inclinadas, dentro de um frame rodado também), o encaixe do ÂNGULO, e a regra do tldraw de que um
filho alinha com a casa dele (irmãos + o frame), nunca com o que está fora.

## §1 — Como os apps fazem (fontes)

Pesquisa da documentação oficial, com citação literal e URL de cada lei, em dois ficheiros:
[`ferramentas/guias_pesquisa_miro_figma_penpot.md`](ferramentas/guias_pesquisa_miro_figma_penpot.md) (os
652 artigos do help do Miro e os 896 do Figma pela API Zendesk, Penpot pelo guia e pelo `CHANGES.md`) e
[`ferramentas/guias_pesquisa_inkscape_affinity_illustrator_godot.md`](ferramentas/guias_pesquisa_inkscape_affinity_illustrator_godot.md)
(+ Lucid e Canva). Os oráculos: [`ferramentas/excalidraw_oracle/`](ferramentas/excalidraw_oracle/README.md)
§W5b e [`ferramentas/tldraw_oracle/`](ferramentas/tldraw_oracle/README.md).

| pergunta | Miro (dono) | Figma | Excalidraw 0.18.1 (corrido) | tldraw 5.5.2 (corrido) | Inkscape · Affinity · Illustrator · Godot |
|---|---|---|---|---|---|
| rodado alinha por | silente; guias só «vertical or horizontal» | «centers and outermost points» (caixa, inferido) | **4 cantos rodados + centro** (G1: canto de dentro a x 300 exacto) | **4 cantos rodados + centro** (G1b: 8/8 sondas) | caixa e geometria são categorias separadas; Illustrator «geometry of objects» |
| guias na direcção da forma | não | não | **não** (0 oblíquas em 142 sondas) | **não** (G3: 0 oblíquas) | só por comando (Inkscape «Object to Guides»; Illustrator construction guides em ângulos FIXOS) |
| encaixe do ângulo | silente; `Shift` = 45° (fórum) | `Shift` = 15° | não; `Shift` = 15° mais próximo | não; arredonda ao grau ao largar; `Shift` = 15° | Inkscape `Ctrl` 15° (`rotationsnapsperpi` 12, medido no 1.4.4 instalado); Affinity `Shift` 15°; Godot passo configurável; nenhum ao ângulo de outro |
| frame como alvo / arrastado | silente; frames NÃO rodam (developers.miro.com) | frames rodam; nada dentro de um rodado (staff: *«We won't do any snapping…of rotated grids»*) | alvo; arrastado alinha a caixa dele; **não roda** | alvo; arrastado alinha a caixa dele; roda | Godot «Snap to Parent»; artboards são alvos (Affinity, Illustrator) |
| filho dentro de um frame | silente | alinha ao frame-pai | irmão, frame **e o de fora** | irmão e frame, **nunca o de fora** (G6) | silente |
| dentro de um frame rodado | — | nenhum encaixe | — | só eixos da página (G6: prendeu a um vértice) | silente em todos |
| espaçamento igual | **sim** (*«smart guides … equal spacing»*) | sim (fórum) | sim, na fila (G5: 43/37/41/46 → 40; meio 53/47 → 50) | sim (G5: 48 → 40; meio) | Inkscape «Same distances», Affinity «gaps and sizes», Illustrator «Spacing Guides» |
| tolerância | silente | silente | **8 px** de ecrã, zoom 1 e 2 (G8) | **8 px** de ecrã, zoom 1 e 2 (G8) | Affinity/Illustrator configuráveis, sem número |
| suspender | `Ctrl` (grelha) | `Ctrl` (help) | Alt+S liga/desliga o modo | `Ctrl` | Inkscape `Shift`; Affinity `Alt` |
| setas / traço livre como alvo | — | — | pela caixa | setas NÃO; traço pela caixa | — |

## §2 — As leis do PH2D (lei → fonte → número)

| # | lei | fonte | número / teste |
|---|---|---|---|
| L1 | Os alvos são os vivos **à vista** que não se movem: formas, notas, traços da caneta e **frames**; setas não | tldraw G6/G7; Penpot «only in sight» | `a_frame_is_a_snap_target` |
| L2 | Cada objecto alinha com os da **mesma casa** (o frame onde vive, ou solto) **e esse frame** — um filho nunca com o de fora | tldraw G6 (*divergência escolhida* do Excalidraw G6, que alinha com o de fora: a queixa 4 do dono é que dentro do frame a busca se perde) | `a_child_does_not_snap_to_what_lives_outside_its_frame` · `a_child_snaps_to_the_edge_of_its_own_frame` |
| L3 | Num referencial, uma caixa alinha pelas bordas, o centro e — rodada — os **cantos de dentro** (os 4 cantos + centro) | Excalidraw G1/G2, tldraw G1/G2 (os dois iguais) | `a_rotated_mover_snaps_by_an_inner_rotated_corner` · `a_rotated_target_offers_its_inner_corners` |
| L4 | ⭐ Um rodado tem também os **eixos PRÓPRIOS**: lá entram só os alvos ao mesmo ângulo (mod 90°), pelas caixas rodadas; quando encaixa neles, ganham; a guia desenha-se **inclinada** | **pedido do dono** (nenhum app o faz) | `two_shapes_at_the_same_angle_align_along_their_rotated_edges`; pixels `a_rotated_guide_paints_along_the_rotated_edges` |
| L5 | ⭐ Dentro de um frame rodado, os filhos (ao ângulo do frame) alinham e espaçam-se nos **eixos do frame** | pedido do dono; consequência de L2 + L4 | `inside_a_rotated_frame_children_align_in_the_frame_axes` · `equal_spacing_inside_a_rotated_frame_runs_along_the_frame_axes` |
| L6 | **Frame arrastado** alinha a caixa DELE (vazio ou com filhos); os filhos andam com ele e não são alvo | Excalidraw G6, tldraw G6; smoke do dono 08/10 | `dragging_a_frame_aligns_the_frame_box` · `dragging_a_frame_does_not_snap_to_its_own_children` |
| L7 | ⭐ **Encaixe do ângulo**: ao rodar, o ângulo cola no de um alvo da mesma casa (mod 90° — um direito dá 0/90/180/270), o mais perto em ângulo; tolerância = **8 px no ARCO** do ponteiro (`|Δθ|·r ≤ 8`); mostra-se com duas guias no ângulo, uma por cada centro | pedido do dono (nenhum app); a tolerância no arco segue a de posição (px de ecrã) | `rotating_near_the_angle_of_another_shape_snaps_to_it` · `the_angle_snap_is_measured_on_the_arc_and_ctrl_suspends_it` · `…modulo_a_right_angle` |
| L8 | `Shift` a rodar = passos de **15°**, o mais próximo, e não procura ângulos | Excalidraw G4, tldraw G4, Figma, Affinity, Inkscape | `shift_still_steps_fifteen_degrees` |
| L9 | **Espaçamento igual** na fila (os que se sobrepõem no outro eixo): ao lado de um par, o vão do par; entre dois vizinhos, ao meio; pela caixa envolvente; a guia são os dois vãos | Miro (help), Excalidraw G5, tldraw G5 | `equal_spacing_beside_a_pair_and_between_two_neighbours` (43 → 40; 53/47 → 50) |
| L10 | Tolerância **8 px de ecrã** (`Spacing::Md`), igual em qualquer zoom | Excalidraw G8 = tldraw G8 = 8 | `board_view::metrics` |
| L11 | `Ctrl` suspende tudo (posição e ângulo) | tldraw G8, Figma, Miro (grelha) | `…ctrl_suspends_it` |
| L12 | Redimensionar um rodado encaixa a pega nos eixos dele | consequência de L4 (antes: nenhum encaixe com o frame rodado) | `resizing_a_rotated_shape_snaps_in_its_own_axes` |

## §3 — ⛔ Recusas MEDIDAS

| proposta | estado |
|---|---|
| alinhar um rodado pela caixa envolvente (o que o quadro fazia até 08/10) | ⛔ os dois oráculos medem o contrário: os cantos rodados (L3); a caixa só entra nos vãos (L9) |
| arredondar o ângulo ao grau inteiro ao largar (o tldraw G4: 29,5 → 29) | ⛔ desfaria o encaixe a um ângulo não inteiro (um alvo a 37,3°) — e o encaixe é a lei que o dono pediu |
| o filho alinhar com o que está FORA do frame (o Excalidraw G6) | ⛔ divergência escolhida: tldraw (L2) — a queixa 4 do dono |
| os eixos do mundo e os próprios a competir pelo mais perto | ⛔ medido no teste do espaçamento dentro do frame rodado: os cantos de dentro de um vizinho (L3) ficavam a < 1 px no mundo e roubavam o encaixe nos eixos do frame. O próprio ganha quando encaixa |
| guias rodadas a qualquer ângulo de qualquer alvo (projectar todos nos eixos de quem se move) | ⛔ a caixa de uma forma DIREITA vista a 30° não tem bordas no desenho — guias sem nada debaixo; só entram os alvos ao mesmo ângulo (mod 90°) |
| índice espacial dos alvos | ⛔ a régua (§4) não o pede: < 2,6 ms por movimento a 100 mil à vista |
| setas como alvo (o Excalidraw G7, pela caixa) | ⛔ o tldraw G7 recusa; a caixa de uma curva não é nada que se veja |
| tecla para LIGAR/desligar o encaixe na barra | ⛔ não nesta onda: o `Ctrl` suspende (L11) e nenhum controlo novo entra sem pedido do dono |

## §4 — Medir antes de limitar (§0.0)

Régua `crates/ph2d-board-render/tests/it/measure_snap_cost.rs` — N rectângulos 120×80 em grelha, todos à
vista, 1 em 3 a 30°, arrastado um DIREITO e um RODADO (e o `Ctrl` como chão), variantes no mesmo processo
intercaladas, o mínimo de 7 rodadas (mediana ao lado), `--release`, 08/10, `load` 17,8 (a máquina
partilhada — os números são um TECTO):

| N à vista | carregar (o gesto nasce) | só o `hit` do clique (frio) | mexer, direito | mexer, rodado |
|---|---|---|---|---|
| 1 000 | 0,09 ms | 0,24 | 0,015 | 0,025 |
| 10 000 | 0,97 | 2,6 | 0,15 | 0,24 |
| 100 000 | 15 | 36 | 1,55 (med 2,1) | 2,5 (med 3,4) |

⇒ nenhum tecto de alvos, nenhum índice: o pior movimento a 100 mil é ~1/7 de um quadro a 60 Hz; o
carregar paga uma vez por gesto (e o `Ctrl`, que não encaixa, paga o mesmo — o custo é recolher, não
encaixar). O `hit` frio é o da cache das setas na 1.ª chamada (na app ela está quente).

## §5 — O que o dono vê (cena 7)

`PH2D_BOARD_SMOKE=7` (`crates/ph2d-app-board/src/smoke_guides.rs`): dois rectângulos a 30° com as bordas
18 px fora da mesma linha inclinada; um a 45° e um direito para RODAR; um frame rodado a 20° com filhos a
vãos 40 e 62 e um 18 px fora da esquerda de outro; dois frames direitos com os topos 25 desencontrados;
uma fila com vãos 60 e 75.
