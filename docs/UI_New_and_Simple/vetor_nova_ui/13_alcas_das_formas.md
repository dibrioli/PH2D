# 13 — As alças das formas (F9, 11/10)

> Pedido do dono (11/10): *«tirar do inspector as opções de customização de cada shape e colocar nos
> gizmos, como apps de alta qualidade como o Affinity fazem.»* Prompt:
> [`12_prompt_F9_gizmos_das_formas.md`](12_prompt_F9_gizmos_das_formas.md). Código: a lei em
> `ph2d_tool_vector::shape_handles` (+ a tabela `shape_handles_table`), o mundo em
> `ph2d_app_vec::shape_handles_live`, o desenho `ph2d_vec_render::draw_shape_handles`, a cena 15.

## §1 — O desenho

- ⭐ **A alça É um ponto do contorno COZIDO** (`Spot`: um vértice, o meio de dois, o início do
  arredondamento de uma quina, um ponto da elipse da caixa) — nunca uma fórmula re-derivada ao lado
  da geometria. O que se agarra e o que se desenha saem do MESMO `cook`. Gate:
  `every_handle_sits_on_its_outline` (todas as formas × 4 valores, a `1e-6`).
- **O arrasto é uma MÉTRICA** com que esse ponto persegue a mão, dentro da faixa do DESCRITOR (os
  limites perguntam-se a `shapes::desc`, nunca se copiam). As métricas foram medidas (§2):

  | lei | o que mede | exemplo |
  |---|---|---|
  | `Nearest` | o ponto da alça o mais perto da mão — num trilho recto, a projecção | o raio do canto; as frações das setas |
  | `Radial` | o raio NORMALIZADO (a caixa vira círculo) da alça = o da mão | o vale da estrela; o furo da pizza |
  | `Count` | o vértice que a contagem põe mais perto do ângulo da mão | lados, pontas |
  | `Sweep` / `ArcStart` | o ângulo NORMALIZADO da mão, desenrolado desde o press; o outro fim parado | o fim / o início de um arco |
  | `Winding` | voltas inteiras da mão à volta do centro | as voltas da espiral |

- **A pega viaja com a mão** (o Inkscape faz o mesmo — §2): agarrar a alça pela borda não a faz
  saltar; o arrasto é uma função da mão desde o PRESS, não um acumulador.
- **Estacionar:** uma alça que, no mínimo, cairia numa alça da caixa (o canto de raio zero) ou no
  pivô (o furo zero) desenha-se a `PARK_PX` = meio lado da alça da caixa + o raio de acerto
  (`6 + 12 = 18 px`) do ponto mínimo, na direcção em que o campo cresce — o widget de canto do
  Illustrator nasce recolhido pela mesma razão. A pega mede-se ao ponto VERDADEIRO: agarrar a alça
  estacionada e não mexer não arredonda nada (gate `a_parked_handle_grabbed_and_held_still_changes_nothing`).
- **Modificadores** (teclas do Illustrator; «em pares» do Affinity): **Shift** = ângulos em passos de
  15° (o passo medido no Inkscape); **Ctrl** = o início do arco leva o fim (o arco RODA inteiro);
  **Alt** = só este canto (o rectângulo arredondado — sem Alt os quatro cantos andam juntos, o
  *Single radius* de omissão do Affinity).
- **Duplo-clique numa alça** repõe os campos DELA no valor de fábrica (o Affinity: *«double-click the
  red handle that has been previously moved»*); as outras ficam.
- **As portas:** só no **Select** (a seta preta) com a forma seleccionada; o press entra no cluster de
  alças do Select (texto em caminho, padrão, forma) ANTES do gizmo e do pick — dentro do raio dela a
  alça ganha, mesmo com Shift/Ctrl/Alt (ali são modificadores do arrasto). O arrasto escreve pela
  porta ÚNICA `shape_params::edit_selected_shape` (a mesma dos campos); o undo é o do diff do quadro
  (o botão premido segura-o: **um arrasto = um passo**); o **Esc** a meio repõe os valores do press.
  O toque entra pelo mesmo caminho (o reconhecedor da F6 converte o dedo em press/arrasto).
- **A guia:** com o cursor sobre uma alça de trilho, o trilho dela desenha-se (a guia vermelha do
  Affinity: diz para onde a mão a pode levar).
- **A cor:** o apelido `shape-handle` → `success`, MEDIDO contra as alças da caixa (`accent`,
  `accent-soft`, `selection`) nos 8 temas: pior caso `0,122` (Workshop). O vermelho do Affinity
  mediria `0,062` e o laranja `0,080` — no Sunstone o acento é laranja. Gate
  `a_shape_handle_never_wears_the_colour_of_the_box` (barra `0,10`, a da alça do osso).

## §2 — Os oráculos

### O Inkscape 1.4.4 — CORRIDO (`oraculo_alcas/`)

Triagem por artefacto (`pacman -Qi inkscape`): **GPL / LGPL** ⇒ caixa preta, só a SAÍDA sobre
formas nossas. `corre_inkscape.py` (Xvfb `:96` + xdotool, HOME isolado): 8 famílias (rectângulo,
rectângulo rodado 25°, estrela, estrela rodada, polígono, arco/fatia, arco rodado, espiral), cada uma
num documento com UMA forma; 107 arrastos (com e sem Shift/Ctrl/Alt), calibração tela→documento
MEDIDA por um rectângulo desenhado, controlo do último `Ctrl+Z` (8/8 repuseram a forma). Fixtures
`fixtures/<família>.json`; tabela: `python3 -I resume_fixtures.py`.

| lei medida | número |
|---|---|
| o `rx` do canto = a **projecção** da mão na aresta, a componente perpendicular não conta | `rx_x-40_y+25` = `rx_x-40` = 50,161 |
| o nó **guarda a pega** (o press + o deslocamento da mão) | 20/20 arrastos a 0,000; a mão absoluta erra 0,04–0,14 |
| satura em meia aresta | 100,000 (o canto dele é elíptico: `w/2`; o nosso é um círculo — satura em meia aresta CURTA) |
| o `r2` da estrela = a **distância** ao centro (não a projecção no raio) | a 15° fora do raio: 70,0 contra 67,6 |
| Ctrl no vale/ponta = só radial (sem torcer) | `arg2` = −54,00 |
| o `start` do arco = o **`atan2` normalizado** (a elipse vira círculo) | longe da curva a 40°: 40,1° (o ponto mais perto daria 31,3°) |
| arrastar o início deixa o **fim parado** | `end` = 270,00 nos 10 arrastos |
| Ctrl = passos de **15°** | 37° → 30° |
| Shift = roda o arco inteiro | `start` 37,01 · `end` 307,01 |
| as `revolution` da espiral só mudam com o **ângulo** | 150 e 70 px a 0° → 3,000; +90° → 3,250; −90° → 2,750 |

Os gates reproduzem-nos com os NOSSOS parâmetros (o ponteiro real pela calibração de cada fixture):
o raio a `2e-3` (18 arrastos), o vale a `5e-3` (12), os ângulos a `0,3°` (24), as voltas
arredondadas (6) — `shape_handles_tests.rs`.

### O Affinity — o MANUAL (a lei do produto do dono)

Não há build para Linux: lido o help oficial (Designer 2 e o app novo) e as figuras «handle position
(in red)». O que se adoptou: alças vermelhas **junto** das 8 da caixa; o arrasto segue um eixo próprio
(recto ou em arco); **duplo-clique repõe** só essa alça; Rounded Rect = 1 alça no canto de cima
(uma por canto com *Single radius* desligado); Pie = início e fim em arco + o furo a partir do
**centro**; Trapezoid = os cantos de cima na horizontal; Heart = o entalhe na vertical; Cog = o furo
do centro; Callout = o bico 2D; Arrow = espessura do corpo, comprimento e tamanho da cabeça.
**Contagens (lados, pontas, dentes) não têm alça** no Affinity (só a barra) — o Illustrator tem
(o *side widget*, 3–11 no polígono, 3–20 na estrela); nós seguimos o Illustrator com a faixa do
NOSSO descritor. Spiral: sem alças no Affinity; o Inkscape tem (as voltas pelo ângulo).

## §3 — Forma a forma

Os campos de cada forma e a alça de cada um vivem numa tabela só (`shape_handles_table.rs`); um
campo sem gesto natural (uma ESCOLHA, a suavização do canto) fica só na fila ② e a conta deles é
`shape_handles::fields_without_handle` (gate `the_fields_without_a_gesture_are_counted_not_listed`).

| forma | alças (campo → onde → lei) |
|---|---|
| Rectângulo arredondado | os 4 cantos, cada um no lado HORIZONTAL do canto → `Rounding` → `Nearest`; sem Alt mexe o raio base (os 4), com Alt só aquele (os outros guardam o raio) · a suavização: só na ② |
| Polígono | lados → 2.º vértice → `Count` · raio do canto → `Rounding` do 1.º |
| Estrela | pontas → 2.ª ponta → `Count` · interior → 1.º vale → `Radial` · arredondar a ponta / o vale → `Rounding` |
| Elipse / Pizza | furo → meio da fatia, do centro → `Radial` (estaciona) · início → `ArcStart` · fim → `Sweep` (numa elipse inteira o fim está sobre o início e ganha: puxá-lo abre a pizza) |
| Arco / Segmento | início → `ArcStart` · fim → `Sweep` |
| Espiral | voltas → a ponta de fora → `Winding` |
| Seta / Seta dupla | haste → o canto de trás dela → `Nearest` · cabeça (comprimento + largura) → a quina de trás da cabeça → `Nearest` 2D |

## §4 — Onde fica o número exacto

A secção da forma **saiu do Inspector**. O número exacto ficou na **fila ②** (a barra de opções de
cima), com os MESMOS ids dos campos de antes (`vector_shape_field_id(i)`, `vector_shape_choice_id(i)`):
é onde o Affinity o põe (a Context Toolbar) e onde o Illustrator o põe (o Control panel). Aparece com
a forma seleccionada (Select) e com a forma armada (Shape: o próximo desenho). Um arrasto de alça
re-semeia os números no quadro seguinte (`shape_params::frame_fields`, a 2.ª razão de semear).
⚠️ **Pergunta ao dono** (no relatório da F9): fica na fila ②, ou também num cartão do Inspector?

## §5 — ⛔ Recusas MEDIDAS

| proposta | porque não |
|---|---|
| reproduzir o SALTO do Inspector quando o fim do arco passa pelo início (270° → 70°) | a forma desaparece debaixo da mão; o nosso fim desenrola-se e SATURA na volta inteira e em 1° — a única divergência do oráculo, e o gate confere que ele saltou mesmo |
| trocar o TIPO por uma alça (o Inkscape troca fatia ↔ arco conforme a mão está dentro ou fora) | o tipo nunca muda por um arrasto (a mesma lei do Edit Shape, F7e) |
| torcer o vale (`arg2` do Inkscape) | o nosso catálogo não tem o parâmetro; o vale lê só a distância (o Ctrl do Inkscape) |
| a alça no ponto MAIS PERTO da elipse | longe da curva dá 31,3° onde a mão aponta 40° (o oráculo diz `atan2` normalizado) |
| a alça a saltar para o centro do ponteiro no press | o oráculo guarda a pega (20/20) |
| o vermelho/laranja do Affinity | 0,062/0,080 contra as alças da caixa no Sunstone (barra 0,10) |
| uma alça nas ESCOLHAS (o «Viewed» das isométricas) | não há gesto natural para um booleano: fica na fila ② |

## §6 — A cena 15

`PH2D_OBJECT_MODE_SMOKE=15`: no layout *Vetor*, um rectângulo arredondado, uma estrela e uma pizza;
a Seleção pelo botão dela; cada forma sozinha na seleção e UMA alça arrastada pelo MESMO press e o
MESMO arrasto da shell (`shape_handles_live::on`): o canto arredonda (+0,45 m), o vale abre, o fim da
pizza dá a volta até aos 300°. Gate `the_scene_pulls_a_corner_a_valley_and_the_end_of_a_pie`.
