# F33 — Filtros × Efeitos: o plano (com o mecanismo medido)

> Ordem do dono (26/10, smoke F32): uma forma com **Bevel** (+ Drop Shadow) ganha vincos ao ligar o
> efeito **Flag** — nas reentrâncias e na borda. Pedido: auditar a INTERACÇÃO filtro × efeito, não
> só o par. Fotos: [sem](handoffs/img_F32/bevel_sem_flag.png) · [com](handoffs/img_F32/bevel_com_flag.png).

## 1 — Como um filtro vê a forma (lido no código)

- O FX vetorial (`ph2d-app-vec/src/fx_live.rs`) rasteriza a forma **cozida** (`VecPath::cooked`, a
  pilha de efeitos já aplicada) e a caixa vem da mesma geometria (`path_screen_bounds` →
  `build_bezpath` → `cooked()`). ⇒ **a caixa do filtro já segue o efeito** (sombra/brilho não são
  cortados por um Bulge/Repeater). Medido no código, nada a curar aqui.
- Os filtros de DISTÂNCIA (Bevel, Inner/Outer Glow, Inner/Drop Shadow, Outline, Feather) leem o pé
  **exacto** da silhueta: `ph2d-vec-render/src/silhouette.rs` achata a forma cozida em segmentos
  (`FLATTEN_TOL = 0,02 texel`). O Bevel lê a **DIRECÇÃO** desse campo — é o filtro mais sensível a
  qualquer quebra de tangente ou laço na curva. Os outros leem a DISTÂNCIA (menos sensíveis, mas a
  mesma geometria).
- ⇒ **A interacção filtro × efeito é inteira pela GEOMETRIA COZIDA.** Um filtro não sabe que há
  efeito; ele vê a curva que o efeito devolveu. A régua tem de ser da geometria, por efeito, e a
  imagem confirma por filtro.

## 2 — O mecanismo (MEDIDO, sonda sobre uma nuvem com 6 quinas e sobre um círculo)

Sonda: para cada efeito com parâmetros típicos (os do dono no Flag: Bend 17,45 · H 12,21 · V −11,66),
conta (a) **segmentos que RECUAM** (a derivada inverte contra a corda = laço/cúspide dentro do
segmento), (b) vértices `Smooth` com alças não colineares, (c) **quinas > 5°**.

| efeito | nuvem: recuos | nuvem: quinas (fonte 6) | círculo: recuos |
|---|---|---|---|
| Arc / Arc Upper / Arc Lower / Flag / Squeeze / Fisheye | 4 | **0** | 0 |
| Bulge / Wave / Rise | 3 | **0** | 0 |
| Twist | 4 | **0** | 0 |
| Zig Zag (Smooth) | 2 | 0 | 0 |
| Sketch | 3 | 0 | 0 |
| Pucker & Bloat | 8 | 9 | 4 |
| Trim · Repeater · Knot · Hatch | 0 | preservadas | 0 |

⭐ **O reamostrador partilhado (`resample_displace`, Warp ×9 + Twist) com DESLOCAMENTO NULO já dá 3
recuos e 0 quinas** — o defeito não é do Flag, é do esqueleto:

1. **As quinas morrem.** A âncora de quina entra na amostragem, mas sai como `Smooth` com alças de
   Catmull-Rom `(next − prev)/6`: a reentrância vira uma volta apertada. O Bevel desenha a quina
   como uma curva de raio quase nulo → o vinco nas reentrâncias da foto.
2. **Amostras quase coincidentes fazem laços.** A grade uniforme ∪ as âncoras deixa pares a
   `~5,8 %` do passo (razão mínima de vizinhos medida `5,82e-2`); a alça de Catmull-Rom tem um
   sexto do vão TOTAL `prev→next`, maior que o segmento curto ⇒ o segmento recua (laço). O Bevel lê
   a direcção do laço → os vincos na borda (setas à direita da foto).
3. Zig Zag Smooth e Sketch: a alça segue a tangente de UM lado na quina (o `frame_at` dá a de
   saída) e o Sketch nem amostra as âncoras ⇒ o segmento que atravessa a quina recua.
4. Pucker & Bloat: as pétalas com bico SÃO o efeito (a lei do Illustrator: âncoras para um lado,
   alças para o outro). Os «recuos» medidos são a dobra da pétala. **Não é defeito da família**;
   mede-se à parte na imagem e não se cura sem decisão do dono.

## 3 — A cura (uma família, um bloco)

Uma porta partilhada de REAMOSTRAGEM (`fx_resample.rs`, na `ph2d-vec-scene`), usada por Warp, Twist,
Zig Zag e Sketch:

- **A grade conhece as âncoras**: cada amostra sabe se é a âncora `i`; ao fundir amostras
  coincidentes, a âncora ganha (nunca a amostra da grade).
- **Na âncora, as DUAS tangentes exactas** (`tangent_at` do segmento que chega em `t=1` e do que sai
  em `t=0`); a saída é `Corner` se elas diferem, `Smooth` se não.
- **Alças de Hermite, cada uma com o SEU vão**: braço = vão até ao vizinho / 3 (o que o Zig Zag já
  fazia, agora para todos). Nenhuma alça passa o seu segmento ⇒ nenhum laço de espaçamento.
- **Nos deformadores de campo (Warp, Twist)** a direcção da alça é a tangente **transportada pelo
  campo** (`J·t`, diferença central do deslocamento — incluindo o peso do Falloff), não um palpite
  de Catmull-Rom sobre pontos: liso onde a fonte era lisa, quina onde a fonte tinha quina.

## 4 — Réguas (gates), cada uma vista VERMELHA por mutação antes

- **G1 `no_effect_folds_a_segment_back`** — para todo efeito do catálogo (menos Pucker & Bloat, §2.4)
  sobre a nuvem e o círculo: zero segmentos que recuam.
- **G2 `a_continuous_deformer_keeps_every_corner`** — Warp ×9, Twist (e os que já passam: Trim,
  Repeater, Knot): as quinas da fonte sobrevivem na saída. A quina conta-se por QUALQUER quebra de
  tangente (`1e-6` rad), não por `> 5°`: um campo liso leva quina a quina mas muda o ângulo (medido:
  o Twist a 90° leva a de 9,5° da nuvem a 3,5°).
- **G3 `a_smooth_vertex_out_of_an_effect_is_g1`** — todo vértice `Smooth` emitido tem alças
  colineares.
- Mutações: (a) voltar ao `(next − prev)/6` ⇒ G1 vermelho; (b) emitir `Smooth` nas quinas /
  tangente de um lado só ⇒ G2 vermelho; (c) a fusão sem preferir a âncora ⇒ G2 vermelho.

## 5 — Régua na IMAGEM (por par filtro × efeito)

Cena de smoke nova `PH2D_OBJECT_MODE_SMOKE=40`: a nuvem repetida, uma por efeito, com Bevel +
Drop Shadow (o par do dono). Com `PH2D_FX_DUMP` a saída de cada filtro é gravada; a régua é o
**salto máximo de luminância entre píxeis vizinhos na banda do relevo** (o vinco é um salto), com
o efeito ligado contra a mesma forma sem ele. Antes × depois da cura; os outros filtros de
distância por troca do filtro da cena.

## 6 — Fora desta onda

Os abertos 2–4 do handoff F32 §6 (pose dos estados por índice; chave sobre `from`/`to`; F31/F30).
