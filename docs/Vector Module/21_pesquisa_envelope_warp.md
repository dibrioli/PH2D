# 21 — Pesquisa: Envelope / Puppet Warp (a wave que o `20_*` §4 mandou fazer)

> **Por que existe:** o [`20_pesquisa_ferramentas_de_artista.md`](20_pesquisa_ferramentas_de_artista.md) §4
> nomeou a armadilha do warp, listou 4 famílias (FFD/MLS/ARAP/BBW) e parou de propósito:
> ***"a família de algoritmos vale um estudo próprio. Não fazer isso de improviso."***
> Este é o estudo. Ele fecha num ADR; **não é o ADR**.
>
> Método: fan-out de agentes de pesquisa (2026-07-16), fontes primárias, cada claim com URL.
> O que não foi verificado está marcado **[NÃO-VERIFICADO]** — e continua assim.

---

## §0 — TL;DR (as 6 decisões que a pesquisa entrega prontas)

1. **A espinha é `sample + fit`, e o kurbo já a tem.** `ParamCurveFit`/`fit_to_bezpath` (kurbo 0.13,
   já na árvore) foi **desenhado para isto** — a doc do trait cita *"distortion effects such as
   perspective transform"*. §2.
2. **O deformador é ENTIDADE, em forma de CONTAINER** (o warp group do Affinity). 5 de 5
   referências; **zero** usam o modelo de parâmetro. §3.
3. **Envelope e puppet são UMA espinha e DOIS gestos** — o que troca é `warp: R2 → R2`. §4.
4. **Nada de Bézier racional.** O caminho exato existe para o projetivo e está fora do nosso
   alcance (exigiria peso por ponto de controle). §2.3.
5. **A gaiola convexa mata o horizonte** sem clipping. §2.4.
6. **O gate-mãe é invariância à subdivisão** — de graça, e vem de um bug aberto do Inkscape. §6.

---

## §1 — A armadilha, enunciada com precisão

Só transformações **afins** comutam com a avaliação de Bézier (elas preservam combinações
convexas / coordenadas baricêntricas). Logo:

```rust
for v in verts { v.anchor = warp(v.anchor); }   // ERRADO
```

produz uma curva que **não é a imagem** da curva original. Erra *quase* funcionando: forma pouco
curva parece certa; o erro aparece quando o envelope curva.

**Grau da composição.** A regra: `deg(W ∘ C) = deg(C) × grau_total(W)`. Para um tensor de bigrau
`(p,q)`, o grau total é `p+q`, logo uma cúbica sai em `3(p+q)`. Verificado em aritmética **racional
exata** (coeficientes genéricos), 3 agentes independentes, e conferido contra o exemplo publicado
do DeRose 1988:

| Mapa `W` | Imagem de uma cúbica | Exato? |
|---|---|---|
| **Afim** | cúbica | ✅ trivial — é o caso que funciona |
| **Projetivo** (homografia) | **racional de grau 3** (num. 3 / den. 3) | ✅ **mas exige PESO por ponto de controle** |
| **Bilinear** (1,1) | polinomial **grau 6** | teórico, e desnecessário (vide abaixo) |
| **Coons, lados cúbicos** | **grau 12** — *não* 18 | idem |
| **Bicúbico pleno** (3,3) | **grau 18** | idem |
| **MLS-rigid** | **nem polinomial nem racional** (pesos `1/|p−v|^{2α}` + normalização) | ❌ **impossível por construção** |

> **O refinamento do Coons (12 e não 18) é achado próprio desta wave — nenhum paper o afirma.** Um
> Coons de lados cúbicos *é* um bicúbico, mas a construção **restringe o suporte de monômios**:
> `max(a+b) = 4`, não 6 (os 12 monômios presentes foram enumerados). Logo `3 × 4 = 12`. O interior
> do Coons é **derivado** dos 12 pontos de bordo; os 4 pontos interiores que um bicúbico genérico
> tem livres são exatamente o que custa os 6 graus a mais. **Um envelope Coons é estritamente mais
> barato de fitar que um FFD bicúbico, para o mesmo bordo.**

**O MLS decide a arquitetura sozinho:** como ele não é sequer racional, composição exata está fora
de questão. E como queremos **uma** espinha para os dois gestos (§4), a espinha tem de ser
`sample + fit`. O caso projetivo exato vira uma otimização que não vale o preço (§2.3).

---

## §2 — A espinha: `ParamCurveFit` do kurbo

### 2.1 O achado

`kurbo 0.13.0`, `src/fit.rs` (lido do nosso próprio registry; **Apache-2.0 OR MIT**).
Doc do trait, textual:

> *"A major motivation is computation of offset curves... It is also intended for conversion
> between curve types (for example, piecewise Euler spiral or NURBS), **and distortion effects
> such as perspective transform**."*

Algoritmo: *"cubic Bézier curve fitting based on a quartic solver making signed area and moment
match the source curve"* — Levien, casamento de momentos (mesma família do `cubic_fit.rs` do
foundational velho, que o motor novo **não** depende).

### 2.2 O contrato

```rust
pub trait ParamCurveFit {
    fn sample_pt_tangent(&self, t: f64, sign: f64) -> CurveFitSample;  // t em [0..1]
    fn sample_pt_deriv(&self, t: f64) -> (Point, Vec2);
    fn break_cusp(&self, range: Range<f64>) -> Option<f64>;
    fn moment_integrals(&self, range: Range<f64>) -> (f64, f64, f64) { /* default */ }
}
pub fn fit_to_bezpath(source: &impl ParamCurveFit, accuracy: f64) -> BezPath;
pub fn fit_to_bezpath_opt(source: &impl ParamCurveFit, accuracy: f64) -> BezPath;
```

- **3 métodos obrigatórios.** `moment_integrals` tem default (Gauss-Legendre 16 + Green).
- **`accuracy` ≈ distância de Fréchet.** A doc é honesta: *"not a rigorous guarantee, as the error
  metric is computed approximately"*.
- **`fit_to_bezpath`** subdivide recursivamente ao meio; **`fit_to_bezpath_opt`** otimiza mais, a
  *"considerably more runtime cost"*.
- **`sign`** em `sample_pt_tangent` escolhe o lado da descontinuidade — é o que faz quina funcionar.

**Tudo que temos de fornecer** é `t -> (ponto, derivada)` da curva deformada. Isto é:
`W(C(t))` e, pela regra da cadeia, `J_W(C(t)) · C'(t)`.

### 2.2.1 O que NÃO fazemos: composição simbólica exata (e é MEDIDO, não opinião)

A composição funcional exata (DeRose 1988 / blossoming 1993 / convolução de Sánchez-Reyes 2003) é
**real e SHIPA** — o Inkscape a faz em S-basis (`compose_each`, o comentário no fonte diz literalmente
*"Here comes the magic"*). Rejeitamos mesmo assim, por três medições:

1. **Custo não é a objeção.** Uma cúbica através de um bicúbico = **896 lerps** pelo algoritmo de
   1993 (contado, não estimado). Trivial.
2. **É o MESMO problema de aproximação.** Compor→subdividir→reduzir vs amostrar `W(C(t))`→fitar dão
   erro **idêntico a 4 algarismos** (4 spans: 2.860e-03 vs 2.860e-03; 16 spans: 1.729e-05 vs
   1.729e-05). E tem de ser assim: a curva de grau 18 **é** `W(C(t))`. **Quem disser que uma das duas
   é "mais exata" está errado.** Uma redução única 18→3 dá **50,8% de erro** — inutilizável; o
   ganho vem da subdivisão, nos dois caminhos igualmente.
3. **O grau é a objeção, e é onde o Inkscape se trai.** Grau 18 = 19 pontos de controle, e não há
   onde pôr isso: nem SVG, nem PostScript, nem `kurbo::BezPath`. O Inkscape compõe exato e
   **destrói a exatidão uma chamada depois**, em `path_from_piecewise(pwd2, LPE_CONVERSION_TOLERANCE)`
   — uma constante `0.01` marcada `// FIXME: find good solution for this.`

**A ÚNICA vantagem real da composição** é um **limite de erro certificado sem amostragem** (elevar a
cúbica ao grau composto, subtrair pontos de controle, casco convexo → medido **1,13× do erro real**).
Amostrar-e-fitar só *estima*, e pode perder uma excursão entre amostras. **Não precisamos disso** — a
tolerância é do artista e o gate é de aparência (§6). Construir uma álgebra polinomial para ganhar um
limite que não vamos consumir é reimplementar a nossa própria dependência.

> **O que de fato dirige a subdivisão** (medido, e é a resposta da pergunta certa): **não é o grau
> composto — é quanto da não-linearidade do mapa a curva atravessa.** Mesma curva, mesmo grau 18:
> atravessando 100% da grade = 16 spans; 25% = 8; **5% (escala de glifo) = 2**. Bate com o único dado
> publicado (Surazhsky & Elber relatam **2** segmentos para grau 12 — porque deformam *glifos*).
> Isto é a forma prática do limite do Gain 2000 (eqn 5.2): `ε̃ ≤ ε_r + δ·(l_r/s)²`, onde **δ** = maior
> deslocamento de ponto de controle e **s** = extensão mínima de célula. **O erro é função do MAPA,
> não da curva.**

### 2.3 O que NÃO fazemos: Bézier racional

Sob homografia a imagem exata de uma cúbica **é** uma cúbica racional — o `w` da homografia por
ponto de controle *vira* o peso. Não se aproxima; muda-se de representação.

**Rejeitado.** O nosso `VecVertex` é `{anchor, in_handle, out_handle, kind, corner_radius}` — **não
tem peso**. Adicionar significaria bump de `VEC_SCENE_SCHEMA_VERSION` + ensinar peso a *render,
booleana, hit-test, bbox e gradiente*. Preço desproporcional por um único mapa da família — e o
gesto de pinos (MLS) continuaria a precisar do fit de qualquer jeito. **Uma espinha só.**

### 2.4 O horizonte (`w = 0`), e por que não vamos clipar

Só o gesto **Quad/perspectiva** é projetivo, e só ele tem linha de fuga.

- **SVG, PDF, PostScript e Cairo não têm transformação projetiva** — a linha de baixo da matriz do
  SVG é literalmente `0 0 1` ([W3C SVG 1.1 §7.4](https://www.w3.org/TR/SVG11/coords.html);
  [PDF 32000-1:2008 §8.3.3](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf)).
  O modelo 2D majoritário resolve o horizonte tornando-o **irrepresentável**.
- **Skia** é a única lib vetorial 2D que faz projetivo em Bézier de verdade: clipa contra
  `w > 1/16384` **antes** da divisão, com clipper que preserva curvas (`SkPathPriv::PerspectiveClip`).
  O comentário no fonte admite: *"Not a perfect solution"*.
- **CSS Transforms L2** é a única spec que define o caso degenerado, e nomeia o lixo: dividir por
  `w` negativo **espelha a geometria pela origem** — *"might incorrectly display this point as
  (−x, −y, −100), dividing by −1 and mirroring the box"*. Renderiza, e por isso é pior que NaN.
- **Inkscape** faz a math errada (bug aberto — §6) **mas** tem um *clamp de convexidade* nas alças
  (`overflow_perspective`, default `false`).

**A nossa escolha: o clamp de convexidade.** Uma homografia de retângulo para quadrilátero
**estritamente convexo** não consegue pôr a linha de fuga dentro do retângulo — o caso degenerado
fica **inalcançável pelo gesto**, sem clipping e sem epsilon. Gaiola não-convexa é sem sentido de
qualquer forma. **[NÃO-VERIFICADO]**: que o Inkscape tenha esse clamp *por este motivo* é inferência
do que o código faz, não intenção documentada.

---

## §3 — O modelo: ENTIDADE, em forma de CONTAINER

### 3.1 O placar

| Ferramenta | Onde mora o deformador | Artista vê como coisa? |
|---|---|---|
| **Inkscape LPE** | elemento `<inkscape:path-effect>` em `<defs>`, path aponta por id | não (só no diálogo) |
| **Blender lattice** | **objeto** Lattice na cena | **sim** — selecionável, animável, parenteável |
| **Cavalry** | **nó** Behaviour ligado ao atributo `deformers` | **sim** |
| **Affinity Designer 2** | **warp group** que CONTÉM a arte | **sim** — é a linha da árvore |
| **Illustrator** | **envelope group** que consumiu o top object | parcial (uma linha + toggle de modo) |
| **PH2D Live Corners (hoje)** | `VecVertex.corner_radius` | não |

**5 de 5 modelam como coisa separada; zero usam parâmetro.** O Inkscape *parece* a exceção e não é.

### 3.2 Por que não parâmetro — e nós já temos o recibo

**Estado autorado guardado dentro de geometria derivada é varrido pelo próximo produtor.** Não é
bug; é a definição. O repo já registrou a quebra: *"uma **Live Shape NÃO tem alça** (o `recook_into`
reescreve `verts` e varreria o raio)"* → [[feedback_works_then_silently_forgets_recook_wipes_authored_state]].
Uma gaiola é ordens de grandeza mais estado autorado que um `f32` por vértice.

Mais três razões:

- **A gaiola É geometria, e o ADR-0110 já decidiu o que geometria é** (toda forma é entidade ECS).
  Enterrar a gaiola no `VecPath` cria uma **segunda representação** que o modo Node, o marquee, o
  snapping e o undo global não enxergam — e teríamos de reimplementar tudo. Os próprios devs do
  Inkscape escreveram o contra-argumento: querem a gaiola como path real *"so you can reuse all
  node-editing stuff that is already present"*.
- **Keyframability.** A timeline liga **por entidade** (`wire_id` = hash do `Name`), com `PropKind`
  fechado. Gaiola-parâmetro é **inanimável**. Gaiola-entidade é animável no dia 1, com zero
  superfície nova. É o que o lattice do Blender compra ("parenteia num osso").
- **Custo de schema.** `corner_radius` sozinho custou `VEC_SCENE_SCHEMA_VERSION` 7→8. Gaiola de
  tamanho variável dentro do `VecPath` sob postcard **posicional** = quebra de schema a cada mudança
  de topologia. Como componente ECS é registro append-only e **não bumpa nada** (o `VecBlend`/
  `VecMorph` não bumparam).

### 3.3 Container, não lista de referências

Duas sub-formas existem: **stack de referências** (Cavalry/Blender/Inkscape) e **container/pai**
(Affinity/Illustrator). **Container**, porque reusa o que já está construído e depurado:

| Pergunta | O container responde com | Já temos |
|---|---|---|
| "edito a gaiola ou a arte?" | **seleção na Hierarquia** | árvore única (ADR-0110) |
| "como encadeio deformadores?" | **aninhar** — nesting *é* o stack | parentesco cruzado |
| "qual o z do resultado?" | projeção da árvore | `vec_entities::z_order` |
| "undo/save pegam?" | é entidade no `WorldSnapshot` | `ProjectState` |
| "como ponho arte dentro/fora?" | arrastar na árvore | drag da Hierarquia |

**O modo evapora.** O Illustrator construiu `Edit Contents / Edit Envelope`; o Inkscape construiu a
tecla `7` + alças invisíveis (e um pedido de socorro no fórum: o usuário não achava as alças). O
Affinity construiu um container e não precisou de nenhum dos dois.

### 3.4 A costura do ADR-0121 não muda

O deformador é **segundo consumidor** do `cooked()`, não substituto:

```
verts autorados → corner_live → deformador₁ → deformador₂ → … → mundo
```

É o `Piecewise<D2<SBasis>> → Piecewise<D2<SBasis>>` do Inkscape: **função pura geometria→geometria**
— é *por isso* que uma pilha de 50 efeitos compõe. Duas consequências para o ADR:

- **`Cow::Borrowed` tem de sobreviver.** Pilha vazia + raio zero = mesmo ponteiro, custo zero. Foi
  essa propriedade que permitiu ligar o `cooked()` em TODO consumidor sem mudar comportamento.
- **As alças vivem no espaço da FONTE.** O Inkscape diz textual: o knotholder é *"totally unaffected
  by the visible distorted path"*. Corolário: a gaiola **não é deformada por si mesma**; e numa
  gaiola aninhada, a de dentro **é** deformada pela de fora. É
  [[feedback_derived_coordinate_seed_must_match_sample]] de chapéu novo.

---

## §4 — Uma espinha, dois gestos

O `20_*` §4 e o handoff supuseram que envelope e puppet fossem coisas diferentes. **São o mesmo
pipeline**; o que troca é a função:

```
gesto  →  warp: R2 → R2  →  [ densificar → deformar → fit_to_bezpath ]  →  cooked
```

| Gesto | `warp` | Estado |
|---|---|---|
| **Preset** (Arc/Flag/Fish…) | mapa fechado, dirigido por `Bend %` | gerador de gaiola |
| **Quad / perspectiva** | homografia (Heckbert, forma fechada — 2 Cramer 2×2, sem sistema 8×8) | gaiola de 4 cantos convexos |
| **4 curvas de lado** | Coons | 4 paths (o Node edita de graça) |
| **Malha m×n** | Coons/bilinear por célula | grade |
| **Pinos (puppet)** | **MLS-rigid** | pinos + malha? **NÃO — sem malha** |

**O puppet NÃO precisa de malha.** Isto contradiz a suposição comum (puppet = ARAP = triangulação):
MLS-rigid é `R2→R2` puro, ~30 linhas na forma complexa, sem solver e sem fatoração. Ver §5.

**Consequência:** o preset só vale primeiro **se for gerador de gaiola** (é o que o Affinity faz:
Arc e Mesh são o mesmo warp group). Como saco de floats solto, não leva a lugar nenhum; como
gerador, Quad e 4-curvas saem quase de graça.

---

## §5 — MLS (Schaefer, McPhail & Warren, SIGGRAPH 2006)

Paper: <https://people.engr.tamu.edu/schaefer/research/mls.pdf> · DOI 10.1145/1179352.1141920

### 5.1 A forma que vamos escrever (complexa, verificada a ~1e-14)

Tratando pontos como complexos, todo o algoritmo colapsa em multiply-accumulate:

```
w_i  = 1 / |p_i − v|^(2α)                    // α = 1
p_*  = Σ w_i p_i / Σ w_i ;  q_* = Σ w_i q_i / Σ w_i
S    = Σ w_i · q̂_i · conj(p̂_i)               // complexo
μ_s  = Σ w_i · |p̂_i|²                        // real
f_s(v) = S·(v − p_*) / μ_s  + q_*            // similaridade
f_r(v) = S·(v − p_*) / |S|  + q_*            // RÍGIDO  (μ_r = |S|)
```

Nenhuma matriz 2×2 é materializada. Rígido e similaridade diferem **só no divisor**.

### 5.2 As três armadilhas (todas medidas, todas reais)

1. **A Eq. 8 do paper produz NaN.** `f_r = |v−p_*| · f⃗_r/|f⃗_r| + q_*` dá **0/0 no meio de dois
   pinos** (`v = p_*`). Isto *shipou*: é o bug das "white dots" do Jarvis73, remendado com
   interpolação por cima do buraco. A forma `f⃗_r/μ_r + q_*` é **algebricamente idêntica** (verificado,
   erro 5e-15) e o NaN não nasce. **Usar sempre esta.**
2. **1 pino = NaN, não translação.** Com um handle `p̂_1 = 0` e `q̂_1 = 0`: o resíduo é 0 para
   qualquer `M`, o minimizador não é único, tudo dá 0/0. **Um pino é o primeiro gesto de qualquer
   usuário** — caso especial explícito, ou bug no dia 1.
3. **"α maior = mais local" é FALSO.** Medido e refutado: o MLS usa os pesos só em **razões**, então
   α não tem efeito nenhum no campo distante. Um ponto a 1e5 de distância move 1,1e5 **para todo α**.
   Expor α como slider de "influência" seria bug de design ([[feedback_an_escape_that_never_helps_is_a_design_bug]]).
   **α = 1** — é o que todo mundo shipa, e é a fronteira da suavidade (o paper: `f` é suave em todo
   lado *exceto nos `p_i` quando α ≤ 1*).

### 5.3 A fraqueza real, e a mitigação que já temos

**Suporte global:** o deslocamento **cresce linearmente com a distância**; o campo distante converge
para um rígido global em torno do centroide, não para a identidade. O paper admite (§5), com o
exemplo das pernas do cavalo: partes geometricamente próximas mas topologicamente distantes sangram
uma na outra, porque o peso é distância euclidiana pura. **α não conserta** (medido: vazamento perto
de um pino *fixado* satura em ~19% do arrasto, idêntico para α = 2, 4 e 8).

**Mitigação: o container É o escopo.** O warp group contém a arte; os pinos deformam exatamente os
filhos. O mundo raster (Krita, Adobe) sofre aqui porque deforma um *plano de pixels* e tem de
inventar a fronteira; nós deformamos um *conjunto de paths* e a fronteira já é a seleção. **A
fraqueza definidora do MLS vira uma affordance que já possuímos.**

**Fold-over** ≥ ~90° de rotação de pino (medido: `det(J)` muda de sinal). O ARAP **não escapa** disso
— só o combate com a energia de rigidez.

### 5.4 Perf

Paper, Tabela 1 (3 GHz de 2006, **10.000** vértices): rígido **2,6–3,8 ms**. Arte vetorial
densificada tem 10²–10³ pontos ⇒ sub-milissegundo, single-thread. `f(v)` depende só de `v` e dos
arrays de handles (read-only) ⇒ **`par_iter().map()` sem redução, sem sincronização**.

**`A_i` não depende de `q`** — durante o arrasto, `p` e os pontos densificados estão fixos; só `q`
move. Precomputar `A_i`/`p_*`/`|v−p_*|` colapsa o frame num somatório ponderado (medido: ~2600× numa
implementação MATLAB que fez isso; a assinatura do construtor é o que força o acerto).

### 5.5 Licença e prior art

- **Krita é o único produto que comprovadamente shipa MLS** (os 3 variants; o slider "Flexibility"
  *é* o α). **GPL-2.0-or-later ⇒ comportamento, nunca código** (mesma regra do
  `reference/blender-texture-paint/`).
- `rust_mls` é **MPL-2.0** (copyleft por arquivo), f32, α travado em 1, sem precompute.
  `imgwarp-opencv` é **MIT**. **Com ~30 linhas, escrever do paper é mais barato que herdar licença.**
- **[NÃO-VERIFICADO] "Adobe Puppet = ARAP".** A página do próprio Igarashi **não menciona** Adobe,
  After Effects nem o Puppet; não há patente Adobe localizada. A Adobe usa a *frase* "as rigid as
  possible" sem citar paper. **Não repetir como fato.**
- **Nenhum produto vetorial/de animação shipa MLS.** Todo tool de animação 2D usa **LBS** (Rive
  verificado: `weight.cpp`, 4 influências). **Live2D usa lattice FFD de Bézier** — a família do
  envelope, num produto 2D shipado. MLS mora na linhagem de *edição raster*.
- **Cage do GIMP/Krita = Green Coordinates** (Lipman) — família que nem o `20_*` §4 listou.

---

## §6 — Os gates que a pesquisa entrega

### 6.1 O gate-mãe: invariância à subdivisão (de graça)

Do **[Inkscape #10547](https://gitlab.com/inkscape/inbox/-/work_items/10547)** (aberto, 2024-06-10,
Hendrik Roehm): *"LPE Perspective / Envelope is mathematically flawed... only the control points are
transformed... A check is required if the bezier segments need to be splitted."*

O repro dele é o nosso gate: **pegue uma cúbica, divida-a em duas subcurvas, deforme as duas, e o
resultado renderizado não pode mudar.** Uma transformação correta é invariante a como se subdivide a
entrada; a do Inkscape não é. Isso:

- não precisa de implementação de referência;
- não precisa de golden image;
- pega o erro de aproximação **e** o lixo do horizonte com uma asserção;
- modela **aparência**, não regra — é exatamente o que [[reference_topic_oracle_discipline]] cobra.

### 6.2 Os outros

- **Fixture CURVO, sempre.** Um polígono não exibe o problema — a `[[feedback_a_boolean_leaves_slivers_and_a_zero_area_piece_paints_a_line]]`
  já ensinou isto: os 11 gates do Build rodavam sobre polígonos e a hairline **só nasce em curva**.
- **NUNCA gate por contagem de pontos.** `assert!(pontos < N)` é gate da *regra do filtro*, não do
  artefato — fica verde pelos motivos errados, igual ao `assert!(area > 0.0)` da lasca. Gate a
  **deviação** cozido↔assado (distância máxima, nos dois sentidos) + preservação de quina.
- **Identidade = byte-idêntico.** Gaiola na pose de repouso ⇒ `Cow::Borrowed`, mesmo ponteiro.
  E o irmão de **presença** ([[feedback_absence_gate_needs_a_presence_sibling]]): "não deforma" fica
  verde num renderer que não desenha nada.
- **`is_straight`/`sub_cubic`:** a reta é `(P0,P0,P3,P3)` e a booleana **testa isso** para emitir
  `line_to`. Um envelope que move pontos de controle vai encostar nisto — decidir cedo se aresta
  reta deformada continua reta (sob afim sim; sob não-afim **não**).

---

## §7 — "Release adds points": a lição de UX

Todo mundo reclama que o envelope do Illustrator cospe pontos ao expandir. A causa **não** é só a
matemática:

- **É um dial, e está exposto:** o **Fidelity** do Illustrator — *"Increasing the Fidelity percentage
  can add more points to the distorted paths"*. A comunidade "resolve" baixando o Fidelity.
- **O amplificador que ninguém nomeia:** as **linhas de grade**. *"Adding a Mesh Point will add four
  more anchor points to the border when it intersects with the boundary."* Cada cruzamento de
  gridline **força um split**, independente de curvatura. Então a contagem não é `f(erro)`, é
  `f(erro) + f(densidade da grade × complexidade do path)`. E como até o envelope "with top object"
  é internamente malha, **todo** envelope do Illustrator paga isso. **É escolha de arquitetura, não
  lei da matemática.**
- **Falta o simplify.** Não há refit no Expand — os usuários recorrem a *abusar do Pathfinder* para
  limpar. Quando o usuário inventa uma booleana para limpar a tua saída, tu shipaste sem a etapa (c).

**O que fazemos:** (1) **não assar até pedirem** — a densidade do cozido é invisível, só vira
artefato no *Convert to Curves*; (2) **refit obrigatório no bake**, não checkbox; (3) **nunca dividir
em linha de grade** — dividir onde o **erro de curvatura** manda (célula é detalhe de *avaliação*, não
pode vazar para a topologia da saída); (4) **quina sobrevive ao round-trip** (já é invariante do repo);
(5) **Apply é prefixo-só** (regra do Inkscape: com pilha, dá para assar 1–3, nunca só o 3).

---

## §8 — As duas armadilhas para resolver no ADR, não no smoke

1. **A gaiola invisível-mas-presente entra na árvore** ⇒ cai no `RootOrder`/ponto-fixo de z que esta
   linha já pagou (empate em `u32::MAX` desempatado por `Entity::to_bits()`, que o undo TROCA), e no
   `settle_origins`-durante-gesto (a gaiola sob arrasto é *path em gesto* — tem de entrar na lista de
   ignorados, senão foge do cursor, igual à caneta).
2. **Regra de um-input-só (Cavalry):** *"An animation curve (keyframes) is considered an input so
   overwriting the connection with another input will replace the animation curve meaning any
   keyframe data will be lost."* A Cavalry é a única referência que tem timeline **e** grafo de nós —
   a mesma colisão que o PH2D tem — e a resposta dela é: último a escrever ganha, e os keyframes
   somem. A entidade-gaiola é onde a nossa timeline e o cook do Motion se encontram pela primeira vez.

---

## §9 — Nota de escopo: o Deform do Painter (`docs/Deform/`)

**Existe um módulo de deformação landado que não é este.** `docs/Deform/` = Transform + Liquify +
Puppet do **Painter**, sobre **pixels**:

- **Wave 1 (Reshape)** e **Wave 2 (Transform)** landaram: Push/Twist/Pinch/Wrinkle/Fold/Reconstruct;
  Uniform/Free/**Distort (homografia)**/**Warp (grade 4×4, homografia por célula)**.
- Kernel **inverse-warp** (gather): `out[dst] = sample(dst − D(dst))`, `warp/apply.rs`,
  campos em `warp/field.rs`, math em `warp/transform_geom.rs` (`homography_from_quads`), `f32`.
- **MLS/puppet do plano dele (§2.1) NUNCA foi construído** — verificado: `MLS`, `puppet` e `ARAP` não
  têm uma linha no repo.

**A relação com este documento:** raster quer o mapa **inverso** (gather de pixel); vetor quer o
**direto** (mapear a curva). São problemas diferentes, mas a **família de campos é a mesma**. Já
existem **duas** homografias em `f32` (o nó `four-point-warp` e o `transform_geom` do Painter); uma
terceira em `f64` para o vetor é justificável (precisão e domínio diferentes), mas **um segundo MLS
seria duas portas para a mesma pergunta**. Se o puppet vetorial e o puppet raster forem construídos,
o campo devia nascer numa crate isolada compartilhada. **Isto é decisão do Enio** — a linha Painter
está viva e o seu `warp/` é território dela; esta linha **não** o refatora.

---

## §10 — Fontes

**Primárias lidas**
- kurbo 0.13.0 `src/fit.rs` (registry local) — `ParamCurveFit`, `fit_to_bezpath`
- Schaefer, McPhail & Warren 2006 — <https://people.engr.tamu.edu/schaefer/research/mls.pdf>
- Igarashi, Moscovich & Hughes 2005, ARAP — <https://dl.acm.org/doi/10.1145/1186822.1073323>
- Sederberg & Parry 1986, FFD (SIGGRAPH) · Heckbert 1989/1994, *Fundamentals of Texture Mapping and
  Image Warping* — <https://www.cs.cmu.edu/~ph/texfund/texfund.pdf>
- Skia `SkPath.cpp` / `SkPathPriv.h` — `PerspectiveClip`, `kW0PlaneDistance = 1/16384`
- CSS Transforms L2 — <https://www.w3.org/TR/css-transforms-2/#processing-of-perspective-transformed-boxes>
- W3C SVG 1.1 §7.4 · PDF 32000-1:2008 §8.3.3 · OpenGL 4.6 §13.7
- Inkscape #10547 — <https://gitlab.com/inkscape/inbox/-/work_items/10547>
- Inkscape LPE (comportamento; **GPL — nunca código**) · Krita `kis_warptransform_worker` (**GPL — idem**)
- Affinity Designer 2 — <https://affinity.help/designer2ipad/en-US.lproj/pages/ObjectControl/warp.html>
- Cavalry — <https://cavalry.studio/docs/nodes/behaviours/lattice/> · Blender lattice/hook manual
- AE — <https://helpx.adobe.com/after-effects/using/animating-puppet-tools.html>

**Marcados [NÃO-VERIFICADO]:** Adobe Puppet = ARAP · "presets são o envelope mais usado" (sem dado;
só sinais estruturais) · "Make with Top Object converte para malha" (fórum) · intenção documentada do
clamp de convexidade do Inkscape · citações verbatim da Adobe (relay de índice de busca; helpx deu
timeout) · Appendix B do MLS (integrais de segmento — OCR não confiável; **ler do paper se
implementar**).

**Beco sem saída, registrado para ninguém repetir:** os *mapping modes* do CorelDRAW
(Original/Putty/Horizontal/Vertical) **não estão em nenhum formato de arquivo**. Uma sub-wave foi
atrás deles no **CMX** e provou um negativo caro: CMX é *metafile de intercâmbio*, **assa** o envelope
em geometria, não tem comando de envelope, e o `MappingMode` dele é mapeamento de coordenadas estilo
GDI (`BOOL` + rect origem + rect destino) — homônimo, sem relação. Os modos do Corel são feature de
**autoria**, documentada só na doc de usuário.
