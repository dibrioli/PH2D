# 05 — Especificação COMPLETA da UI de cartões (para a linha de implementação real)

> **Estado desta linha.** `line/TrelloUITeste` (worktree `Worktrees/line-TrelloUITeste`) foi **só um
> TESTE de UI**: nunca foi integrada e nunca teve push. O dono **fechou-a em 02/11/2026**
> («Já conseguimos testar a UI e ela funciona muito bem e é linda»). A implementação real será uma
> **linha nova a partir do `main`**, que lê ESTE documento primeiro. O código desta linha é
> referência de leitura (mapa em [03](03_mapa_do_codigo.md)); o que sobrevive como lei são os docs.
>
> **MUITOS outros painéis NÃO foram trabalhados** porque a linha era só um teste de UI. Trabalhados
> (e só estes): **Inspector em modo cartões** (Objeto, Transform, Appearance, Fill, Stroke, Ordering,
> Material & Blend, pé «+ Add component», cartões de COMPONENTE com o corpo de hoje dentro, o
> Physics Body com pares, o Platform Player), **cartões soltos no canvas** (instâncias, alfinete,
> corda, retrato do preso, face por cópia), **painel Bones** (piloto, 7 cartões), **estilos de cor
> do Vector (Fill/Stroke)** e o menu «…» da lista. **Todo o resto está por fazer:** Hierarchy,
> Timeline, Painter, Pixel, Flip, Motion, Kanban/Quadro (só a mola foi extraída), Áudio, Código,
> Constraints, Luz 2D, Macros, Game Engine (HUD/Câmera na lista), Assets, Project, etc. — e, mesmo
> nos painéis tocados, o que está listado em «ABERTO» (§9).

## 0 — Regra de referência (ordem vinculativa, dono 02/11)

1. **Referência PRIMÁRIA somos NÓS**: os painéis/cartões cujo smoke o dono APROVOU nesta linha
   (tabela §1). Antes de desenhar, olhe o que já foi aprovado (fotos em [`maquete/`](maquete/)) e repita-o.
2. **Figma** — para qualquer widget ainda não criado, consulte o Figma PRIMEIRO (layout, nomes das
   propriedades, o campo numérico com glifo, camadas empilháveis, estilos).
3. Complexidade que o Figma não resolve ⇒ **Rive** é a segunda referência mais importante.
4. Ainda por resolver ⇒ **Inkscape, Godot e Unity** podem servir de referência.

Corolário (CLAUDE.md §0.9): outro app que já faz isto é ORÁCULO que se corre, não fonte que se lê;
triagem de licença por artefacto primeiro. Alvo do dono: superar o Figma e a Godot (D8).

## 1 — Tabela «Smoke OK» (só o que os docs registam como aprovado pelo dono)

Decisões em [01](01_decisoes_do_dono.md); fotos em [`maquete/`](maquete/); comandos de cena: ver §10.

| o quê | decisões | data do smoke OK | cena | foto |
|---|---|---|---|---|
| Maquete A/B (Bones, scrub, cartão no ar) e C (Inspector) e D (solto) — referência visual VINCULATIVA | D9 · D14 | 01/11 | — | `bones.png`, `canvas.png` |
| Kanban com a mola extraída (`ph2d-card-stack`) igual a antes | D19 (C1) | 01/11 | `PH2D_KANBAN_SMOKE=1` | — |
| Campo Figma (scrub no ícone, cursor preso `Locked`) + Bones em cartões | D20 (C2, C3), T12 | 01/11 | `PH2D_VEC_BONE_SMOKE=8` | `antes_painel_bones.png` (o «antes») |
| Inspector em cartões, tema do cartão (T25), notas, ícones +30 % (D22), menu «…» (D21) | D23 (C4) | registado na ordem de 01/11 (a linha de D23 não traz data própria) | `PH2D_VEC_CARTOES_SMOKE=1` | `c4_maquete_x_app.png`, `c4_tema_do_cartao_antes_depois.png` |
| Lista flutuante sem coluna escura, alça na beirada; «+» Add component no topo só em Object | D24 · D25/D25b · D26 · T26 | 01/11 | 1 | `d24_antes_depois.png`, `d25_mais_no_topo.png` |
| Length do osso em px; cartão solto (C5); componentes em cartões; lista rola; dicas em todo ícone mudo | D27 · D29 · T29 · T30 · T31 · T32 · T33 · T34 | 02/11 | `PH2D_VEC_CARTOES_SMOKE=1\|2` | `d27_length_px.png`, `c5_cartao_solto.png`, `c4_componentes_e_rolagem.png` |
| Estilos de cor LIGADOS (grelha ⠿, busca, «Edit style»); lista abaixo da régua; sem animação de hover; segmentos pela largura do conteúdo | D28 · D30 · D31 · D32 · T35 | 02/11 | `PH2D_VEC_CARTOES_SMOKE=3\|4\|5` | `d30_estilos_ligados.png`, `d31_lista_abaixo_da_regua.png`, `d32_bend_largura_do_texto.png`, `b3_*.png` |
| Alfinete + subtítulo («Bone 5»), auto-scroll na borda, barra com faixa própria | T36 · T38 | 02/11 | `PH2D_VEC_BONE_SMOKE=8` | `c5_alfinete_e_indisponivel.png` |
| Bloco 4: largura do solto, preso sem tirar a secção da lista, animações suaves, visibilidade do solto, barra que não atravessa o fundo | T39–T43 | 02/11 | idem | `b4_t39_solto_largo.png`, `b4_t43_barra_antes_depois.png` |
| Alfinete com CORDA (3 estados), solda, vertical, sem travar ao voltar | T44 · T44b · T44c · T44d | 02/11 | `PH2D_CARTAO_CORDA_SMOKE=1` (com `PH2D_VEC_BONE_SMOKE=8`) | `b5_t44*.png` |
| Alfinete gravado por `StableId`+nome; entradas/saídas suaves; alfinete+corda no Transform do Inspector | T45 · T46 · T47 | 02/11 | idem | `b5_alfinete_gravado_e_inspector.png` |
| Corda por cima dos outros soltos (T48 revogada); solto com a aba atrás; corda só rompe ao encaixar; relógio da corda por painel | T50 · T51 · T52 · T53 | 02/11 | idem | `b6_smoke_curas.png` |
| Uma cópia solta por objecto (instâncias); todo solto anda com a mão | T54 · T55 | 02/11 | — | `b7_duas_copias.png` |
| Physics Body com dois campos por linha | T56 | 02/11 | `PH2D_VEC_CARTOES_SMOKE=2` | `t56_fisica_dois_por_linha.png` |
| Solto alto rola por dentro; cartão aberto arrasta; Player sem sobreposição e em pares | T57 · T58 · T59 | 02/11 | `PH2D_VEC_CARTOES_SMOKE=6` | `t57_t59_player_solto_*.png`, `smoke_0211_player_solto_*.png` |
| ↺ (reset) em todo cartão de componente | T60 | 02/11 (após a cura da re-semeadura) | 6 (e 2) | `t60_reset_antes_depois.png` |
| Alfinete em todo cartão de componente; Ctrl+Z re-semeia as caixas | T61 · T62 | 02/11 | 6 (e 7) | `t61_alfinete_antes_depois.png` |

### 1.1 — Implementado, smoke NÃO confirmado / parcial

| o quê | decisões | estado honesto |
|---|---|---|
| Tema/contorno/notas por CÓPIA solta («Leva cor e notas» ao nascer; «Passam para a lista» ao voltar) | T63 · commit `9f647fd135` | implementado e testado (`each_loose_copy_keeps_its_own_theme_and_notes`, 7/7 contraprovas, foto `t63_tema_por_copia_antes_depois.png`); o dono NÃO o comentou no smoke de 02/11 ⇒ «implementado, testado, smoke não confirmado explicitamente» |
| Caixas de marcar à Figma no cartão (22 + 8 chamadas por `paint_check_row(s)`) | T64 · T29 parte 1 · `31c228f299` | teste `a_check_in_a_card_is_a_figma_checkbox_row` verde; sem foto; **o smoke do dono revelou o cartão Camera ainda defeituoso** (§8) |
| Cara de chip nas listas suspensas dentro do cartão (`paint_dropdown_chip` → `corpo::chip`) | T29 parte 1 | só pinta; nenhum teste a vê; sem foto; o popover aberto continua no tom do Inspector |
| T29 2.ª metade (corpos dos componentes com a cara do cartão) | T29 | corpos de 1.ª metade aprovados (rótulo em cima, campo Figma, segmentos pelo conteúdo); o resto em §9 |
| iPad (pressão longa, scrub relativo) | T49 | NÃO construído: não há fonte de toque nem porta de pressão longa com relógio |
| Travamento com Scale X enorme | — | não reproduzido com a placa livre (até 1e9 ≤ 30 ms/quadro; ≥1e10 os quadros esperam a placa) |

## 2 — A lista e o cartão (geometria; valores = `crates/ph2d-card-stack/src/metrics.rs`)

Anatomia completa e desenho ASCII em [02 §1](02_especificacao.md). Resumo vinculativo (px lógicos; **nunca
literal na pintura — HR-15: a geometria vive em `metrics`**):

- **Lista**: flutua sobre o canvas, sem fundo de dock (D16/D24); `LIST_RADIUS` 12 · `LIST_MARGIN` 5 (5 px da régua e da borda, D31) ·
  `LIST_PAD_X` 8 · `LIST_PAD_BOTTOM` 8 · `LIST_BAR_GAP` 4 (faixa da barra de rolagem) · `LIST_HEAD_H` 44 ·
  título 13 px/700, tracking `LIST_TITLE_TRACKING_EM` 0,06 · sombra `LIST_SHADOW_FAR` (6,18) + `LIST_SHADOW_NEAR` (1,2) · pé `LIST_FOOTER_H` 32 («+ Add component», T5). A altura abraça o conteúdo e rola (T30, a porta de rolagem de todos os painéis). A alça de largura é a beirada esquerda da lista (D24, T26).
- **Cartão**: `CARD_RADIUS` 8 · `CARD_PAD_TOP` 8 · `CARD_PAD_X` 10 · `CARD_PAD_BOTTOM` 10 · `CARD_GAP` 8 · `CARD_OUTLINE_GAP` 4 (contorno decorativo, D6).
- **Cabeçalho**: `CARD_HEAD_H` 26 · `CARD_HEAD_BOTTOM` 4 · título `CARD_TITLE_PX` 12/600 · ícones `HEAD_ICON_BOX` 24 / `HEAD_ICON_GLYPH` 16 / `HEAD_ICON_RADIUS` 6 · seta de dobrado `CARD_CHEVRON` 14 + `CARD_CHEVRON_GAP` 2 (T19).
- **Rótulo**: `LABEL_PX` 11 · `LABEL_TOP` 8 · `LABEL_BOTTOM` 4 · marca de animação `KEY_MARK_PX` 6 (box `KEY_MARK_BOX` 12).
- **Campo**: `FIELD_H` 28 · `FIELD_RADIUS` 6 · `FIELD_GAP` 6 · `FIELD_ROW_GAP` 6 · `FIELD_PAD_R` 8 · `FIELD_BORDER` 1 · célula do ícone `ICON_CELL` 26 (é a PEGA do scrub) · glifo `ICON_GLYPH` 18 (D22: 14 + 30 %; os do cabeçalho ficam em 16) · valor `VALUE_PX` 12 · `FIELD_VALUE_MIN_W` 56 (mínimo do campo ao lado da caixa da cor).
- **Segmentado**: `SEG_PAD` 2 · `SEG_RADIUS` 4 · `SEG_TEXT_PX` 11 · `SEG_ICON_GAP` 5 · `SEG_INSET` 6 · larguras por `seg_widths` (D32: cada segmento tem a largura do que mostra; só-ícone fica estreito; a falta corta só os de texto).
- **Caixa de marcar (T64)**: `CHECK_BOX` 16 · `CHECK_GAP` 8.
- **Scrub**: `SCRUB_SHIFT_MUL` 10 · `SCRUB_ALT_MUL` 0,1 (a convenção do Figma, [02 §3](02_especificacao.md): varrer os consumidores do arrasto numérico antes de trocar o Shift que hoje afina).
- **Solto**: `LOOSE_GHOST_H` 34 · `LOOSE_DASH` (4,3) · `LOOSE_SHADOW_FAR` (8,20) + `NEAR` (1,3) · `LOOSE_KEEP_VISIBLE` 48 · `LOOSE_FLASH_S` 0,6 · `LOOSE_EDGE_GRIP` 8 · `LOOSE_MIN_W` = 2·`CARD_PAD_X` + 3·`HEAD_ICON_BOX` + `LOOSE_MIN_TITLE_W` (72) = 164 · `LOOSE_PIN_BLINK_S` 1,6 / `_MIN` 0,3.
- **Corda**: `LOOSE_ROPE_W` 2 · `KNOT_R` 4 · `WELD_R` 5 · `POINTS` 32 · `ITERATIONS` 12 · `SLACK` 1,12 · `SLACK_MIN` 24 · `PLUMB_TENSION` 0,06 · `GRAVITY` 1800 · `DAMPING` 0,06 · `BEND` 0,4 · recolher `LOOSE_ROPE_REEL_S` 0,35 s.
- **Largura**: referência 320 px de dock; tem de funcionar ESTREITO (iPad, D13): as caixas encolhem, o número nunca colide com o ícone, o rótulo pode reticenciar ou **quebrar em duas linhas na última palavra que cabe** (`cards_rotulos.rs`), nunca a unidade dentro da caixa.

## 3 — Cabeçalho do cartão

- Título (clique = dobra/desdobra; dobrado: «›» antes do título, título um tom abaixo `Text2`, T19) + **resumo** à direita quando dobrado (ex.: «Z 3», T4).
- **Ícones à direita**, registados DEPOIS do cabeçalho (T34: senão o rect do cabeçalho os cobre): ações da secção (Rest Pose, Mirror, Release, Look At…, T2), olho (o MESMO id do olho do cartão Objeto, T21), «–» (tirar), «+» (secção ausente = cartão só-cabeçalho esmaecido, T3), **↺ reset** em todo cartão de componente (T60: repõe o componente principal + linhas em cadeia pelo Desfazer), **alfinete** (T7/T36/T44/T61), «×» no solto (= *Return to panel*).
- **Todo ícone sem texto tem tooltip** (T31; censo `every_icon_in_a_card_has_a_tooltip`); o campo numérico sem dica autorada mostra «<Rótulo> — Drag the icon to change · Shift ×10 · Alt fine».
- **Grip**: o cabeçalho (título ou zona livre, nunca os ícones) é a pega de arrastar o cartão; limiar ~4 px. Menu do botão direito no cabeçalho = tema/contorno/notas (D6). No cartão de componente o cabeçalho é o da secção ABSORVIDO (`absorb_next_header`, T29); T58: a moldura do corpo não pode registar o cabeçalho absorvido por cima do título.
- **Menu «…» da lista** (D21): Reset Positions · Reset Colors · Reset All (as notas NÃO se apagam). Resets e devoluções animam com mola (T41).

## 4 — Corpo: tipos de linha (vocabulário)

Lei de layout ([02 §2](02_especificacao.md)): **rótulo EM CIMA** (obrigatório se não for inteligível sozinho, D10) **com a unidade nele** («Length (px)», D13); um par de uma grandeza = um rótulo, letras de `Eixos` (X·Y, W·H, X·Y·W·H); duas coisas irmãs = dois rótulos sobre cada caixa (T56). Glifo de um campo só = a letra DECLARADA ou a inicial da ÚLTIMA palavra do rótulo.

| tipo | desenho |
|---|---|
| **Campo numérico Figma** | caixa `CardField` com glifo (ícone/letra) na `ICON_CELL` que é a pega do scrub; arrasto sem fim com cursor preso (`Locked`+movimento cru; reserva `Confined`+reposicionar; T12 medido); Shift ×10, Alt ×0,1 a meio sem salto; Esc devolve; duplo clique no ícone = fábrica; texto aceita contas e sufixo de unidade; ↑↓; «Mixed»; um gesto = um Desfazer; dois passos (setas × arrasto por px, T10) |
| **Segmentado** | pílula `SEG_*`, segmento ativo com sombra leve; ícone e/ou texto; larguras pelo conteúdo |
| **Lista suspensa** | chip `corpo::chip` (caixa do cartão, valor, seta); opções com os ids do painel dono, popover pintado no fim da lista (T16) — **ainda no tom do Inspector (ABERTO)** |
| **Amostra de cor** | `Swatch` + código hex (Enter = escolher no seletor partilhado, T28), opacidade ao lado; estilos ⠿ (D28/D30) |
| **Botão / Bare / Badge / Text** | botão de destaque (ex.: «Bind to Skeleton»), ícone nu, emblema, texto |
| **Caixa de marcar (T64)** | quadradinho `CHECK_BOX` com ✓ (acento do cartão) ou vazio, nome à direita, a linha inteira é o alvo, marca de animação no fim; 2 por linha se os dois nomes cabem |
| **Marca de animação (D17)** | ponto 6 px (Livre) · losango vazio (Animado) · cheio (Chave) · cheio âmbar (Mudado); na linha do rótulo, à direita da caixa; um par = uma marca; ajustes de ferramenta sem marca; ids = os que o Vector já marca (T22) |
| **Camadas empilháveis** | Fill/Stroke: `[amostra│código] [opacidade] 👁 –`, cabeçalho com ⠿ e «+» |

## 5 — Cor e tema: `CardInk` (T25)

TODA cor de dentro do cartão passa por `widget::card_ink::CardInk` (`of_card(store, secção, tema da lista)` → `.color(token)`). Sem tema próprio, ou com o da lista ⇒ tokens `Kanban*` (byte a byte o aprovado). Com tema próprio ⇒ `derive::card_tone`: fundo `bg-1` (cerca do OLED), caixa −1 degrau, hover −½/−1½, aceso +1/−½; `Text3`/`Accent`/`AccentFg`/`Text1` no lugar de rótulo/azul/tinta. O tema pinta o cartão INTEIRO (fundo, caixas, segmento, lista suspensa, rótulos, pontos); o contorno decorativo envolve cartão + notas a 4 px; notas = post-its dentro do cartão. Tokens: `docs/design/tokens.json` (`kanban-*`, `card-field`, `card-field-hover`, `card-label`). **Proibido** `paint_text`/hex directo com cor do Inspector dentro de um cartão (ABERTO: 76 casos).

## 6 — Arrastar e largar

Kanban ([02 §4](02_especificacao.md)): voo com inclinação 4° (pivô 60 %), sombra de voo, sem escala; vão por mola criticamente amortecida (`ph2d-spring::Follow`); auto-scroll a 64 px da borda, ≤ 900 px/s, só com a mão dentro da lista (T38); largar assenta por mola. **Lei T41/T46: toda animação de cartão é suave, sem saltos** (a secção que entra nasce fechada e abre com a mola; a que sai fecha-se pintada pela FOTO do último quadro; o solto que a seleção esconde esmaece; a corda solta recolhe). Sem animação de hover nos widgets (D32a: o fundo acende e apaga no quadro).

## 7 — Cartões soltos no canvas

- **Descolar/largar (T33)**: sair da coluna fecha o vão; largar fora = solto em coordenadas de ECRÃ; fica a linha-fantasma tracejada «<Título> · on canvas» (clique traz e pisca). Persistência em `sections.txt` (`loose.<secção>=<painel>,x,y,w`, 5.º campo = título), por MÁQUINA.
- **Largura (T39)**: bordas esquerda/direita arrastáveis, mín. `LOOSE_MIN_W`, máx. o canvas. **Rolagem (T57)**: solto mais alto que o canvas rola por dentro (cabeçalho fica; pára acima do HUD de baixo).
- **Instâncias (T54)**: o solto é uma INSTÂNCIA por objecto (`loose_instance_of(secção, n)`; cópia 1 = a própria secção); com a cópia presa, soltar de novo cria a cópia 2; cópia que segue a seleção deixa a linha-fantasma. **Todo solto anda com a mão (T55)**.
- **Visibilidade (T42, revê T37)**: não preso + outra classe ⇒ some (esmaece, T46); mesma classe ⇒ segue a seleção; preso ⇒ fica e o nome do sujeito («Bone 5») é destacado (`Accent`) e pisca devagar com o rato por cima. Cartão preso não tira a secção da lista (T40).
- **Alfinete e corda (T7/T36/T44/T47/T61)**: 3 estados pelo mesmo ícone — solto → preso → preso COM corda → solto (dicas «Pin to this object» · «Show a rope to this object» · «Unpin (follow the selection)»). Alfinete durável: `loose_pin.<secção>=<StableId>,<corda 0|1>,<nome>` (T45; nome impede prender a estranho noutro projecto). **Retrato do preso (T61)**: a shell publica o retrato POR OBJECTO (não pela seleção); o painel instala-o ao pintar, ao semear (ensaio) e no gesto; vale para Bones (osso) e para todo cartão de componente do Inspector. Objecto apagado ⇒ solta-se sozinho. Custo ≈ 90 µs/quadro por preso (cena 7), com cache pela chave.
- **Corda (T44–T53)**: lei do nó `motion.verlet-rope` (porta pura `ph2d_node_motion_verlet_rope::corda`), da borda do cartão ao MEIO do objecto; solda de 5 px na cor do cartão; cor do corpo do cartão (`KanbanCard`); o troço que passa por cima de outro solto pinta-se no tom `CardField` (T50, REVOGA o contorno da T48); fio de prumo na vertical; só rompe quando o cartão ENCAIXA ao largar (T53, `CardLists::suspended`); relógio por painel (T52).
- **Ctrl+Z (T62)**: as caixas re-semeiam-se a cada quadro, salvo as vivas (foco, arrasto, botão premido) e enquanto há pedido no barramento.
- **Face por cópia (T63)**: cada solto tem uma FACE (`loose_face_of(k)`) com tema, contorno e notas. Nasce COM os da lista («Leva cor e notas»); depois cada um muda sozinho; ao VOLTAR à lista (arrastar de volta, «×», encaixar o preso) tema e notas da cópia **passam para a lista** (notas somam-se). Menu do botão direito sobre o solto escreve na face; «Create Note» no corpo dele cria a nota da face.
- **Clique**: o solto é chrome — recebe o clique antes do canvas (`panel_at` pergunta-lhe ANTES).
- iPad: pressão longa para levantar — NÃO construída (T49).

## 8 — Camera card (smoke de 02/11 do bloco 11): defeitos ABERTOS

Palavras do dono sobre o cartão **Camera** no modo cartões: «cartão câmera todo bugado, sem título, mal organizado, com cores diferentes do padrão, Ponto de animação mal posicionado». Depois: «Mas já chega para essa linha. Já conseguimos testar a UI e ela funciona muito bem e é linda.» Registados como ABERTOS para a linha real (não diagnosticados no código nesta janela; a causa abaixo é a hipótese pelo que o T29 parte 1 deixou):

1. **Sem título** — o corpo começa sem o cabeçalho/título do cartão (o cabeçalho absorvido não foi pintado ou o cartão da câmera não passa por `cards_componentes::cartao`).
2. **Mal organizado** — estilos misturados: linhas de formulário antigas (ex. «Game Format» em lista suspensa; «Frame» segmentado) com linhas de cartão; o par Active / Look Through com o rótulo de Look Through elidido.
3. **Cores diferentes do padrão** — linhas pintadas com tokens do Inspector em vez de `CardInk`; caixas escuras de formulário em vez do `CardField` do cartão.
4. **Ponto de animação mal posicionado** — o ponto fica entre as duas caixas de marcar / fora da linha do rótulo (D17: tem de ficar na linha do rótulo, à direita da caixa).

Cura provável = fechar a 2.ª metade do T29 (§9) para a câmera, e fotografar o cartão Camera ABERTO antes de dizer que passou (§11b).

## 9 — ABERTO para a linha real (consolidado)

**T29 2.ª metade** ([04](04_plano_de_construcao.md) «Falta do T29», handoffs 3–7/8/9/10):
- 6 `Checkbox::new` directos: `anchor_mount_row`, `slice_nine` ×2, `visibility` ×2, `script_controlo` (confirmado por `git grep` em `ph2d-panel-inspector/src/sections`).
- 11 chamadores de `property_label_row` (o nome vai numa coluna que o chamador monta; converter pede o chamador).
- Popover das listas suspensas ainda no tom do Inspector (só o chip tem a cara do cartão).
- `paint_text` directos fora de `CardInk` (a medição do bloco 11 diz 76; contagem solta de hoje 104 em `sections/` — recontar com critério antes de prometer).
- Restos da contagem do bloco 3–7: listas suspensas ~109, avisos 123 (frases, já quebram), número directo 8, grupos de cor 4.
- Camera card (§8).
- O fold/dobra pertence ao id da secção e é partilhado entre a cópia 1 e o cartão da lista (a face T63 cobre tema, contorno e notas, não a dobra).
- Caixas de marcar/listas presas: a sobreposição de valores do preso leva números; confirmar caixas e listas no preso.

**Solto**: um solto colado ao topo do canvas tem o cabeçalho sob a barra de modos («Rail Backdrop» regista-se depois) — o alfinete não se clica ali · a corda só se pinta por cima dos soltos do SEU painel · iPad (T49).

**Outros antigos**: dobra do cartão Tool não reproduzida · Skin não aparece com osso no Transform · listas de ações do Smart Bone e pop-ups de Tip de um cartão preso leem o osso em foco · atalho logo após clicar num cartão preso vai ao osso preso · Travamento com Scale X (repetir sem contenção de placa).

**Vermelhos que NÃO são desta linha** (a tratar na mudança definitiva): `ph2d-panel-inspector --test it` `seam_*` (134–139, instável) · `node_id_collisions::every_non_literal_hash_is_named` · 16 em `ph2d-editor-core --tests` e 16 em `ph2d-panel-skeleton/tests/seam.rs` (lista no handoff dos blocos 3–7) · censo `toast_reading` do `ph2d-app-kanban`.

**Ferramenta**: `.claude/hooks/prazo-do-agente.sh:17` sem `--path-format=absolute`; existe um `docs/CartoesUI/.git/ph2d-agentes/` perdido (apagar com nenhum agente activo).

**Mudança definitiva** ([04](04_plano_de_construcao.md) final): varrer consumidores do arrasto numérico antes de trocar o Shift; `PanelEvent=4` e §6 não mudam; regras de rótulo/unidade candidatas a gate de censo; Fill/Stroke no Inspector ⇒ o painel Vector deixa de os repetir (decidir); medir a largura mínima legível no iPad.

## 10 — i18n, HR-15 e como rodar

- **Geometria** em `metrics` (nunca literal); **cor** por `CardInk`/tokens; **texto** em `ph2d-i18n` (`crates/ph2d-i18n/src/cartoes.rs`, chaves `card.*`) — o app é SÓ em inglês ([memória](../../project-memory/feedback_app_ui_english_only.md)), ajuda/tooltip em Plain English.
- Fonte padrão **Noto Sans** (`UiFont::NotoSans`; D11); outras escolhíveis. Algarismos tabulares da fábrica (T8).
- Cenas: `PH2D_VEC_CARTOES_SMOKE=1` Inspector · `2` componentes (Rigid Body+Collider+Vida) · `3`/`4` estilos (lista aberta em 4) · `5` «Edit style» · `6` Platform Player · `7` Player solto preso (custo/foto T61/T63); `PH2D_VEC_BONE_SMOKE=8` Bones; `PH2D_CARTAO_CORDA_SMOKE=1` pede a corda. `PH2D_CARDS=0` volta ao painel de hoje (comparação, T13).
- Comando (a partir do worktree): `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TrelloUITeste && PH2D_VEC_CARTOES_SMOKE=6 cargo run --profile smoke -p ph2d-host-desktop` (confirme o binário/variáveis em [04](04_plano_de_construcao.md) antes de reproduzir; fotos por `docs/Components/ferramentas/fotografa_cena.sh <VAR=valor> <w> <h> <saida.png>`).
- Código: endereços em [03](03_mapa_do_codigo.md) (não duplicados aqui). Gates relevantes: `the_shell_only_shrinks` (a shell NÃO cresce; `panel/cards.rs` nunca cresce), `architecture_every_editable_field_is_a_channel`, `every_icon_in_a_card_has_a_tooltip`.

## 11 — Lições de processo que valem para a linha real

a. **Id da instância == id da secção na cópia 1** faz toda preferência por id ficar partilhada com o cartão da lista (T63): use uma CHAVE derivada (face), nunca o id cru.
b. **Uma foto do corpo de um cartão exige o cartão aberto e acima da dobra**; o script de foto não clica ⇒ semeie um solto (`FOTO_HOME_SEMENTE`) ou escolha a cena; um cartão abaixo da dobra "passa" sem ser visto (foi assim que o Camera escapou).
c. **Contraprova por patch reverso** é o jeito limpo de tirar a foto «antes» com a cura INTEIRA desligada: `git diff > p; git apply -R p; foto; git apply p`. Meia cura desligada faz a foto mentir.
d. Medir custo por quadro com instrumentação temporária marcada e retirá-la antes do commit; verificar com cache pela chave.
e. Teste de clique sobre título: use a pega MAIS LARGA com o id (`iter_registrations`), nunca `rect_for`.
f. Moldura pintada antes do conteúdo: meça a altura numa cena à parte.
g. Uma escrita no mundo que o Inspector deve mostrar sem trocar de objecto exige re-semear as caixas; estenda o teste até ao número na caixa.
h. `fotografa_cena.sh` RECOMPILA a árvore: o «antes» tira-se desligando a cura, não com o binário antigo.
