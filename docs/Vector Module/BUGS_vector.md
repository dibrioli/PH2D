# BUGS — Vector

Registro dos bugs do módulo Vector: o **sintoma** que o Enio viu, a **causa real** (que
quase nunca é a primeira suspeita), a **correção** e o **gate executável** que impede a
volta. Espelha o [`docs/Painter/BUGS_painter.md`](../Painter/BUGS_painter.md).

Regra deste arquivo: um bug só é considerado fechado quando existe um teste que **falha**
se a correção for revertida. "Não reproduzi mais" não fecha nada.

---

## Bug #1 — Panic ao mexer nos parâmetros do balão de fala (2026-07-12) — **FECHADO**

### Sintoma

Editor morre com `SIGSEGV` ajustando os sliders de `Bubbles:Speech`:

```
PH2D PANIC frame=15973
message="min > max, or either was NaN. min = 0.5, max = 0.49999999999999994"
core/src/num/f64.rs:1502
```

### Causa

Não é o `clamp`: é uma **janela que colapsa**.

`speech_rect` assenta a base do rabo na aresta direita, entre as duas quinas arredondadas.
Ela nunca pode cavalgar uma quina, então a meia-largura da base satura na aresta reta que
sobrou:

```rust
let hb = 0.5 * base_w.clamp(0.05, 0.6).min(1.0 - 2.0 * q);   // q = raio da quina
let cy = t.1.clamp(q + hb, 1.0 - q - hb);                     // ← PANIC
```

Quando a base pedida é **maior que a aresta livre**, o `.min()` morde e `hb = (1 − 2q)/2`.
Substituindo:

- piso = `q + (1 − 2q)/2` = **0,5**
- teto = `1 − q − (1 − 2q)/2` = **0,5**

Os dois limites são **o mesmo número em matemática real** — a janela tem largura zero, o que
é geometricamente correto (a base ocupa a aresta inteira, e só há uma posição possível).
Mas em ponto flutuante `q + hb` e `1 − q − hb` percorrem caminhos de arredondamento
diferentes, e para certos `q` o teto cai **1 ulp abaixo** do piso. `f64::clamp` entra em
pânico com `lo > hi` — por projeto, e com razão: não existe intervalo onde clampar.

O `q` culpado não é especial. Um teste que varre `q ∈ [0, 0.45]` encontra cruzamentos
espalhados; é aritmética, não lógica.

### Por que passou por todos os gates

Os testes do módulo varriam **um parâmetro por vez**. Este pânico exige `radius` grande
**e** `base_w` grande ao mesmo tempo — nenhuma varredura unidimensional o alcança. O gate
novo confirma isso na prática: `dragging_any_single_slider_end_to_end_never_breaks_a_shape`
fica **verde** com o bug presente; só a varredura combinada o pega.

### Correção

Uma primitiva, em [`space.rs`](../../crates/ph2d-vec-scene/src/space.rs):

```rust
pub(crate) fn clamp_window(x: f64, lo: f64, hi: f64) -> f64 {
    if lo.is_nan() || hi.is_nan() { return 0.0; }   // o OUTRO jeito de o clamp explodir
    if lo > hi { return (lo + hi) * 0.5; }          // colapsou: não há intervalo, há um PONTO
    x.clamp(lo, hi)
}
```

**A regra:** todo `clamp` cujos DOIS limites são calculados usa `clamp_window`. Um `clamp`
com limites literais (`x.clamp(0.05, 0.6)`) é seguro e continua como está. Os cinco sítios
de `bubbles.rs` foram convertidos.

O `is_nan()` explícito não é decoração: comparação com `NaN` é sempre falsa, então
`lo > hi` sozinho deixaria o `NaN` passar direto para o `clamp` cru — que também explode
com ele.

### Gates

1. **`space::tests::a_collapsed_window_yields_a_point_instead_of_a_panic`** — não postula o
   `q` culpado, **procura**: varre 200 mil valores e exige achar um que cruze. Se nenhum
   cruzasse, o pânico do smoke não teria acontecido — o teste também valida a própria
   teoria do bug. Depois exige que o cruzamento seja de **1 ulp** (é ponto flutuante, não
   erro de lógica) e que a função devolva o ponto.

2. **`robust_tests::cook_survives_every_slider_position_and_every_gesture_box`** — o gate de
   classe. Para cada uma das 48 formas, 3.000 vetores de parâmetros (sorteados de uma tabela
   determinística que inclui os extremos exatos das faixas, valores fora delas, zero,
   negativos e os pontos de colapso) × 9 caixas de gesto — incluindo as **degeneradas**:
   caixa nula (o primeiro frame de todo arrasto, entre o `pointer_down` e o primeiro
   `pointer_move`), largura zero, altura zero, invertida, gigante e minúscula. Exige que
   nada exploda e que **toda coordenada saia finita**, âncoras e handles, contorno principal
   e sub-contornos.

   Este gate reproduziu o pânico do Enio com os mesmos números antes da correção.

### Lição

> **Dois limites que saem da mesma conta se cruzam em ponto flutuante.** Sempre que um piso
> e um teto são derivados do mesmo parâmetro e se encontram na fronteira, eles são iguais em
> ℝ e ordenados de forma imprevisível em `f64`. `clamp` é a operação errada ali: a janela
> colapsada não é um erro a evitar, é um **caso legítimo** (só há uma posição possível) que
> precisa de resposta, não de pânico.

E: **um `NaN` é pior que um pânico.** O pânico morre no ato, com o stack trace. Um `NaN`
contamina em silêncio — vira uma bbox `NaN`, o gizmo some, o hit-test para de responder, e o
usuário reporta "a forma sumiu" três telas depois da causa. Por isso o gate exige
finitude, não apenas ausência de pânico.

---

## Bug #2 — Formas assimétricas nasciam espelhadas na vertical (2026-07-12) — **FECHADO**

### Sintoma

"Cone e pirâmide estão de cabeça para baixo. Símbolos de cabeça para baixo."

### Causa

**Uma causa, vinte bugs.** O mundo do PH2D é **Y-para-cima** (a câmera leva mundo→tela com
`scale(k, −k)`), mas *toda* referência de geometria — SVG, a geometria pré-definida do
OOXML/ECMA-376, os stencils do draw.io — é **Y-para-baixo**. Os módulos `symbols`, `flow`,
`arrows` e as isométricas foram escritos tratando `cy − hh` como "o topo".

As formas **simétricas em Y** (retângulo, elipse, losango) escondiam o defeito. As
assimétricas saíam todas invertidas: 20 de 48.

Não foi só o que o Enio viu. **Triângulo, polígono e estrela apontavam para baixo** — e havia
um teste **verde** consagrando isso (`regular_polygon_first_vertex_is_at_top` afirmava que o
topo era `cy − ry`). O oráculo tinha sido derivado do código em vez da aparência: ele não
pegava o bug, **carimbava**.

### Correção

Não acertar trinta sinais — **não ter sinal para errar**. O
[espaço de autoria](../../crates/ph2d-vec-scene/src/space.rs): a forma é escrita em
`(u, v) ∈ [0,1]²` com `v = 0` no TOPO (a convenção da referência), e `Unit::p` é o **único**
lugar do catálogo que sabe para que lado o mundo aponta. Uma fórmula copiada de uma
referência entra sem tradução mental.

### Gate

**`orient_tests`** — um `match` exaustivo sobre `ShapeKind` **sem braço `_`**: toda forma
declara `Symmetric` ou `Asymmetric(predicado)`. A forma #49 **não compila** até o autor dizer
para que lado ela fica de pé.

E a asserção que importa: **o predicado tem de REPROVAR a forma espelhada**. Um predicado que
passa também no espelho não distingue cima de baixo — é **vazio**, e ficaria verde com a forma
de ponta-cabeça na tela. Não é hipótese: o agente que reescreveu os símbolos reportou que suas
primeiras asserções de coração e gota eram exatamente isso, e só descobriu espelhando cada
forma na mão.

O gate rendeu na primeira execução:

- a **pirâmide** é uma armadilha de vacuidade viva — o predicado óbvio ("o ponto mais alto é
  uma quina no eixo") vale para cone e gota, mas de cabeça para baixo o vértice frontal da
  base *também* é uma quina no eixo, no extremo;
- um teste do **cilindro** era vazio: o contorno principal dele é exatamente simétrico em Y,
  então asserir sobre a barriga passava idêntico invertido. Quem distingue o cilindro do seu
  espelho é a **tampa**.

### Lição

> Um teste de orientação que não distingue a forma do seu espelho **não testa nada**.
> Espelhe a forma e exija que o teste fique vermelho — se ele não ficar, ele é decorativo.

---

## Bug #3 — A seta curvada era um polígono (2026-07-12) — **FECHADO**

**Sintoma:** "Seta curvada com segmentos retos, deveriam ser arredondados."

**Causa:** o corpo era amostrado em cordas retas de 15°. Medido: sagita de `8,6·10⁻³·r` — a
um raio de 1000 px são **8,6 px** de erro, perfeitamente visível. A espiral tinha o mesmo
defeito (24 quinas por volta).

**Correção:** arcos em cúbicas de ≤45° com o comprimento de handle exato,
`(4/3)·tan(θ/4)·r` — erro `4,2·10⁻⁶·r`. A espiral virou 8 cúbicas por volta com a tangente
analítica nos handles (e **um terço** dos vértices do polígono antigo).

**Armadilha na correção:** o handle de Hermite ingênuo (`Δθ/3`) fica **1,5% curto** num
quarto de volta — é exatamente a diferença entre `π/6 = 0,5236` e o `KAPPA = 0,5523`. A
espiral saía visivelmente "murcha" para dentro. O teste pegou.

**Gate:** o desvio radial é medido *entre* as âncoras (âncoras ficam **sobre** o círculo
mesmo num polígono — um teste que só as olhasse passaria por acidente), e a mesma régua é
aplicada a uma cópia poligonizada do path, que tem de ser ≥100× pior. Assim o teste prova
que **a própria régua funciona**.

---

## Bug #4 — Balão de fala com contorno auto-intersectante (2026-07-12) — **FECHADO**

**Sintoma:** invisível no preenchimento, visível no traço — um zigue-zague cruzado na base
do rabo.

**Causa:** o rabo era inserido **antes** do primeiro vértice da aresta de baixo, quando o
percurso do round-rect exige inseri-lo **depois**. O `NonZero` fill escondia.

**Correção:** o balão passou a ser construído sempre com o rabo saindo à direita e girado
por quartos de volta no fim — **um** caminho de código para os quatro lados, e a ordem de
percurso deixa de ter como estar errada.

**Gate:** varredura de 75 configurações de ponta × folga, detecção de auto-interseção na
curva achatada — **mais um meta-teste que alimenta uma gravata-borboleta ao detector** e
exige que ele a pegue. Sem isso, um detector quebrado passaria verde sem detectar nada.

---

## Bug #5 — Contorno ABERTO recortava o preenchimento ("cubo 3D imperfeito", 2026-07-12) — **FECHADO**

### Sintoma

Um triângulo **escuro** comendo metade da face direita do cubo isométrico. A silhueta e as
arestas estavam certas; faltava tinta no meio.

### Causa

O cubo é uma silhueta hexagonal **fechada** + três arestas internas, que são contornos
**abertos**. O renderer construía **um** `BezPath` com tudo e o usava para preencher *e*
traçar.

Preenchimento **fecha contorno aberto implicitamente** — é a semântica de fill de qualquer
rasterizador, não um bug do Vello. A corda que fecha a polilinha `V1 → M → V3` recorta um
triângulo com winding próprio que, sob `NonZero`, **cancela** o hexágono onde coincide. O
triângulo escuro é literalmente a aresta interna do cubo, fechada.

**Era uma classe, não um caso.** A mesma doença, em forma de lente, atingia a boca da base
do cone e a tampa do cilindro (arcos abertos fechados pelas suas cordas) — só que menos
visível. As barras da sub-rotina e a cruz da junção escapavam por acidente: sendo segmentos
de 2 pontos, a corda tem área zero.

### Correção

Uma regra semântica, não um remendo: **um contorno aberto não tem interior — ele é uma linha
de construção.** Em [`ph2d-vec-render`](../../crates/ph2d-vec-render/src/lib.rs):

- `build_fill_bezpath` — só os contornos FECHADOS. É o que se preenche.
- `build_lines_bezpath` — só os ABERTOS. É o que dá volume ao sólido.
- `build_bezpath` — tudo. É o que se traça.

**Corroboração de que a regra é a certa, e não uma invenção:** `path_contains_point` (o
hit-test do gizmo) **já** pulava os contornos abertos (`if !closed { continue; }`). O resto
do código sempre tratou "aberto = sem interior" como a semântica correta. O renderer é que
estava fora do passo.

### Gates

`open_contour_tests::an_open_contour_never_punches_a_hole_in_the_fill` mede o **winding no
baricentro do triângulo** — o miolo exato da mancha da foto. Ele prova as duas metades da
história:

1. o path do preenchimento cobre a face (`winding ≠ 0`);
2. o path COMPLETO — que era o que se preenchia antes — **a perfura** (`winding = 0`).

A segunda asserção é o que impede o teste de perder o poder de discriminar: se um dia os
dois paths coincidirem, ela cai e avisa.

Mais `fill_and_lines_partition_every_shape_in_the_catalogue`: para as 48 formas, os dois
caminhos **particionam** o path inteiro, e toda forma aberta tem preenchimento vazio.

---

## Bug #6 — Previews do painel espelhados (2026-07-12) — **FECHADO**

### Sintoma

As formas certas no canvas, **de cabeça para baixo na grade de thumbnails** do painel.

### Causa

**A mesma fronteira do Bug #2, do outro lado.** O mundo é Y-para-CIMA; a tela é
Y-para-baixo. No canvas, a câmera faz a inversão (`world_to_screen_affine` = `scale(k, −k)`).
O painel pinta **direto em coordenadas de tela** e montava a transformação do thumbnail com
`Affine::scale(s)` — uniforme e **positiva**. Faltava o sinal.

É o jeito mais confuso possível de o bug se apresentar: a geometria está certa, o canvas
está certo, e só o preview mente.

### Correção

`Affine::scale_non_uniform(s, −s)` (e o `+ scale·bcy` na translação). Extraído para
`thumb_affine`, uma função pura — para poder ser testada.

### Gate

`the_thumbnail_flips_y_so_the_preview_is_not_upside_down` exige que **o ápice do cone seja
pintado ACIMA da base na célula**. Um teste que conferisse apenas a ESCALA (que a forma cabe
na célula) passaria feliz com o preview invertido — é a mesma armadilha de vacuidade do
Bug #2, e a única asserção que a evita é a que compara a ordem vertical.

---

## Bug #7 — O `spread` de conectores paralelos era um PLACEBO (2026-07-12) — **FECHADO**

### Sintoma

Nenhum. E é esse o ponto.

O bug foi encontrado **lendo o código para expor o parâmetro no painel** — não por uma queixa.
Ele teria aparecido para o Enio no instante em que ele arrastasse o slider e nada acontecesse.

### Causa

Dois conectores entre o mesmo par de formas nascem com um deslocamento para não se
sobreporem. A primeira versão o aplicava aos **vértices do meio** da rota:

```rust
fn offset_middle(pts: &mut [[f64; 2]], spread: f64) {
    let n = pts.len();
    if n < 4 { return; }
    for p in &mut pts[2..n - 2] { p[0] += spread; }   // ← range VAZIO quando n == 4
}
```

Parece razoável até você contar os vértices. Duas caixas **empilhadas** dão uma rota de
quatro pontos — `p0, s0, s1, p1` —, e `pts[2..2]` é **vazio**. O laço não itera. Os dois
conectores saíam idênticos, um exatamente por baixo do outro, e o segundo era invisível.

E quando `n > 4`, o deslocamento era em `x` **sempre** — mesmo quando o trecho do meio corria
na horizontal, caso em que somar a `x` desliza o vértice *ao longo* do próprio segmento e não
desloca linha nenhuma.

### Por que passou por todos os gates

Não passou: **nenhum gate olhava**. Havia seis testes de rota, todos com `spread: 0.0`. O
parâmetro era exercitado por zero asserções — código que roda, não faz nada, e ninguém nota,
porque a única prova de que ele funciona seria um teste que ninguém escreveu.

### Correção

A separação não é do **meio** da rota: é do **ponto de saída**. Arestas paralelas encostam em
pontos diferentes da face (é o que o draw.io e o Visio fazem). Isso exige a FORMA — o contorno
real em que o ponto desliza —, que o roteador não conhece de propósito; então quem aplica é o
chamador, deslizando a **origem do raio** antes de chamar o `boundary_hit`:

```rust
let n = perp(ray);
let limit = SPREAD_FACE_K * (n[0].abs() * hw + n[1].abs() * hh);   // não escorregar da face
let from = [c[0] + n[0] * s, c[1] + n[1] * s];
boundary_hit(p, xform, from, ray, GAP)      // o ponto continua NO contorno, por construção
```

Deslizar a origem em vez de deslocar o resultado não é preciosismo: mover o ponto *depois* de
achá-lo o tiraria do contorno, e numa estrela ou numa nuvem ele passaria a flutuar ao lado da
forma.

Duas consequências que caíram de graça:

- **As duas pontas deslizam com sinais OPOSTOS** (`spread` e `−spread`). As normais das duas
  faces apontam uma contra a outra, então o mesmo sinal as moveria em direções contrárias no
  mundo — e o conector sairia **torto** em vez de paralelo.
- **O feixe nasce centrado** (`auto_spread`: `0, +1, −1, +2, −2 …`). O índice cru empilhava
  todos para o mesmo lado, e o *primeiro* conector — o caso comum! — já saía fora do centro.

### Gate

`two_connectors_between_the_same_pair_never_overlap`, com as caixas **empilhadas de propósito**
(é a geometria que expunha o bug), medindo a **menor distância entre as duas polilinhas** — que
é o que o olho vê: duas linhas que se tocam são, para o usuário, uma linha só.

### Lição

Um parâmetro que nenhum teste exercita não está implementado — está **escrito**. `spread: 0.0`
em todos os seis testes era o aviso, e ele estava à vista.

---

## Padrões que se repetem (leia antes de caçar o próximo)

1. **O sintoma quase nunca é a causa.** "Cone de cabeça para baixo" era uma convenção de
   eixo em quatro módulos. "Panic no clamp" era uma janela geométrica que colapsa.

2. **Teste que não morde é pior que teste nenhum** — dá confiança falsa. Antes de aceitar um
   teste de propriedade, **quebre a propriedade de propósito** e confirme que ele fica
   vermelho. Três bugs desta lista foram encontrados assim, e não pela leitura do código.

3. **Varra a INTERAÇÃO, não os eixos.** Bugs de parâmetro moram nas combinações. Um gate que
   move um slider por vez é barato, roda rápido e **não encontra o bug que derruba o editor**.

4. **Renderize.** O agente do padrão-ouro dos balões descobriu três defeitos *desenhando* as
   formas, com todos os testes verdes. O do banner idem — a dobra tinha um buraco e nenhuma
   asserção olhava para lá.

5. **Uma fronteira mundo↔tela é sempre DUAS.** O Bug #2 (formas espelhadas) e o Bug #6
   (previews espelhados) são o mesmo `−1` faltando, em dois lugares diferentes. Ao corrigir
   uma travessia de coordenadas, **procure as outras**: quem mais pinta geometria de mundo
   sem passar pela câmera? (Resposta: o painel. E qualquer export futuro.)

6. **Quando um subsistema discorda dos outros, ele é o errado — mas confirme.** O hit-test
   já pulava contornos abertos; o renderer não. Duas implementações da mesma pergunta
   ("o que é o interior desta forma?") com respostas diferentes é sempre um bug. A maioria
   costuma estar certa, mas o valor está em *notar a discordância*, não em contar votos.

7. **Um parâmetro que nenhum teste exercita não está implementado — está escrito** (Bug #7).
   O sinal é visível a olho nu na suíte: se **todos** os testes passam o mesmo valor para um
   campo (`spread: 0.0`, seis vezes), esse campo nunca foi testado. Grepar o nome do parâmetro
   nos testes é mais rápido que ler a implementação, e encontra a mesma coisa.
