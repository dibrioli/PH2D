# 25 — Os três relatos do smoke da F14 (F14b, 13/10)

> Relatos do dono ao testar a F14 ([doc 24](24_arredondar_as_quinas.md)). Mecanismo, medida e cura de
> cada um; commit `8edf9c75a0`. Tabela do traço (Miter, `2·asin(1/4)`): [doc 19](19_o_traco_das_formas.md).

## §1 — O ⇕ sobre o ponto

*«A seta e o ponto não devem ficar na mesma posição seja vertical ou horizontal»* (as setas ficam como estão).

- **Mecanismo**: `shape_handles::handle_grip`, ramo `Spot::Rail`, assentava a fila de ⇕ olhando só a caixa;
  o ⊙ do *Radius* do Speech desce pela borda direita.
- **Medido** (sonda, `park_at` 18 px, cada campo em 9 valores, caixas 100×95,4 · 200×100 · 200×400 px):
  **300 casos** de ⇕ a menos de 24 px do grip de outra alça, em 9 formas. SpeechRect 47/55 (pior 5,8 px,
  no padrão), RoundRect 34/46 (18,9 px), IsoCube 37/37 a **0,0 px** (o vértice 0 da profundidade É o
  sítio do 1.º ⇕), mais Drop, IsoCone, IsoPyramid, Star, Spiral, ArrowDouble, Thought, Burst.
- **1.ª cura, RECUSADA pelo dono** (`801a9d58d9` a substitui): a fila andava um passo para fora (até 3)
  só quando um grip chegava perto — 300 → 0, mas *«não fixou afastado um do outro. a posição está
  variando»* (duas fotos: o ⇕ saltava da borda para fora ao arrastar o *Radius*).
- **Cura** (`spots::rail_grip`): a fila tem um lugar FIXO na caixa VISÍVEL (`visible_bbox`: o contorno
  E os sub-contornos — a do contorno principal deixava as bolhas do pensamento 68 px fora):
  `RAIL_OUT = 1,5` passo (40 px) à direita da borda, a `h/4` do topo (caixa baixa: meia altura, F13 D).
  Medido antes de escolher: nenhum grip do catálogo passa da borda direita da caixa visível, em todo
  valor de cada campo, a 100×95,4 · 200×100 · 200×400 · 60×200 px. Depois: o par ⇕↔grip mais perto a
  **32,8 px**; a fila nunca anda quando outra alça anda. Custo: zero (não lê as outras alças).
- **Com traço** (F13 §7.2): a caixa das alças da caixa é a VISUAL (com o traço); a fila soma a margem do
  que a forma pinta (`shape_handles_live::Target::paint_margin`, a conta do gizmo) — caneta 0/10/60 px →
  sempre 40,5 px da caixa visual. Gate `the_arrow_row_stands_off_the_painted_box_whatever_the_stroke`.
- **Gates**: `the_arrow_row_has_a_fixed_place_beside_the_visible_box` (todo valor de cada campo;
  vermelho com a caixa só do contorno principal) e `no_arrow_lands_on_another_handle` (vermelho com
  `RAIL_OUT = 0`: 332). Foto: [`f14b_01`](fotos/f14b_01_cena21_o_speech_com_o_seta_um_passo_por_fora.png).

## §2 — «Radius 0» e as orelhas

*«Radius 0» desenha orelhas no traço dos quatro cantos do corpo do Speech.*

- **Medido**: o ⊙ do *Radius* (`Law::Nearest`) levado ao canto, ou além, guardava `3,9e-8`–`2e-5` px
  (nunca 0): `best_on_track` desempata a favor do valor do PRESS (`(x−cur)` menor) e o mínimo é plano
  ali; a fila mostra «0». Com um raio assim (ex. 1e-5 px) o «Corner round» (F14) deixa de arredondar
  aqueles cantos (os dois vértices do micro-arco são lisos; `corner_live` não vê quina).
- **Cura** (`spots::snap_to_ends`): no arrasto de um campo contínuo (Nearest/Radial) o valor vai à ponta
  da faixa quando a alça andaria menos de meio pixel no ECRÃ (`0,5·px_to_world`) entre ele e a ponta,
  medido pela TAXA do ponto junto da ponta (`(hi−lo)·1e-3`) — um trilho saturado (ponto parado) nunca
  encaixa. Depois: mão no canto ou além → `0` exacto; 0,1 px adentro → 0; 1 px → 0,97 px.
- **Gate** `a_rounding_dragged_to_the_corner_is_exactly_zero` (toda `Spot::Rounding` do catálogo, ≥ 20;
  controlo: 2 px adentro guarda um raio, nunca o máximo). `the_star_valley_never_twists` e
  `the_tip_handle_slides_down…` (F10) passam a `px_to_world = 0,01` (com 1 px = 1 m a estrela tinha 2 px).
- ⚠️ **As ORELHAS em si NÃO foram reproduzidas.** Sonda GPU (o `VelloPass` do produto, `encode` →
  `stroke_uniform`, e o traço NATIVO do Vello) do Speech (raio do corpo e Corner round), do Rectangle e do
  RoundRect; raios `0 · 1e-6 · 1e-3 · 0,01 · 0,1 · 0,5 · 1 · 3 · 10 · 14 · 16 · 30` px, caneta 3 e 30 px,
  caixa 400×380 px; régua por janela em volta de cada vértice vivo (tinta fora do preenchimento a mais de
  `w/2·√2 + 1,5` px; furo no traço; furo no preenchimento): **0 orelhas, 0 furos** nos cantos do corpo (o
  bico, com Miter, pinta a ponta — esperado). Geometria: raio do corpo `1e-9`..`1` px (passo √10) ×
  Corner round `0 · 5 · 15`, Speech/RoundRect/Rectangle: **0 auto-cruzamentos**. A placa do Motion
  (`PlacaDeFormas::decide`) recusa uma cópia só — não medida. A foto do dono não está no repo nem na
  máquina ⇒ pedir a foto de novo / confirmação no smoke com o binário novo.

## §3 — O bico do balão

*«Nenhuma opção do painel lateral produz uma ponta pontuda, mas apenas quadrada e redonda.»*

- **Medido**: `bubbles_ring::tail_flanks` entregava o bico com as duas alças NO EIXO (agulha, ângulo
  interno 0°) em todo valor; o Miter (limite 4) corta abaixo de `2·asin(1/4) = 28,96°` (doc 19 §2,
  SVG 2 `miter-clip`) ⇒ um toco rectangular de 2 larguras.
- **Cura**: `TIP_OPEN = 0,8`, o piso da mistura eixo→corda (`beta.clamp(TIP_OPEN, 1)`) na chegada ao bico;
  o flanco segue côncavo junto ao bico (a tangente fica DENTRO do ângulo da corda).
- **Medido depois** (caixa 100×95,4): triângulo recto do rabo 52,2° (rect) / 43,3° (oval) → bico
  **41,9° / 34,8°** (era 0°). Pior da varredura: 8,2° (rect, base 0,05) / 6,0° (oval, meia-abertura 2°);
  um rabo fino tem o próprio triângulo abaixo de 29° e continua cortado (como em qualquer app).
  Candidatos 0,5–0,7: oval a 21,7°–30,4°; **0,8** (margem de 6°).
- **Gates** (`shape_handles_f14b_tests.rs`): `the_speech_tips_have_an_angle` (padrão > 31,96°; nenhum
  < 5° na varredura). Ajustados em `ph2d-vec-scene`: `bubbles_tests` (o gate do bico) e `orient_tests::tail_points_down`.
- **Oráculo**: o Affinity foi DESINSTALADO a pedido do dono (11/10) — não medido.
  Foto antes/depois: [`f14b_02`](fotos/f14b_02_cena18_o_bico_antes_esq_depois_dir.png).

## §4 — O censo das pontas

Quinas < 29° (padrão + cada campo varrido): SpeechRect 52 → 5, SpeechOval 45 → 10.

- **Cúspides de ângulo ZERO = lista escrita** (gate `the_zero_angle_cusps_are_the_written_list` =
  `[Polygon, Star, Brace, Moon]`): estrela e polígono só no *Curve* ±1 (doc 19 §5); os chifres da lua
  (`symbols::moon`, terminador tangente ao limbo; o ECMA tem 20,6°); o bico da chave (o `{` do OOXML).
- **Pontas FINAS no padrão, ângulo do PARÂMETRO** (geometria certa; o Miter corta pela lei do traço):
  Burst 16–25°, Bolt 21,9°, Cloud 23,0° (vale), Thought 26,4°. Querê-las pontudas com Miter pede subir o
  limite do Miter — pergunta ao dono (ABERTO).

## ⛔ Recusas MEDIDAS

| proposta | por que não |
|---|---|
| ler os TRILHOS (`track`) para assentar a fila de ⇕ | custa 0,27–21 ms/quadro (Star 21 ms), ~4–38× os grips |
| trilho saturado encaixar na ponta | o ponto parado não anda: a taxa junto da ponta decide, não a distância |
| `TIP_OPEN` 0,5 / 0,6 / 0,7 | o oval fica a 21,7° / 26,1° / 30,4° do corte: margem de 6° só em 0,8 |
| subir o limite do Miter para as pontas finas | recusado e medido antes (doc 19: `MITER_LIMIT` da F39 do esqueleto) |
| «curar» as orelhas sem as reproduzir | 0 orelhas e 0 furos em GPU e 0 auto-cruzamentos na geometria; sem a foto, cura às cegas |
