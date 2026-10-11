# 19 — O traço das formas: o bico que saltava, os laços e os ganchos (F12 A, 11/10)

> Pedido do dono (11/10, fotos em `/var/tmp/ph2d_F12_dono/`, fora do repo): *«a depender da altura
> da seta o stroke buga»* · *«Coração a depender do ajuste o stroke buga. O mesmo acontece com Cloud
> e vários outros»* · *«Poly ficou todo bugado (fotos) com stroke bizarro»*. Prompt:
> [`18_prompt_F12_…`](18_prompt_F12_sem_modais_e_o_traco.md).

## §1 — As espécies (a leitura das fotos, MEDIDA)

| foto | o que se vê | mecanismo medido |
|---|---|---|
| 1 polígono, *Curve* 0,010 | «X» a sair das quinas | **gancho**: com raio 0 a lei do *Curve* punha as alças na PERPENDICULAR da aresta (`out_h = bend`), então qualquer envergadura ≠ 0 vira a ponta da aresta 90° num troço de ~2 px; a junta lê essa tangente e o traço grosso desenha o «X». Não é laço nem miter. |
| 2/3 pentágono, *Smooth* −0,02/−0,01 | abas escuras dentro do traço | **laço**: o *Smooth* negativo (o *Rounded* do Inkscape) aponta as alças para trás — um laço por vértice; o corpo do traço sobre o laço tem winding oposto e o NonZero abre um furo. |
| 4 seta, duas alturas | a farpa inteira numa, cortada noutra | **salto bico → chanfro**: a quina da farpa cruza os `~29°` do limite `4`. |
| 5/6 coração | o bico do vale comprido numa, cortado noutra | o mesmo salto. |

A caixa (espécie «c» do prompt) NÃO estava errada: nas fotos ela abraça as pontas do «X». Ela passa
agora pela mesma porta do contorno com os cortes (§2), então abraça-os por construção.

## §2 — O oráculo e a lei do bico (`miter-clip`)

Os apps de desenho SALTAM todos (Illustrator no limite `10`, Inkscape/SVG/Figma/Affinity em `~29°`);
o SVG 2 criou o `miter-clip` para o salto deixar de existir: além do limite o bico é CORTADO por uma
recta perpendicular à bissectriz, a `limite × ½w` do vértice. Triagem: `resvg 0.48.1` (Apache-2.0/MIT,
`pacman -Qi resvg`) desenha-o; o Inkscape 1.4.4 e o `rsvg-convert` caem no `miter`. Corrido por
[`oraculo_traco/corre_resvg.py`](oraculo_traco/corre_resvg.py) → [`fixtures/miter_clip_resvg.json`](oraculo_traco/fixtures/miter_clip_resvg.json)
(quina de pernas 200 px, vértice a y = 60, traço 24, limite 4; topo da tinta):

| ângulo interno | 60° | 40° | 30° | 29° | **28°** | 25° | 10° | 0° (ida-e-volta) |
|---|---|---|---|---|---|---|---|---|
| `miter` | 37 | 26 | 15 | 14 | **57** | 58 | 59 | 60 |
| `miter-clip` | 37 | 26 | 15 | 14 | **12** | 12 | 12 | 12 |

⇒ **43 px de salto por 1°** com o bico de antes; contínuo com o corte, e nunca mais alto que o maior
bico que já se desenhava (`2w` com o limite `4`). O limite NÃO mudou (`MITER_LIMIT = 4`); mudou o que
acontece além dele.

**Uma lei, cinco leitores.** A lei: `ph2d_vec_scene::miter_clip` (`clip_patch`, `clip_patches`,
`oriented`, `subpaths`). Os leitores juntam ao contorno que já expandem um quadrilátero por corte, no
sentido do winding do corpo (sem costura no NonZero): o desenho (`ph2d_vec_render::stroke_uniform` —
o Vello traça como sempre e só a geometria com quina além do limite, sinal `PathTess::agudo` pago uma
vez, vai pelo contorno preenchido), a caixa visual, a placa do Motion conforme (`ph2d-shape-gpu`,
`contornos_dos_tracos`), o *Outline Stroke* (`ph2d-vec-boolean`) e o SVG (`stroke-linejoin="miter-clip"`).
`PH2D_MITER_CLIP=0` volta ao chanfro (o antes × depois do dono).

## §3 — A lei do polígono e da estrela (o gancho e o laço)

- ***Curve*** com uma lei só: alças a ⅓ da corda + `4h/3` pela normal (a mesma do
  `bend_between_corners`, que já valia com raio > 0). A tangente na quina roda `atan(4h/corda)` a
  partir de ZERO — sem gancho. A rotação nunca passa da metade da quina que ela fecha (côncava fecha o
  ângulo interno; convexa, o externo): no limite uma cúspide, nunca um laço.
- ***Smooth*** de `0` a `1`: o meio negativo do Inkscape é um laço por definição, e o produto do dono
  (Affinity) tem o *Smooth points* só ligado/desligado. Com *Curve* ≠ 0 o troço a ⅓ cede às tangentes
  do liso em `1 − 3·smooth`; com *Curve* = 0 a curva do Inkscape fica exacta (gate de paridade: 10
  casos de 0 a 1; o `−0,25` do fixture é o laço recusado).

## §4 — O censo (todas as formas × cada campo, a proporção da caixa, e os pares)

Sonda `ph2d-tool-vector/src/stroke_census_tests.rs` (traço 7,5 % da caixa). ANTES → DEPOIS da §3:
laços em 1D: Poly *Curve* 35 + *Smooth* 38, Star *Curve* 63 + *Smooth* 40 → **0**. Saltos bico/chanfro
em ~30 linhas (estrela, setas, coração, nuvem, raio, explosão, balão-pensamento, iso…) — curados pela
§2 sem tocar na geometria.

## §5 — A cauda (F12 A2): cada espécie, o mecanismo e a cura

Sondas `print_the_stroke_census`, `print_the_pair_census` e `print_the_tail_detail` (junta a junta:
posição, volta, que ponta engancha, os pontos de controle; e que troços se cruzam). ANTES → DEPOIS:
laços nos pares **145 → 0** (e 0 em 1D, que já era); ganchos **509** amostras nos pares e **214** em
1D pela régua antiga → **0** pela régua nova (fora as excepções abaixo; parte dos 723 eram da régua,
não da forma — a última linha da tabela). O gate `no_cooked_shape_loops_or_hooks` vale agora para TODAS as formas,
campos, proporções e pares.

| espécie | mecanismo medido | cura |
|---|---|---|
| estrela lisa de muitas pontas / inner 0,05 | as cúbicas de arestas VIZINHAS cruzam-se perto do vértice (as alças do liso não se cruzam — a cura «alças sem cruzar» do handoff não morde) | `ph2d_vec_scene::untangle`: se o contorno cozido se cruza, torção, *Curve* e *Smooth* recuam JUNTOS até ao último desenho sem cruzamento (bissecção, 32 amostras por troço, janela 3 troços, quase-cúspide > 120° conta como cruzamento, recuo 3 % depois do limiar) |
| torção extrema, recorte redondo × *Curve* côncavo | arestas rectas que se cruzam; o arco do canto com a alça rodada para fora | a mesma guarda |
| *Smooth* × *Curve* | cada aresta enverga pela SUA normal: o vértice «liso» tinha uma quina de 35° com gancho | o vértice liso é G1: alças na direcção média (sem *Curve* já eram colineares — paridade Inkscape intacta) |
| engrenagem côncava de muitos dentes | os dois flancos do dente cruzam-se | `untangle` sobre a envergadura |
| rabo do balão curto e largo | o bico em agulha obrigava o flanco a dobrar ~77° no último toco | o bico abre em V pela medida `cos(eixo, corda) × l / corda` (agulha ≥ 0,7, V ≤ 0,1; o balão por omissão mede 0,82 e não muda) |
| rabo de lado (raio enorme × faixa mínima) | a alça da RAIZ saía da faixa (para lá da altura do bico) e o flanco cruzava o outro | a alça da raiz nunca passa de meia altura do bico, medida em `+u` (fora do corpo) |
| bico pedido dentro do corpo | encostado na borda: rabo de altura zero, cruzado | o bico pára a meia faixa (`TIP_FLOOR`) |
| seta dupla com cabeça 0,5 | haste de comprimento zero: fenda de ida-e-volta | sem haste o contorno é o losango |
| cantinho côncavo, furo pequeno, bolhas | **não era gancho**: curva menor que a caneta | a régua mede a viragem em `min(½ caneta, ⅛ do troço)`; a caixa achatada mede a caneta no lado CURTO |

Fica de fora, MEDIDO: a lua de fase < 0,12 (a borda de dentro é uma elipse de semi-eixo menor que
meia caneta) e a nuvem/pensamento a menos de 1:2,9 (bolhas autoradas como círculos na caixa
quadrada) — a viragem é a da elipse, não de uma alça; e a cúspide da espiral com *Expansion* ≥ 4,8,
no miolo, num raio de `~0,002` da caixa (o Inkscape faz o mesmo). As fatias de 1° (pizza, oval,
segmento no mínimo do *Sweep*) são agulhas de verdade.

**As pontas que voltam 180° por desenho** (chifres da lua, bico da chave, ponta do rabo, as
cúspides do *Curve* ±1): com o bico cortado ganham um toco de `2 ×` a largura (o SVG 2 faz o mesmo:
o fixture do resvg a 0°). Antes eram chanfro rente. É a mudança MAIS visível do bico cortado — vai
nas fotos da cena 18 para o dono decidir.

**A placa da GPU do Motion** (`ph2d-shape-gpu/src/contorno_cpu.rs` + `contorno.wgsl` +
`contorno_subgrupo.wgsl`, `limite_esquadria`) ainda salta para o chanfro nas cópias não conformes.
Não entra nesta linha: é o motor do Motion e só vale portar depois de o dono aprovar o bico cortado.

## ⛔ Recusas MEDIDAS

| proposta | porquê não |
|---|---|
| subir o limite (o `10` do Illustrator) | já esteve e voltou (`MITER_LIMIT`, F39 do esqueleto: bicos de 5 larguras perfuravam a peça); e só muda o ângulo do salto |
| junta redonda além do limite | também salta (de `2w` para `½w`) |
| corrigir forma a forma | o salto é a lei do traço, não da forma: ~30 linhas do censo |
| o *Smooth* negativo | laço por definição (fotos 2/3) |
| limitar as alças do liso para «os dois segmentos-alça não se cruzarem» | medido na estrela de 9 pontas: as alças nem se encontram; o laço é entre arestas vizinhas |
| limitar a alça do liso ao raio do vizinho (o sector) | partia a paridade com o Inkscape na estrela de 5 pontas a liso > 0,38 |
| limitar o recuo da alça do BICO pela normal da elipse | criou ganchos no balão oval de base aberta (a normal do lado aponta de costas para o bico) |
