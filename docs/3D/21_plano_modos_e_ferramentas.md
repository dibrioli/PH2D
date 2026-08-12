# Plano — OS TRÊS MODOS DE REFERÊNCIA, o Basic/Pro, e as ferramentas que faltam

*Ordem do Enio, 2026-08-12: cada tool que não é idêntica nas três referências
ganha um dropdown de três modos (**l-mode** = literatura · **b-mode** = Blender ·
**s-mode** = SculptGL); todo tool ganha dois níveis de UI (**Basic** sem ajustes
específicos, **Pro** com eles); e a lista de implementação inclui as ferramentas
que faltam do Blender que mais importam — Cloth, Mesh Filters, Cloth Filters —
evitando as menos essenciais (Face Set Paint, Color Filters). **Estado da arte,
padrão-ouro, sem pensar em custo.***

O levantamento que fundamenta cada linha daqui é o [doc 20](20_divergencias_tools.md)
(D1-D27, os negativos N1-N7, a tabela E1-E14, o catálogo por família e o
padrão-ouro). Este doc **não repete** os achados: ele decide o que fazer com eles.

---

## §0 — O achado que reordena tudo: **o s-mode já existe**

⚠️ **`crates/ph2d-sculpt3d/src/ref_kernels.rs` são 651 linhas de porte 1:1 do
SculptGL**, aritmética `f64` com armazenamento `f32`, semântica do JS onde ela
difere — **treze kernels gateados bit a bit contra o JS EXECUTANDO**
(`tests/sculptgl_parity.rs` + o oráculo de 14 casos). Eles não são um harness de
teste: `falloff`, `area_normal_with` e `area_center_with` já são a **porta única**
que o produto chama hoje.

E o cabeçalho deles nomeia a razão de o módulo existir, que é a mesma razão de
este plano ser barato:

> *o nosso motor escreve `lerp(base, target, accum)` sobre o `pre` congelado. Os
> dois **não podem** coincidir… **a estrutura do kernel é a lei**.*

E a estrutura já é um **tipo**: `GripLaw { frozen, from_live, unit_accum, additive }`.

⇒ **s-mode = uma `GripLaw` + um `ref_kernels::*`, e as duas metades existem e são
gateadas.** O que falta é a rota que as liga ao produto. Isto tira a wave mais
arriscada do plano e a transforma em fiação.

**Corolário que governa o programa inteiro:** o s-mode é o **CONTRATO DE
PARIDADE** — o gate que afirma *s-mode ≡ oráculo ao ULP* nunca pode ficar
verde por vácuo, e ele é o que torna seguro mover o default. O default é
**produto**; o s-mode é **contrato**. (O precedente é o `PH2D_FLIP_NEW_ENGINE=0`
do Flip: o rasterizador antigo continua vivo e testado como rota de bissecção
enquanto o produto usa outro motor.)

---

## §1 — A espinha: o que um MODO é

### 1.1 — `RefMode`, e o que ele governa

```rust
pub enum RefMode { S, B, L }   // SculptGL · Blender · Literatura
```

Um modo governa **duas metades**, e separá-las é o que impede a explosão
combinatória:

| metade | o que é | onde mora |
|---|---|---|
| **declarativa** — o PERFIL | a tabela de números e flags pequenas que o verbo arma: curva, força, fator de raio, sinal, `accumulate`, lado do plano, `normal_radius_factor`, front-face contínuo, `hardness`, resposta do slider | `VerbProfile`, uma tabela `const` |
| **imperativa** — o KERNEL | onde o modo escolhe uma FUNÇÃO diferente: laplaciano × HC × Taubin, deslocamento × Kelvinlets, plano × MLS, euclidiana × geodésica | um braço no `compute_target`/`GripLaw` |

⚠️ **A leitura que o doc 20 §7 já entregou:** os quatro achados mais visíveis
(D1-D4) são **tabelas**, não algoritmos. Então a metade declarativa sozinha
entrega a maior parte do que o artista vê — e ela é a que **não pode quebrar**
nada, porque `VerbProfile::S` é, por construção, o mundo de hoje.

### 1.2 — `VerbProfile`

```rust
pub struct VerbProfile {
    falloff: Falloff,          // D1 / E1
    strength: f32,             // D3 / E2
    radius_factor: f32,        // D4 / E3
    sign: Sign,                // E4
    accumulate: bool,
    plane_side: PlaneSide,     // E5  Bilateral | Up | Down
    normal_radius_factor: f32, // E11
    front_face: FrontFace,     // E12 Binary | Continuous
    hardness: f32,             // E14
    strength_curve: StrengthCurve, // E13  Linear | Squared
    direction: Direction,      // E8  AreaNormal | PointNormal | MlsNormal
    pinch_space: PinchSpace,   // E9  Tangent | World
    normal_freeze: bool,       // E10
}
```

⚠️ **Uma tabela, N consumidores** — o padrão que o painel de física instalou e
que o `rows.rs` deste painel já segue (`show:` pergunta à porta do MOTOR,
`Verb::uses_plane`, *"nunca a uma lista paralela de nomes"*). O perfil é
consultado pelo **kernel** (para computar), pelo **painel** (para pintar o valor
que o modo armou), pelo **gate de costura** e pela **varredura de modos mortos**
(§3). Quatro listas à mão derivam, e a deriva é muda.

### 1.3 — Onde o modo MORA, e a porta única

O modo é **por verbo** (o artista quer o Clay do Blender e o Smooth do SculptGL),
guardado em `Sculpt3dUi::mode_by_verb: [RefMode; N]`, e o `Brush` carrega o modo
**resolvido** do verbo corrente — exatamente como o `radius` de mundo é derivado
do `radius_px` e ninguém edita os dois.

⚠️ **Trocar de verbo ARMA o modo lembrado daquele verbo**, pela mesma porta que
hoje arma `strength` e `accumulate` (`Verb::default_*`). Hoje ela **não arma o
falloff**, e é por isso que o D1 atravessa o app inteiro: o seletor de verbo é a
porta e ela estava incompleta.

**Um estado, um gesto de massa:** o dropdown por-verbo é o **estado**; ao lado
dele um botão **"Apply to all tools"** carimba o modo corrente nos demais. Um
seletor global *e* um por-verbo seriam duas portas para o mesmo fato — a falha
que esta casa varre a cada wave. Um botão é um gesto, não uma segunda verdade.

### 1.4 — Os rótulos

Os chips leem **`S` · `B` · `L`**, e a row se chama `Reference`
(`panel.sculpt3d.ref_mode`). ⚠️ **Decisão, com o motivo:** o artista não sabe o
que é o SculptGL, e o nome de um produto de terceiro num botão é ruído que
envelhece; as letras são exatamente o vocabulário que o Enio pediu e o significado
vive no readout e no tooltip. Trocar para os nomes por extenso é uma linha de
i18n, se ele preferir.

---

## §2 — Basic × Pro

### 2.1 — A regra, em uma frase

**Basic mostra o que TODO pincel tem; Pro abre os knobs que o MODO tinha armado.**

Isto não são dois conjuntos de features — é **divulgação progressiva do mesmo
estado**, e essa escolha é o que impede duas fontes de verdade. Em Pro o artista
não ganha números novos: ele ganha **acesso** aos números que o perfil escreveu.

| | Basic (o vocabulário do SculptGL) | Pro (+ o do Blender) |
|---|---|---|
| sempre | Radius · Strength · Symmetry · Accumulate · **Reference (S/B/L)** | tudo do Basic |
| por tool | — | Falloff · Plane Offset · Plane Trim · Pinch · Hardness · Normal Radius · Auto-Smooth · Front-Face · Strength Curve · os params da ferramenta (§5) |
| filtros | o filtro inteiro (§6) | + o eixo e a curva do filtro |

⚠️ **A `Reference` fica no BASIC**, e é a única escolha "avançada" que fica: o
doc 20 mede que a curva sozinha (D1) muda o pincel em **1,08× a 1,44×** ao longo
do raio, e é *o único achado que o artista encontra sem tocar em nada*. Escondê-la
atrás de um interruptor Pro seria esconder a decisão mais consequente do app.

### 2.2 — ⚠️ O modo é um PRESET, e um preset editado tem de DIZER que foi editado

Em Pro, mexer num knob que o modo armou faz o readout ler **`B*`** (modificado).
Um rótulo que continua dizendo *"Blender"* sobre números que o artista mudou é a
ferramenta mentindo — a mesma lei do
[[feedback_a_label_must_promise_what_the_model_delivers]] que custou o *"Air
Drag"* do módulo de física. Re-escolher o modo **re-arma** (e o gesto é
explícito, então não destrói trabalho em silêncio).

⚠️ **Consequência para o gate de paridade:** ele afirma sobre `RefMode::S`
**não-modificado**. Um perfil editado sai do contrato por definição, e o gate tem
de saber disso — senão ele fica vermelho sobre um artista fazendo o trabalho dele.

### 2.3 — Onde o interruptor mora

Um chip `Basic | Pro` no topo da seção BRUSH, estado do PAINEL (não do
documento): escolher com que profundidade olhar não muda a escultura — o
precedente exato do `matcap`, que já mora no `Sculpt3dUi` e não é salvo.

---

## §3 — A MATRIZ: quantos modos cada verbo tem

⚠️ **A lei anti-controle-morto:** um chip só existe se o resultado dele **diferir
do vizinho acima do piso de paridade** (1 ULP de `f32` = `5,960e-8`). Um dropdown
com três entradas idênticas é pior que dropdown nenhum — o artista troca, nada
muda, e ele conclui que o app está quebrado.

⇒ **Cada l-mode nasce como CANDIDATO e só ganha o chip depois de MEDIDO** contra
o b-mode na mesma cena (a disciplina do CLAUDE.md §0: meça antes de declarar). A
sonda é `measure_mode_divergence.rs`, irmã da `measure_reference_divergence.rs`
que já existe.

| verbo | s-mode | b-mode | l-mode — **o mecanismo publicado** | chips |
|---|---|---|---|---|
| **Draw** | normal do PONTO · quártica · int. 1,0 | normal de ÁREA · curva · `strength²` · hardness · front-face contínuo | ⚠️ normal do ajuste **MLS/PCA** da pegada (Alexa 2003) — **o l-mode mais fraco da tabela**; se medir dentro do piso, o Draw shipa com **2** | 2 ou 3 |
| **Inflate** | normal viva, int. 0,30 | normal viva, pressão assimétrica 0,25/0,125 | **normal de curvatura média por cotangentes** (Meyer/Desbrun/Schröder/Barr 2003) | 3 |
| **Smooth** | laplaciano, **sem falloff**, `smoothTangent` OFF | Smooth + `SURFACE_SMOOTH` como pincel próprio | **HC-Laplacian** (Vollmer/Mencl/Müller, EG 1999) — o que **não encolhe** | 3 |
| **Sharpen** | — (o SculptGL não tem) | Enhance Details | **passo μ do Taubin λ\|μ** (SIGGRAPH 1995) / fairing implícito (Desbrun 1999) | 2 |
| **Flatten** | **unilateral** (negativo) | `height`/`depth`, dois números | **projeção MLS** (Alexa et al. 2003): a superfície acompanha a curvatura em vez de ser um plano | 3 |
| **Fill** | — | `PLANE` com lado | projeção MLS, lado positivo | 3 |
| **Scrape** | — | `PLANE` com lado + Multiplane | projeção MLS, lado negativo | 3 |
| **Clay** | plano + `sqrt(r²)·0,1`, bilateral no kernel | `area_co` **descartado**, âncora no cursor | MLS + offset | 3 |
| **Pinch** | tangencial em **3D** | `x_disp` + `z_disp` projetado | **Kelvinlets afins — modo *pinch*** (de Goes & James, SIGGRAPH 2017) | 3 |
| **Magnify** | oposto do Pinch | Blob | **Kelvinlets afins — modo *scale*** | 3 |
| **Crease** | Draw + pinch, força 0,75, raio ½, sinal negativo | Crease + Blob | Draw + Kelvinlets *pinch* | 3 |
| **Mask** | raio euclidiano | euclidiano + automask topológico | ⚠️ **distância GEODÉSICA pelo Heat Method** (Crane/Weischedel/Wardetzky, TOG 2013) | 3 |
| **Move** | grab congelado | Grab + `ORIGINAL_POSITION` | **Kelvinlets regularizados — *grab*** | 3 |
| **SnakeHook** | âncora que anda | Snake Hook + rake | Kelvinlets *grab* com âncora móvel | 3 |
| **Twist** | rotação em torno do eixo de vista | Rotate | **Kelvinlets — *twist*** | 3 |
| **LocalScale** | escala radial | (só via Elastic Deform) | **Kelvinlets — *scale*** | 3 |

⚠️ **A geodésica é a maior ideia da tabela e ela não é do Mask.** Um falloff
euclidiano vaza através de um vão fino — mascarar um dedo pinta o dedo de trás.
O Heat Method resolve isso para **TODOS os verbos de uma vez**, e nenhuma das
duas referências o tem (o Blender tem apenas o automask topológico, que é
conectividade e não distância). É por isso que ela é a última wave: ela toca a
pegada, que é a entrada de tudo.

---

## §4 — O que "l-mode" pode significar, e o que ele **não pode**

**A regra:** um l-mode só existe onde há um **paper com nome, ano e critério de
aceitação**. Não há l-mode "de gosto" — inventar um mecanismo e chamá-lo de
literatura seria a pior linha deste plano, porque ele viria com a autoridade que
não tem.

| paper | o que ele dá | verbos |
|---|---|---|
| Vollmer, Mencl & Müller 1999 — *Improved Laplacian Smoothing of Noisy Surface Meshes* | **HC**: laplaciano de dois passes que devolve o encolhimento | Smooth |
| Taubin 1995 — *A signal processing approach to fair surface design* | **λ\|μ**: passa-baixa sem encolher; o μ invertido **é** o Sharpen | Smooth · Sharpen |
| Desbrun, Meyer, Schröder & Barr 1999 — *Implicit Fairing…* | alisamento implícito + preservação de volume | Smooth (passo grande estável) |
| Meyer, Desbrun, Schröder & Barr 2003 — *Discrete Differential-Geometry Operators…* | laplaciano por **cotangentes**, normal de curvatura média | Inflate · o operador dos dois acima |
| Alexa et al. 2003 — *Computing and Rendering Point Set Surfaces* | **projeção MLS**: a superfície local em vez de um plano | Flatten · Fill · Scrape · Clay · (Draw) |
| **de Goes & James 2017** — *Regularized Kelvinlets* (Pixar, SIGGRAPH) | deformação **elástica analítica**: grab · twist · scale · pinch, com suporte regularizado | Move · SnakeHook · Twist · LocalScale · Pinch · Magnify · **Elastic Deform** |
| de Goes & James 2018/2019 — *Dynamic* e *Sharp Kelvinlets* | suporte limitado e resposta dinâmica | os mesmos, refinamento |
| Crane, Weischedel & Wardetzky 2013 — *Geodesics in Heat* | distância **geodésica** rápida sobre a malha | a PEGADA de todos |
| Müller et al. 2007 / Macklin, Müller & Chentanez 2016 — *PBD* / **XPBD** | tecido com rigidez independente do passo | Cloth · Cloth Filter |
| Sorkine & Alexa 2007 — *As-Rigid-As-Possible* | deformação por handles que preserva rigidez local | Pose · Boundary (l-mode) |
| Jacobson et al. 2011 — *Bounded Biharmonic Weights* | pesos de handle sem artefato | Pose (l-mode) |

---

## §5 — As FERRAMENTAS que faltam

### 5.1 — As que ENTRAM, e a ideia que cada uma traz

O doc 20 §10 fecha com a leitura que decide esta lista: as 20 ferramentas
exclusivas do Blender **não são 20 features — são quatro ideias**. A lista abaixo
é ordenada pelas ideias, não pelos nomes.

**Ideia 1 — ler o estado ORIGINAL**
1. **Draw Sharp** — o Draw sobre as posições/normais do pen-down. Vinco duro em
   vez de domo. ⚠️ Custo quase nulo: o `pre` congelado já existe (`GripLaw::frozen`).

**Ideia 2 — um dab que não é um disco**
2. **Clay Strips** — dab **retangular** com falloff parabólico no eixo do plano.
   É a ferramenta de blocagem do Blender, e a que mais muda o que se consegue
   fazer numa sessão. ⚠️ Ela obriga o falloff a receber uma **coordenada local**
   em vez de uma distância escalar — o refactor que as duas seguintes reusam.
3. **Multiplane Scrape** — **dois** planos com um ângulo: a quina em V numa passada.
4. **Clay Thumb** — plano inclinado cujo ângulo cresce ao longo do traço.
5. **Blob** — o Crease com o pinch invertido (cai de graça depois do Crease).

**Ideia 3 — estado PERSISTENTE por vértice**
6. **Layer** — demão de **altura constante**, saturante e **apagável**.
   ⚠️ Ela introduz um plano por-vértice novo (`displacement`), e a lei do repo
   para isso está escrita: *ao adicionar um plano, adicione-o ao snapshot de undo
   no MESMO commit* — o buraco que custou um bug no impasto do Painter.

   ⚠️ **E o Layer estava BARRADO por uma cerca cuja premissa morreu** — achado
   ao escrever este plano, corrigido no `brush.rs` no mesmo commit. O
   doc-comment do `Verb` dizia, e estava certo no dia em que foi escrito, que
   *"sob a lei do traço (`accum` é um ENVELOPE em `[0,1]`) o Draw já é limitado
   a um `reach` por traço… os dois colapsam"*. **Não há mais envelope:** a wave
   da paridade (2026-08-11) fez o `Grip::Stamp` **COMPOR**, e o doc da
   `GripLaw::additive` diz a frase inteira — *"nenhum grip é mais um envelope"*.
   Hoje o Draw com Accumulate **ON** empilha sem teto, e com **OFF** ele se
   auto-limita por GEOMETRIA (*"o vértice andou mais que o raio"*), que **não é**
   o teto do Layer — o dele é uma **ALTURA escolhida**, um número que não muda
   quando o artista muda o raio. ⇒ os dois deixaram de colapsar.

   *É a regra do CLAUDE.md §0 mordendo em casa: quem move o número que tornava
   algo inalcançável tem de reconferir a nota. A wave do accumulate moveu, e a
   nota sobreviveu ao fato por um dia.*

**Ideia 4 — um modelo FÍSICO em vez de um deslocamento**
7. **Elastic Deform** — **Kelvinlets** (grab · scale · twist · pinch).
   ⚠️ Aqui **b-mode ≡ l-mode** (o Blender *é* o paper) ⇒ **um chip, sem dropdown**,
   e essa coincidência é o teste de sanidade da matriz do §3.
8. **Cloth** *(pedido explícito)* — simulação local de tecido sob o pincel.
9. **Pose** — pose por cadeia com IK.
10. **Boundary** — deforma a partir de um contorno aberto.

**Puxar (baratos, sem ideia nova)**
11. **Nudge** — empurra tangencialmente na direção do traço.
12. **Thumb** — o Grab projetado no plano da vista.

**Alisar**
13. **Surface Smooth** — chega como o **l-mode do Smooth** (HC) *e* como pincel próprio.
14. **Slide Relax** — redistribui vértices **sobre** a superfície sem mudar a
    forma. ⚠️ Sem face sets ele perde a regra de fronteira do Blender — a
    **aproximação está nomeada**, não escondida.

**Filtros** *(pedido explícito)* — §6.
15. **Mesh Filter** · 16. **Cloth Filter**

### 5.2 — ⚠️ As que ficam de FORA, e o motivo de cada uma

Escrito para ninguém "completar a lista" daqui a seis meses sem saber que houve
uma decisão.

| fora | por quê |
|---|---|
| **Draw Face Sets**, Face Set Edit, Relax Face Sets, automask por face set | **ordem do Enio.** E o motivo estrutural: face set é um atributo inteiro por-face **mais uma UI inteira** (expand, extract, visibilidade) — um substrato, não uma ferramenta |
| **Color Filter**, Paint, Smear/Blur de cor | **ordem do Enio.** ⚠️ E a nota que fica: **o kernel do `Paint` já está portado** (doc 20 §10) — o que falta é UI, não motor. Ninguém precisa reimplementá-lo |
| Simplify · Displacement Eraser · Displacement Smear | dependem de **multires**, que não temos. Exclusão **estrutural**, não de gosto |
| Scene Project | precisa do resto da cena como alvo de raycast; o viewport de escultura é de um objeto |
| Draw Vector Displacement | ⚠️ **adiado, não excluído** — pede um frame tangente por vértice, e nós já temos imagens de alpha; é atraente depois do Layer |
| Topology Rake | ⚠️ **adiado** — pede o *decimate* do dyntopo, que ainda não temos (só `refine_in_sphere`) |

---

## §6 — Os FILTROS, e o precedente que os torna baratos

Um filtro **não é um pincel**: é o verbo aplicado à **malha inteira**, dirigido
por um arrasto (esquerda/direita = força e sinal), num **passo de undo**.

⚠️ **Nós já construímos exatamente isto, noutro módulo.** O *Filter Layer* do
Painter (W5b) diz, no próprio doc: *"não há kernel novo, e isso é o desenho
inteiro"* — o render já é `alvo + k·Δ(verbo)` e um traço só preenche o `amount`
andando dabs, então o filtro **preenche o `amount` direto, uniforme, e chama o
MESMO kernel**. A mesma frase vale aqui, palavra por palavra.

E a **mesma recusa** vale: o Painter recusa os verbos de PLANO no filtro de camada
porque *o alvo deles é ajustado à PEGADA, e uma camada não tem pegada*. Uma malha
inteira também não tem. ⇒ `Verb::filters_mesh()` é a **porta única** — o painel
pergunta para OFERECER, o motor pergunta para HONRAR. (E o Blender concorda: o
Mesh Filter dele também não tem "flatten".)

**Mesh Filter** — 9 tipos, todos reusando verbo existente ou de §5:
Smooth · Surface Smooth · Sharpen · Enhance Details · Inflate · Scale · Sphere ·
Random · Relax.

**Cloth Filter** — 5 tipos sobre o solver do Cloth: Gravity · Inflate · Expand ·
Pinch · Scale.

⚠️ **`Sphere` e `Random` são os dois únicos kernels realmente novos** da lista de
filtros (projetar na esfera ajustada; deslocar por ruído por-vértice) — o resto é
fiação.

---

## §7 — AS WAVES

Ordenadas por **valor entregue cedo** e por **risco contido**, não por tamanho.
Toda wave fecha com smoke próprio e não deixa knob morto atrás.

| # | wave | entrega | depende de |
|---|---|---|---|
| **W0** | ✅ **A espinha — LANDOU** (`8b207e505` + `f4677c8cd`) | `RefMode` · `VerbProfile` · a tabela `S` lida das fontes · `default_strength`/`_accumulate`/`_falloff`/`_radius_px` **delegam** · a porta única `arm_verb_defaults` arma os **quatro**. 11 gates, 4 mutações provadas | — |
| ~~**W1**~~ | ⛔ **BLOQUEADA — trocou de lugar com a W3** (ver §7.1) | o perfil `B` de DEFAULTS não é construível: o arquivo que os declara não está no clone | — |
| **W1'** | **A UI, sobre o `B` de KERNEL** | o dropdown · o *apply to all* · o chip Basic/Pro — depois que a W3 der ao `B` o que declarar | W3 |
| **W2** | **Os knobs de Pro** | as rows condicionais por tool (`show:` já existe): Hardness · Normal Radius · Plane Trim · Auto-Smooth · Front-Face · Strength Curve | W1 |
| **W3** | **Os kernels divergentes baratos** | E5 lado do plano · E11 `normal_radius_factor` · E12 front-face contínuo · E13 `strength²` · E14 hardness · E8 direção · E9 espaço do pinch · E10 normal viva | W2 |
| **W4** | **O Smooth que não encolhe** | l-mode **HC** · Taubin λ\|μ · **Surface Smooth** · **Slide Relax** · o laplaciano por **cotangentes** | W1 |
| **W5** | **Kelvinlets** | l-mode de Move/SnakeHook/Twist/LocalScale/Pinch/Magnify + o pincel **Elastic Deform** | W1 |
| **W6** | **Os dabs que não são discos** | o falloff passa a receber **coordenada local** → **Clay Strips** · **Multiplane Scrape** · **Clay Thumb** · **Draw Sharp** · **Blob** | W3 |
| **W7** | **O plano MLS** | l-mode de Flatten/Fill/Scrape/Clay (+ o candidato do Draw, medido) | W3 |
| **W8** | **Estado persistente** | **Layer** (+ o plano no snapshot de undo no MESMO commit) | W0 |
| **W9** | **Os FILTROS** | **Mesh Filter** (9) — barato porque reusa os verbos | W4 |
| **W10** | **A FÍSICA** | **Cloth** (XPBD) + **Cloth Filter** (5) | W9 |
| **W11** | **Handles** | **Pose** · **Boundary** · Nudge · Thumb | W5 |
| **W12** | **A GEODÉSICA** | Heat Method na PEGADA → l-mode de falloff para a família inteira | W6 |

⚠️ **A W0 MUDA o desenho, e a frase que eu ia escrever — *"zero mudança, `S` ≡
hoje ao bit"* — era FALSA.** Hoje o app roda o **kernel** do SculptGL (medido:
1,00× a `5,960e-8` em Draw/Clay/Fill/Scrape/Inflate — doc 20 §11.1, *"o kernel é
idêntico ao ULP; o produto não"*) com **defaults NOSSOS** (curva `Smooth`, força
0,5 em tudo). Isso não é o s-mode nem o b-mode: é um **terceiro** que ninguém
escolheu, e é a causa raiz do D1-D4.

⇒ O que a W0 entrega não é *"nada muda"*, é algo mais forte e testável: **o
default do app deixa de ser um terceiro sem nome e passa a ser uma REFERÊNCIA
nomeada.** Pela primeira vez a frase *"estamos em s-mode"* fica literalmente
verdadeira.

⚠️ **E a W0 achou uma coisa que a W1 precisa saber:** o arming da **CURVA** é
hoje **inobservável em quinze dos dezasseis verbos** — todas as tools de
geometria do SculptGL compartilham a MESMA quártica, então trocar entre duas
delas arma um valor idêntico ao que já estava. O único par que a distingue é o
`Sharpen` (perfil `None` ⇒ cai no nosso `Smooth`), e é ele que o gate usa. **No
dia em que o perfil `B` declarar a curva editável do Blender, esse arming passa
a mover a curva em toda troca de ferramenta** — o gate já existe e passa a
morder em todo verbo.

⚠️ **E o default nasce em `S`, não em `B`, por uma razão de dependência e não de
gosto:** metade do b-mode (front-face contínuo, `strength²`, hardness,
`normal_radius_factor`) é **kernel**, e ele só existe na W3. Mover o default para
`B` é uma **decisão de produto com smoke próprio**, depois da W3 — e o §0 já diz
por que ela é segura: `S` é o contrato, o default é produto.

### §7.0 — ⚠️ O `brush.cc` FOI TRAZIDO, e ele responde METADE (2026-08-12)

*Ordem do Enio: "trazer o `blenkernel/intern/brush.cc` para o clone".* Feito —
o clone é um **partial clone** (`blob:none`) com sparse-checkout, então bastou
`git sparse-checkout add` das quatro famílias (`intern/brush*`, `BKE_brush*`,
`makesdna/DNA_brush*`, `makesrna/intern/rna_brush*`). O que ele mudou:

#### ✅ DESBLOQUEADO — as nove curvas, que o doc 20 declarava INVERIFICÁVEIS

O estudo escreveu, com honestidade, que *"as fórmulas dos presets `BRUSH_CURVE_*`
não são verificáveis"* e por isso **nunca as afirmou**. Elas estão em
`brush.cc:1489-1610`, e são estas — com `u = 1 − t`, `t = d/r`:

| preset | fórmula | temos? |
|---|---|---|
| `CONSTANT` | `1` | ✅ o nosso `Constant`, **idêntico** |
| `LIN` | `u` | ❌ |
| `SHARP` | `u²` | ❌ (o nosso `Sharper` é `(1−t²)⁴`, outra curva) |
| `POW4` | `u⁴` | ❌ |
| `ROOT` | `√u` | ✅ o nosso `Root`, **idêntico** |
| `SPHERE` | `√(2u − u²)` | ✅ o nosso `Sphere` — ver a correção abaixo |
| `INVSQUARE` | `u(2 − u)` | ❌ |
| `SMOOTH` | `3u² − 2u³` — o **smoothstep** | ❌ |
| `SMOOTHER` | `u³(6u² − 15u + 10)` — o **smootherstep** de Perlin | ❌ |
| `CUSTOM` | a curva editável (`BKE_curvemapping_evaluateF`) | ❌ (o `ph2d-curve` existe) |

⚠️ **CORREÇÃO da linha do `SPHERE`, feita ao implementar.** Este parágrafo dizia
*"o `Sphere` dele é a esfera em `u`, o nosso é em `t`"* — e a álgebra desmente:
`2u − u² = u(2 − u)`, e com `u = 1 − t` isso é `(1 − t)(1 + t) = 1 − t²`. **São
a mesma curva.** Uma diferença de FORMA não é uma diferença de CURVA, e eu
comparei duas expressões sem reduzir nenhuma. ⇒ **TRÊS das nossas seis já eram
do Blender** (`Constant`, `Root`, `Sphere`) e **SEIS faltavam**, não sete.

**FEITO (mesma sessão):** as seis entraram em `Falloff` —
`Linear` · `Sharp` · `Pow4` · `InvSquare` · `Smoothstep` · `Smoother` —, o
`ALL` foi de 6 para **12** e o `SCULPT3D_FALLOFF` junto. O catálogo do painel é
agora a **UNIÃO** das duas referências: as nove analíticas do Blender + as três
nossas (`Smooth`, `Sharper`, `Plateau`, a última sendo a quártica do SculptGL).
Gate `every_blender_preset_is_the_formula_the_reference_writes` — o oráculo é a
transcrição **literal** do C em `u`, porque um expoente trocado **não** falha
nenhum gate de forma (uma `u³` no lugar de `u⁴` continua valendo 1 no centro, 0
na borda e descendo o caminho todo).

⚠️ **DUAS armadilhas de NOME, e as duas mordem calado.** O Blender rotula o
`POW4` de **"Sharper"** e o nosso `Sharper` é `(1 − t²)⁴` — a nova veste o
identificador (`Pow4`), não o rótulo. E ele rotula o `SMOOTH` de **"Smooth"**,
que aqui já é `(1 − t²)²` — daí `Smoothstep`, o nome matemático. *Duas entradas
com o mesmo rótulo são dois botões que o painel pinta lado a lado.*

⛔ **E a frase *"o `B` pode declarar curva"* estava ERRADA — ler o arquivo a
matou.** As nove são o que o artista pode **ESCOLHER**, não o que um pincel
**VESTE**: o `curve_preset` de um `Brush` zero-inicializado é
`BRUSH_CURVE_CUSTOM = 0` (`DNA_brush_enums.h:148`), e o `brush_init_data`
(`brush.cc:66`) semeia a *curvemapping* dele com `CURVE_PRESET_SMOOTH` — uma
**bézier editável, nenhuma das nove**. ⚠️ E os dois nomes se parecem de
propósito para enganar: `curve_preset` (o enum das nove) e
`BKE_brush_curve_preset()` (que escreve a `CurveMapping` e toma um
`eCurveMappingPreset`) são **famílias diferentes**. ⇒ `VerbProfile::falloff`
do modo `B` continua **`None`**, e agora por um motivo lido em vez de suposto.

#### ⛔ NÃO desbloqueado, e a razão é ESTRUTURAL — não é o trim

**`BKE_brush_sculpt_reset` não existe mais em C** (`git grep` sobre a árvore
inteira: **zero**). Desde o **Blender 4.3 os pincéis são ASSETS** — o tree traz
`assets/brushes/essentials_brushes-*.blend`, arquivos **binários**. O
`brush_defaults()` que sobrou (`brush.cc:597`) copia de `Brush brush_def = {}`,
os defaults de DNA: **um** conjunto para todos, não uma tabela por tool.

⚠️ **E nem os defaults de DNA existem como arquivo:** `git ls-files
'source/blender/makesdna/DNA_brush*'` devolve só `DNA_brush_enums.h` e
`DNA_brush_types.h` — **não há `DNA_brush_defaults.h`**, ao contrário de dezenas
de outros tipos. `Brush brush_def = {}` é literalmente **zero-inicialização**, e
é por isso que o `curve_preset` de fábrica é `0` = `CUSTOM`. A ausência do
arquivo é a mesma medição por outro ângulo.

⇒ *"a força de fábrica do Clay Strips"* **não é lida de fonte nenhuma** — ela
está dentro de um `.blend`. Isto **não é uma lacuna do nosso clone**: é onde o
Blender passou a guardar a resposta, e trazer mais arquivos não muda.

**As três saídas, agora que a pergunta está certa:** parsear o `.blend` de
assets (projeto próprio, formato DNA-tagged) · ler os **defaults de RNA**
(`rna_brush.cc`, legível: faixas e `RNA_def_property_float_default` por
propriedade — é o default do CAMPO, não do TOOL) · ou **aceitar que o `B` não
declara defaults por tool**, que é o que ele faz hoje e é honesto.

⚠️ **E a §7.1 abaixo continua VÁLIDA na conclusão e ERRADA no motivo** — ela
dizia *"o arquivo não está no clone"*, e o arquivo agora está: o que falta é
outra coisa, e está escrito acima.

### §7.2 — ✅ W3a LANDOU: o `S` deixa de ser um rótulo (2026-08-12)

O modo passou a governar a metade **IMPERATIVA** — a LEI do kernel — e não só a
tabela de defaults. `RefMode::kernel() -> KernelLaw { lateral, plane }`, derivada
**uma vez** e perguntada onde o verbo decide (três `match mode` espalhados são
três lugares onde o quarto verbo nasce sem a resposta).

| eixo | `S` | `B`/`L` | quem lê |
|---|---|---|---|
| `LateralPull` | `Direct` — o delta CRU até o centro (`Pinch.js:52-58`) | `Tangential` — projeta na tangente da área | Pinch · Magnify · o termo lateral do Crease |
| `PlaneReach` | `OneSided` — o `comp = −1` de fábrica (`Flatten.js:11,57,64`) | `Bilateral` — o `plane.cc` (Height acima, Depth abaixo) | Flatten |

**O NÚMERO, medido pelo atlas** (`measure_reference_divergence`): em `S` os
**dez** verbos de carimbo chegam ao piso do `f32`. Antes da wave, três não
chegavam — **Flatten `1,717e-3` · Crease `8,087e-4` · Pinch/Magnify `5,776e-4`**
contra `5,96e-8` nos outros sete. Não era ruído numérico: era **lei diferente**,
rodando sob um chip que dizia `S`. É a mesma doença que a W0 curou nos
DEFAULTS, agora na LEI — e é o que o Enio pediu por escrito (*"paridade
bit-idêntica"*).

⚠️ **CORREÇÃO — o `Tangential` é NOSSO, não do Blender**, e chamá-lo de `B`
seria repetir o erro que a §7.0 acabou de consertar na curva. O `pinch.cc:39-60`
monta um frame `(X ao longo do TRAÇO, Z na normal)` e devolve `x_disp + z_disp`,
com o comentário *"the Y component is removed"*: ele descarta a tangente
**perpendicular ao traço** e **guarda** a componente normal — quase o oposto do
que a nossa projeção faz. São **três** leis, não duas. Fechar a dele pede o
frame do traço dentro do `Dab`; fica **nomeada** em vez de contrabandeada num
`match` que diria `B` sem ser.

⚠️ **MUDANÇA DE PRODUTO, e ela é o ponto — não um efeito colateral.** O default
é `RefMode::S`, então **o Flatten passa a raspar** (um lado) e **o Pinch a puxar
em 3D**. Quem quiser o bilateral tem `Fill`+`Scrape` hoje e o chip `B` na W1'. A
§7 já previa esta decisão (*"mover o default é decisão de produto com smoke
próprio, depois da W3"*) — o que a wave faz é **tornar as duas posições
verdadeiras**, e o smoke julga qual delas o app deve abrir.

**Gates.** O `in_s_mode_the_stamp_verbs_reach_the_floor` (no arquivo do atlas,
porque precisa EXATAMENTE daquele harness: mesma malha, mesma pegada, mesma
polaridade de máscara, os flags de fábrica por tool) + os dois de
`verb_mode_tests.rs`, irmão novo cortado por ASSUNTO (*o que o verbo faz* × *qual
referência ele está seguindo*), cada um com o outro modo de CONTROLE.
**3 mutações, 3 sangram.**

⚠️ **E uma delas achou um gate meu que passava pelo motivo errado:** eu havia
escrito o controle *"em `B` os quatro divergem"* dentro do gate do piso, e
colapsar o `B` sobre o `S` **não o derrubava** — porque o `B` também declara
`strength²` (o E13), então o dab difere da referência **pela força** mesmo com a
lei idêntica (Flatten `7,26e-3` com a lei colapsada). *Um controle que não isola
a grandeza que diz isolar é pior que nenhum*; ele saiu, e quem prova que o chip
escolhe são os gates de geometria, onde a força está fora da conta.

⚠️ **E uma lição de EDIÇÃO, que custou um gate silenciosamente errado:** um
`str.replace(old, new, 1)` cujo `old` aparecia **duas vezes idênticas** no
arquivo pousou o `mode: RefMode::B` no gate ERRADO — que passou de qualquer
forma, porque o `plane_offset` é mode-agnóstico. *Afirmar a PRESENÇA da âncora
não basta; afirme a CONTAGEM.*

**Aberto na W3** (o resto da linha da tabela): E8 direção do Draw/Crease (a
normal do PONTO — pede um campo novo no `Dab`, vindo do shell) · E10 normal viva
do Inflate (⚠️ **as DUAS referências dizem viva e nós congelamos** — é cerca de
Chesterton COM motivo escrito, e o passo seguinte é MEDIR a deriva que ela
alega, não removê-la) · E11 `normal_radius_factor` · E12 front-face contínuo ·
E14 hardness — os três últimos **acrescentam** ao `B` em vez de restaurar
paridade, e são o que dá conteúdo ao chip na W1'.

### §7.3 — ✅ W3b: a DUREZA entra, e a cerca do Inflate ganha NÚMERO (2026-08-12)

**`Brush::hardness`** — porte literal do `apply_hardness_to_distances`
(`sculpt.cc:7549-7575`), em distância normalizada:

```text
t' = 0                          se t < hardness
t' = (t − hardness)/(1 − h)     caso contrário
```

Um **platô de peso cheio** de raio `h · r`, com o falloff inteiro espremido na
casca que sobra; em `h = 1` o dab vira **disco duro** (braço próprio no original,
porque a fórmula geral dividiria por zero).

⚠️ **Ele remapeia a distância de TODOS os consumidores da curva**, o canal de
máscara incluído — é a ordem do original (o remap roda **antes** do
`BKE_brush_calc_curve_factors`, e nenhuma curva sabe que ele existe). Aplicá-lo
só na geometria faria a máscara ler uma distância diferente da que o pincel usa
no mesmo dab.

⚠️ **`hardness` NÃO é `mask_hardness`, e os dois nomes se trocam em silêncio.**
Aquele é a forma da CURVA do canal (`(1 − t)^{2(1−h)}`, do `Masking.js`); este
reescreve a **DISTÂNCIA** que qualquer curva depois lê. Curva e distância são
perguntas diferentes; os dois coexistem.

⚠️ **O default é `0.0` e ele é o NEUTRO DO PRÓPRIO ORIGINAL** — o
`apply_hardness_to_distances` abre com `if (hardness == 0.0f) return;`. Não é um
número escolhido: é o early-out deles, e é ele que faz esta metade da wave ser
**byte-idêntica no produto de hoje**. E o valor de FÁBRICA de um pincel do
Blender **não é legível** (§7.0), então o knob nasce no neutro e o número passa
a ser do ARTISTA — nunca uma tabela inventada com o nome de outro produto.

**Gates.** Três de unidade (identidade ao bit em zero · a fórmula contra a
transcrição do C · o disco duro) + **um de PRODUTO** que mede o deslocamento na
MALHA, e é ele que carrega a wave: ⚠️ *os três de unidade passariam com a
chamada REMOVIDA do laço*. **2 mutações, 2 sangram**, cada uma no gate certo — e
a que apaga a chamada sangra **só** o de produto, que é a prova de que ele não é
redundante.

---

#### ⚠️ E10 — a cerca do Inflate foi MEDIDA, e ela está CERTA

O `stroke_target.rs` congela a normal no pen-down enquanto **as duas
referências** leem a viva, com o motivo escrito ao lado: *"um traço parado
passaria a inflar numa direção que gira sozinha"*. Isso é uma **afirmação sobre
um número que ninguém tinha medido** — o §0 manda medir antes de decidir.

`tests/measure_inflate_normal_drift.rs` (traço PARADO, raio 0,45, pior caso):

| força | 1 dab | 4 | 16 | 64 |
|---|---|---|---|---|
| 0,3 | 2,8° | 10,9° | 33,4° | **53,4°** |
| 0,6 | 5,6° | 20,3° | 45,9° | **57,4°** |
| 1,0 | 9,2° | 30,2° | 52,2° | **58,4°** |

⇒ **Meio ângulo reto.** A cerca fica, agora com o preço nomeado: ela custa
paridade, e quem quiser a lei da referência ganha um ramo de MODO — não uma
troca de default. *O miolo gira menos* (15-18°), porque ali a superfície sobe
sem inclinar tanto.

⚠️ **E a sonda nasceu com a régua errada:** ela media a distância ao centro na
posição **VIVA**, e o Inflate empurra os vértices para FORA — o conjunto
`dist < r/4` esvaziava, o `max` sobre o vazio devolvia o inicializador, e a
coluna do miolo imprimia **`0,000°`**, que se lê como *"o miolo não gira"* e
significava *"não sobrou miolo pela minha régua"*. A pegada é ancorada no
pen-down; a régua tem de ser também — e a **CONTAGEM** entrou ao lado da coluna
para um conjunto vazio nunca mais poder se disfarçar de zero.

---

#### ⚠️ E12 — a linha do doc 20 conflita DOIS consumidores (corrigido aqui)

A tabela E1-E14 diz *"o front-face é binário? nós binário · S binário · B
contínuo"*. Lendo os dois lados, os **consumidores são diferentes**:

- **o nosso** binário pesa a **ESTIMATIVA DO PLANO** (o `front` do
  `fit_plane_over`: um vértice de costas entra com peso zero na normal e no
  centro de área) — e o dab **não filtra nada**;
- **o do Blender** (`sculpt.cc:7283-7295`) faz `factors[i] *= max(dot, 0)`, ou
  seja pesa **o FATOR DE CADA VÉRTICE** do dab;
- **o do SculptGL** é o `_culling`, um **checkbox do usuário desligado de
  fábrica** em dez tools.

⇒ Portar *"contínuo"* sem escolher o consumidor mudaria QUAL coisa é pesada.

**✅ CONSTRUÍDO (mesma sessão), com o consumidor certo:** `FrontFace::Ignored`
(nós e o SculptGL) × `FrontFace::Continuous` (o Blender) é o **terceiro eixo** da
`KernelLaw`, e ele pesa **o FATOR de cada vértice** — a metade do plano fica onde
está, e é dela que o `S` depende.

⚠️ **É o único dos três eixos em que o `B` ACRESCENTA** em vez de guardar o que o
app já fazia: os outros dois nasceram preservando o produto, este liga uma lei
que ninguém tinha. Em `S` o `facing` é `1.0` e `x * 1.0` é a identidade em
IEEE-754 ⇒ **byte-idêntico**, e o gate do piso o prova (a mutação que liga o
front-face no `S` derruba a paridade junto com dez gates de unidade).

⚠️ **O SINAL, e o gate que o mede:** o `Dab::eye` aponta *do olho para a
superfície*, então um vértice de frente tem `n · eye` **NEGATIVO** — o
`max(dot, 0)` do original vira `max(−dot, 0)` aqui. Inverter o sinal dá um pincel
que só pega o que está de costas, e no MIOLO isso é indistinguível: por isso o
gate mede a **SILHUETA**.

⚠️ **E a fixture teve de ser MEDIDA para conter o fenômeno.** A pegada é uma
consulta por esfera, então uma corda `r` numa esfera unitária varre `2·asin(r/2)`
— com `r = 0,8` o pior vértice ainda olha a **47°** (cosseno `0,683`) e a lei mal
se distingue de não existir; com `1,2` são **74°** (cosseno `0,28`), e a pegada
de fato atravessa o terminador. *Um gate cujo fixture não atravessa a fronteira
que ele afirma mede o vácuo.*

**O oráculo é uma RAZÃO, não um piso:** em `B`, `deslocamento / cosseno` tem de
ser o MESMO em toda a pegada — que é a lei `factors *= max(dot, 0)` escrita como
propriedade —, e em `S` a `Constant` tem de sair constante. **3 mutações, 3
sangram** (o sinal invertido · o `B` voltando a ignorar · o `S` passando a pesar,
que derruba também o gate do piso).

### §7.4 — ✅ W1' LANDOU: o chip `Reference` está na tela (2026-08-12)

A row **`Reference`** vive logo abaixo da lista de ferramentas, no **Basic**
(§2.1: a escolha muda o pincel em `1,08×-1,44×` e a lei em `1,7e-3` — é o achado
mais consequente do estudo, e escondê-lo num interruptor Pro seria esconder a
decisão que mais importa). Ao lado dela, o botão **Apply to all tools**.

**A escolha é POR VERBO** (`Sculpt3dUi::mode_by_verb`, espelhada na cena), e o
`Brush::mode` é o **derivado** — como o `Brush::radius` é derivado do
`radius_px`. Trocar de ferramenta **re-resolve** pela porta única
`arm_verb_defaults`, que é onde os outros quatro campos já eram armados. ⚠️ E a
referência **não** passa pelo teste de *"o artista mexeu?"* dos quatro knobs: ali
não há um número a proteger, há uma tabela a consultar.

⚠️ **O chip NÃO re-arma a tabela de defaults, e isso é decisão com motivo:** os
quatro `Verb::default_*` continuam lendo `RefMode::S` explicitamente. O `B` não
declara defaults (§7.0: eles vivem num `.blend` binário), então *"armar os
defaults do B"* seria armar os NOSSOS fallbacks — trocar de referência jogaria a
força do artista para `0,5` e a curva para `Smooth` sem nada disso ser do
Blender. O chip governa **a lei do kernel e a curva de força**; os sliders ficam
onde o artista os deixou.

#### ⚠️ E o `L` NÃO é oferecido — a razão mudou durante a implementação

A primeira `declares()` era derivada (*"este modo duplica um anterior?"*) e ela
**oferecia** o `L`. O motivo é o achado: **o `L` não é uma duplicata do `B`, ele
é `B` sem o `strength²`** — o `profile_l` devolve `None`, então a `StrengthCurve`
dele cai no `Linear` do `SILENT`. Ou seja, hoje o `L` já significa alguma coisa,
e essa coisa **não é literatura**: é um acidente da tabela vazia. *Um chip que
funciona e mente sobre o próprio nome é pior que um chip ausente.*

⇒ A resposta do `L` virou uma **afirmação sobre o que foi construído**, e a
direção de falha é a segura (esquecer de virá-la deixa a feature **invisível**,
nunca errada). Dois gates a cobram: *dois modos oferecidos nunca são o mesmo* e
*o `L` não tem perfil próprio nem lei própria* — o segundo cai no dia em que
qualquer uma das duas coisas deixar de valer, e cobra a decisão.

**Seam.** Quatro gates novos: cada modo oferecido tem chip que o pega **e escreve
na tabela do verbo** · a escolha **sobrevive à troca de ferramenta** (o gate que
separa *"o modo é do pincel"* de *"o modo é da ferramenta"*) · o carimbo alcança
os dezasseis · e os chips entram na **varredura anti-widget-morto**, que pergunta
ao motor exatamente como o pintor pergunta (`offered_for`) — senão ela exigiria
um chip do `L` que o painel não desenha e ficaria vermelha sobre produto correto.
**3 mutações, 3 sangram** (os chips fora do `populate` · o evento escrevendo no
pincel em vez da tabela · a row não pintada).

⚠️ **LOC: dois cortes por assunto** — `paint/tool.rs` (*que ferramenta está na
mão, e que referência ela segue*: as duas rows respondem à MESMA pergunta em dois
níveis) tira o `body.rs` de 645 para 560.

⚠️ **E o `LITERAL-PX-OK` do `BASE_RADIUS_PX` foi ÓRFÃO por uma inserção minha:**
eu ancorei no `pub const … = 50.0;` sem o comentário de fim de linha, então a
função nova pousou **entre** a const e o marcador — que foi parar no `}` dela. O
gate pegou. *O marcador tem de estar NA linha*, e uma âncora que ignora o que vem
depois dela move o que vem depois dela.

### §7.1 — ⛔ Por que a W1 trocou de lugar com a W3 (medido em 2026-08-12)

**Os defaults de fábrica do Blender não estão no clone.** Eles vivem em
`BKE_brush_sculpt_reset`, no `blenkernel/intern/brush.cc`, e o nosso clone é um
**trim de escultura**: `source/blender/blenkernel/intern/` traz `paint.cc`,
`pbvh.cc` e a família `multires_*`, e **não traz `brush.cc`** — `grep -rl
"BKE_brush_sculpt_reset\|brush->alpha ="` sobre `source/` devolve **vazio**.

⚠️ **Escrever esses números de memória seria exatamente o que a §4 proíbe** —
*inventar um número e shipá-lo com a autoridade de uma referência que não o
declara* —, e aqui seria pior que num l-mode: o chip diria **`B`**, um nome
próprio, sobre uma tabela que o Blender nunca escreveu.

✅ **Mas a metade ALGORÍTMICA do `B` é toda legível** e não estava bloqueada —
`editors/sculpt_paint/mesh/sculpt.cc` e `mesh/brushes/*.cc` estão inteiros. O
`alpha = root_alpha * root_alpha` do `brush_strength` (**`sculpt.cc:2338-2339`**,
o E13) é lido literalmente, e sozinho ele é uma divergência **enorme e
declarativa**: num slider a meio curso o `B` deposita **0,25 contra 0,50** — o
dobro de diferença, muito acima do piso de paridade, então o chip nasce vivo
pelo critério do §3 sem depender de default nenhum.

⇒ **A ordem certa é: a W3 dá ao `B` o que ele DECLARA, e só então a UI o
oferece.** Um dropdown que ship antes disso teria um chip `B` idêntico ao `S`,
que é precisamente o controle morto que a §3 existe para impedir.

**As duas saídas para os defaults, quando o Enio quiser:** trazer o
`blenkernel/intern/brush.cc` para o trim (uma decisão sobre o clone, não sobre o
código) · ou aceitar que o `B` **não declara defaults** e que trocar para ele
muda só o que o kernel faz — o que é honesto e já é a maior parte do que
distingue os dois.

**Se for para escolher onde parar:** W0-W3 já entregam o pedido inteiro do Enio
(os três modos, o Basic/Pro, e o app deixa de esculpir mal). W4-W6 são as três
que mudam **o que o app consegue fazer**. W9-W12 é o padrão-ouro.

---

## §8 — Os gates que cada wave deve trazer

Não é cerimônia: cada linha aqui já falhou uma vez neste repo.

1. **O contrato:** `s-mode não-modificado ≡ ref_kernels ≡ o oráculo, ao ULP` —
   com **controle positivo** (uma mutação no perfil `S` tem de sangrar), senão o
   gate fica verde por vácuo no dia em que a rota mudar de nome.
2. **Nenhum chip morto:** para cada `(verbo, modo)` oferecido, existe uma cena em
   que o resultado difere do vizinho **acima do piso de paridade**. A varredura
   percorre a MATRIZ, não uma lista escrita à mão.
3. **Costura:** toda row nova nasce **pintada, registrada, viva sob o mouse e
   varrida** — o `rows.rs` já entrega isso por percorrer UMA lista; o gate é o
   que impede alguém de pintar à mão fora dela.
4. **Basic não esconde estado:** um valor editado em Pro **continua valendo** em
   Basic (o nível é divulgação, não um segundo modelo) — e o readout do modo diz
   `B*`.
5. **O verbo que arma:** trocar de verbo arma **modo + curva + força + raio +
   sinal + accumulate**, e o gate afirma os seis. (Hoje ele arma dois, e é
   exatamente esse buraco que produziu o D1.)
6. **Undo:** todo plano por-vértice novo entra no `ModelSnapshot` **no mesmo
   commit** que o cria; um filtro é **um** passo.
7. **DEBUG e RELEASE** — precedente registrado nesta linha.

---

## §9 — Os riscos, nomeados

| risco | por que ele é real | o que o contém |
|---|---|---|
| **explosão combinatória** (16 verbos × 3 modos × 2 níveis) | 96 caminhos é o que mata um plano assim | a divisão §1.1: a metade declarativa é **uma tabela** (não código) e cobre a maioria; só ~8 pares pedem função própria |
| **o default move e a paridade some** | é como se perde um oráculo de ULP, que é raro | §0: `S` é contrato, o default é produto, e o gate 1 afirma o contrato |
| **l-mode inventado** | traria a autoridade de um paper sem ter um | §4: paper com nome e ano, ou não há chip |
| **chip que não faz nada** | o artista troca, nada muda, conclui que o app quebrou | gate 2, com o piso de paridade como régua |
| **Cloth vira um projeto dentro do projeto** | um solver tem cadência, estabilidade e undo próprios | W10, atrás de tudo, e XPBD (rigidez independente do passo) é escolhido justamente para não afinar constante por cena |
| **o Slide Relax sem face sets** | a regra de fronteira do Blender depende deles | a aproximação fica **escrita** no doc-comment e no chip |

---

## §10 — O placar-alvo

| | hoje | depois |
|---|---|---|
| ferramentas | **16 verbos** | **32** (16 + 14 pincéis novos + 2 filtros com 14 tipos) |
| modos por ferramenta | 1 | **1 a 3, medidos** |
| níveis de UI | 1 | **2** |
| paridade ao ULP com o oráculo | 13 kernels | **13 kernels, num modo NOMEADO e contratado** |
| ideias do Blender que nos faltavam (doc 20 §10) | 0 de 4 | **4 de 4** |
| mecanismos publicados implementados | 1 (Surface Nets) | **8** |

⚠️ **E o padrão-ouro que continua sendo só nosso:** o ZBrush não pode ser
auditado e o Blender **não se testa contra ninguém**. Um oráculo executável com
paridade ao ULP é raro — e é exatamente ele que torna seguro **trocar de alvo**,
porque dá para adotar a curva do Blender, o HC e os Kelvinlets *sabendo qual bit
deixou de ser idêntico e qual não foi tocado*.
