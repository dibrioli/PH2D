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

## Bug #8 — O filete caía num fallback frouxo (sinal de Cramer trocado) (2026-07-13) — **FECHADO**

### Sintoma

Nenhum, na tela. O arredondamento de quina "funcionava" — só era **errado**: o arco não era
o arco. Um gate que comparava o motor novo (`corner_live`) com o velho (`corners`, o gerador
das Live Shapes) na MESMA quina reta pegou: handle em `1.0572` onde devia ser `0.8954`.

### Causa

A cúbica do filete precisa da interseção das duas tangentes. Resolvendo
`s·t_in + w·t_out = d` por Cramer:

```
s = (d × t_out) / (t_in × t_out)        w = (d × t_in) / (t_out × t_in)
```

Os **denominadores são opostos** (`t_out × t_in = −(t_in × t_out)`). Eu dividi os dois pelo
mesmo. Resultado: `w` saía **negativo** numa quina perfeitamente boa, o guard de "a
interseção fica à frente dos dois pontos" a **reprovava**, e todo filete caía no fallback de
terço-de-corda. O número batia exato: `hypot(2,−2)/3 = 0,9428`.

### Lição

Um fallback que funciona **esconde** o bug do caminho principal. O guard estava certo; o que
ele testava é que estava errado. **Se um caminho de exceção nunca é exercitado nos testes,
ele pode estar sendo o caminho NORMAL** — e a única coisa que denuncia é comparar o
resultado com um oráculo independente. Aqui o oráculo já existia: o outro motor.

---

## Bug #9 — A "reta" é um smoothstep, e de Casteljau a envenenava (2026-07-13) — **FECHADO**

### Sintoma

Também nenhum na tela — e este teria estragado a **booleana** silenciosamente, meses depois.

### Causa

Uma aresta reta, neste modelo, é uma cúbica de **handles nulos**: `[(0,10), (0,10), (0,0),
(0,0)]`. Ela é geometricamente o segmento, mas **não é uma reta uniforme** — é um
*smoothstep* (velocidade zero nas duas pontas). Cortá-la com de Casteljau devolve pontos de
controle **no meio da aresta**, não nulos.

A curva traçada fica **idêntica**. A **representação**, não: ela deixa de ser a convenção de
reta. E é exatamente essa convenção que o `to_bez` da `ph2d-vec-boolean` testa para emitir
`line_to` — sem ela, o `linesweeper` devolve as quinas como `Smooth` com handles pendurados
no meio das arestas. **É o bug que aquele código já documenta e contorna**, e eu ia
recriá-lo pela porta dos fundos.

### Correção

`sub_cubic` corta a aresta reta **como reta** (lerp das pontas, handles nulos), e não por de
Casteljau. Detectado por igualdade EXATA (é uma convenção de representação, não uma medida
de geometria — e um afim leva handle nulo em handle nulo).

### Lição

**Geometria igual não é representação igual.** Duas cúbicas podem traçar exatamente a mesma
curva e significar coisas diferentes para o subsistema seguinte. Antes de reconstruir
geometria, pergunte quem mais LÊ o formato dela — e o que ele infere da forma dos pontos de
controle, não só da posição deles.

---

## Bug #10 — A alça escorregava do dedo (2026-07-13) — **FECHADO**

### Sintoma

Arrastar a alça de raio de quina: a bolinha **ficava para trás do cursor**, e a distância
crescia ao longo do gesto.

### Causa

A alça de uma quina afiada não pode ser desenhada em cima da âncora (sumiria dentro dela),
então ela **estaciona** alguns pixels adiante, sobre a bissetriz. Eu implementei isso como um
**PISO** no desenho (`max(setback, park)`), enquanto o arrasto lia `setback = projeção −
park`. Os dois não se cancelam: agarrar a alça estacionada e levar o cursor a `2,0` deixava a
bolinha em `1,86` — a diferença é o `park`, e ela **não some**, fica.

### Correção

O `park` é um **deslocamento CONSTANTE**, não um piso: desenhar em `park + setback` e ler
`− park` se cancelam, a bolinha pousa exatamente no cursor, e o raio ainda cresce de zero.

### Gate

`the_handle_stays_under_the_cursor_all_the_way_through_the_drag` mede a deriva em **quatro**
pontos do arrasto. Um ponto só, perto do início, teria passado — o erro do piso é constante
em valor absoluto, mas só fica óbvio depois de andar.

### Lição

É a família do bug que o `connector.rs` já documenta (*"desenhar num raio e agarrar noutro"*),
na versão que **se acumula ao longo do gesto**. Uma alça tem TRÊS números que precisam
concordar, não dois: onde ela é desenhada, onde ela é capturada, e **como o arrasto dela é
lido**. Os dois primeiros eu já sabia. O terceiro me pegou.

---

## Bug #11 — A alça funcionava e depois esquecia (2026-07-13) — **FECHADO**

### Sintoma

Nenhum teste vermelho. Encontrado **simulando o caminho do smoke do Enio no papel**, o que é
o método que este doc inteiro recomenda e que eu quase pulei.

Desenhar um retângulo com a Shape tool → ele nasce **Live Shape** → modo Node → a alça de raio
aparecia nas quinas dela e **funcionava**. Até encostar num slider do painel: `recook_into`
reescreve `path.verts` INTEIRO a cada mudança de parâmetro, e o `corner_radius` mora dentro
do vértice. O raio sumia **sem erro nenhum**.

### Por que os gates passaram

Eu havia gateado que a alça é do **modo Node**, e achei que estava coberto. Mas uma **forma
viva selecionada dentro do modo Node** é outra coisa — o gate de MODO nunca a alcança.

### Correção

Uma forma viva não tem alça de raio, nem no render nem no **hit-test** (senão a alça ficaria
invisível e ainda agarrável — um alvo fantasma). E não há conserto por preservar os raios no
recook: a **contagem** de vértices é função dos parâmetros. O raio de uma forma viva é um
campo dela (o painel); o por-vértice é para caminho desenhado (ADR-0121).

### Lição

**"Funciona e depois desfaz sozinho" é pior que "não funciona".** O usuário confia e perde o
trabalho em silêncio.

E: um gate sobre o **modo** não é um gate sobre a **política**. Se a regra é "isto não vale
para objetos do tipo X", teste objetos do tipo X — não os modos em que eles costumam
aparecer.

---

## Bug #12 — Um CLIQUE no Shape Builder dissolvia a arte (2026-07-13) — **FECHADO**

### Sintoma

O Enio smokou, desenhou um pentágono, uma estrela e um retângulo arredondado sobrepostos,
entrou no modo Build e reportou quatro coisas: *"undo não funciona"*, *"as silhuetas das
formas sobrepostas somem"*, *"o véu está estranho e grande e não bate com as formas"*, *"há
problemas no algoritmo"*. **Dezesseis gates verdes.**

### O que estava acontecendo (medido no app, não deduzido)

`resolve()` devolvia a sobra do gesto como `união(todas as fontes) − o que foi levado`: **uma**
forma só, com **um** estilo só. Um clique numa face trocava as três formas por (a face) + (um
blob de 24 vértices com a bbox da união inteira) + (uma lasca de 0,05 unidade). O pentágono
laranja e a estrela verde **desapareciam** — e as fronteiras entre as formas, que são
justamente o que define as faces seguintes, deixavam de existir.

Ou seja: as queixas 2 e 4 do Enio eram a MESMA coisa, e não eram sobre o véu.

### Correção

Cada fonte sobrevive como **ela mesma menos o que foi levado**, com o estilo dela e no z dela.
A forma que o gesto **não atravessou não é sequer tocada** — mantém id, entidade, `Transform`,
raio de quina e params de Live Shape. É o que o Illustrator faz: o Shape Builder divide o que
você percorre e deixa o resto em paz.

### Lição

**Um gate que olha o RETORNO de uma função não vê o que o artista fica.** Todos os 16 mediam
`resolve()`; nenhum mediu a **cena depois do gesto**. O oráculo de uma ferramenta de edição é
o documento, não o valor de retorno.

---

## Bug #13 — A borda do véu tinha 150 pixels (2026-07-13) — **FECHADO**

### Sintoma

O "véu estranho e grande que não bate com as formas" (queixa 3).

### Causa

`draw_build_faces` emitia `Stroke::new(EDGE_PX)` **sob o afim mundo→tela**. O Vello escala o
traço pelo afim: `1.5` não era 1,5 px, era **1,5 unidades de MUNDO** — no zoom default, ~150 px
de tinta opaca por cima exatamente da arte que o artista precisa ver.

Toda a crate já fazia o contrário (`draw_corner_handles`: *"o ponto sobe pelo afim, o raio
não"*). Este era o único sítio que destoava, e era o mais novo.

### Correção

`edge_strokes()` — função pura que devolve **a geometria já em coordenadas de tela e a largura
já em pixels**. O gate mede exatamente a propriedade: dar zoom não muda a largura, e muda a
geometria. Mutei a largura para acompanhar o afim (o bug original): vermelho.

### Lição

**Uma constante com o sufixo `_PX` não é uma promessa — é uma intenção.** Quem verifica a
unidade é o afim que você passa junto. Ao criar a 2ª função de desenho de uma crate, leia a 1ª:
a convenção estava escrita, com o motivo, a 40 linhas de distância.

---

## Bug #14 — O realce pairava sobre nada (2026-07-13) — **FECHADO**

### Sintoma

*"As silhuetas das formas sobrepostas somem — deveriam continuar visíveis no Build."*

### Causa

As faces do arranjo seguem as bordas das formas sobrepostas — mas **uma forma coberta por
outra não está na tela**. O artista via um realce recortado por contornos que não
correspondiam a nada que ele enxergasse. Nenhum bug de geometria: um bug de **desenho ausente**.

### Correção

As silhuetas das formas de origem são redesenhadas **por cima do véu** (traço fino, em px). É o
que o Illustrator faz, e é literalmente o que o Enio pediu.

### Lição

**O realce É a feature.** A booleana já existia; o que o Shape Builder acrescenta é a
capacidade de VER as regiões antes de escolher uma. O agente anterior escreveu o realce por
último, sem gate — e é onde estavam duas das quatro queixas.

---

## Bug #15 — "O undo só faz uma etapa e não funciona mais" (2026-07-13) — **FECHADO**

### Sintoma

O Enio: um Ctrl+Z funciona; do segundo em diante a cena não sai do lugar. Reprovado no smoke.

### Causa

**Não estava no undo.** A pilha de z das formas é a **projeção da árvore** (ADR-0110), e o passe
que a aplica projetava da **lista do PAINEL** — publicada no *prólogo* do frame, **antes** de
`vec_entities::sync` dar entidade à forma recém-criada. Quem o `VecScene::reorder_to` não conhece
recebe chave 0 e vai pro **FUNDO**; a cena só convergia **um frame depois**, em silêncio.

A captura do undo é tirada no fim do frame da **AÇÃO** — ou seja, **antes de convergir**. Ela
**não era ponto fixo dos sistemas**: restaurá-la e deixar o frame rodar produzia *outra coisa*.
O diff por-frame do `post_frame_undo` lia essa diferença como ação do usuário → nascia um **passo
espúrio** (mesmos ids, só a ORDEM diferente) que **limpava a pilha de redo** e re-empurrava o
estado. O Ctrl+Z seguinte desfazia **o lixo que ele mesmo acabara de criar**.

Medido (`PH2D_BUILD_SMOKE=6 PH2D_UNDO_LOG=1`): em regime a cena ficava `[3,4,5,6,1]` (ordem de
**inserção**); depois de qualquer undo convergia para `[6,5,4,3,1]` (a **projeção real**).

### A segunda mina, que o gate expôs

Um sprite importado nasce **sem `RootOrder`** (`image_import.rs`) ⇒ colate em `u32::MAX` ⇒ o sort
de raízes desempata por **`Entity::to_bits()`**, o id de **ALOCAÇÃO** — e o restore do undo
**despawna e re-spawna o mundo inteiro**. O gate do ponto fixo com uma forma pendurada num sprite
nasceu **VERMELHO**: a pilha invertia (`[0,1]`→`[1,0]`) a cada Ctrl+Z. O passo espúrio voltaria
vestido de outra coisa, e ninguém teria ligado uma coisa à outra.

### Correção

1. **A projeção lê a árvore DEPOIS do `sync`** — do `build_hierarchy_snapshot` (a **mesma** função
   que alimenta o painel; um DFS próprio seria uma segunda porta, e duas portas divergem), com
   scratch próprio (`z_walk_state`/`z_snapshot`), para não descasar a `bridge` do painel.
2. **`ph2d_ecs::assign_missing_root_order`** — toda raiz ganha número explícito, na ordem que a
   tela já mostra. **Não se escolhe um desempate melhor: não se tem empate.** A ordem vira função
   do CONTEÚDO e sobrevive ao respawn.

**Gates:** `vec_zorder_fixpoint_tests.rs` (5, um deles *nasceu vermelho*: o do sprite) +
`tests/the_z_projection_reads_the_tree_after_the_sync.rs` (arch-gate de **ordem do frame**, sobre
o arquivo do produto — os unit tests rodam um *espelho* da sequência, e um espelho não vê o dia em
que alguém reordena o frame de verdade).

### Lição

**Toda captura de estado tem de ser PONTO FIXO dos sistemas:** *capturar → rodar um frame →
capturar* tem de dar a mesma foto. Quem captura por diff (undo) ou por instantâneo (save) lê
qualquer normalização atrasada como se fosse o usuário. E o corolário que ninguém viu: **o SAVE
gravava a mesma captura não-convergida** — o bug não era só do undo.

---

## Bug #16 — O Build deixava "pedaços de linha" sobrando (2026-07-13) — **FECHADO**

### Sintoma

O Enio: *"a ferramenta Build funciona bem, mas está deixando pedaços de linha sobrando"* — linhas
soltas (uma curva longa, um "Λ") flutuando no meio do desenho depois do gesto.

### Causa (medida — a primeira teoria estava errada)

Toda sobra é `fonte − união(faces levadas)`, e as faces vêm do **arranjo**. A borda que elas
devolvem é uma **re-derivação** da borda da fonte, não a mesma sequência de bytes — então
**subtrair uma curva dela mesma, depois de ela ter dado a volta pelo arranjo, deixa resíduo**.

A booleana comum (os botões do painel) é **limpa**: 2 operandos, zero lasca. Isto é do Build.

**E é por isso que o resíduo aparece como LINHA:** sem área, não há preenchimento para pintar —
o que sobra na tela é o *traço* da fonte, percorrendo a borda e voltando.

Medido (`PH2D_BUILD_LOG=1`), duas populações que não se tocam:

| | área (fração da fonte) | verts | densidade (área/bbox) |
|---|---|---|---|
| resíduo de borda **curva** | **0,0000%** (área ~1e-13) | 144 | 0,00 |
| resíduo de **quina** | 0,07% – 0,30% | 3–4 | 0,44 |
| **arte** | **6,5% – 81%** | 3–21 | 0,41 – 0,79 |

### O fixture escondia o bug

Os 11 gates do Shape Builder rodavam sobre **polígonos** — e o polígono só produz o resíduo de
quina (uma pontinha gorda). A **hairline só nasce em borda curva**, que é o que o produto entrega.
É a lição §6.1 desta linha outra vez, e ela custou o smoke: *mutar o código dentro de um universo
de quadrados só prova coisas sobre quadrados.*

### Correção

`drop_slivers` no `resolve`: uma peça abaixo de **0,5% da área da fonte** não é região, é resíduo
de borda (13× abaixo da menor peça de arte medida; 1,7× acima do maior resíduo). Vale para a sobra
E para a forma nova. `PH2D_BUILD_LOG=1` diz o que foi descartado e por quê — **geometria que some
em silêncio é pior que uma lasca**, então ela não some em silêncio.

### Lição — **o gate quase nasceu MORTO**

A 1ª versão do gate assertava `área > 0`. Ficou **verde com o filtro desligado**: a hairline não
tem área *exatamente* zero, tem **1e-13**. O oráculo certo não é a regra do filtro (isso é
circular) — é a **aparência**: uma peça sem *espessura* pinta uma linha, e isso é **densidade**
(área/bbox), que mede 0,00 na lasca e ≥ 0,41 na arte. Só mutando o código é que o gate morto
apareceu.

---

## Bug #17 — O quadrado GIRAVA a caminho do círculo (2026-07-14) — **FECHADO**

### Sintoma

O Enio, no 2º smoke do Blend: *"o porquê da rotação?"*. Um quadrado a caminho de um círculo: os
intermediários rodavam **45°** e voltavam.

### A causa não estava na busca — estava no CONJUNTO DE CANDIDATOS

Medido (o probe imprimiu âncoras e viradas do catálogo):

```
quinas do quadrado:  -135°  -45°   45°  135°     virada (sen ±1, cos 0)  ← quinas de verdade
âncoras do círculo:     0°   90°  180°  -90°     virada (sen  0, cos 1)  ← PERFEITAMENTE SUAVES
casamento escolhido: cada quina → a âncora 45° adiante  ⇒ giro de 45°
```

**Uma âncora de contorno suave não é uma feature — é artefato da parametrização.** As 4 âncoras do
círculo existem porque a elipse é **cozida em 4 cúbicas**; o artista nunca as autorou. Mas o motor
**obrigava** cada quina a casar com uma âncora, e a resposta certa — a quina a 45° vai para o ponto
do círculo a 45°, que cai **no MEIO de um segmento** — **não estava no conjunto**. O melhor de
quatro casamentos ruins é o giro de 45°.

### Correção

1. **Só uma QUINA é candidata a nó** (`features()`): virada acima de um limiar (cos < cos 15°). O
   limiar é escrito em **cosseno**, não em seno — uma **cúspide** vira ~180°, onde o seno volta a
   zero, e `|sen| > ε` classificaria o bico mais afiado que existe como suave.
2. **Sem quina dos dois lados, a correspondência é uma FASE CONTÍNUA** (`phase_only`): correlação
   circular sobre 256 amostras de arco + refino parabólico. A fase deixa de estar presa às âncoras.

### Os DOIS defeitos que ele escondia (nenhum tinha sido visto)

- **Dois círculos iguais, com parametrizações desalinhadas, ENCOLHIAM para raio 0,924** no meio do
  caminho (7,6%): os pontos atravessavam o disco em vez de caminhar radialmente. O catálogo esconde
  isso — as elipses nascem todas com as âncoras nas mesmas posições de arco. Só aparece quando
  **não** nascem (o artista desenha a segunda à mão).
- **Uma aresta inteira SUMIA do pareamento.** Os cortes de B são as **imagens** dos cortes de A, e a
  imagem que devia cair na origem volta do ida-e-volta `map_backward`→`map_forward` como
  **`1 − 3,7e-14`**: o `f64` não fecha o ciclo. O `cut` então **truncava** a peça em `1.0` e o arco
  entre a origem e o corte seguinte ficava **sem peça nenhuma**. O invariante ("a origem está na
  lista") era **esperado do chamador**; agora é **estabelecido por quem depende dele**.

### E um terceiro, que só apareceu porque eu fui procurar

**Picar uma aresta reta em 20 pedaços mudava a correspondência.** Geometria idêntica, âncoras a
mais — e o quadrado casava com **outros** vértices da estrela. O centro e a escala que normalizam o
custo saíam da **média das âncoras**, e âncora é parametrização: os pontos novos arrastavam a média
para o lado da aresta picada. Agora o quadro sai de amostras equiespaçadas em **arco**.

Não é caso de laboratório: todo caminho traçado, importado ou passado por um `Simplify` tem âncoras
onde o algoritmo as deixou. **Duas formas que se veem iguais têm de blendar igual.**

### O limiar do "o que é uma quina" não podia sentar num ângulo que o catálogo faz

A 1ª versão do fix usou `cos(15°)` — e **um polígono regular de 24 lados vira exatamente 15°**. A
peneira das features é uma comparação estrita; o `f64` erra o cosseno no último bit; as **24 quinas
idênticas** caíam dos dois lados da cerca por ruído. Medido: **transladar a cena** (só mover, sem
deformar) mudava a forma do intermediário em **5,5%**; um ruído de 1e-13 no raio dava **68 resultados
distintos**. É o mesmo `f64`-decide-um-empate que já custou o z-order (#15) e a aresta que sumia
(acima) — desta vez no **limiar**, não na fronteira.

Um polígono regular de N lados vira `360/N`. O limiar virou **`cos(16°)`**: `360/16 = 22,5` não é
inteiro, então nenhum polígono regular consegue produzir 16°, e a cerca fica num vale vazio. O gate
`the_corner_threshold_does_not_sit_on_a_polygon_the_catalog_can_make` **enumera** N até 128 e é ele
que impede o próximo a mexer no número de reintroduzir o empate.

### O Reverse Match era um botão que NUNCA ajudava (smoke seguinte do Enio)

O Enio testou estrela→círculo e pediu para conferir **Rotate/Reverse** ("resultados estranhos"). O
probe mostrou os intermediários **rasgados/colapsados** no meio. Medido, por par de formas:

| | `min\|área\|` ao longo de t |
|---|---|
| estrela→círculo, **Reverse** | **0,026** (colapso total) |
| quadrado→estrela, **Reverse** | **0,040** |
| círculo→círculo, **Reverse** | **0,000** |

**O Reverse inverte o sentido de percurso de B, o que inverte o WINDING** (área +π → −π no círculo
final). Interpolar de um contorno CCW (a estrela) para um CW (o círculo revertido) cruza winding zero
no meio → colapso. **Todo o catálogo nasce com o mesmo winding**, então o botão colapsava em 100% dos
casos. E era **redundante**: quando B de fato tem winding oposto (uma forma importada ao contrário), o
`search` já testa os dois sentidos e fica com o de menor custo — provei dando uma B em CW e o auto
escolheu `reversed=true` sozinho, meio limpo. **Removido** (botão + ação + campo `BlendOpts.reverse` +
const `VECTOR_BLEND_REVERSE`). Nenhuma ferramenta profissional (Illustrator, GSAP) tem um "reverse"
que inverte winding — elas detectam a direção automaticamente, que é o que o nosso auto já faz.

### O Rotate Match tinha o quantum errado no caso suave

Estrela→círculo, o quantum era `1/âncoras-do-círculo = 1/4 = 90°/toque`, e o 2º toque (180°)
colapsava. As âncoras do círculo são **arbitrárias** (artefato de cozer a elipse em 4 cúbicas). O
quantum agora vem das **quinas da forma que TEM quinas** (a estrela, 10 → 36°/toque): passos finos, a
torção cresce em vez de colapsar de uma vez. A torção em rotação grande é intrínseca ao lerp de
coordenadas (o gap Sederberg/Alexa, ainda aberto). Gate: `rotate_steps_by_the_corners_not_by_the_
smooth_shapes_anchors` (nos dois sentidos).

### Lição — **a pergunta certa era "o que é uma FEATURE?"** — e um escape que nunca ajuda é um enfeite

Os dois handoffs anteriores desta linha apontaram um suspeito nº 1, e os dois estavam errados. O que
resolveu foi **medir**: imprimir a virada de cada âncora do catálogo, ver `(0, 1)` nas quatro do
círculo, e entender que **não havia nada ali para casar**. O defeito não era um erro de conta — era
uma **pergunta mal formulada** ao dado.

E o Reverse ensina o corolário do escape: **um botão de escape que produz lixo em 100% dos casos não é
um escape — é um enfeite que engana.** O sinal foi a MEDIÇÃO (colapso em 3 pares distintos), não a
leitura do código. Quando um controle "tem resultados estranhos", meça-o em vários casos antes de
calibrá-lo: pode ser que ele nunca devesse existir.

E dois corolários que valem além do blend:

- **Quando um algoritmo escolhe entre candidatos e a resposta é sempre ruim, desconfie do CONJUNTO
  antes de desconfiar do critério.**
- **Um limiar mora onde o domínio é vazio.** Não se calibra um limiar para "quase nunca cair em cima
  de um valor real" — escolhe-se um valor que **nenhuma** entrada legítima consegue produzir, e
  gateia-se isso enumerando as entradas.

---

## Bug #18 — "Round não muda" e "só apply aplica" — a saga do Offset ao vivo (2026-07-20/21) — **FECHADO**

Cinco rodadas de report do Enio sobre a MESMA feature, com **três causas diferentes** empilhadas
umas sobre as outras. Cada rodada tem uma lição própria, e a última é a que valia desde o começo.

### Sintoma (as cinco rodadas, na ordem)

1. *"queda de fps, muda em tempo real para round mas não muda para Miter e Bevel"*
2. *"funciona corretamente, mas com queda de FPS"*
3. *"melhorou FPS mas regrediu: Round para Bevel ou Miter não muda mais"*
4. *"não resolveu! se selecionar Round, não consegue mudar"*
5. *"no momento que aperto Round a curva já tem todos os vertex novos criados antes de apertar
   apply … a idéia é bevel desfazer round e aplicar bevel. Só apply aplica definitivamente"*

### Causa 1 — o FANTASMA no `d` extremo (rodada 1)

Com caneta `2|d|` maior que o próprio laço, o contorno interno da banda degenerava em ruído de
winding e o refugo **atravessava o sweep**. Medido no donut: encolher além da morte devolvia NADA
no Miter (correto) e uma **ilha de 12 verts / área 2,52 no Round e no Bevel** — não-monotônico e
**diferente por join**, que é o report ao pé da letra. Fix: `drop_phantoms` na porta única
`loop_region` (discriminador = distância ao laço fonte, teste pelo **MÁXIMO** do contorno).

### Causa 2 — **o DIAL era o bug** (rodadas 3-4), e é a lição mais transferível daqui

Depois do fantasma morto, o report virou *"não muda mais"* — **num build cujo código de produto era
byte-idêntico ao que ele tinha aprovado**. A janela de retune sempre funcionou (o smoke prova: undo
andando, verts mudando, janela viva). O que falhava era o **slider**:

- ele mapeava **±4 unidades de MUNDO sobre ~104 px** de track, numa forma de 2,4 ⇒ o gesto natural
  (arrastar com vigor) **saturava sempre**;
- e os dois extremos são regimes **join-inertes**: à esquerda a forma **aniquila** (os três joins
  produzem o mesmo nada — clicar chip não muda um pixel, literalmente) e à direita ela cresce
  **4,3×** e estoura a tela, levando embora as quinas, que é onde o join mora.

Medido: **37% do curso do slider era forma MORTA**. A rodada 1 era o fantasma NESSE mesmo regime (a
`d = −4` o Round ressuscitava o blob — mudança visível! — e Miter/Bevel aniquilavam); o
`drop_phantoms` igualou os três no nada, e daí *"não muda mais"*.

A faixa antiga confessava a origem no próprio comentário: *"a vista do editor mede ~10 unidades"* —
**derivada da VISTA**, quando o alcance útil de um offset é propriedade da **FORMA**. Fix: o slider
passou a falar **fração do tamanho da seleção** (`d = fração × maxdim/2`, chip em percentual):
**−100% = morte garantida** (o inradius de qualquer forma é ≤ maxdim/2) e **+100% = dobrar a forma**
(quinas na tela ⇒ todo retune muda pixels visíveis). Curso morto 37% → ≤ 3%.

E a fileira gêmea: o painel tinha **duas** fileiras "Join · Miter/Round/Bevel" idênticas (Stroke = a
quina do traço, no-op visível num shape sem traço; Expand = a quina do offset). A do Expand virou
**"Corner"**.

### Causa 3 — **o MODELO** (rodada 5): o offset materializava no release

A queixa final não era sobre nada disso: era que o resultado do offset **já era um objeto do
documento**. No modo Node o artista via as ~238 âncoras dos arcos do Round e concluía — com razão —
que a curva já tinha sido assada, e que o Bevel seguinte chanfraria *aqueles* vértices.

⚠️ **MEDI ANTES DE MEXER, e a medição mudou o alvo:** a geometria **nunca compôs**. O gate
`each_preview_re_derives_from_the_source_never_from_the_previous_preview` compara Round→Bevel com um
Bevel **fresco da forma pristina, âncora a âncora (1e-6)**: idênticos. O `clone_from(&pre)` já fazia
*"bevel desfaz round e aplica bevel"*. O defeito era o **modelo**, não a conta.

**Fix (o que fechou):** componente `VecOffset{d, join, side}` na entidade; a shell **cozinha por
frame** (memoizado) e o renderer desenha o cozido; o documento continua guardando a **curva
autorada** — no Node aparecem os nós DELA, editáveis, e o offset acompanha. **Apply Offset** (ou
Convert to Curves) é o único momento em que os vértices novos entram no documento. É a costura
fonte≠cozido do ADR-0121/0132, um nível acima. Prova no smoke 19: `src=26` **parado** durante o
arrasto e os quatro retunes, `live` 220 (Round) → 16 (Bevel) → 8 (Miter); no Apply **`src 26→238`,
`live→0`**.

⚠️ **A arquitetura foi decidida pelo COMPILADOR, não por gosto:** montar o offset na pilha de Live
Path Effects **não compila** — um efeito é avaliado dentro da `ph2d-vec-scene` (crate pura) e o
offset precisa da `ph2d-vec-boolean`, que depende dela; `error: cyclic package dependency`. A saída
"registrar o motor por ponteiro global" foi **recusada**: faria `cooked()` — a resposta a *"o que
este documento desenha?"* — depender de alguém ter chamado um instalador, e o mesmo arquivo
desenharia diferente em processos diferentes, **em silêncio**.

**Aniquilação:** a entrada fica **presente e vazia**, nunca ausente — a arte não sai da cena e volta
ao subir o slider.

### Os DOIS remendos meus que não eram a correção (e o que custaram)

1. **Fazer o retune substituir o próprio passo de undo.** Legítimo por si (N retunes = 1 passo), mas
   respondia a uma pergunta que o Enio não fez. Virou **código morto** no modelo certo.
2. **Esconder as âncoras do preview** — e aqui eu causei uma **regressão**: gatear a chamada
   `draw_overlays` inteira apagou os nós de **TODAS** as formas no modo Node enquanto a janela
   vivesse. Revertido no report seguinte. ⚠️ **Suprimir um overlay globalmente para resolver um
   problema de UMA forma é sempre grande demais** — o raciocínio "essa geometria é transiente, então
   não decore" estava certo; o alcance da implementação, não.

O padrão dos dois: **remendo sobre a premissa errada**. A premissa era *"o offset materializa no
release"*, e enquanto ela ficou de pé, toda correção era em cima do sintoma.

### O ORÁCULO que faltava (e por que a suíte era verde o tempo todo)

O gate da cadeia afirmava *"Round, Bevel e Miter produzem verts DISTINTOS"*. Essa asserção **passa
mesmo se o preview compuser** — bevel-de-round também difere de round. Quem pega a composição é a
**identidade com um resultado FRESCO da fonte**. Um oráculo de *diferença* é quase sempre mais fraco
do que parece: ele descarta o caso trivial e nada mais.

### Três mutações SOBREVIVERAM, e todas eram a FIXTURE

- **cook lendo `verts` cru** sobreviveu sob escala **uniforme** — a pilha de efeitos é invariante a
  ela *de propósito*. RED só com `scale(3,1)`: a fixture que testa *em que ESPAÇO o efeito é medido*
  tem de ser **não-uniforme**.
- **`expand_selection` lendo cru**, idem.
- **o resolver ignorando o id do caminho** é indistinguível com **uma forma só** na cena.

### Interferência do AMBIENTE no harness (custou uma investigação inteira)

O desktop é vivo: o WM reposiciona a janela recém-aberta sob o cursor **físico** parado e isso emite
`CursorMoved` **reais**, que o slider agarrado obedece. Um hold sem re-assert foi teleportado a
`d = −4` pelo ambiente, e eu persegui esse fantasma como se fosse do app. Os roteiros auto-dirigidos
re-afirmam a posição sintética **todo frame do hold E no frame do release** (o `up` solta onde o
ponteiro está, e o cursor físico pode ter falado por último).

### Lições

- **Quando o mecanismo está inocentado e o usuário insiste, desconfie do DIAL.** Um slider de faixa
  fixa é usado nos EXTREMOS; se lá a feature é inerte por construção, o relato é "X não funciona" com
  todo o mecanismo de X saindo limpo de teste após teste.
- **A faixa de um controle é função do OPERANDO, não da vista.** ±4 unidades de mundo era um número
  sobre o viewport; `fração × maxdim/2` é um número sobre a forma, e por isso o curso inteiro é útil
  em qualquer escala.
- **Duas fileiras idênticas no mesmo painel para perguntas diferentes** ("Join" do traço × do offset)
  é meio caminho para "cliquei e não fez nada".
- **Meça antes de aceitar o diagnóstico do usuário** — a composição que ele descrevia não existia; o
  que existia era a *evidência visível* dela (âncoras materializadas). Corrigir o que ele descreveu
  teria sido corrigir o que já estava certo.
- **Uma âncora sobre geometria transiente é uma alça MORTA** (arrastá-la é trabalho que o próximo
  clique apaga) — mas a cura é o modelo (não materializar), não esconder o overlay.

---

## Bug #19 — "A gaiola do envelope não aparece, mas o efeito se aplica" (2026-07-24) — **FECHADO**

### Sintoma

O artista seleciona uma forma, clica **Envelope**, entra no modo **Node** — a forma deforma pela
gaiola de perspectiva (o efeito aplica), mas **a gaiola não aparece**: no lugar dos 4 cantos
arrastáveis, o que se vê são as **alças de nó da própria forma** (quadrados de âncora + pontos de
tangente), que não editam nada útil. Reportado com screenshot; *"Some depois de quê? Nem aparece."*

⚠️ **Quase foi diagnosticado como build velho.** O smoke `=27` (a estrela envelopada) renderizava a
gaiola **corretamente** — provado por três vias: um teste headless (`gizmo.selection` = container,
`is_envelope` = true, `view()` = Some, estável por 4 frames), o diagnóstico de runtime do app real
(`[vec-diag] render selection=…(container) is_envelope=true`) e uma **captura de tela** do `=27`
(trapézio + 4 círculos, estrela limpa). Foi essa contradição — smoke OK, produto quebrado — que
apontou a causa: **a ordem do smoke não era a ordem do produto**.

### Causa

As duas metades do overlay do Node dependem de o gizmo carregar o **CONTAINER** do envelope:

- as alças de nó da forma são **suprimidas** só quando `is_envelope(hero.gizmo.selection)` (a forma
  sob a gaiola é derivada — seus nós são a SAÍDA do warp, não editáveis);
- a gaiola é desenhada por `envelope_gesture::view(sim, hero.gizmo.selection, …)`, que devolve `None`
  quando a seleção não carrega um `VecEnvelope`.

Alças aparecendo **e** gaiola ausente = `gizmo.selection` era o **FILHO**, não o container.

A promoção filho→container mora no `vec_selection::sync_selection` (a regra "seleção-só-o-container"
do ADR-0129 Fatia 3, via `envelope_live::sole_container`). Mas o `sync_selection` só **reroda** essa
promoção quando **o pen muda** (`pen_now != state.paths`, branch 1) ou **o conjunto vetorial do
gizmo muda** (branch 2). E o gesto do produto — *selecionar a forma, DEPOIS clicar Envelope* —
não faz nenhum dos dois: enveloparr **re-parenteia o filho sem tocar o pen**. Sequência exata:

1. artista seleciona a forma → sync branch 1 roda, `sole_container([forma])` = `None` (ainda sem
   envelope) → `gizmo = forma`, `state.paths = [forma]`;
2. artista clica **Envelope** → `create()` re-parenteia a forma sob o container novo, **pen intacto**;
3. próximo `sync_selection`: `pen == state.paths` ⇒ branch 1 pulado; a forma ainda é "vetorial-nossa"
   ⇒ branch 2 vê o mesmo conjunto e retorna cedo ⇒ **gizmo fica na forma, para sempre**.

O `create()` **já DEVOLVE** os bits do container "para a seleção/gizmo" (o próprio doc dele diz
isso) — o `render_loop` simplesmente **descartava o valor** (`Some(_) =>` só imprimia).

### Correção

`VecSelSync::invalidate()` (limpa `state.paths`) forçando o `sync_selection` do MESMO frame a
rerodar a promoção pela branch 1 — que agora acha `sole_container([forma])` = o container recém-criado
e promove o gizmo. O `create` do `render_loop` a dispara ao ter sucesso; o smoke `=27` faz o mesmo.
Reusa a lógica de promoção que já existe (zero 2ª porta).

### Correção do SMOKE (a fixture mascarava o bug)

O `=27` criava o envelope e **só então** selecionava a forma — **tudo num frame**. Nessa ordem o
`select_many` MUDA o pen, então a branch 1 roda e a promoção acontece **por acidente**, com a gaiola
aparecendo. Passou à **ordem do produto**: seleciona no **frame 4**, envelopa no **frame 5** (dois
frames separados — é a separação que reproduz o bug). O smoke agora exercita a correção de verdade.

### Gate

`enveloping_a_selected_shape_promotes_the_gizmo_to_the_container` (`vec_selection.rs`) espelha o
gesto do produto: seleciona → sync (gizmo = forma) → `create()` → `invalidate()` → sync, e exige
`gizmo.selection == container` **e** `view().is_some()`. **Mutação-provado:** sem o `invalidate()`,
o gizmo fica no filho e o gate fica **VERMELHO** (`assertion left == right failed`).

### Lição

**A ordem de SETUP de um fixture esconde bugs dependentes de ORDEM.** O smoke montava o estado no
caminho *conveniente* (criar-depois-selecionar), que dispara a promoção por outra via; o produto usa
a ordem *inversa* (selecionar-depois-criar), onde nada a dispara. Quando o report descreve um GESTO
(*"acrescentei envelope"*), o gate/smoke tem de reproduzir a MESMA sequência, em frames separados se
o produto os separa. E o corolário do processo: quando o smoke passa e o produto falha, **não
declare "build velho" — instrumente** (headless + runtime diag + captura) até a contradição nomear a
causa. Família: [[reference_topic_fixture_discipline]].

---

## Bug #20 — O contorno TRACEJADO do Feather, e a premissa de cor que o módulo afirmava (2026-07-26) — **FECHADO**

### Sintoma

Enio, com screenshot, três rodadas: *"ainda com artefatos"* → *"um pouco melhor mas não é o efeito
real"* → *"bevel OK / o outro ruim"*. Ao longo do contorno da forma, uma **linha tracejada** clara;
no bevel, **linhas pretas** no fio da borda.

### Causa

Três causas empilhadas, e cada uma escondia a seguinte.

**(a) O PERFIL do bevel.** `1 − smoothstep(0, w, dist)` vale **1 em `dist = 0`**, ou seja punha o
valor EXTREMO do sombreado no texel mais externo. Um bevel é uma quina arredondada: a superfície
começa PLANA na silhueta, e o sombreado é a INCLINAÇÃO dela — logo se anula nas DUAS pontas. O
perfil certo é `4t(1−t)`, a derivada normalizada de um smoothstep.

**(b) O ESPAÇO de cor.** O `cs_resolve` fazia `rgb/a` sobre bytes sRGB, o que **não é** a
des-premultiplicação: é ela composta com uma transferência não-linear. E o borrão somava bytes
codificados, que é a franja escura clássica.

**(c) A CONVENÇÃO DE ALFA DA FONTE — e esta derrubou uma frase do próprio doc do módulo.** O
`fx_stack` afirmava *"premultiplicada — é o que o Vello escreve"*. Medido no rasterizador REAL:
**1696 de 1696** texels de cobertura parcial trazem a cor CHEIA `(235,175,60)` com o alfa ao lado.
Alfa **reto**. O `render_to_intermediate` nunca premultiplicou.

### Por que atravessou dezenas de gates verdes

Num texel **OPACO** e num **VAZIO** as duas convenções de alfa produzem exatamente os mesmos bytes.
Só a cobertura PARCIAL as separa — e **toda fixture de cobertura parcial do módulo foi escrita pela
mesma mão que escreveu a premissa**. Nenhum gate perguntava ao Vello.

E o (a) tinha um buraco de oráculo próprio: todos os gates de FX mediam **variação AO LONGO de uma
aresta** (ondulação, pente, dente). Um defeito **constante ao longo da aresta** — uma linha dura,
uma cor lavada — é invisível a esse oráculo. Foi o mesmo buraco duas vezes.

### Correção

Perfil `4t(1−t)` no bevel. `cs_ingest` novo faz `sRGB reto → linear → ×α`; `cs_resolve` faz o
inverso; o `tint` atravessa a porta única `tint_lin`. **O miolo fala linear premultiplicado e só as
duas pontas falam sRGB reto.**

### Gates

`fx_stack_linear_gpu.rs` (6): a cor de um texel parcialmente coberto é a cor da forma (rampa inteira
dentro de **2 níveis**) · a média de um borrão na fronteira preto/branco é **187,5**, não 128 ·
overlay e halo pintam exatamente a cor da swatch · a ida e volta devolve os 256 bytes ·
**`the_source_carries_straight_alpha_not_premultiplied`**, que interroga o Vello de verdade · e um
arch-gate que pina o `g.tint.rgb` cru dentro da única porta que o converte. Mais
`the_relief_vanishes_at_the_silhouette_and_peaks_inside_the_band` para o perfil. **6 mutações, 6
sangram.**

---

## Bug #21 — A fixture do Feather era uma CÓPIA, e ficou para trás (2026-07-26) — **FECHADO**

### Sintoma

Ao corrigir a fixture oblíqua partilhada, o gate `the_feathered_band_keeps_the_shapes_colour…`
acusou **149 níveis** de desvio de cor — **sobre um produto correto**.

### Causa

`fx_stack_feather_gpu.rs` carregava a própria `source()`: mesmos números, mesmo ângulo, outra
função. Quando a partilhada aprendeu a convenção do Vello, a cópia ficou a montar
`byte · cobertura`, uma fonte que o produto nunca produz.

### Correção

A cópia morreu; o arquivo importa `oblique_source` do `fx_stack_common`. Duas portas para *"como é
uma aresta antialiasada?"* divergem — e divergiram.

De carona: `make_src` não tinha `COPY_SRC`, então o modo analítico da sonda `fx_look_probe`
**nunca rodava** (só o `PH2D_FX_VELLO=1`), o que tornava a comparação entre rasterizadores
impossível — e era exatamente essa comparação que ia nomear a causa (c) do #20.

---

## Bug #22 — A sombra interna DESCOLAVA do contorno quando deslocada (2026-07-26) — **FECHADO**

### Sintoma

Enio: *"reveja Inner Shadow que está bugado"*. Com deslocamento, a banda escura não encosta na
borda: sobra uma **tira clara** entre o contorno e a sombra, e a sombra tem borda dura dos dois
lados — lê como recorte, não como sombra.

### Causa

Em modo Contour a força era `1 − smoothstep(0, w, dist)` com `dist` **SEM SINAL**. Um texel cujo
ponto amostrado cai FORA da forma volta a ter distância grande, então a sombra **desvanece
justamente do lado onde devia estar saturada**.

Medido numa aresta reta com deslocamento 8 (luminância por profundidade, tinta crua ≈ 180):

```
110  96  81  64  45  24   3   9  31  52  70 …
```

O ponto **mais escuro ficava 7 texels dentro**, e a borda saía **3,6× mais clara** que ele. Uma
sombra interna é mais escura NA BORDA, sempre.

⚠️ O modo **Proximity nunca teve o defeito**: ele borra uma REGIÃO (a máscara invertida), e deslocar
uma região preenchida continua a cobrir a borda. Deslocar um campo de DISTÂNCIA não.

### Correção

`sdist` — a distância **com sinal**, que o shader já computava duas linhas acima. Fora da forma ela
é negativa, o `smoothstep` satura em 0 e a força vai a 1. Perfil com o mesmo deslocamento:

```
0   0   0   0   0   0   0   9  31  52  70 …
```

⚠️ **Sem deslocamento é byte-idêntico ao anterior** (`sdist == +dist` para todo texel de dentro, e
um de fora é morto pelo `over.a` do `inner_tint`) — o defeito era só do caso deslocado.

### Gates

`fx_stack_modes_gpu.rs`: `the_inner_shadow_is_darkest_at_the_edge_even_when_offset` varre **três**
deslocamentos (o zero é metade do gate: ali as duas leis coincidem, e um gate que só o medisse
ficaria verde sobre o defeito inteiro) e
`the_offset_saturates_the_shadow_as_far_as_it_pushes` pina o que o artista controla.

⚠️ Este segundo nasceu vermelho **por culpa do oráculo**: eu esperava que um deslocamento de 16
saturasse 16 texels, e o produto deu 14. O deslocamento é um **VETOR** e a profundidade é medida
**perpendicular** à aresta — contra uma aresta oblíqua o que satura é a PROJEÇÃO,
`16 · cos(23,7°) = 14,7`. O gate afirma a projeção.

---

## Bug #23 — O halo externo não tinha a escolha que os de dentro tinham (2026-07-26) — **FECHADO**

### Sintoma

Enio: *"coloque opção em Glow como em inner glow (proximity e contour)"*.

### Causa

Não é bug de cálculo: os modos existiam e chamavam-se `INNER_MODES` porque só os degraus de dentro
os ofereciam. O nome era um **acidente de quem chegou primeiro**, não uma propriedade da escolha —
a pergunta *"perto da borda é pouco fora por perto, ou é pouca DISTÂNCIA até ela?"* é a mesma para
um halo externo. E o roteador dizia `spec.inner && Contour`, uma **enumeração disfarçada de regra**.

### Correção

`INNER_MODES` → `FALLOFF_MODES`; Glow ganha `modes`; o `plan_of` passa a perguntar *"este tipo
oferece escolha, e ele escolheu Contour?"* — o que faz o Glow entrar por construção, e o próximo
tipo com modos também. Braço novo no `cs_op_field` (banda externa com queda por distância),
`seeds_shell` passa a incluir o Glow (sem semente o modo desenharia NADA no caminho do raster) e o
`op_reach` dele passa a ser a LARGURA, não `3σ`.

⚠️ **O Glow nasce em Proximity, não no Contour do `BLANK`.** Ele SEMPRE foi a silhueta borrada;
ganhar uma opção não pode repintar o que "Add Glow" quer dizer. Um Glow salvo antes desta wave
carrega `mode = 0` — que é exatamente este —, então nenhum arquivo muda de aparência, e o
`PROJECT_SCHEMA` não se move (o campo já existia).

### O que a medição corrigiu na minha prosa

Escrevi o gate copiando o enredo do irmão de dentro (*"a reentrância quase não acende"*) **sem
medir**. Num halo EXTERNO o sinal se inverte. Medido a 3,5 texels da borda:

| sítio | Proximity | Contour |
|---|---|---|
| quina reentrante | 156 | 202 |
| aresta reta | 110 | 202 |
| quina **convexa** | **45** | 202 |

Quem fica no escuro num halo externo é a **PONTA**, não o vão — há mais silhueta perto de uma
reentrância, não menos. A lei que sobrevive à medição é exata: **à mesma distância, o Contour dá o
mesmo halo (202, 202, 202); o Proximity varia 111 níveis.**

### Gates

`the_contour_glow_is_a_function_of_distance_alone_and_proximity_is_not` (a lei + o controle + o
ponto fraco da ponta) e `the_contour_glow_reaches_its_width_and_stops`, este nos **DOIS** caminhos
do campo — sem a metade "sem geometria" a entrada do Glow no `seeds_shell` ficaria sem prova, e
uma forma com traço (que cai no raster) desenharia nada. `the_falloff_choice_is_offered_where_it_
means_something` mora na tabela, porque o seam do painel é dirigido por ela e por isso **não pode**
testemunhar que a capacidade existe. **5 mutações, 5 sangram.**

⚠️ E o gate nasceu vermelho por culpa da fixture: a minha sonda "de aresta reta" caía **DENTRO** do
braço da cruz e lia o alfa da FORMA (255) em vez do halo. O controle atropelado pelo experimento,
pela quarta vez neste projeto — agora a premissa (*as três sondas estão FORA*) é **declarada e
verificada** no próprio gate.

---

## Bug #24 — "Linhas no Bevel": o PENTE era o caminho do raster, e toda forma com traço caía nele (2026-07-27) — **FECHADO**

### Sintoma

Enio, com foto (estrela roxa, contorno branco, bevel cobrindo a forma inteira): *"problemas voltaram
a aparecer no main: Linhas no Bevel"*. Cada faceta do relevo vem hachurada por um **pente** de
linhas diagonais finas.

### Causa

Duas coisas, e só a segunda é o bug.

**(1) O pente é o caminho do RASTER, e ele é real.** O campo de distância tem duas rotas: semeado
pela GEOMETRIA (o pé exato sai de um laço sobre os segmentos da silhueta) ou pelo **JFA sobre o
raster**, quando não há geometria. O raster semeia em texels DISCRETOS ⇒ a direção salta na fronteira
de célula de Voronoi e a distância escadeia. O bevel lê a DIREÇÃO, então o erro sai como hachura.
Reproduzido lado a lado na sonda (`PH2D_FX_RASTER=1`, cena 13): a mesma estrela sai **com a hachura
da foto** pelo raster e **lisa** pela geometria.

**(2) Toda forma com TRAÇO caía no raster.** O `silhouette_segments` recusa a curva autorada de uma
forma traçada — e **com razão**: num traço centrado ela passa pelo MEIO da faixa de tinta, então
semeá-la poria a fronteira DENTRO da forma. O doc dele já nomeava o item aberto: *"a união exata é
trabalho da booleana, e entra quando houver quem a peça"*. A estrela do Enio tem contorno branco.

⚠️ **A cena de smoke não continha o fenômeno.** As dezasseis estrelas do `PH2D_BUILD_SMOKE=33`
tinham UMA traçada (a do Outline grosso), e nenhuma era traçada **e** biselada — o bug reportado não
podia aparecer em nenhuma delas.

### Correção

Três peças, uma porta cada:

1. **`ph2d_vec_boolean::silhouette_paths`** — a união `preenchimento ∪ contorno-do-traço`. Sem traço
   devolve o próprio caminho **ao bit** (zero sweeps).
2. **`shells/desktop/src/fx_silhouette.rs`** — resolve por forma, com memo. Mora na shell porque o
   `ph2d-vec-render` **não depende** da booleana (skew de kurbo, declarado no `Cargo.toml` dele) e a
   cerca é boa: o desenhista não precisa saber resolver interseção.
3. **`silhouette_segments` ganha `sil: &LiveGeometry`**, consultada PRIMEIRO. Mapa vazio =
   byte-idêntico ao mundo pré-wave.

**Medido antes de adotar** (§0): **0,19–0,31 ms** numa estrela de 5 pontas (10 âncoras) · **0,46–0,67
ms** com 12 · **1,67–2,54 ms** com 40 (80 âncoras) — linear, ~0,02 ms por âncora. O memo é chaveado
na geometria de **MUNDO**, que é função da POSE e não da câmera ⇒ acerta durante todo pan e zoom: a
união é paga por **EDIÇÃO**, não por frame (gate que **conta** os cozimentos).

### O que a medição corrigiu em mim — três vezes

**O primeiro oráculo falhou sobre produto CORRETO.** Ele exigia meia-largura de folga em TODA
âncora; num espeto o join **clampa** (miter vira bevel) e a corda passa a **0,0101** da ponta numa
estrela de 12 pontas — legítimo, e é o que o rasterizador desenha também. A pergunta certa é *"a
âncora está DENTRO?"*; a folga exata só vale longe das quinas. São **dois** gates.

**Uma mutação sobreviveu a quatro gates.** `return ink` (a tinta sem unir com o preenchimento) deixa
um **ANEL**, e o furo do anel é uma fronteira correndo pelo meio da forma — literalmente o defeito
que a união existe para apagar. Os quatro gates não a viam porque medem **perto da curva autorada**,
e ela fica dentro da faixa de tinta nos dois casos. O oráculo que faltava é o **MIOLO**.

**O instrumento do gate de custo não podia falhar.** Eu usei `ph2d_vec_boolean::__sweep_calls`, que
conta entradas em `offset_path` — caminho que a união **não percorre**. O gate media zero contra zero.
Trocado por um contador do próprio módulo, que mede a frase que ele afirma.

### E um doc-comment FALSO que eu ia shipar

Escrevi *"a normalização não é cosmética"* sobre o `regions_of` (que zera `stroke` e põe `fill`). A
mutação que a remove **sobreviveu**, e a medição diz por quê: a booleana já devolve região sem traço
e com preenchimento. Ela **fica** — o `push_path` recusa peça estilizada **em silêncio**, e a forma
voltaria ao raster com tudo verde —, mas como **cinto**, e o fato de hoje está PINADO num gate
(`the_boolean_already_hands_back_regions_and_this_normalisation_is_a_belt`).

### Gates

5 na booleana + 5 no `ph2d-vec-render` + 9 na shell, incluindo a **costura ponta a ponta** com a
estrela do report (`the_reported_stroked_star_reaches_the_exact_field_instead_of_the_raster`): sem a
silhueta resolvida ela dá **zero** segmentos, com ela dá segmentos na borda da TINTA. **9 mutações;
7 sangram, 2 sobrevivem e estão documentadas** (a normalização, acima; e o `retain` do memo, que é
higiene de memória — uma entrada órfã nunca é lida).

### 2ª rodada — `stroke = 0` (mesmo dia)

Enio: *"para stroke maior que 0 funciona. Mas para stroke = 0 linhas aparecem"*.

**A mesma doença, com a pergunta errada num lugar a mais.** O slider de Width chega a `0`, e `0`
significa **sem traço** — o `stroke_zero_tests` da `ph2d-vec-render` já o prova do lado do desenho.
Mas `stroke.is_some()` continua **verdadeiro** com largura zero, e era isso que o campo perguntava:
a forma caía no raster **sem sequer haver tinta de contorno** para justificar. Medido: com
`width = 0` o `outline_stroke` devolve **0 peças** ⇒ `silhouette_paths` devolvia `Vec::new()`, que o
chamador lê como *"não sei responder"*.

**A pergunta tinha TRÊS respostas espalhadas** e a terceira discordava das outras duas: o desenho
confiava no Vello (que não encoda nada com largura 0), a booleana devolvia tinta vazia, e o campo
perguntava pela PRESENÇA. A cura é a porta única **`StrokeSpec::lays_a_band()`**, onde a largura
mora, com os três consumidores a perguntar a ela.

⚠️ **O escopo da porta é a FAIXA, não a tinta toda** — uma ponta (seta, losango) é desenhada à parte
pelo `stroke_plan`. Quem pergunta pela silhueta de uma forma **preenchida** pode parar aí, e o
motivo é geométrico: `stroke_head` devolve `None` para caminho FECHADO (não há extremo onde pôr a
ponta). Medido, não suposto.

3 gates novos (um por consumidor), **3 mutações, 3 sangram** — cada uma no seu lugar: `push_path` a
voltar a `is_some` mata o gate do render, `silhouette_paths` a voltar a `is_none` mata o da
booleana, e `lays_a_band` a mentir mata os dois de fora.

⚠️ **E o comando de smoke que eu tinha passado estava ERRADO** — escrevi `PH2D_FX_RASTER_SMOKE=1`,
que **não é lido em lugar nenhum**; a cena é `PH2D_BUILD_SMOKE=33`. O Enio testou na cena dele e
achou os dois bugs assim mesmo, mas um comando errado num handoff é o próximo a perder uma tarde.

### Smoke

**`PH2D_BUILD_SMOKE=33`** — a estrela do **BEVEL (14)** leva **traço branco** e a do **FEATHER
(13)** leva traço de **largura ZERO**. O relevo do bevel tem de sair **liso**, sem hachura
diagonal; a rampa do feather tem de sair lisa e sem contorno visível nenhum. O bevel sem traço continua provado pelos gates e pela sonda
`fx_look_probe` (cenas 12/13), que é onde as fotos do antes/depois foram tiradas.

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

8. **A FIXTURE é parte do gate, e é a parte que ninguém audita** (Bugs #12-#14). Dezesseis
   gates verdes usavam quadrados eixo-alinhados, construídos à mão, na identidade — enquanto
   o produto entrega formas do **catálogo** (curvas), **centradas no local 0**, com a pose num
   `Transform` (ADR-0111). Mutar o código e ver vermelho **dentro de um universo de quadrados**
   só prova coisas sobre quadrados. Duas perguntas, antes de confiar numa suíte:
   *quantos gates usaram uma forma do catálogo?* e *quantos usaram um `Transform`?* Se a
   resposta for zero, a suíte não fala do seu produto.

9. **Instrumente o app antes de teorizar.** Os três bugs acima foram achados montando a cena
   do print DENTRO do app (`PH2D_BUILD_SMOKE`), dirigindo o gesto no frame de verdade e
   olhando a tela — em ~20 minutos. A hipótese principal do handoff anterior (xform stale)
   estava **errada**, e o gate novo do arranjo nasceu **verde**. Uma tarde de leitura de
   código não teria chegado lá.

10. **O DIAL pode ser o bug** (#18). Quando o mecanismo de X sai inocentado de teste após teste e o
    usuário insiste que "X não funciona", pergunte **onde o gesto natural estaciona o parâmetro** — e
    se ALI X é inerte por construção (aniquilação, fora da tela, clamp). O relato então é verdadeiro
    e o mecanismo também: o que está errado é o mapa do controle. Gate: a fração do curso do slider
    que cai em regime morto.

11. **"Eles diferem" é um oráculo fraco** (#18). O gate dizia que Round, Bevel e Miter produzem
    resultados distintos — e essa asserção **passa mesmo com o defeito** (compor bevel sobre round
    também difere). O que pega é a **identidade com um resultado FRESCO** computado fora do caminho
    sob teste. Antes de confiar num gate de diferença, pergunte: *que defeito ele deixaria passar?*

12. **Remendo sobre a premissa errada** (#18). Três correções seguidas atacaram sintomas — undo,
    âncoras, faixa — enquanto a premissa (*"o offset materializa no release"*) ficou de pé; uma delas
    ainda causou regressão. Quando o mesmo relato volta pela terceira vez com palavras diferentes, o
    que precisa mudar é **o modelo**, não mais um detalhe dele. E a pista costuma estar no
    vocabulário do usuário: *"só apply aplica definitivamente"* é uma frase sobre **quando a
    geometria nasce**, não sobre botões.

14. **Uma premissa que só a cobertura PARCIAL pode contradizer não é contradita por fixtures suas**
    (#20). O módulo afirmava *"a fonte é premultiplicada"*; num texel opaco e num vazio as duas
    convenções dão os MESMOS bytes, então só a banda macia as separa — e toda fixture de banda macia
    tinha sido escrita pela mesma mão que escreveu a premissa. **Pergunte ao produtor a montante**,
    não ao seu modelo dele: o gate que faltava renderiza com o Vello de verdade e conta os texels.

15. **Todo gate de um módulo pode medir o MESMO eixo, e o outro fica cego** (#20). Os gates de FX
    mediam *variação AO LONGO de uma aresta* — ondulação, pente, dente. Um defeito **constante ao
    longo dela** (uma linha dura, uma cor lavada) é invisível a esse oráculo, e passou duas vezes.
    Antes de confiar numa suíte, pergunte de que EIXO ela fala; o defeito seguinte costuma estar no
    outro.

16. **Copiar o enredo de um gate irmão sem medir** (#23). Escrevi *"a reentrância quase não acende"*
    para o Glow porque é o que vale no irmão de DENTRO — e num halo EXTERNO o sinal se inverte (há
    MAIS silhueta perto de uma reentrância; quem morre é a ponta convexa). A asserção passava e a
    prosa mentia. Mede-se primeiro, escreve-se depois.

17. **Suprimir um overlay globalmente para resolver o caso de UMA forma é grande demais** (#18).
    O raciocínio ("esta geometria é transiente, não a decore") estava certo; o alcance
    (`draw_overlays` inteiro, todas as formas, enquanto a janela vivesse) apagou o modo Node.
    Overlay é política por-ALVO; quando a política nasce global, o gate que falta é o que pergunta
    pelas formas que **não** são o alvo.

18. **Um caminho de fallback que ninguém consegue FOTOGRAFAR acumula bugs** (#24). O campo tem duas
    rotas e a sonda desenhava só a boa; o pente vivia na outra havia meses, e o que o revelou foi uma
    chave de ambiente de três linhas (`PH2D_FX_RASTER=1`). Quando um módulo diz *"pior, mas nunca
    trava"* sobre uma rota, pergunte como se OLHA para ela — senão *"pior"* é uma palavra sem imagem.

19. **A cena de smoke tem de conter o fenômeno, e a lista de fixtures apodrece sozinha** (#24). O
    smoke tinha dezasseis estrelas, uma traçada e uma biselada — **nenhuma as duas**. Cada wave
    acrescentou o seu caso e ninguém perguntou pelas COMBINAÇÕES; o bug reportado não podia aparecer
    ali. Ao fechar um bug, o passo final não é o gate: é pôr o caso na cena que alguém olha.
