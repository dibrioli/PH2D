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
| L2 | Cada objecto alinha com os da **mesma casa** e **esse frame** — um filho nunca com o de fora. A casa de quem se ARRASTA é a do PONTEIRO (o frame onde se largaria, a lei do largar, oráculo A4/A5) e muda durante o arrasto; a de quem roda ou se redimensiona é a dele; a de quem nasce, o frame debaixo do início | tldraw G6 (*divergência escolhida* do Excalidraw G6, que alinha com o de fora: a queixa 4 do dono); o tldraw re-parenta durante o arrasto | `a_child_does_not_snap_to_what_lives_outside_its_frame` · `a_child_snaps_to_the_edge_of_its_own_frame` · `dragging_into_a_frame_aligns_with_its_children` |
| L3 | Num referencial, uma caixa alinha pelas bordas, o centro e — rodada — os **cantos de dentro** (os 4 cantos + centro) | Excalidraw G1/G2, tldraw G1/G2 (os dois iguais) | `a_rotated_mover_snaps_by_an_inner_rotated_corner` · `a_rotated_target_offers_its_inner_corners` |
| L4 | ⭐ Um rodado tem também os **eixos PRÓPRIOS**: lá entram os alvos ao mesmo ângulo (mod 90°), pelas caixas rodadas — e SÓ lá (não nos do mundo); quando encaixa neles, ganham; a guia desenha-se **inclinada** | **pedido do dono** (nenhum app o faz); o «só lá» medido no gate da cena 7 (abaixo, §3) | `two_shapes_at_the_same_angle_align_along_their_rotated_edges`; pixels `a_rotated_guide_paints_along_the_rotated_edges`; `every_lesson_teaches_its_own_alignment` |
| L5 | ⭐ Dentro de um frame rodado, os filhos (ao ângulo do frame) alinham e espaçam-se nos **eixos do frame** | pedido do dono; consequência de L2 + L4 | `inside_a_rotated_frame_children_align_in_the_frame_axes` · `equal_spacing_inside_a_rotated_frame_runs_along_the_frame_axes` |
| L6 | **Frame arrastado** alinha a caixa DELE (vazio ou com filhos); os filhos andam com ele e não são alvo | Excalidraw G6, tldraw G6; smoke do dono 08/10 | `dragging_a_frame_aligns_the_frame_box` · `dragging_a_frame_does_not_snap_to_its_own_children` |
| L7 | ⭐ **Encaixe do ângulo**: ao rodar, o ângulo de quem roda (de VÁRIOS, o comum; ângulos diferentes não encaixam) cola no de um alvo da mesma casa (mod 90° — um direito dá 0/90/180/270), o mais perto em ângulo; tolerância = **8 px no ARCO** (`|Δθ|·r ≤ 8`) com `r` o MAIOR entre o raio do ponteiro e a meia diagonal (perto do centro, o canto continua a andar 8 px); mostra-se com duas guias no ângulo, uma por cada centro | pedido do dono (nenhum app); a tolerância no arco segue a de posição (px de ecrã); o chão do raio pela auditoria (a r = 10 a janela chegava a 45°) | `rotating_near_the_angle_of_another_shape_snaps_to_it` · `the_angle_snap_is_measured_on_the_arc_and_ctrl_suspends_it` · `…modulo_a_right_angle` · `near_the_centre_the_angle_tolerance_is_the_corner_arc` · `rotating_several_snaps_their_common_angle` |
| L8 | `Shift` a rodar = passos de **15°** da moldura (de vários: da volta, a moldura nasce direita), o mais próximo, e não procura ângulos | Excalidraw G4, tldraw G4, Figma, Affinity, Inkscape | `shift_still_steps_fifteen_degrees` |
| L9 | **Espaçamento igual** na fila (os que se sobrepõem no outro eixo): ao lado de um par, o vão do par; entre dois vizinhos, ao meio; pela caixa envolvente; a guia são os dois vãos | Miro (help), Excalidraw G5, tldraw G5 | `equal_spacing_beside_a_pair_and_between_two_neighbours` (43 → 40; 53/47 → 50) |
| L10 | Tolerância **8 px de ecrã** (`Spacing::Md`), igual em qualquer zoom | Excalidraw G8 = tldraw G8 = 8 | `board_view::metrics` |
| L11 | `Ctrl` suspende tudo (posição e ângulo) | tldraw G8, Figma, Miro (grelha) | `…ctrl_suspends_it` |
| L12 | Redimensionar encaixa a pega nos eixos da moldura (os próprios de um rodado) e SÓ nos eixos que ela mexe (a de um lado, um); em proporção (`Shift`, notas, vários rodados) não encaixa | consequência de L4; a auditoria (a pega de E colava em y e desenhava a guia que a forma não seguia) | `resizing_a_rotated_shape_snaps_in_its_own_axes` · `a_side_handle_snaps_only_its_own_axis` |
| L13 | Criar: o canto encaixa nos eixos do mundo; uma nota só na largura (a altura é a da proporção); com `Shift` (quadrado) não encaixa; um PONTO não tem espaçamento igual | auditoria: a guia mentia (o lado do quadrado era o maior, o vão era de uma caixa que não existe) | `a_square_being_drawn_does_not_snap` · `a_shape_being_drawn_has_no_equal_spacing` |
| L14 | Arrastar com `Shift` (um eixo) encaixa SÓ no eixo livre, nos do mundo; `Alt` (duplicar): os originais ficam e são alvo | auditoria (o encaixe destravava o eixo; a cópia não alinhava com o original) | `shift_keeps_the_axis_lock_while_snapping` · `an_alt_copy_aligns_with_its_original` |
| L15 | Um PAR do espaçamento igual partilha a fila (os dois sobrepõem-se no outro eixo) | Excalidraw G5c, tldraw | `a_pair_that_does_not_share_a_row_gives_no_gap` |

## §3 — ⛔ Recusas MEDIDAS

| proposta | estado |
|---|---|
| alinhar um rodado pela caixa envolvente (o que o quadro fazia até 08/10) | ⛔ os dois oráculos medem o contrário: os cantos rodados (L3); a caixa só entra nos vãos (L9) |
| arredondar o ângulo ao grau inteiro ao largar (o tldraw G4: 29,5 → 29) | ⛔ desfaria o encaixe a um ângulo não inteiro (um alvo a 37,3°) — e o encaixe é a lei que o dono pediu |
| o filho alinhar com o que está FORA do frame (o Excalidraw G6) | ⛔ divergência escolhida: tldraw (L2) — a queixa 4 do dono |
| os eixos do mundo e os próprios a competir pelo mais perto | ⛔ medido no teste do espaçamento dentro do frame rodado: os cantos de dentro de um vizinho (L3) ficavam a < 1 px no mundo e roubavam o encaixe nos eixos do frame. O próprio ganha quando encaixa |
| dois rodados ao MESMO ângulo a alinharem também canto a canto no ecrã (o Excalidraw/tldraw G3) | ⛔ medido no gate da cena 7 (08/10): o filho de baixo do frame rodado SALTAVA 0,97 antes da lição, para um alinhamento de ecrã entre cantos de irmãos que ninguém vê. Ao mesmo ângulo, só os eixos próprios (L4) |
| a cena 7 com tudo «quase alinhado» (14–25 px fora) e solta no quadro | ⛔ lente de costura (08/10): o 1.º encaixe era o do centro, guias direitas, o canto de um vizinho — a cena ensinava outra coisa. Cada lição no seu frame, a 40, e o gate conduz cada uma a dois zooms |
| guias rodadas a qualquer ângulo de qualquer alvo (projectar todos nos eixos de quem se move) | ⛔ a caixa de uma forma DIREITA vista a 30° não tem bordas no desenho — guias sem nada debaixo; só entram os alvos ao mesmo ângulo (mod 90°) |
| índice espacial dos alvos | ⛔ a régua (§4) não o pede: ≤ 2,4 ms (mediana) por movimento a 100 mil à vista |
| recolher só a família que o gesto usa (rodar não precisa das caixas; mover não precisa dos ângulos) | ⛔ por agora: o carregar inteiro é 11,7 ms a 100 mil, uma vez por gesto, e o `Ctrl` (que não encaixa) paga o mesmo — o custo é percorrer os vivos, não o que se guarda |
| setas como alvo (o Excalidraw G7, pela caixa) | ⛔ o tldraw G7 recusa; a caixa de uma curva não é nada que se veja |
| tecla para LIGAR/desligar o encaixe na barra | ⛔ não nesta onda: o `Ctrl` suspende (L11) e nenhum controlo novo entra sem pedido do dono |

## §4 — Medir antes de limitar (§0.0)

Régua `crates/ph2d-board-render/tests/it/measure_snap_cost.rs` — N rectângulos 120×80 em grelha, todos à
vista, 1 em 3 a 30°, arrastado um DIREITO e um RODADO (e o `Ctrl` como chão), variantes no mesmo processo
intercaladas, o mínimo de 7 rodadas (mediana ao lado), `--release`, 08/10, sobre o código final do
fecho, `load` 9,0 (a máquina partilhada — os números são um TECTO):

| N à vista | carregar (o gesto nasce) | só o `hit` do clique (frio) | mexer, direito | mexer, rodado |
|---|---|---|---|---|
| 1 000 | 0,09 ms | 0,24 | 0,016 | 0,019 |
| 10 000 | 0,89 | 2,6 | 0,16 | 0,19 |
| 100 000 | 11,7 (med 14,4) | 33,6 | 1,61 (med 2,1) | 1,94 (med 2,4) |

⇒ nenhum tecto de alvos, nenhum índice: o pior movimento a 100 mil é ~1/7 de um quadro a 60 Hz; o
carregar paga uma vez por gesto (e o `Ctrl`, que não encaixa, paga o mesmo — o custo é recolher, não
encaixar). O `hit` frio é o da cache das setas na 1.ª chamada (na app ela está quente).

## §5 — O que o dono vê (cena 7)

`PH2D_BOARD_SMOKE=7` (`crates/ph2d-app-board/src/smoke_guides.rs`): cada lição no seu frame (L2 — uma
não acende guias da outra), cada «arraste-me» a 40 do que ensina — «Tilted shapes» (B desliza ao longo
dos 30° até a borda esquerda ficar na linha inclinada da de A) · «Rotate» (o direito encaixa em 30°, 45°
e direito) · «Tilted frame» a 20° (o 3.º da fila: vão 80 → 40; o de baixo: a esquerda à do 1.º) ·
«Equal spacing» (vãos 60 e 100 → 60) · «Frame A» e «Frame B» (topos 40 desencontrados). O gate
`every_lesson_teaches_its_own_alignment` conduz cada lição com o editor e as medidas da interface, a
zoom 1 e ao do enquadrar: nada salta antes, e no fim fica o que o texto promete.
