# 04 — Planear: mapa mental e modelos (W6)

> **Pedido do dono (plano §3 W6, ordem de 08/10 «a próxima janela segue com a W6»):** mapa mental (`Tab`
> filho, `Enter` irmão, setas navegam, ramos recolhem, layout automático) e modelos (retrospectiva,
> brainstorm, kanban, mapa mental, fluxograma, mapa de jornada). **O dono vê:** começar um planeamento a
> partir de um modelo e expandi-lo pelo teclado.
>
> Briefing: [`handoffs/HANDOFF_CONTINUACAO_W6_2026-10-08.md`](handoffs/HANDOFF_CONTINUACAO_W6_2026-10-08.md).
> Fontes: a ajuda do Miro e de cinco apps ([`ferramentas/planear_pesquisa_miro_e_apps.md`](ferramentas/planear_pesquisa_miro_e_apps.md)),
> os três apps de mapa mental CORRIDOS ([`ferramentas/mindmap_oracle/`](ferramentas/mindmap_oracle/README.md))
> e a imagem oficial do modelo «Mind Map» do Miro medida ao píxel (§1.3).
> Código: `crates/ph2d-board-model/src/mind.rs` (a árvore, o layout e o `settle`), `crates/ph2d-board-edit/src/mindmap.rs`
> (as teclas e o arrastar), `crates/ph2d-board-edit/src/templates.rs` (os modelos).

## §0 — Veredito

O layout do mapa mental do Miro é **por FAIXAS, não compacto**: cada subárvore ocupa uma faixa inteira, o pai
fica no MEIO da faixa dos filhos, e cada nó de uma linha ocupa **54** de altura — entre folhas, entre
subárvores e a qualquer profundidade (§1.3). Os dois apps que editam (mind-elixir, simple-mind-map) fazem o
mesmo tipo de layout, com outros números; o markmap compacta (tidy tree) e fica de fora. Por isso **não é
preciso Reingold–Tilford**: a lei cabe numa passagem O(n) de cima para baixo. O `taffy` exprime-a (linha
`[nó, coluna de filhos]` centrada), mas é uma dependência de layout de caixas para uma conta de três linhas
(§4).

O layout é uma **invariante do documento**, não um gesto: o `ph2d-board-model` arruma o mapa em cada lote de
operações que lhe toca (`apply_batch`), no MESMO passo de desfazer. Escrever, apagar, colar, desfazer ou
receber uma operação de outra pessoa (Etapa 2) deixa sempre o mapa arrumado. Os ramos não são setas
guardadas: desenham-se a partir da relação pai→filho com a curva MEDIDA do Miro (braço = 0,39 do vão, §1.3),
que **não** é a das setas (0,45 da distância).

Os modelos inserem-se no centro da vista, num passo só de desfazer, já seleccionados, e são feitos das peças
que o quadro já tem (frames, notas, formas, setas, mapa mental).

## §1 — Como os apps fazem (fontes)

### §1.1 — Miro (o produto do dono): a ajuda

[`ferramentas/planear_pesquisa_miro_e_apps.md`](ferramentas/planear_pesquisa_miro_e_apps.md) tem cada lei
com a citação e o URL. A ajuda do mapa mental (<https://help.miro.com/hc/en-us/articles/360017730753-Mind-map>):

- `Tab` = filho · `Enter` = irmão (já em edição) · `Shift+Enter` = quebra de linha · as setas *«move between
  nodes»* · `Esc` sai da edição.
- `Delete` apaga o nó **com os filhos**. Arrastar um nó troca-o de pai; `Ctrl` seguro ao arrastar move-o sem
  trocar de pai.
- Layout horizontal (esquerda/direita) ou vertical (cima/baixo), escolhido na raiz. O **auto layout** vem
  ligado e desliga-se no nó-pai.
- Recolher: um «−» ao pairar; o ramo recolhido mostra um **número** (nós + comentários escondidos), e
  clicar nele expande.
- Fonte só pela raiz, e escala o mapa todo (REST: 10–288, omissão **14**). Sem limite de nós.
- O lado de um filho da raiz decide-o o «+» clicado (ou, pelo SDK, a posição inicial); **não há equilíbrio
  automático** (a imagem oficial tem 3 à esquerda e 2 à direita).
- **Silente:** se o irmão nasce antes ou depois, o que cada seta faz, `Backspace`, tecla de recolher, os
  números do espaçamento.

### §1.2 — Oráculos corridos

[`ferramentas/mindmap_oracle/`](ferramentas/mindmap_oracle/README.md): mind-elixir 6.0.0-next.4,
simple-mind-map 0.14.0-fix.3, markmap 0.18.12 (todos MIT, triados pelo pacote INSTALADO), sobre as árvores
NOSSAS T1–T6, N100 e N1000; duas corridas iguais byte a byte (39/39). O que se mediu:

| lei | mind-elixir (`SIDE`) | simple-mind-map (`mindMap`) | markmap |
|---|---|---|---|
| layout | faixas, pai no meio da faixa | faixas, pai no meio (0 exacto) | **compacto** (A2\|A3 −19,8; até −32,9) |
| vão entre níveis (borda) | 75 / 40 / 60 | 104 / 54 / 54 | 80 |
| vão entre irmãos (prof. 1 / ≥ 2) | 61–67 / 22 | 44 / 4 | 6,2–6,4 |
| lados | alterna por índice, o 1.º à ESQUERDA | os ⌈n/2⌉ primeiros à DIREITA | só direita |
| recolher | o nó fica; os de baixo sobem (−150) | idem (−91) | recentra tudo |
| `Tab` | filho, o ÚLTIMO, seleccionado e em edição | idem | — |
| `Enter` | irmão logo DEPOIS; na raiz cria filho | idem; na raiz nada | — |
| setas | estruturais: ↑↓ irmão do mesmo lado (pára no 1.º/último), para fora o 1.º filho, para dentro o pai, na raiz o filho do MEIO | espaciais (o mais perto na direcção) | — |
| `Delete` | apaga o ramo; selecção → pai | apaga o ramo; → irmão seguinte | — |

### §1.3 — Miro: a imagem oficial medida ao píxel (inferência por medição)

`Mind_Map_View-thumb-web.png` da página <https://miro.com/templates/mind-map-basic/> (1440×960, a 100 %; a
letra de 14 px). Régua: scripts Python sobre os píxeis no scratchpad da sessão, números transcritos aqui.

- **Passo vertical = 54**, sempre: as 7 folhas de «Use cases» em 175, 229, …, 499; a seguinte, de OUTRO pai
  («Add as many…»), em 553; «Give pre-made…» 716 → «All users…» 770. Entre subárvores o vão é o mesmo que
  entre folhas.
- **O pai no meio da faixa:** «Use cases» 337 = meio de 175…499; «More than a mind map» 634,5 = meio de
  553…716; a raiz 472 = meio de 175…770 (lado esquerdo) e de 202…742 (lado direito: «Different themes», com
  dois filhos, ocupa duas faixas). ⛔ Não é a média do 1.º e do último filho (daria 553,5).
- **Cada lado da raiz centra-se nela por si.**
- **Vão entre níveis = 122 de texto a texto**, contado da borda de FORA do pai (697 → 575, 607 → 485, 690 →
  568: as folhas de pais diferentes NÃO se alinham numa coluna); da pílula da raiz ao texto do filho, 113.
  A linha encosta a ~9 do texto ⇒ caixa do nó = texto + **9** de cada lado, vão de caixa a caixa **104**.
- **A curva:** cúbica com tangentes horizontais nas duas pontas e braço **40,5 = 0,39 × 104**, igual em
  todas as que se mediram (rms 0,32–0,97 px), seja qual for o desnível (135, 162, 298). ⛔ O braço das
  setas (0,45 da DISTÂNCIA) daria 142 na mais inclinada.
- Traço de **1,5**; uma cor por ramo da raiz (laranja `#ffab00`, violeta `#7a28ff`, roxo `#b736ff`, magenta
  `#fc66e7`, verde-água `#74e6d2`, os píxeis mais saturados); a raiz é uma pílula **170 × 66** (o texto de
  104 com 33 de cada lado = meia altura) com contorno lilás `#afa8fe`; os outros nós são SÓ texto.
- **Recolher não rearruma** (GIFs oficiais do help, quadro a quadro): com «Mind mapping features» e «More
  than a mind map» recolhidos, os vizinhos ficam a 35 e 66 da raiz, como antes. O nó mostra um selo da cor
  do ramo com o número.

## §2 — As leis do PH2D (lei → fonte → número)

| # | lei | fonte | número / teste |
|---|---|---|---|
| M1 | **Faixas**: a faixa de um nó é a soma das faixas dos filhos (com o vão entre elas) ou a altura dele, a maior; o pai no MEIO da faixa dos filhos | Miro §1.3; ME e SMM iguais | `the_parent_sits_in_the_middle_of_its_childrens_band` |
| M2 | Vão entre faixas vizinhas **igual a qualquer profundidade**: passo de **54** para nós de uma linha | Miro §1.3 (divergência escolhida dos dois oráculos, que abrem mais na prof. 1) | `single_line_siblings_step_fifty_four` · `cousins_step_like_siblings` |
| M3 | Vão entre níveis **104** de caixa a caixa (122 de texto a texto), a partir da borda de FORA do pai; a borda de DENTRO dos irmãos alinha | Miro §1.3 | `a_child_starts_one_level_gap_past_its_parents_outer_edge` |
| M4 | Cada lado da raiz centra-se nela por si | Miro §1.3 | `each_side_of_the_root_is_centred_on_it` |
| M5 | O lado de um filho da raiz é dele (guarda-se). `Tab` na raiz põe-no no lado MAIS LEVE (a soma das faixas), empate à direita; o «+» de um lado põe-no nesse lado | Miro: o «+» clicado decide; o teclado é silente ⇒ o XMind (equilíbrio pela altura). ⛔ não o alternar do ME nem a contagem do SMM, que ignoram o tamanho | `tab_on_the_root_fills_the_lighter_side` |
| M6 | `Tab` cria o filho **último**, seleccionado e em edição; a escrever, o `Tab` primeiro guarda o texto | Miro; ME = SMM | `tab_adds_the_last_child_and_edits_it` |
| M7 | `Enter` a escrever cria o irmão **logo depois**, em edição; na raiz, um filho. `Shift+Enter` quebra a linha | Miro; ME = SMM (a raiz: o ME) | `enter_adds_the_next_sibling_right_after` |
| M8 | Setas (seleccionado, não a escrever) **navegam**: para dentro = o pai; para fora = o filho mais perto do meio; ↑/↓ = o vizinho visível à MESMA profundidade e do mesmo lado (atravessa para o primo) | Miro silente ⇒ estrutural como o ME; ⚠️ divergência escolhida: o ME pára no último irmão e dá o 1.º filho — o primo e o do meio são o que o olho espera numa faixa | `arrows_walk_the_tree` |
| M9 | `Delete` apaga o ramo inteiro; a selecção passa ao pai | Miro; ME (SMM: o irmão) | `delete_takes_the_whole_branch_and_selects_the_parent` |
| M10 | Recolher esconde os descendentes **sem rearrumar**; o nó mostra um selo da cor do ramo com o número de escondidos; clicar no selo expande; `Ctrl+↑` recolhe e `Ctrl+↓` expande (e o botão da barra de contexto). Escondido = fora do desenho, do clique, do laço, da busca (sai de `BoardDoc::live`, a porta única) | Miro (GIF do help); ⛔ não o ME/SMM, que rearrumam. A tecla é a dos outliners (Workflowy): a `/` do XMind não chega ao quadro (a shell traduz só letras, algarismos e as teclas de edição, e a shell só encolhe) | `collapsing_hides_without_moving_anyone` · `collapsing_hides_and_the_badge_expands` |
| M11 | Auto layout ligado por omissão; desligado num pai, os filhos dele ficam onde a mão os pôs e **andam com ele** | Miro | `a_parent_without_auto_layout_carries_its_children` |
| M12 | Arrastar um nó: largado sobre outro nó → filho dele (o último); largado no vazio → muda de lugar entre os irmãos (e de lado, se cruzar a raiz); `Ctrl` = só reordena. As guias calam-se num nó arrumado pelo layout | Miro | `dropping_a_node_on_another_reparents_it` · `dropping_beside_siblings_reorders` |
| M13 | Os ramos são **derivados** (não guardados): cúbica da borda de fora do pai à de dentro do filho, a meia altura, tangentes no eixo, braço **0,39 do vão**; traço **1,5** à letra 14; a cor do ramo do filho | Miro §1.3 | `a_branch_is_the_measured_miro_curve` |
| M14 | Cor: cada filho NOVO da raiz recebe a seguinte da paleta medida (5); os descendentes herdam a do pai | Miro §1.3 | `a_new_branch_takes_the_next_colour_and_children_inherit` |
| M15 | A raiz nasce pílula 66 de alto (texto + meia altura de cada lado); os outros nós só texto, caixa = texto + 9. A largura cresce com o texto (sem quebra; a linha nova é do `Shift+Enter`) | Miro §1.3 | `a_node_grows_in_width_with_its_text` |
| M16 | Layout vertical (opção da raiz): as mesmas leis com os eixos trocados | Miro | `the_vertical_layout_is_the_same_law_turned` |
| M17 | O layout é invariante do documento: todo `apply_batch` que toca um mapa arruma-o no mesmo passo de desfazer; a raiz não se move. Os caminhos AO VIVO (escrever, colar, largar uma cópia) arrumam com `settle_live` e guardam só o seu inverso — desfazer volta a arrumar, porque o layout é função do documento | decisão (Etapa 2: uma operação remota também chega arrumada) | `undo_brings_back_the_layout_of_before` |
| M18 | Um nó novo mora na casa do PAI (o frame dele); a raiz, no frame onde nasce — o frame leva o mapa ao mexer-se | achado da foto da cena 8 (o mapa dentro de um frame; os filhos novos ficavam soltos) | `a_new_node_lives_in_its_parents_frame` |
| M19 | `Tab` e `Enter` a escrever só são «filho» e «irmão» num NÓ; numa nota o `Tab` continua a ser a nota seguinte e o `Enter` uma linha nova (W3); seleccionado, as setas de um nó andam pela árvore em vez de empurrar | a colisão de teclas do briefing §2 | `a_note_keeps_its_tab_and_enter` · `a_note_has_no_mind_child` |

### Modelos

| # | modelo | o que traz (Miro, [imagem oficial] da página do modelo) | onde e como |
|---|---|---|---|
| T1 | **Retrospective** (Start · Stop · Continue) | 4 áreas com título (Start, Stop, Continue, Action items), uma pergunta em cada, notas vazias da cor da coluna | 4 frames lado a lado |
| T2 | **Brainstorm** (Brainwriting) | linha «Participant 1…n», uma cor por pessoa, notas «Idea» por baixo | um frame |
| T3 | **Kanban** | Backlog · In progress · Done, raias por equipa, cartões | 3 frames, rótulos das raias |
| T4 | **Mind map** | raiz pílula, ramos de cores, nós só texto, dos dois lados | um mapa mental (M1–M15) |
| T5 | **Flowchart** | Start → Process → Decision (Yes/No) → … → End, setas em cotovelo | formas do fluxograma + setas presas |
| T6 | **Customer journey map** | Persona + estágios × (Actions, Touchpoints, Emotions, Pain points, Opportunities), notas por estágio | um frame, rótulos das linhas |

Inserir (o Miro é silente em tudo menos «chega desagrupado e por cima do que existe»): **no centro da vista**,
à escala do mundo (as notas de 199 de nascença), **seleccionado**, num passo de desfazer; o texto vem do
`ph2d-i18n` (inglês, como o resto da interface). A galeria é um botão da barra com os seis.

## §3 — ⛔ Recusas MEDIDAS

| proposta | estado |
|---|---|
| layout compacto (Reingold–Tilford / *tidy tree*, o do markmap) | ⛔ o Miro (§1.3) e os dois oráculos que editam são por faixas; compactar mudaria a posição de um ramo quando o vizinho cresce |
| os ramos como SETAS guardadas no documento | ⛔ uma segunda verdade sobre a mesma relação (apagar, recolher, mudar de pai teriam de manter duas coisas); o Miro não os deixa seleccionar |
| o braço das setas (`END_ARM` = 0,45 da distância) nos ramos | ⛔ medido §1.3: o braço do Miro é 0,39 do VÃO, igual em ramos com desníveis de 135 a 298 (o das setas daria 76 a 142) |
| vão maior entre os filhos da raiz (os dois oráculos: 61–67 e 44 contra 22 e 4) | ⛔ o Miro mede 54 a qualquer profundidade |
| recolher a rearrumar os vizinhos (ME, SMM) | ⛔ o Miro não rearruma (GIF do help) — o mapa não salta debaixo da mão |
| equilibrar os lados por índice (ME) ou contagem (SMM) | ⛔ ignoram o tamanho: um ramo de 9 nós conta como uma folha |
| alinhar as folhas numa coluna por profundidade | ⛔ o Miro mede o vão a partir de CADA pai (§1.3) |
| o layout num passo de desfazer à parte, ou como gesto do editor | ⛔ desfazer devolveria o texto e deixaria os vizinhos fora do sítio; a invariante (M17) cobre também quem não passa pelo editor |

## §4 — Medir antes de limitar (§0.0)

Régua `crates/ph2d-board-render/tests/it/measure_mind_layout_cost.rs` (molde da `measure_snap_cost.rs`):
um mapa de N nós (a raiz, até seis filhos por nó, larguras variadas, metade de cada lado). **Arrumar** =
`mind_settle` sobre o mapa já arrumado (o que cada lote que toca o mapa paga — uma letra num nó);
**Tab** = a tecla inteira pelo editor (o nó novo, o lote, o layout e os vizinhos que se mexem);
**taffy** = o MESMO layout pelo `ph2d-vec-layout` (uma linha `[nó, coluna dos filhos]` centrada por
subárvore). Variantes no mesmo processo, intercaladas, o mínimo de 7 rodadas (mediana ao lado),
`--release`, 08/10, `load` 64,9 na máquina partilhada (os números são um TECTO):

| N nós | arrumar | Tab | taffy |
|---|---|---|---|
| 100 | 0,066 ms (med 0,071) | 0,172 (0,216) | 0,291 (0,293) |
| 1 000 | 0,571 (0,618) | 0,889 (0,957) | 2,197 (2,944) |
| 10 000 | 7,26 (7,85) | 9,91 (10,93) | 27,96 (31,12) |

⇒ **nenhum tecto de nós**: a 10 mil (dez vezes o maior mapa que um planeamento à mão faz) uma tecla
cabe num quadro de 60 Hz, com a máquina a 65 de carga. ⇒ **o `taffy` exprime a lei** — o gate
`taffy_expresses_the_mind_layout` prova as mesmas posições (a < 1e-3: ele conta em `f32`) — **e custa
~3,8×**: montar a árvore de caixas e resolvê-la paga mais que a conta de três linhas que ela faz. Fica
a passagem própria (`ph2d_board_model::mind::layout`), com o `taffy` como prova da lei.

## §5 — O que o dono vê (cena 8)

`PH2D_BOARD_SMOKE=8` (`crates/ph2d-app-board/src/smoke_mind.rs`, foto `ferramentas/cena8_foto.png`): um
mapa mental começado — «Plan» com «Ideas» (dois filhos) e «Risks» (três filhos, RECOLHIDO: o «3» à vista)
à direita e «Next steps» à esquerda — e, por baixo, seis notas, uma por lição, com os nomes dos nós que
usam: `Tab` (nasce o filho, a escrever) · `Enter` a escrever (o irmão logo abaixo; `Shift+Enter`, linha
nova) · as setas andam e `Delete` leva o ramo · o número abre o ramo (`Ctrl+↑` volta a fechar) · arrastar
«Sketch» para «Next steps» · o botão dos modelos. Tudo dentro de um frame com a margem do que os painéis
e a barra de estado tapam: o 1.º quadro enquadra o quadro INTEIRO na janela (handoff W5b §10.1), e as três
primeiras fotos mostraram-no — a galeria aberta tapava o ramo da esquerda, depois a fila de baixo ficava
debaixo da barra de estado e as notas das pontas debaixo dos painéis. O gate
`every_mind_lesson_does_what_it_says` conduz cada lição pelo editor real com as medidas da interface, a
zoom 1 e 0,8.
