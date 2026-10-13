# 24 — Arredondar as quinas em toda forma que as tem (F14, 13/10)

> Pedido do dono (13/10): *«Um ajuste legal que ainda está ausente em muitas formas é o ajuste que
> arredonda as quinas. Poderia existir em todas as formas que tem quinas, desde que não crie
> problemas»*. Prompt: [`23_prompt_F14_…`](23_prompt_F14_arredondar_as_quinas.md).

## §0 — A lei, uma porta

- **Um campo apenso**, o ÚLTIMO de cada forma, `radius("Corner round")` (`Px`, neutro em zero — o
  rótulo da pizza, a tradução já existia). A tabela `ShapeKind::corner_round_field` (cena) diz o
  índice; o gate `the_corner_round_field_is_the_last_and_neutral` prova que o catálogo concorda.
- **Uma função**: `ph2d_vec_scene::round_shape_corners(path, r)`, chamada pelo `cook` por cima do
  contorno já feito (o sítio onde o *Smooth* da F13 corre). Ela carimba o raio em TODO vértice e o
  motor da quina viva (`corner_live`, ADR-0121: o tecto `min(r/tan(θ/2), ½ corda mais curta)`, G1
  junto das curvas, satura sem pânico) arredonda as que a GEOMETRIA vê.
  - O tipo do vértice NÃO decide: medido, a engrenagem marca os 32 cantos dos dentes como lisos e o
    documento 2 (`VertexKind::Smooth` com tangentes que viram).
  - Contornos: o principal e todo sub-contorno FECHADO. As linhas ABERTAS de um composto são desenho
    sobre a face (as arestas do cubo e da pirâmide, as dobras da faixa, as barras da sub-rotina):
    não arredondam, e a ponta que tocava um contorno segue-o pela própria direcção (o corte da recta
    da ponta com o contorno cozido, o mais perto). Gate `the_lines_on_the_face_keep_touching_the_outline`.
- A pizza e a elipse-fatia perderam a função delas (`round_slice_corners`): passam pela mesma porta.
- O **Motion** (`motion_shape_gen::build_shape_path`) tinha o seu próprio arredondamento geral (só o
  contorno principal): agora chama a mesma porta.
- **Persistência**: `VecShape::Param.values` é `[f64; 8]` fixo e o índice novo de cada forma era um
  lugar vazio (zero) — um save antigo lê `0` = a forma de sempre. `PROJECT_SCHEMA` não muda.

## §1 — Duas curas no motor da quina (medidas)

1. **O `acos` mentia perto de ±1.** O teste de colinear era `π − acos(dot) ≤ 1e-9`, mas o `acos` só
   resolve `~1,5e-8` rad ali (√ε): o vértice liso de uma elipse esticada (seno EXACTO `0`, medido
   pelo produto vetorial) lia-se como quina de `2,1e-8` rad e arredondava — o contorno achatava (o
   gate da F12 `only_the_slice_corners_round_the_smooth_arc_vertices_stay` atribuía-o às alças;
   eram o `acos`). A menor quina verdadeira do catálogo tem `0,18` rad (a faixa, aspecto 4).
   Cura: `corner_live::straight_or_folded` — o seno pelo produto vetorial, a mesma função para o
   cozimento, a alça (`corner_at`) e o gerador (`corners.rs`). Gate `a_straight_vertex_never_moves`.
2. **O motor recusava todo contorno de menos de 3 vértices.** O coração tem 2 (dois troços) e duas
   quinas reais (o vale, `~43°` entre as tangentes; a ponta, `~107°`): ninguém as via — nem o censo,
   que lia pelo mesmo predicado. Cura: `n < 2`; um vaivém recto continua recusado («dobrado»). Gate
   com a régua INDEPENDENTE do motor: `the_engine_sees_every_corner_the_geometry_has`. Efeito no
   Motion: o `corner` passa a mexer o coração (o painel mostra-o; o gate de bytes do coração varre só
   `corner = 0`, a nota da engrenagem).

## §2 — Quem arredonda o quê (o censo `every_live_corner_has_a_rounder`)

Toda forma está numa classe só (`every_shape_is_in_exactly_one_class`); o censo varre o default,
cada campo na faixa dele e 17 proporções da caixa, e conta só a quina que um raio mexe de forma
visível (recuo máximo `> 1e-6` da caixa).

- **«Corner round» (37)**: Rect · Oval e Pie (a fatia) · Segment · as quatro setas · Decision · Data ·
  os dois trapézios · Prepare · Document · Delay · Display · Subroutine · Off-page · Note · os três
  balões (o bico) · Burst · Heart · Bolt · Drop · Shield · Tag · Gear · Cross · Check · Banner · Cube ·
  Cone · Pyramid · Bone · Rope. As duvidosas da lista do prompt (a agulha do Burst, a ponta da Gota,
  os vales do pensamento, o bico do balão oval, o ápice do cone) entram: o tecto do motor (½ corda)
  é honesto e o censo de laços e ganchos (`no_cooked_shape_loops_or_hooks`, 0 → 500 px e aos pares)
  passa limpo.
- **Arredondador próprio (4)**: Round (o *Radius*), Poly (o *Radius*), Star (*Tip/Notch round*,
  8/8 campos), Cloud (o *Smooth*, o filete do vale) — o gate prova que cada um apaga as quinas.
- **De fora (8)**, com a medida: Line · Arc · Spiral (aberta; com *Expansion* ≥ 4,17 o centro tem
  troços de `~1e-9`, recuo máximo `2e-9`: ruído) · Pill · Cylinder (elipses tangentes, seno ≤ 2e-15)
  · Junction · Brace (o bico é uma cúspide de ângulo zero) · Moon (os chifres: seno 0, cosseno 1 em
  toda a *Phase* — entre duas curvas tangentes não cabe arco).

**O rectângulo** (decidido medindo): o produto do dono (Affinity) dá raio de canto ao rectângulo, e o
«Corner round» do nosso É o *Radius* do rectângulo arredondado — iguais a `1e-9` em três caixas e
cinco raios, saturação incluída (`the_rectangle_corner_round_is_the_round_rect_radius`; a diferença
de `3e-11` é o `KAPPA` que o gerador escreve com 8 casas). Ele recebe o campo.

## §3 — A alça

- Um **⊙ dentro do canto** (`corner_round(field, corner, next)`: `Spot::Rounding`, `in_corner`,
  estaciona) onde há uma quina convexa entre duas arestas RECTAS em toda variante (medido por forma):
  24 formas. Um **⇕ na fila** onde as quinas são entre curvas ou o número de vértices muda: Double,
  Bent, os três balões, Burst, Heart, Drop, Gear, Cone (10). A elipse, a pizza e o segmento: o ⊙ da
  pizza (F12).
- As outras alças dessas formas leem a forma com o raio a ZERO (`on_sharp(…, &[F])`): a quina que o raio
  parte em duas mudava os índices dos vértices. Gate `the_corner_round_moves_no_other_handle`.

## §4 — A cena 21

`PH2D_OBJECT_MODE_SMOKE=21` (`vector_mode_quinas`): as 36 (a elipse inteira não tem quina; a pizza
mostra a família) em quatro filas; o botão *Forma*; o raio de
cada uma subido pelo press e arrasto da shell — o ⊙ pelo trilho que o cursor por cima dele mostra,
o ⇕ para cima. `PH2D_CENA21_FICA=<i>` deixa seleccionada a forma `i` (omissão: o cubo).

## ⛔ Recusas MEDIDAS

| proposta | por que não |
|---|---|
| carimbar o raio só nos vértices `Corner` (a lei da pizza da F12) | a engrenagem e o documento ficavam com quinas vivas (32 + 2, marcadas lisas) |
| um limiar de ângulo no `acos` (p.ex. `1e-7`) | mediria o ruído da conta e não a geometria; o seno pelo produto vetorial é exacto (`0`) |
| arredondar também as linhas abertas da face | o «Y» do cubo curvava no centro e a 3.ª aresta descolava dele |
| a cena puxar o ⊙ para o centro da forma | no raio (Bolt) e na cruz o centro quase não anda ao longo da aresta: o raio não subia |
