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

### §7.5 — ✅ W2 LANDOU: `Basic` × `Pro`, e a DUREZA ganha porta (2026-08-12)

O interruptor da §2, mais os quatro controles que ele revela.

**O que o artista vê.** Um par `Basic` · `Pro` no TOPO da seção do pincel, sob o
rótulo **Detail**. Em Basic ele vê **Radius · Strength** (mais a Hardness da
máscara, as pistas do padrão e os dois números do extract); em Pro aparecem
**Falloff · Plane Offset · Pinch · Hardness**.

**A regra de quem pode ser Pro, e ela é a §2.1 lida ao pé da letra.** Só uma row
cujo valor **alguém ARMOU**:

| row | por que é Pro | por que não amputa |
|---|---|---|
| **Falloff** | o `arm_verb_defaults` o escolhe por verbo (`VerbProfile::falloff`) | em Basic o artista está com a curva da REFERÊNCIA, não com um vazio |
| **Plane Offset** | — | os quatro verbos de plano rodam na referência EXATA com ele em zero; o Clay levanta o plano dele no KERNEL |
| **Pinch** | `Brush::default().pinch = 0.5` | o Crease aperta desde o primeiro traço |
| **Hardness** | `0` é o NEUTRO do próprio original | esconder um neutro não tira capacidade nenhuma |

⚠️ **E o que NÃO pode ser Pro tem gate:** `the_basic_level_never_hides_the_two_knobs_every_brush_has`
varre os dezasseis verbos exigindo Raio e Força. Esconder um knob que alguém
armou é divulgação progressiva; esconder os dois que todo pincel tem é
amputação, e a diferença é testável.

**O default é `Basic`, e a razão é a mesma do `RefMode::default() == S`:** a tese
deste módulo é que a referência do SculptGL é a linha de base sã, e um painel que
abre mostrando mais knobs do que a referência que o kernel roda é o painel
discordando do motor. O chip está no topo da própria seção, a um clique e
nomeando-se.

⚠️ **O interruptor governa a seção do PINCEL e nada mais** (§2.3): sombreamento e
topologia descrevem *como a forma é lida* e *quão fino é o barro*, e nenhum dos
dois é um knob que o verbo armou.

**A DUREZA ganhou porta.** O `Brush::hardness` nasceu no kernel na W3b, gateado
dos dois lados e medido — e **sem nenhum controle**. Um campo sem porta é uma
capacidade que ninguém tem.

**Gates: 5 novos, 6 mutações, 6 sangram.**

- `the_detail_chip_discloses_and_never_decides` — a propriedade load-bearing:
  trocar de nível deixa todo o resto do estado autorado byte a byte onde estava.
  ⚠️ **Ele nasceu testando UMA direção e a mutação passou:** *esconder* é o gesto
  em que o reflexo de zerar aparece, então subir de Basic para Pro não o vê
  ([[feedback_layered_defenses_need_per_layer_gates]]).
- `a_pro_row_is_reachable_in_pro_and_absent_in_basic` — as duas metades, e
  ⚠️ **DUAS fixtures**, porque `plane_offset` e `pinch` **se excluem por
  desenho** (verbo de plano × Crease): uma fixture só nunca varreria as três.
- `the_basic_level_never_hides_the_two_knobs_every_brush_has` · `every_ui_level_has_a_chip_that_selects_it` · `the_hardness_row_writes_the_field_the_kernel_reads`.

⚠️ **E DOIS gates existentes tiveram de declarar a premissa nova**, senão ficavam
verdes pelo motivo errado: a varredura anti-widget-morto passa a rodar em **Pro**
(em Basic ela pularia quatro controles que nunca seriam clicados — *a fixture tem
de conter o fenômeno*, a quarta vez neste arquivo) e o
`a_conditional_row_is_absent_with_the_wrong_tool` fixa **Pro nas DUAS metades**,
porque em Basic a metade negativa passaria pelo NÍVEL em vez de pelo verbo — um
gate que não pode falhar pela razão que alega.

⚠️ **A chave i18n `panel.sculpt3d.level` JÁ EXISTIA** (é o readout da multires) e
o compilador a pegou como braço inalcançável — a terceira coisa deste painel
disputando a palavra *nível*, exatamente o que o doc do id `SCULPT3D_UI_LEVEL` já
prevenia. Ficou `panel.sculpt3d.ui_level`.

⚠️ **E uma substituição por âncora foi RECUSADA pelo próprio `assert count == 1`:**
`tr("panel.sculpt3d.level")` aparece **duas** vezes no `body.rs`, e a outra é o
readout de multires que eu teria renomeado por engano.

**Três tetos de LOC, três cortes por ASSUNTO** — `paint/brush.rs` (*a cabeça e a
cauda da seção do pincel*, irmão do `mask_tools.rs`) · `rows_shading.rs` (*os
knobs da LEITURA da forma*) · e o `group_chip_ui` extraído do `apply_event`
(**seis grupos, uma porta** — o irmão exato do `table_intent` e do
`arm_alpha_chip`). ⚠️ O primeiro corte por INTERVALO levou junto o `paint_bake`,
que é outra seção e só estava no meio do arquivo.

**Fora desta wave, com o preço:** `Normal Radius` (E11) precisa de uma **segunda
consulta ao octree** quando o fator passa de 1 — o plano teria de recolher
vértices FORA da pegada do dab, mudança estrutural, não um multiplicador · `Plane
Trim` foi lido no clone e **o Blender o APOSENTOU** no pincel `PLANE` unificado
(`properties_paint_common.py:887`: `if sculpt_brush_type != 'PLANE'`), que o
trocou por **`height`/`depth`** — dois números que são a versão CONTÍNUA do nosso
`PlaneReach` binário, e por isso a próxima wave é essa, não o trim · `Front-Face`
e `Strength Curve` **não ganham row de propósito**: o MODO já os responde, e um
segundo controle para a mesma pergunta é a falha de duas-portas que este módulo
varre a cada wave.

### §7.6 — 📐 A W4 tem alvo, e o PAPER mudou: **Taubin, não HC** (medido em 2026-08-12)

A W4 abriu pela §0 — medir antes de construir —, e as duas medições estão em
`tests/measure_smooth_shrinkage.rs` (`--ignored --nocapture`).

**O DEFEITO, pela porta do produto** (esfera unitária, dab cobrindo a malha
inteira, `Falloff::Constant`, força 1 — o *Filter Layer*, que é onde o
encolhimento é o efeito e não um detalhe de borda):

| passadas | raio médio | encolhimento |
|---|---|---|
| 0 | 1,000000 | 0,00% |
| 1 | 0,999092 | **0,09%** |
| 10 | 0,990924 | 0,91% |
| 20 | 0,981938 | 1,81% |
| 40 | 0,964246 | **3,58%** |

⚠️ **O número não é o achado — a FORMA é.** É **linear e não satura**: o
laplaciano umbrella (`lerp(p, média do anel, w)`, `stroke_target.rs:225`) contrai
o volume a cada aplicação, para sempre. *Alisar até ficar liso é alisar até
sumir* — a objeção com que o Vollmer/Mencl/Müller 1999 abre.

**A CURA, medida ANTES de ser construída** — 20 pares λ|μ, ou seja os MESMOS 40
passos laplacianos:

| λ / μ | raio final | desvio | pior |
|---|---|---|---|
| **0,33 / 0,34** | 1,000183 | **−0,018%** | 0,018% |
| 0,50 / 0,53 | 1,000546 | −0,055% | 0,055% |
| 0,60 / 0,64 | 1,000727 | −0,073% | 0,073% |

**200× menos** — e ⚠️ **o *"LIMITADO"* que esta linha dizia era MEU e está
REFUTADO pela §7.7**: eu deduzi um teto do SINAL ter invertido, e a medição pela
porta do produto mostra as duas colunas **lineares no número de dabs**. O que
muda é a inclinação (~87×), não a existência da deriva. *Uma inversão de sinal
não é um teto.*

⚠️ **E a escolha do paper muda em relação ao §3, com motivo.** A tabela do §3
dava ao Smooth o **HC** (Vollmer/Mencl/Müller 1999) e listava o Taubin no §4
como alternativa. O que a medição e a leitura do nosso kernel dizem:

- **as duas metades do Taubin JÁ SÃO verbos deste motor** — o passo `λ` é o
  Smooth e o passo `μ` é o Sharpen (*"o laplaciano com o sinal trocado"*,
  `stroke_target.rs:234`). É *uma lei, dois consumidores*, a doutrina desta casa
  — ⚠️ e o *"o l-mode do Sharpen sai no mesmo movimento"* que esta linha dizia
  **não foi construído, com motivo**: ver a §7.7. O par λ|μ descreve um
  passa-baixa que não encolhe, e o paper não fala de afiar;
- **o HC pede estado que este motor não tem**: um vetor `b` POR VÉRTICE e uma
  SEGUNDA passada sobre o anel para a média dele. O `compute_target` é uma função
  pura por-vértice, então o HC exige um buffer de pegada no molde do
  `render_inflate` do Painter — estrutural, não um kernel a mais;
- **o Taubin não precisa do ORIGINAL.** O `o` do HC teria de ser a pose do
  pen-down, e num traço longo ele puxaria de volta para ela — brigando com o
  artista que QUER que o alisamento acumule.

⇒ **O l-mode do Smooth é Taubin λ|μ**, e o HC fica nomeado como a alternativa que
foi medida contra ele em vez de esquecida.

**O que a implementação pede, e é a única parte estrutural:** *um dab = um PAR
λ|μ*, senão um traço de N dabs são N passos `λ` sem nenhum `μ` — e aí o l-mode
encolhe igual. A porta é uma só (`Brush::passes()`, quantos passes este pincel
faz e com que peso), e ⚠️ **todo pincel que não é o l-mode do Smooth devolve
EXATAMENTE um passe — ele próprio**, o que torna o resto do motor byte-idêntico
por construção e é o que o gate afirma.

⚠️ **E é isto que dá ao chip `L` o primeiro conteúdo dele.** O §7.4 o retém com
a razão escrita (*"o `L` é `B` sem o `strength²` — um acidente da tabela vazia"*);
com o Taubin ele passa a declarar uma lei com paper, ano e critério, e o
`RefMode::declares` deixa de ser universal para ser **por verbo**.

### §7.7 — ✅ W4 LANDOU: o chip `L` ganha conteúdo, e o Smooth para de encolher (2026-08-12)

O par λ|μ do **Taubin 1995** é o primeiro paper portado, e com ele o `L` deixa
de ser retido: ele passa a ser oferecido **exatamente** onde declara uma lei.

**O RESULTADO, pela porta do PRODUTO** — o mesmo gesto nos dois modos, medido na
mesma corrida (`tests/taubin_pair.rs::the_numbers_the_gates_assert`):

| dabs | `S` | `L` |
|---|---|---|
| 1 | 0,0908% | **−0,0011%** |
| 10 | 0,9076% | −0,0104% |
| 20 | 1,8062% | −0,0206% |
| 40 | 3,5754% | −0,0409% |

⚠️ **A INCLINAÇÃO cai ~87×, e o resíduo NÃO É LIMITADO — a §7.6 dizia que sim e
era uma dedução minha.** Eu li o teto no SINAL ter invertido (o `μ` sobre-corrige
e a esfera CRESCE em vez de encolher); as duas colunas são **lineares no número
de dabs**, e o que muda é a taxa — `0,0894%/dab` contra `0,00102%/dab`. Alisar
para sempre ainda deriva: 87 vezes mais devagar, e para o outro lado.

⚠️ **E o par NÃO foi afinado para zerar a coluna nesta esfera.** O `k_PB = 0,1` é
o do paper e o `μ` é **DERIVADO** dele (`1/λ + 1/μ = k_PB` ⇒ −0,341262); um `μ`
ajustado até a coluna zerar seria um número ajustado a UMA fixture, que passa a
mentir na malha seguinte.

**A ÚNICA PARTE ESTRUTURAL — `Brush::passes()`** (`brush_pass.rs`), a porta única
de *quantas vezes o laço do dab roda e com que fator*. Três propriedades:

- **um dab é um PAR** — se o λ e o μ se alternassem por DAB, um gesto de um
  clique (o *Filter Layer*) seria `λ` puro e encolheria como o `S` com um terço
  da força. O gate mede o gesto **mais curto** que existe, que é onde as duas
  leituras mais divergem;
- **o passe 0 define o CONJUNTO; os seguintes percorrem ELE** — não é
  conveniência: o `μ` existe para desfazer a contração do `λ`, então tem de
  alcançar exatamente quem o `λ` moveu. É também o que mantém a janela publicada
  (o que a GPU re-lê) sendo um SUPERCONJUNTO do que foi escrito;
- **todo pincel que não é o `L` do Smooth devolve EXATAMENTE um passe — ele
  próprio, fator `1.0`** ⇒ o resto do motor é **byte-idêntico por construção**
  (`x * 1.0 == x` no IEEE-754). A prova de que o plumbing não moveu nada são os
  **166 gates** da crate, que passam sem uma linha de fixture mudada.

⚠️ **O FATOR DO PASSE entra depois dos dois guards**, e a ordem é o que mantém a
byte-identidade: o `w <= 0.0` pergunta *"este dab tem alguma coisa a dar?"*, uma
grandeza SEM sinal — aplicado antes, o passe `μ` seria pulado em toda parte e o
par nunca rodaria (mutação M4, sangra).

**O `RefMode::declares` deixou de ser universal e passou a ser POR VERBO.**

⚠️ **Ele é ESCRITO e não derivado de `passes().len() > 1`, e eu escrevi a
derivada primeiro:** ela é elegante, casa hoje, e é **falsa em geral** — o
Kelvinlets da W5 é um campo de deslocamento de **um** passe, e sob a derivada o
chip dele nasceria mudo com todos os gates verdes. *Fazer dois passes* e
*declarar uma lei* são perguntas diferentes que hoje têm a mesma resposta; o gate
`the_literature_mode_is_offered_exactly_where_it_declares_a_law` pina a
coincidência, e o dia em que ela cair é o dia em que alguém tem de vir aqui
decidir.

⚠️ **O SHARPEN fica FORA, revogando o que a §7.6 previa.** Ela dizia *"o l-mode
do Sharpen sai no mesmo movimento"*; o par descreve um passa-baixa que **não
encolhe**, e o Taubin não diz nada sobre afiar. Dar-lhe o par seria pôr o nome de
uma fonte numa lei que ela não declara — o chip mentiroso que o `declares` existe
para impedir.

⚠️ **A `KernelLaw` do `L` deixou de ser FALLBACK à do `B`.** Ela declara
`FrontFace::Ignored` — o Taubin é um filtro de MALHA e não sabe onde está a
câmera —, e é isso que faz `S → L` mudar **uma** coisa só: o par. Herdar o
`Continuous` do `B` faria o chip mudar duas coisas de uma vez, e o artista não
teria como separá-las na tela. Os outros dois eixos (`lateral`, `plane`) são
**inalcançáveis por construção** e trazem os valores do `S`; o gate da
bi-implicação é o que torna impossível passar em silêncio no dia em que o `L`
declarar um verbo de plano.

⚠️ **E a porta "aplicar a todos" era a ÚNICA capaz de pôr um modo onde ele não
tem lei.** Enquanto os três modos respondiam por todo verbo ela era um `fill` e
ninguém notava; com o `L` declarando só o Smooth, carimbá-lo em todos deixaria
quinze verbos rodando uma lei de literatura que não fala deles — **com o chip a
mostrar `S`**, porque o painel pinta os OFERECIDOS. Ela agora só alcança quem
declara, e onde não alcança **PRESERVA** em vez de repor um default.

⚠️ **UMA DEFESA INERTE, dita em vez de afirmada ao contrário.** O
`refresh_region` roda **por passe**, e o comentário que eu escrevi primeiro dizia
que *"o passe seguinte relaxaria contra a vizinhança de antes"* — FALSO: a média
do anel lê `mesh.positions()`, que o `apply_positions` já escreveu. Quem o
refresh conserta são as NORMAIS, e o par não as lê (o `L` declara `Ignored`, então
`facing` vale `1.0` exato nos dois passes). Ela fica porque o dia em que um modo
declarar `Continuous` **e** mais de um passe, o passe seguinte pesaria pela normal
de antes do anterior — um erro que não falha, escurece um anel na borda do pincel
e ninguém sabe por quê.

**O PREÇO, medido:** `0,848 → 1,466 ms/dab` na esfera de 8192 vértices
(`what_the_pair_costs`) — **1,73× e não 2×**, porque a consulta da pegada, a
captura e o ajuste do plano acontecem UMA vez por dab. Fica com folga sob o kill
K1 do ADR-0150.

**Gates: 4 novos em `tests/taubin_pair.rs` + 2 reescritos em `ref_mode_tests.rs`
+ 1 de seam. 6 mutações, 6 sangram** (o par vira um passe ⇒ 6 gates · o passe
único deixa de ser identidade ⇒ 4 gates EXISTENTES do motor · passes posteriores
empurram para a janela · o fator antes do guard · o `L` declara tudo · o carimbo
volta a ser `fill`).

⚠️ **E o gate da duplicata tinha um discriminante FALSO que eu escrevi primeiro:**
ele comparava `verb.profile(a) != verb.profile(b)`, e o `VerbProfile` carrega
quatro campos que **nenhum caminho lê pelo modo corrente** — os defaults são
sempre armados da coluna `S`. `S` e `L` no Smooth têm perfis diferentes, e essa
diferença **não move um vértice**: o gate teria ficado verde por um discriminante
que não é o da feature. Ele passou a comparar a **assinatura observável**
(`KernelLaw` · curva de força · passes).

**LOC:** `stroke.rs` cruzou 700 (754) ⇒ corte por ASSUNTO em `stroke_windows.rs`
(*o que um traço FAZ* × *o que se PERGUNTA a ele* — as sete janelas publicadas),
671 + 104. **Filho e não irmão**, pela mesma conta do `stroke_apply.rs`: elas leem
os planos privados, e um irmão os obrigaria a virar `pub(crate)` — a visibilidade
viraria função do TAMANHO do arquivo.

**Sem schema, sem ADR, sem crate nova, sem dep nova, sem id novo.**

⚠️ **PENDENTE DE SMOKE.** A pergunta é de OLHO e tem controle: pegue o **Smooth**,
alterne o chip **S ↔ L** e alise a mesma região muitas vezes. No `S` a forma
**encolhe** sob o pincel; no `L` ela alisa e **fica onde está**. O `L` não aparece
em nenhuma outra ferramenta — se aparecer, pare.

### §7.8 — ✅ O WIREFRAME REMOVE LINHA ESCONDIDA (2026-08-12, 2º report)

Report do Enio, com foto de uma esfera: *"ainda ruim. veja as bordas"* — a
borda saía numa faixa escura e embolada.

**A medição por ANEL desarmou a hipótese fácil antes de qualquer código**
(`probe_wire_continuity::where_the_wire_ink_falls`). Na esfera 64×128, **59 % de
toda a tinta de wireframe cai nos dois anéis externos** (`u > 0,8`), que são 36 %
da área: os anéis de latitude de uma esfera UV **comprimem-se na silhueta**, e a
faixa escurece por geometria. Curar o vazamento inteiro deixaria aquela banda com
~92 % da densidade. ⇒ *a faixa não é um defeito de profundidade.*

**Mas o vazamento existia e era grande onde a malha é grossa** — 25,7 % da tinta
do anel `0,8-0,9` na esfera 32×64, 14,0 % num toro — e ele **é** um defeito: é a
malha do outro lado da peça atravessando.

⚠️ **A cura tem DUAS metades e nenhuma delas basta**, e isto está medido:

| | miolo estrito | vazada |
|---|---|---|
| sem a nudge (o defeito do 1º report) | 45 % | 0,0 % |
| só a nudge (o que shipava) | **109 %** | 2,2 % |
| nudge + descarte por FRAGMENTO | **86 %** | **0,0 %** |
| nudge + descarte no VÉRTICE | 73 % | 0,0 % |

⚠️ **Os 109 % são a lição da wave: o oráculo anterior estava INFLADO.** Ele media
a tinta total contra o comprimento das arestas de frente, e numa esfera densa o
fio de trás projeta-se **por cima** do da frente — parte do que ele contava como
*"a aresta chegou"* era a malha de trás tapando o buraco de uma aresta cortada.
A queda para 86 % é a ilusão a sair, não cobertura a perder. A régua nova é o
**miolo estrito** (as arestas cujas DUAS pontas encaram o olho com folga, num
sólido convexo), que não tem essa ambiguidade.

⚠️ **E o descarte é do FRAGMENTO, não do vértice** — uma aresta que CRUZA a
silhueta tem uma ponta de cada lado, então decidir no vértice leva a metade
visível junto (86 % → 73 %).

⚠️ **A nudge fica em 3e-3, e não sobe.** Com o descarte ela deixou de ter
orçamento de vazamento, mas a varredura mostra que ela **SATURA** ali (86 % de
3e-3 a 4,8e-2) — e num TORO subir para 6e-3 faz o número *melhorar* para 87 %
porque arestas que encaram o olho e estão **atrás do tubo da frente** voltam a
atravessar. Um oráculo subindo enquanto a remoção de superfície escondida piora.

⚠️ **O preço foi COBRADO, não aceito:** numa casca ABERTA vista por trás toda
normal aponta para longe do olho, e uma regra que lesse *"normal de costas ⇒
escondido"* apagaria exatamente o que o artista está olhando. Por isso o descarte
só se arma numa malha **FECHADA** (`Mesh::is_closed`, um `f32` por-objeto no
uniform — **por-objeto e não do quadro**, senão uma única casca aberta na cena
devolveria o vazamento a todas as peças). Gate: `an_open_shell_keeps_its_wireframe`
compara a MESMA grade plana enrolada nos dois sentidos.

⚠️ **E a nota do resíduo da wave anterior era FALSA e foi corrigida:** ela dizia
que a cura precisava da adjacência aresta→face, que o `wire_indices` não constrói
(+89 % medidos). A normal POR-VÉRTICE já responde, e o toro foi de 14,0 % a 0,0 %
sem grafo nenhum.

**4 mutações, 4 sangram** (descarte sempre armado ⇒ a casca aberta some · sem
descarte ⇒ o vazamento volta · sem a nudge ⇒ as duas · o atalho ortográfico
`n_view.z` ⇒ 0,3 % de vazamento na esfera grossa, que é por que o `facing` é
perspectiva-correto).

**Aberto, com o número ao lado:** a faixa da silhueta continua **59 % da tinta**
por geometria da esfera, e a cura disso não é profundidade — é **anti-aliasing**
(desenhar linhas mais juntas que um pixel é aliasing por definição). O caminho
com nome é o wireframe de passe único por coordenada baricêntrica (Bærentzen
2006), que dá cobertura com `fwidth` e dissolve a faixa num degradê — e ele exige
geometria **não-indexada** no passe do barro (3× a memória de vértice), logo é
decisão de produto, não correção.

### §7.9 — ✅ O FIO SAI DE CIMA DE SI MESMO: o empurrão lateral (2026-08-12, 3º report)

O 2º report fechou o **vazamento** (a malha do outro lado atravessando) e deixou
o miolo estrito em **86 %**. Este fecha a outra metade, e ela vinha do Blender:
*"veja no código do blender como ele faz"*.

**Lendo o `overlay_wireframe_vert.glsl` (GPL, só comportamento):** o Blender
carrega um empurrão **LATERAL** de meio pixel em espaço de tela, pesado por
`facing_ratio = 1 − facing²` e com o sinal virado por `flip = sign(facing)`.
Nós tínhamos só a nudge de profundidade.

⚠️ **A pergunta que decidiu não foi *"o Blender faz?"*, foi *"onde mora o
buraco?"*** — e a medição **REFUTOU o meu próprio raciocínio**. Eu argumentei que
a tinta que falta estaria em `facing ≈ 1`, onde `1 − facing²` é zero, logo o
empurrão não a alcançaria. Binando o miolo estrito por `facing`
(`where_the_interior_miss_lives`):

| facing | cobertura (32x64 / 64x128) | peso do empurrão |
|---|---|---|
| **0,20-0,40** | **79 % / 75 %** | **0,91** |
| 0,40-0,60 | 100 % / 96 % | 0,75 |
| 0,60-0,80 | 103 % / 100 % | 0,51 |
| 0,80-1,00 | 100 % / 97 % | 0,19 |

O buraco está **exatamente** onde o peso é quase 1 — o oposto do que eu previ.
Perto da silhueta o triângulo se projeta quase de perfil e **cobre** a linha que
nasce sobre a aresta dele; a nudge de profundidade **satura** ali (3e-3 a 4,8e-2
dão o mesmo número) porque o problema não é disputa de profundidade.

⚠️ **E o SINAL foi um defeito medido, não uma escolha.** Empurrar ao longo da
normal externa leva o fio para **FORA** da silhueta, sobre o fundo: a tinta total
cai monotonicamente (**11984 → 11750 → 11495 → 11071** em 0 / 0,5 / 1 / 2 px) e o
bin rasante cai de 75,1 % para **72,9 %**. Invertido, os dois sobem. O fio tem de
correr para o **CORPO** da peça.

⚠️ **É `−sign(facing)`, não `−1`** — é isso que torna o empurrão **invariante à
orientação**: virar o enrolamento de uma casca vira a normal *e* o sinal do
`facing`, e o produto não se move. É o que mantém o
`an_open_shell_keeps_its_wireframe` verde.

**O meio pixel é MEDIDO** (e cai no mesmo número que o Blender ships):

| px | continuidade (32x64 / 64x128 / toro) | arestas inteiras | vazada |
|---|---|---|---|
| 0 (o controle) | 43,0 / 48,6 / 28,6 % | 699 | 0,0 % |
| **0,5** | **45,4 / 49,0 / 33,4 %** | **786** | **0,0 %** |
| 0,75 | 43,7 / 47,4 / 31,3 % | 827 | 0,3 % |
| 1,0 | 39,8 / 44,6 / 28,8 % | 766 | 1,4 % |

A 0,75 o vazamento já cruza a barra de 0,1 %; a 1,0 a **continuidade cai abaixo
do controle** — passado meio pixel o fio deixa de estar sobre a própria aresta e
passa a mentir sobre onde a geometria está. O **toro** é quem mais ganha
(28,6 → 33,4 %), porque é a peça com mais superfície de perfil.

**A direção sai de uma DIFERENÇA FINITA** do próprio `view_proj` (um segundo
caminho de *"onde este vértice cai na tela"* divergiria do `vs_core` no dia em
que a `Pose` ganhasse rotação), medida em **pixels** e não em NDC (num viewport
não-quadrado normalizar em NDC torceria a direção), com o passo proporcional a
`clip.w` para a régua acompanhar escala e zoom.

**Superfície nova:** `CameraRaw`/`Camera` ganham `viewport` — ⚠️ ele mora na
CÂMERA porque *"quanto vale um pixel aqui?"* é a mesma pergunta que a projeção ao
lado responde. `camera_uniform_bytes` passa a receber o tamanho e devolve
**144 B**; `view_proj_from_bytes` passa a receber `&[u8]`.

**Gate novo:** `the_grazing_edges_are_not_eaten_by_their_own_surface`, com a
barra em **78 %** — entre o certo (79,2 %) e os **dois** modos de falha (75,1 %
sem empurrão, 72,9 % invertido). **2 mutações, 2 sangram**, com os números exatos
do doc. ⚠️ E as duas primeiras rodadas de mutação foram **vácuo**: sem
`-- --ignored` nenhum teste rodou e os dois "verdes" não eram nada
([[feedback_a_negative_search_needs_a_positive_control]]).

**Aberto e nomeado:** o **59 %** de tinta nos dois anéis externos segue sendo
GEOMETRIA (anéis de latitude comprimindo na silhueta), não defeito — o alvo dele
é o limiar diedral do Blender (`wire_step_param`), que esconde aresta entre faces
quase coplanares; ele tem a adjacência de que precisa (`Mesh::edges()`) e é
**decisão de produto**, porque um wireframe que esconde arestas deixa de ser um
instrumento que lê topologia.

### §7.10 — ✅ W5 (metade A): O AGARRE VIRA UM CAMPO ELÁSTICO (2026-08-13)

O primeiro Kelvinlet ([de Goes & James 2017](https://graphics.pixar.com/library/Kelvinlets/))
entra pelo **Grab**, e com ele o `L` deixa de ser um chip com um verbo só.

**O que o chip troca, numa frase:** a lei do `S` é `gesto × escalar`, e um
escalar **não tem para onde apontar** — todo vértice da pegada anda na MESMA
direção. Um Kelvinlet é a solução fundamental da elasticidade: o termo
`(r·f)·r` faz o barro à **frente** do puxão acompanhar mais que o barro ao
**lado** dele. Medido no campo, a um `ε` do centro (onde o barro ainda anda 45 %
do que o bico anda): **2,72×**.

⚠️ **O `ν` tem TRÊS ATOS, e o do meio é meu.** (1) O argumento físico: barro é
incompressível ⇒ `ν = 1/2`. (2) A refutação: o ganho do bico do modo de escala
vale `(5/2)a − 5b`, que é **exatamente zero** em `ν = 1/2` — escrevi `0,4` e uma
tabela a justificar. (3) **A refutação da refutação, achada MEDINDO**: a
varredura devolveu a MESMA linha para todo `ν` de `0,00` a `0,49`, porque o campo
de escala **FATORA** — usando `r² = rε² − ε²` o colchete inteiro colapsa em
`(1 − 2b)·K(r)·r` e o ganho é `(5/2)(1 − 2b)`, o **mesmo** fator. Depois de
normalizar ele cancela: *a escala não pergunta de que material o barro é*, e o
zero era uma **singularidade removível da minha parametrização**. O argumento
físico do ato 1 volta a valer, e agora paga — os dois eixos que sobram (a
anisotropia do agarre `1,125× → 1,333×` e a divergência `0,5 → 0,0`) melhoram
**monotonicamente** até `1/2`.

⚠️ **E o fatorar também expôs que TWIST e ESCALA são o MESMO kernel** aplicado a
vetores diferentes (`ω × r` e `s·r`) — partilham o `radial()`, e o efeito
colateral é medido: **18,2 → 4,7 ns** por avaliação, 4× mais barato.

⚠️ **O campo é ILIMITADO e a pegada não é** — a outra decisão da wave. Um
Kelvinlet cru decai como `1/r` e a um raio de pincel ainda vale **70,7 %** do
bico: truncá-lo ali é um degrau enorme no anel do cursor. A cura é do próprio
paper (*multi-scale*), e o número escolheu:

| r/ε | Mono | Bi | Tri |
|---|---|---|---|
| 1 | 0,7071 | 0,5198 | 0,4533 |
| **3** | 0,3162 | 0,0778 | **0,0347** |
| 4 | 0,2425 | 0,0379 | 0,0119 |

⇒ **`KELVINLET_REACH = 3`** com a família **Tri**: 3,5 % na borda.

⛔ **E os 3,5 % deixaram de ser um resíduo ACEITÁVEL — ver §7.13.** Estas linhas
escolhem a família que decai mais depressa e depois **CORTAM** o que sobra; com
`ε = raio/3` o corte caía no anel do cursor, onde um degrau se lê como *a borda
do pincel*, e com a §7.11 ele mudou-se para 3× fora dele, onde nada o explica.
O número da tabela continua certo — o que estava errado era o veredito.

⛔ **E a leitura que estas linhas faziam do `3` estava INVERTIDA — ver §7.11.**
Elas diziam `ε = raio/3` (a pegada espremendo o campo) e recusavam a alternativa
*"manter `ε = raio` e crescer a consulta para `3·raio`"* com uma **estimativa**
(*"~9× os vértices por dab"*), que é o que a §0 proíbe. Medida, a pegada
triplicada custa **1,2 % do K1** num pincel de detalhe, e a leitura espremida
punha o cruzamento de zero do Tri **dentro do cursor**. O que shipa é
**`ε = raio`, pegada `3·raio`** — o anel passa a significar *a escala do que eu
deformo*, e o preço é esse, nomeado.

**A ÚNICA parte estrutural é uma COLUNA:** `Grip::law` passou a receber
`carries_field`, e ele move **exatamente uma** — o `unit_accum` do `Grip::Hold`.
Um campo **é** o falloff, então quem o recebe carrega o peso no ALVO; deixar o
aplicador atenuar de novo aplicaria o perfil duas vezes, que é o defeito mais
caro que este verbo já pagou (`0,12226` contra `0,22500`). ⚠️ E ele move UMA
coluna e não quatro **por medição**: os outros quatro grips já carimbam
`unit_accum = true` por razões próprias.

⚠️ **DOIS gates da W4 morderam, e um deles é a profecia dela a cumprir-se:**

- `the_literature_mode_is_offered_exactly_where_it_declares_a_law` pinava a
  coincidência *declara ⟺ mais de um passe*, e o doc dele dizia: *"o Kelvinlets
  da W5 é um campo de UM passe — no dia em que ele chegar, este gate cai e obriga
  quem o traz a vir aqui dizer o que passou a ser o discriminante"*. **Este é o
  dia**, e a resposta é que **não há discriminante derivável**: o `L` declara por
  um par de passes (Taubin) **ou** por um campo (Kelvinlets), e a próxima família
  pode não ser nem uma coisa nem outra;
- `a_mode_is_offered_only_where_it_is_not_a_duplicate_of_an_earlier_one` disse
  *"Move / Grab: S e L são o mesmo modo"* — o `l-mode` do Grab não muda lei de
  kernel, não muda curva e faz um passe, então **sob a assinatura de três eixos
  ele ERA o `s-mode` letra por letra**. É a lição do próprio doc da `signature`
  cobrada uma segunda vez: *uma assinatura que não contém um eixo observável é um
  discriminante falso*, e a defesa é ela crescer no MESMO commit que o eixo (hoje
  são quatro).

⚠️ **UMA MUTAÇÃO NÃO SANGROU E O BURACO ERA MEU.** Trocar o peso `flat` pelo `w`
— ou seja, aplicar a curva POR CIMA do campo, o defeito histórico deste verbo —
passava por **todos** os gates da wave: o bico não se move (a curva vale 1 ali), a
razão *à frente ÷ ao lado* não se move (os dois probes estão à mesma distância,
logo levam a mesma curva) e o degrau da borda só ENCOLHE. **Nenhum media o
PERFIL**, que é o que o defeito deforma. Nasceu daí o
`the_stroke_delivers_what_the_kernel_promises`, que compara o traço inteiro
contra o campo, vértice a vértice.

⚠️ **E o gate do degrau era VERDADEIRO e não DISCRIMINAVA:** sobre a esfera
partilhada (aresta `0,13081`) o degrau é `0,019` com o campo que shipa e `0,108`
com o campo CRU — os dois passam, porque a malha não consegue desenhar nem o
defeito. Quem o expôs foi a mutação da família de escalas, que não sangrou em
lado nenhum; ele passou a medir numa malha subdividida (aresta `0,065`).

⚠️ **E o CONTROLE pegou a minha sonda de anisotropia:** eu exigia que o `s-mode`
desse `1,00×` e ele deu **`1,19×`** — a esfera é UV, `+x` corre num meridiano e
`+y` num paralelo, e a anisotropia da MALHA entrava inteira no número. O oráculo
virou **razão contra razão**, com a malha como fator comum.

**11 gates de campo + 6 de produto. 7 mutações, 7 sangram.** Sem schema, sem
ADR, sem crate nova, sem dep nova, sem id novo.

⚠️ **PENDENTE DE SMOKE — `PH2D_SCULPT3D_SMOKE=28`.** A cena **não arma o
pincel** (a cicatriz do `impasto_smoke`): o artista pega o Grab, vê o chip `L`
nascer ao lado do `S`, e compara o MESMO gesto nos dois. O vértice sob o cursor
tem de seguir o dedo **igual** nos dois modos; o que muda é a vizinhança. E o
`L` **não pode** aparecer em mais nenhum verbo de geometria além do Smooth.

**Aberto (a metade B da W5):** os outros cinco verbos da tabela §5 — SnakeHook
(agarre de âncora móvel) · Twist · LocalScale · Pinch · Magnify —, e o pincel
**Elastic Deform**, que é o único que pede `Verb` novo. O kernel dos quatro
modos **já está construído e gateado**; o que falta é a fiação e o gate de
produto de cada um.

### §7.11 — ✅ OS MODOS `B` E `L` DO GRAB ESTAVAM BIZARROS (2026-08-13)

Report do Enio: *"os modo B e L do grob/move estão bizarros"*. **Dois defeitos
INDEPENDENTES**, os dois diagnosticados pela sonda antes de qualquer hipótese
(`tests/measure_grab_modes.rs`, que dirige o produto com um arrasto de **doze
eventos** — é isso que os torna visíveis).

**B — o barro agarrado COLAPSA no meio do arrasto.** O `Grip::Hold` recomputa
`accum = w` **do zero a cada dab**, e o `w` contém o `facing` do
[`FrontFace::Continuous`], que lia a normal **VIVA** — a normal que o dab
anterior acabou de girar. Isso é realimentação: o vértice roda para longe do
olho, o peso dele cai, o alvo encolhe, ele volta. Medido na ponta, como fração
do que o dedo pediu:

| evento | 1 | 5 | **9** | 12 |
|---|---|---|---|---|
| antes | 0,9956 | 0,7195 | **0,1418** | 0,8874 |
| depois | 1,0000 | 1,0000 | **1,0000** | 1,0000 |

⚠️ **A cura é uma linha e a LEI que ela honra é do repo inteiro:** *o peso é um
fato sobre a superfície CONGELADA*. A máscara já saía do `base_mask`, a distância
do `base_pos`, a normal do `Verb::Inflate` do `base_nrm` — o `facing` era o
**único leitor vivo**, e a assimetria frente/trás que o número mostra (0,7195
contra 0,9391 no mesmo instante) era exatamente ele.

**L — o agarre elástico era uma AGULHA com um colar invertido.** O `ε` era
`raio / REACH`, ou seja **a pegada espremia o campo** em vez de o campo escolher
a pegada. Medido a meio raio, ao LADO do puxão: **0,0296** contra os **0,5473**
do `s-mode` — 5,4 %. E o cruzamento de zero da família **Tri** (`r/ε ≈ 1,5`)
caía **DENTRO do cursor**, então metade da pegada empurrava barro **para trás**
(−3,8 % da ponta).

⚠️ **O defeito era MEU e a §0 nomeia as duas metades.** (1) A sonda que escolheu
o `KELVINLET_REACH` mediu **só À FRENTE** — o lóbulo onde o resíduo multi-escala
é menor e **positivo** —, e nunca AO LADO, que é onde ele fica negativo: *uma
tabela medida num eixo só é um número que não conhece o próprio sinal*. (2) A
alternativa que o doc da §7.10 recusou foi recusada **por ESTIMATIVA**
(*"~9× os vértices por dab"*), que é precisamente o que a §0 proíbe — medido, a
pegada triplicada custa **1,2 % do K1** num pincel de detalhe.

⇒ **`ε` É o raio do pincel**, e a pegada é `KELVINLET_REACH · raio`, pela porta
nova **`Brush::query_radius`** (o consultador do octree pergunta a ela; um verbo
sem campo devolve o raio, byte a byte). A curva do falloff vira a **INDICADORA
do suporte** — o campo já É o perfil —, o que preserva a separação entre cópias
de espelho **e torna o `flat` redundante** (`curva == 1,0` faz `w` ser o `flat`
ao bit): ele morreu.

| | antes | depois | `s-mode` |
|---|---|---|---|
| vértices movidos | 61 | **903** | 1063 |
| soma do deslocamento | 3,060 | **31,385** | 33,462 |
| ao lado, a meio raio | 0,0296 | **0,5185** | 0,5473 |
| à frente, a meio raio | — | **0,7708** | 0,5473 |
| maior passo entre vizinhos | — | **0,0919** | 0,1805 |

⇒ o perfil fica **mais largo que o do `s-mode`** com a anisotropia intacta
(1,49× à frente), e o passo máximo entre vértices vizinhos é **o menor dos
três** — o oposto de uma agulha. A invariância de caminho (o mesmo puxão total
em 1 ou em 12 eventos) mede **0,000000 nos três modos**, em puxões de 0,2 · 0,6
· 0,9.

**3 mutações, 3 sangram, cada uma nos gates certos:**

- a normal viva de volta no `facing` ⇒ **exatamente** os dois gates de B
  (185 passam, 2 falham) e mais nada;
- `query_radius` ignorando o campo ⇒ o gate da pegada + o do traço + o do degrau
  do aro (184/3);
- `ε = raio / REACH` (o defeito original) ⇒ o gate da agulha + os dois acima.

⚠️ **E o gate da agulha NÃO sangra sob a mutação da pegada** — é ele que prova
que *largura de PERFIL* e *alcance de PEGADA* são duas perguntas, e que um gate
por cada é o par mínimo.

**LOC:** o `stroke.rs` cruzou o teto (720 > 700) e foi cortado por
RESPONSABILIDADE em dois irmãos — `stroke_freeze.rs` (*o que um traço CONGELA*,
o assunto que os outros três filhos já pressupunham sem ter casa) e
`stroke_probe.rs` (*o que se consegue MEDIR de um dab*, contra *o que um dab
FAZ*). 671 + 54 + 41.

Sem schema, sem ADR, sem crate nova, sem dep nova, sem id novo.

⚠️ **PENDENTE DE SMOKE — a cena é a `PH2D_SCULPT3D_SMOKE=28` da §7.10**, e o
roteiro dela **muda**: o `L` agora deforma **para além do anel do cursor** (a
leitura do *Elastic Deform* do Blender — o anel passa a significar *a escala do
que eu deformo*, não *o que eu toco*), e o `B` tem de seguir o dedo do primeiro
ao último evento de um arrasto **longo** — o colapso do meio só aparece num
gesto que dure.

### §7.12 — ✅ W5 (metade B): AS TRÊS FAMÍLIAS AFINS E O GANCHO (2026-08-13)

Os cinco verbos que a §7.10 deixou abertos ganham o `l-mode`: **Snake Hook ·
Twist · Local Scale · Pinch · Magnify**. Os kernels já estavam construídos e
gateados desde a metade A — o que faltava era *como um verbo CONSOME um campo*, e
a resposta não é a mesma para os cinco.

**⚠️ O ACHADO QUE DECIDE A WAVE: `rigid()` é `perfil(r) · v(r)`.** O vetor rígido
(`ω × r` para a torção, `s·r` para a escala) **não depende da escala do
Kelvinlet**, então ele sai do somatório e o que sobra é um **escalar**. Isso
importa porque um verbo que GIRA já tem a geometria certa, e somar-lhe o
deslocamento linearizado do paper põe o vértice **fora da circunferência** —
`|r|·√(1 + (θ·perfil)²)`. Medido a meio raio, onde o perfil vale `0,577`:

| θ (rad) | deslocamento | ângulo |
|---|---|---|
| 0,50 | 1,0408 | **1,0000** |
| 1,00 | 1,1546 | **1,0000** |
| 2,00 | **1,5271** | **1,0000** |

⇒ **Twist e Local Scale consomem o ESCALAR** (`kelvinlet::rigid_profile`, porta
nova) e mantêm a própria geometria exata — giram sobre a circunferência, escalam
ao longo do raio —, com o campo a decidir só *quanto cada vértice acompanha*.
**Grab, Snake Hook e Pinch consomem o VETOR**, porque não há geometria de verbo a
preservar (e a `F` do aperto produz um termo `(r·F r)·r` que não é múltiplo de
`F r`: não há escalar a extrair).

⚠️ **E o primeiro número que escrevi para isto era `+12 %` a meio radiano, sem
medição** — a conta com `perfil = 1`, que só vale no BICO, onde `r = 0` e não há
raio nenhum a inflar. É a §0 em casa: *a sonda corrigiu a minha própria
afirmação antes de ela shipar num doc-comment*.

**O que cada verbo passou a fazer, medido pela porta do artista:**

| verbo | vértices S→L | o que o campo acrescenta |
|---|---|---|
| Snake Hook | 84 → **946** | a vizinhança acompanha o gancho, com perfil (`aro÷bico = 0,1035`) |
| Twist | 60 → **902** | gira sem inflar (`1,0000`) e decai do bico ao aro |
| Local Scale | 60 → **902** | dilata radialmente com perfil |
| Pinch | 60 → **902** | **devolve pela normal o que tira do plano** (razão `0,1515 → 0,5043`) |
| Magnify | 60 → **902** | dilata **além do anel**, onde o `s-mode` para |

⚠️ **A MUTAÇÃO ACHOU UM BURACO DE DESENHO, e a cura não foi um gate melhor.**
Escrever na tabela um par (verbo, campo) TROCADO — `Pinch → Field::Scale` —
passava nos **193** gates: o alvo do Pinch casa o próprio variante, não casa,
cai no modo que já shipava … **e a pegada continua a do campo**, porque o
`query_radius` pergunta só `is_some`. O resultado é um `l-mode` **com o alcance e
sem a lei**, e um gate que mede *"L difere de S"* não o distingue de um `l-mode`
são. ⇒ O *qual* mudou-se para o **VERBO** (`Verb::elastic_field`) e o modo ficou
só com o *se*: o par deixou de ter um segundo sítio onde discordar, e a mutação
passou de *não-detectada* a **inexprimível**.

⚠️ **E DOIS gates meus ficaram VERDES sobre a mutação pelo MESMO motivo — a
SOMA.** O gate do gancho contava vértices e o do aperto somava deslocamento
sobre a pegada; tirado o braço de campo, a pegada 3× mais larga com a
curva-**indicadora** (que vale `1` em toda ela) **finge o sinal**: mais vértices,
soma maior, gate verde. O oráculo que ela não consegue fingir é o **PERFIL** —
`aro ÷ bico`, que sem lei nenhuma é ~1,0 e com o campo mede `0,10` e `0,23`. É a
lição da §7.11 outra vez (*o número no lugar errado diz o contrário da foto*),
desta vez no meu próprio gate.

⚠️ **E um discriminante que eu afirmei NÃO EXISTE:** o gate do Magnify nasceu a
dizer que o campo dilata *ao longo de `r`* contra um `s-mode` *lateral* —
medido, os dois dão **cos 1,0000**, porque o `lateral_pull` aponta do centro do
dab para o vértice e sobre a calota raio e tangente são a mesma direção. A
radialidade ficou como CONTROLE (uma propriedade do campo *scale* que um bug de
sinal quebraria) e o discriminante passou a ser o **alcance**.

**7 gates novos** (o censo + cinco de forma + o perfil da escala), **7 mutações,
7 sangram** — e a oitava (o par trocado) é a que deixou de existir. Sem schema,
sem ADR, sem crate nova, sem dep nova, sem id novo.

**LOC:** `stroke_target.rs` cruzou o teto (708) e foi cortado pelo **`e` que o
próprio cabeçalho dele carregava** — *"o alvo de cada verbo, **e** o plano que
quatro deles ajustam"*: o estimador de plano saiu para `stroke_plane.rs` (578 +
147). ⚠️ E a minha inserção do `elastic_field` **ORFANOU o doc-comment do
`grip`** (entrou entre o doc dele e o `#[must_use]`), o mesmo defeito de âncora
que o `paint.rs` do Painter já pagou; quem o pegou foi o `warning: unused
attribute`, não um gate.

⚠️ **PENDENTE DE SMOKE — `PH2D_SCULPT3D_SMOKE=28`**, a cena da §7.10, agora com
cinco verbos a mais para comparar no dropdown. A pergunta de olho é a mesma dos
outros dois: **o chip `L` tem de mudar o que se vê**, e o que muda é a
vizinhança — nunca o barro sob o cursor.

**Aberto:** o **Crease** (a matriz §3 pede *Draw + Kelvinlets pinch*, que é uma
COMPOSIÇÃO e não um campo puro) · o pincel **Elastic Deform**, o único que pede
`Verb` novo · e o resto da §5.

### §7.13 — ✅ O CAMPO ATERRISSA NA BORDA DA PEGADA (2026-08-13, 4º report)

> *"MOdo L o Falloff parece ter borda dura"* — Enio, com screenshot: uma escada
> ao longo de um arco que cruza o anel do cursor e segue pela esfera.

**O que a medição achou, antes de qualquer hipótese.** A curva que o `stroke`
entrega a um verbo de campo era a **INDICADORA do suporte** —
`if dist <= query_r { 1.0 } else { 0.0 }`, um corte C0 — e no raio da pegada o
campo ainda carrega **3,47 %** do deslocamento do bico (`|grab|`, o gate
`the_rim_residual_is_what_chose_the_scale_family` mede exatamente isso e o
**CERTIFICA**). Medido no produto: degrau de **2,90 % ao longo de uma costura de
114 vértices**, contra os **1,57 % / 10 vértices** que o `s-mode` — o que shipa
há meses sem report — deixa na borda dele.

⚠️ **A minha primeira hipótese CAIU e fica escrita.** Eu li o `rigid_profile`
em `r/ε = 3` (**0,00011**) e declarei a tabela §7.10 refutada. O `rigid_profile`
é só o ESCALAR; o que o artista vê é `|grab|`, que inclui o termo anisotrópico
`(r·f)r` — e ele vale **0,03472**, ou seja a tabela estava certa e eu tinha
medido a grandeza errada. *Um número no lugar errado diz o contrário da foto.*

⚠️ **E a causa é a §0 mordendo a minha própria wave anterior.** O degrau sempre
existiu; a §7.11 mudou `ε` de `raio/3` para `raio` e **mudou o corte de lugar**,
do anel do cursor (10 vértices, onde um degrau se lê como *a borda do pincel*)
para 3× o anel (114 vértices, onde nada o explica). *Quem move o número que
tornava algo tolerável tem de reconferir a nota.*

⛔ **Esticar o alcance NÃO é a cura, e isto é medição:** `REACH` 4 → 1,19 % ·
5 → 0,48 % · 6 → 0,215 % — **nunca zero**, com o número de vértices a crescer
como `r²`. Um kernel regularizado tem cauda infinita por construção; o corte é
inerente, e o que se escolhe é **como** ele acontece.

**A cura é uma JANELA no CONSUMIDOR, não no kernel.** `kelvinlet::rim_landing`
é `1,0` até `RIM_HOLD` e desce por `smoothstep` até zero na borda:

- **A concessão pertence a quem corta.** O `grab` é a eq. 5 do paper e as três
  famílias afins são derivadas direcionais DELE; deformá-lo para esconder o
  próprio corte tornaria toda paridade com a referência uma comparação contra
  uma versão nossa. O paper fica intacto e o `stroke` — que é quem decide o
  suporte finito — paga.
- **UM sítio serve os CINCO verbos.** Os cinco consomem a curva **linearmente**
  (grab/pinch a dobram na força, que é linear em `f`/`s`), então um fator na
  indicadora escala os cinco **por construção** — não há tabela por-verbo a
  apodrecer.
- **`RIM_HOLD = 0,75` é MEDIDO, não escolhido.** A varredura: `0,50` deixa o
  degrau em 0,04 % **e** derruba o gate do pinch (0,1686 contra o piso de
  0,1515 — o splash ao longo da normal vive predominantemente na metade
  EXTERNA da pegada, então uma janela que morde metade do raio devolve menos
  volume); `0,75` dá **0,06 %** e o pinch volta a 0,1993. *A barra não foi
  afrouxada — o número foi lido do joelho.*

**Resultado, pela porta do produto:** degrau **2,90 % → 0,06 %** (26× abaixo do
que o `s-mode` aprovado já deixa), bico **byte-idêntico** (o hold cobre 75 % do
raio), o `s-mode` **intocado** (controle), e a subida no aro que o histograma de
saltos mostrava (0,1873 / 0,3521) **desapareceu** — o decaimento é monotônico
até a borda.

**Gates.** `the_elastic_field_lands_at_the_rim_instead_of_being_cut` afirma o
**CONTROLE primeiro** (`at_rim > 0,03`, senão ele passaria por vácuo sobre um
campo que já fosse zero ali) e só então o degrau. E
`the_stroke_delivers_what_the_kernel_promises` foi reescrito para dizer o que o
produto FAZ (`kernel × janela`) e ganhou uma metade que **não existia**: dentro
do hold a janela tem de ser a identidade **AO BIT**.

⚠️ **Três mutações, três sangram — e uma sangrou noutro lugar.** M1 (indicadora
de volta ao corte duro) → 2 RED · M2 (`RIM_HOLD = 1,0`, a janela existe e não
aterrissa) → 1 RED · M3 (`RIM_HOLD = 0,0`, a janela morde o gesto) → 1 RED, **no
gate do PINCH**. A minha asserção de hold é **vazia** com `HOLD = 0` (nenhum
vértice tem `t ≤ 0`) ⇒ quem protege o gesto é o volume do pinch, não ela.

⚠️ **E o gate que estava VERDE sobre isto tinha a MENSAGEM já falsa:** ele dizia
*"o Tri é o que torna a borda do CURSOR honesta"*, frase verdadeira enquanto
`ε = raio/3` e falsa desde a §7.11. Ele mede o resíduo e o **certifica como
aceitável** — um veredito calibrado para uma colocação que deixou de existir.
Corrigido no mesmo commit, com o mecanismo escrito ao lado.

### §7.14 — 📐 O QUE O `l-mode` CUSTA, e as três alavancas que a medição MATOU (2026-08-13)

> *"o resultado ficou muito bom do modo L mas com um pouco de queda de FPS.
> Avalie se pode otimizar sem perder a qualidade"* — Enio.

**Isto é MEDIÇÃO, não wave: nenhum byte do produto mudou.** As duas sondas são
`ph2d-sculpt3d/tests/measure_field_cost.rs` e a
`measure_what_the_refresh_region_is_made_of` (unit, na `ph2d-mesh` — os campos
do `RegionScratch` são `pub(crate)`, e de fora a decomposição mediria o laço da
SONDA em vez do laço do produto).

#### O CONTROLE primeiro: a queda não é da §7.13

`rim_landing` custa **2,14 ns** contra **15,58 ns** de uma avaliação `grab` —
e ⚠️ os dois números carregam o mesmo overhead de closure boxed, então o da
janela é **cota superior**. Sobre a pegada inteira, a aterrissagem é **~2 % do
dab**. *A wave que o Enio aprovou não é a que ele está pagando.*

#### A malha certa, e por que a primeira tabela mentia

⚠️ **O 1º corte da sonda mediu numa `sphere_with_triangles(1M)` — uma
UV-SPHERE**, com vértices amontoados nos polos, e tomava o **MAX** sobre os
dabs. Ali a pegada cresce **linearmente** com o raio (3,00× para 3× o raio),
porque um disco perto do polo engole anéis inteiros. A cena do módulo abre a
`sculpt_sphere` — cubo subdividido, **196 608 triângulos, densidade
quase-uniforme** —, e ali a lei é `r²`:

| raio | pegada `s` | pegada `l` | razão | dab `s` | dab `l` | razão |
|---|---|---|---|---|---|---|
| 2 % | 121 | 661 | 5,46× | 0,013 ms | 0,054 ms | 4,22× |
| 10 % | 1 489 | 8 995 | 6,04× | 0,108 ms | 1,047 ms | 9,67× |
| 30 % | 8 995 | 76 861 | 8,54× | 1,027 ms | 6,406 ms | 6,24× |

**`S(30 %) = L(10 %) = 8 995`** — a porta `query_radius` a funcionar, e o
controle interno da tabela. *Uma fixture com a densidade errada responde a outra
pergunta com confiança.*

#### De que o dab do `l-mode` é feito (raio 30 %)

| parte | ms | % | natureza |
|---|---|---|---|
| normais + curvatura | 3,166 | **49 %** | já `rayon` |
| o laço de vértices | 2,649 | **41 %** | **SERIAL** |
| a consulta do octree | 0,591 | 9 % | — |

#### ⛔ Três alavancas MEDIDAS e mortas

1. **A família de escalas: zero.** `Tri` custa **1,00×** o `Bi` (62,30 contra
   62,43 ms em 4 M avaliações) — o 3º tap é grátis, o kernel é limitado por
   LATÊNCIA e não por vazão. Só o `Mono` é barato (4,41×), e ele é a família
   cujo campo longe **não cancela**: é o look, não o custo.
2. **Paralelizar a descoberta do `refresh_region`: ≤ 5 % do dab.** Ela é
   **3 % / 6 % / 10 %** do passe (os outros 90-97 % são as normais e a
   curvatura, que já são `rayon`). ⚠️ **E isto REFUTA um número que o
   `CLAUDE.md` §5 carrega da W1** (*"o custo real é DESCOBRIR A VIZINHANÇA,
   11,5 ms = 88 % do refresh"*): aquilo era do `apply_dab`, numa UV-sphere de
   5 M, antes de o refresh paralelizar as duas metades. *A nota sobreviveu ao
   fato.*
3. **Pular a curvatura: muda a APARÊNCIA.** Ela parecia desperdício — a
   `ph2d-sculpt3d` não a lê em lugar nenhum —, mas quem a lê é o **RENDERER**,
   todo quadro (`pipeline_upload`: cavidade e sombreado). Fora da mesa.

#### ✅ A alavanca que sobra, com o teto

⚠️ **ADR escrito e aceito: [ADR-0158](../architecture/decisions/0158-sculpt3d-the-dab-vertex-loop-is-a-row-disjoint-map-rayon-exception.md)**
(número PROVISÓRIO — ele se re-conta na integração). A CONSTRUÇÃO está
especificada e **não foi feita**: ver o fecho da §7.15.

**O laço de vértices do `SculptStroke::dab` é 41 % e roda em UM núcleo de
trinta e dois.** As escritas são disjuntas (cada vértice lê `pre` + vizinhança e
escreve a própria posição) — a condição do ADR-0109, e o `rayon` na
`ph2d-mesh` já é sancionado pelo ADR-0150. Teto: dab **6,4 → ~4,0 ms (1,6×)**.
⚠️ **Byte-identidade tem de ser PROVADA por gate, não assumida:** o `fit_plane`
e o banco do `Grip` acumulam em ponto flutuante, e ordem de soma entre threads
move bits — o corte tem de deixar a acumulação fora do laço paralelo.

#### ⚠️ E o que NÃO foi medido, nomeado em vez de estimado

Um dab roda **por EVENTO de ponteiro**, e o `sculpt_at` faz três coisas por
evento: `pick_active` (raycast), `refine_for_dab` (dyntopo) e o dab. **Só o dab
está medido.** Os dois primeiros são **independentes do modo** (o `refine` usa
`brush.radius`, não o `query_radius`), então a razão `l/s` que o report descreve
está atribuída — mas o orçamento por QUADRO precisa dos outros dois, e quantos
eventos o shell entrega por quadro é o número que falta.

### §7.15 — ✅ O PUXÃO DO GRAB É CARIMBADO UMA VEZ POR QUADRO (2026-08-13)

A primeira das duas metades do *"ambos"*. O `Grip::Hold` fazia **um dab por
EVENTO de ponteiro** — o `walk`, que espaça dabs por distância percorrida, só
cobre `Grip::Stamp | Paint` —, e a ~1000 Hz de mouse com 60 fps são **~16 dabs
por quadro** a 1,05 ms cada.

⚠️ **A minha ressalva caiu na medição, e eu a tinha escrito como bloqueio.** Eu
disse que descartar dabs intermediários *"pode mudar quais vértices entram"*,
porque a pegada é consultada nas posições VIVAS. Medido — 16/8/4/2/1 eventos,
puxões de 50 %, 100 % e 200 % do raio — o desvio máximo é **0,000000** e a
contagem de movidos é a MESMA: **17,9 ms viram 1,2**.

O mecanismo já estava escrito no `touched`: o `Hold` é **frozen**, o laço
percorre o conjunto congelado no pen-down e o alvo é medido contra o `pre`. A
pegada viva só pode CRESCER, e o que ela acrescenta pesa zero contra a posição
congelada.

**O evento REGISTA, o QUADRO carimba, o pen-up DRENA antes de fechar** — sem a
última o gesto perde a ponta, um erro que cresce com a velocidade da mão e some
quando ela é lenta. O pen-down limpa o pendente pela razão do `twist` ao lado.

⚠️ **E um gate MEU sobreviveu à mutação na 1ª rodada:** o da ordem no pen-up
fazia `find` sobre o ARQUIVO, e o dreno de quadro chama a mesma função ~170
linhas acima — `flush < close` era verdade **por construção**. A janela agora é
o corpo do pen-up. **4 mutações, 4 sangram.**

#### ⚠️ O que ficou por fazer, e por quê

A **paralelização do laço de vértices** (1,6×) tem o [ADR-0158](../architecture/decisions/0158-sculpt3d-the-dab-vertex-loop-is-a-row-disjoint-map-rayon-exception.md)
escrito e aceito, com as três condições do ADR-0109 **verificadas no código** —
o laço não escreve posições, as leituras são puras dentro de um passe, os slots
são disjuntos e o `compute_target` não lê o `accum`. **O código não foi
escrito.** Ele é uma reestruturação do laço mais gateado da crate (196 gates
mais a paridade ULP com o SculptGL), e a prova exigida é byte-identidade contra
a rota serial CONGELADA — trabalho que não cabia com segurança no que restava
desta sessão, e meio-feito ali é pior que não começado.

### §7.16 — ✅ O VINCO GANHA `l-mode`, e a lei do suporte precisou de um segundo olhar (2026-08-14)

O **Crease** é o único verbo COMPOSTO da matriz §3 (*Draw + Kelvinlets pinch*) e
o último item aberto da W5. Os cinco verbos da W5-B têm o deslocamento INTEIRO
vindo do kernel, então a lei *"com campo, a curva é o SUPORTE do campo"* os serve
toda; aqui ela alcançaria também a metade que **não** é do campo — e a sonda
`measure_the_crease_trench` diz o que isso faz:

| banda | s-mode | campo **INGÊNUO** | composto |
|---|---|---|---|
| 0,25-0,50 r | 0,08594 | 0,18890 | 0,06751 |
| 0,50-0,75 r | 0,00941 | 0,18370 | 0,01547 |
| 1,00-1,50 r | 0,00000 | **0,15410** | −0,00077 |

⇒ **com a indicadora no lugar da quártica o vinco vira CRATERA:** ele afunda
**82 %** do bico a um raio e meio (contra **11 %** do s-mode) e 2,2× fundo demais
no bico, sobre uma pegada 3× mais larga. Um vinco é fundo e ESTREITO.

**A cura não pediu canal novo:** a metade estreita toma a estreiteza do **perfil
do próprio Kelvinlet**, elevado à mesma quártica que o s-mode aplica à curva do
pincel. O verbo composto fica inteiro na linguagem do campo, e o aperto lateral
ganha o alcance elástico que era o ponto de ter um `l-mode` — medido, o aperto do
`s-mode` para **morto** no anel do cursor (`0,00000` a partir de um raio) e o
elástico ainda mede `0,0081` a um raio e meio: a vizinhança escoa para o vinco.

⚠️ **E o meu gate mediu a PEGADA em vez da LEI — a armadilha da §7.12, no meu
próprio arquivo.** A 1ª metade (*"o aperto passa do anel"*) ficou **VERDE** com a
mutação que troca o kernel elástico pelo `lateral_pull` do s-mode: com um campo
declarado a pegada já é 3× e o `w` já é a indicadora, então o puxão cru alcança
dois raios **sozinho**. O que a pegada não finge é o **PERFIL**, e as duas leis
são opostas nele:

| banda | `lateral_pull` × indicadora | Kelvinlet |
|---|---|---|
| 0,50-0,75 r | 0,05708 | 0,01730 |
| 1,50-2,00 r | **0,15174** | **0,00185** |

O puxão cru é o delta ao centro, logo **CRESCE** com a distância até cair de um
penhasco na borda da pegada; o campo **DECAI**. Razão longe÷perto: **2,66**
contra **0,11**.

**1 gate novo, 3 mutações, 3 sangram** (a quártica do perfil ⇒ cratera de 91 % ·
o kernel elástico ⇒ razão 2,66 · o campo declarado ⇒ o aperto não passa do anel).

⚠️ **E os DOIS censos que já existiam pegaram a fiação incompleta antes de mim** —
o `every_declared_field_reaches_the_clay` (*"declara um campo e não tem gate de
FORMA"*) e o `the_literature_mode_is_offered_exactly_where_it_declares_a_law`
(*"a literatura portada até hoje"*). É exactamente o que a §7.12 os construiu
para fazer: um sexto verbo entra por aquelas linhas ou não entra.

**LOC:** `verb_field_tests.rs` cruzou o teto (712) e foi cortado pelo **paper**,
não pelo tamanho — os cinco verbos cujo deslocamento é inteiro do kernel ficam no
pai, o COMPOSTO sai para `verb_crease_field_tests.rs` (592 + 133).

Sem schema, sem ADR, sem crate nova, sem dep nova, sem id novo.

⚠️ **PENDENTE DE SMOKE — `PH2D_SCULPT3D_SMOKE=28`**, agora com o Crease no
dropdown. A pergunta de olho: **o canal tem de continuar estreito** e a
vizinhança tem de escoar para dentro dele.

~~**Aberto na W5:** o pincel **Elastic Deform**, o único que pede `Verb`
novo.~~ — ⚠️ **A MEDIÇÃO DISSOLVEU O ITEM. Ver §7.17.**

### §7.17 — ✅ A W5 FECHA NA LARGURA DO CAMPO, e o `Verb` novo era o item errado (2026-08-14)

O último item aberto da W5 dizia *"o pincel **Elastic Deform**, o único que pede
`Verb` novo"*. Antes de escrever uma linha, lidos os cinco tipos de deformação
que o Blender oferece dentro desse pincel:

| Elastic Deform do Blender | onde ele já vive aqui |
|---|---|
| *Grab* | `Verb::Move` em `l-mode` (§7.10) |
| *Grab Biscale* | **o mesmo**, família `Bi` |
| *Grab Triscale* | **o mesmo**, família `Tri` |
| *Scale* | `Verb::LocalScale` em `l-mode` (§7.12) |
| *Twist* | `Verb::Twist` em `l-mode` (§7.12) |

⇒ **Três dos cinco são o mesmo verbo, e diferem SÓ na família de escalas**;
os outros dois já shipam. Um `Verb::ElasticDeform` seria um sexto botão cujo
conteúdo inteiro é um dropdown para verbos que a lista já tem — a forma exata do
*item de menu morto* que este plano recusa em toda parte. **O que faltava era o
knob**, e é ele que a wave entrega: `Brush.elastic_scales`, a fileira **Field
width** (`Wide` · `Medium` · `Tight`).

⚠️ **Os rótulos dizem LARGURA e não a aritmética.** `Mono`/`Bi`/`Tri` dizem
*quantos kelvinlets a soma tem*, o que não ajuda ninguém a escolher; o que o
artista vê é **quanto a vizinhança acompanha**. Medido pela porta do produto
(esfera unitária, `Move` em `l-mode`, puxão de 0,2 tangente ao polo):

| | ponta (o dedo) | saia a meio raio |
|---|---|---|
| **Wide** (`Mono`) | 0,200000003 | **0,136837** |
| **Medium** (`Bi`) | 0,200000003 | 0,097633 |
| **Tight** (`Tri`) | 0,200000003 | **0,084063** |

**A ponta é bit-idêntica nas três** e a saia do `Wide` carrega **1,63×** a do
`Tight` — a família **redistribui** o que o campo leva, ela não muda quanto o
dedo leva, e é essa a metade do gate que impede um escalar global de passar.

⚠️ **E um segundo desenho meu foi REFUTADO por um doc que já estava no repo.**
Eu havia medido o *resíduo de borda* de cada família em `reach = 3` (`0,3162` ·
`0,0778` · `0,0347`) e ia tornar o `KELVINLET_REACH` **função da família**, para
igualá-los: `28,8` · `4,2` · `3,0`. O doc-comment do `rim_landing` (§7.13) já
tinha medido e recusado exatamente esse movimento — *"alargar o alcance NÃO é a
cura … a janela dá exatamente zero, por construção, a QUALQUER alcance"*. Com a
aterrissagem no lugar as três chegam a `0,00000` na borda, e a única diferença é
a **meia-largura do perfil** (1,74 · 1,04 · 0,93). ⇒ **`KELVINLET_REACH` e o
`query_radius` ficaram intocados**, e a pergunta errada está escrita ao lado da
medição para ninguém a refazer.

⚠️ **O preço do `Wide`, nomeado:** a janela de aterrissagem desenha **40,6%** do
perfil dele no último quarto do alcance (contra 8,7% do `Tight`) — é um ombro
C¹, não um degrau, mas é ele que faz o `Mono` parecer *mais macio* em vez de
*mais largo*.

**Default `Tight`**, delegado ao `Scales::default()` do kernel em vez de
reescrito no `Brush` — a medição do resíduo de borda continua sendo a dona do
número, e o mundo que shipa é **byte-idêntico**.

**A fileira é `Pro`**, pela regra do `UiLevel`: o valor foi **armado** (em Basic
o artista está com a largura que o kernel escolheu por ele, não com um vazio). E
ela é oferecida pela **MESMA porta que o motor pergunta** (`RefMode::field(verb)`)
— nunca por uma lista de verbos ao lado, que seriam três chips que não movem um
vértice no dia em que um verbo entrasse na família.

Gates: `the_field_width_reaches_the_clay_in_the_direction_the_labels_promise`
(as duas metades) + `the_field_width_row_exists_only_where_the_field_does_and_the_chip_lands`
(presença · os dois CONTROLES negativos · o clique pousa). **4 mutações, 4
sangram**: o tool ignorar o knob · o chip não pousar · a row perder a porta do
CAMPO · a row perder a porta do NÍVEL.

Sem schema, sem ADR, sem crate nova, sem dep nova, sem contrato tocado; os três
ids são `hash_node_id`.

✅ **SMOKE OK (2026-08-14).** `PH2D_SCULPT3D_SMOKE=28` — Grab em `L`, painel em
**Pro**, o barro puxado com cada uma das três larguras: a ponta segue o dedo
igual nas três e o que muda é o quanto a vizinhança vem junto.

⇒ **A W5 está FECHADA.**

### §7.18 — 📐 A W6 abriu, e a MEDIÇÃO re-escopou o item mais barato dela (2026-08-14)

A W6 lista cinco tools e chama o **Draw Sharp** de *"custo quase nulo: o `pre`
congelado já existe"*. Antes de escrever uma linha, a frase foi conferida contra
o nosso motor — e ela descreve **metade do que já shipa**.

O `draw_sharp.cc` da referência é o `draw.cc` com **um** troco: os fatores saem
de `orig_data.positions/normals` onde o Draw usa `position_data.eval`. E o nosso
[`crate::Grip::Stamp`] já carimba `from_live = accumulate` ⇒ **o Draw com
Accumulate DESLIGADO já mede a distância no `pre` congelado**.

⇒ A pergunta virou a do gate 2 do §8 (*nenhum chip morto*): quanto um verbo novo
acrescentaria sobre um interruptor que o artista já tem? Medido
(`tests/measure_draw_sharp.rs`, grade de 80², pincel `r = 0,5`, força 0,5,
espaçamento do produto, secção transversal no meio do traço):

| dabs | Accumulate | pico | meia-largura |
|---|---|---|---|
| 1 | ligado | 0,025000 | 0,3000 |
| 1 | desligado | 0,025000 | 0,3000 |
| 9 | ligado | 0,174057 | **0,2500** |
| 9 | desligado | 0,184088 | **0,2500** |

**A meia-largura é IDÊNTICA** e o pico difere **6 %**. ⇒ Um `Verb::DrawSharp`
construído *só* sobre o dado congelado seria um chip cujo resultado o artista já
alcança por um checkbox — e a palavra que o nome promete (*vinco duro em vez de
domo*) **não estaria lá**: os dois perfis são o mesmo domo.

⚠️ **Onde a palavra de facto mora é na CURVA** — o nome do tool é literal, e o
que o separa do Draw no Blender é o preset de falloff, não o `orig_data`. E é
exatamente aí que a §7.1 morde de novo: `BKE_brush_sculpt_reset` **continua fora
do clone** (`grep -rn "BKE_brush_sculpt_reset" source/` devolve **vazio** hoje,
mesmo depois de o `brush.cc` ter sido trazido), então escrever *"o Draw Sharp
nasce com a curva Sharp"* seria inventar um número e shipá-lo com a autoridade de
uma referência que não o declara — o que o §4 proíbe.

⇒ **O Draw Sharp SAI da lista de itens baratos da W6.** Ele não é caro por
kernel; ele está **bloqueado pela mesma tabela que bloqueou a W1**, e a decisão
honesta é a mesma: ou o número aparece numa fonte, ou o chip não nasce.

⚠️ **E a sonda ensinou uma coisa sobre a própria fixture, que fica escrita:** a
1ª versão amostrava a secção por **coordenada** (`|x| < 1e-4`) e reportava pico
**0,000** onde o máximo global era **0,184**. O Draw empurra pela normal da
ÁREA, que se **inclina** à medida que o traço levanta a superfície ⇒ os vértices
andam em `x` também, e o filtro perdia **19 dos 81**. A secção passou a ser
tomada por **ÍNDICE**. *Uma coordenada que o próprio experimento move não é uma
âncora.*

⇒ **O que sobra da W6, re-ordenado pelo que a medição diz:** o **Clay Strips** é
o item que carrega o refactor (o falloff receber coordenada local) e é o único
cujo conteúdo não depende de tabela ausente — a lei dele é **algorítmica** e está
inteira no clone (`clay_strips.cc` + `calc_brush_cube_distances`): moldura local
a partir da normal do plano e da direção do traço, parábola `z·(1−z)` na
profundidade, e a **distância de caixa arredondada** (`tip_roundness`), que em
`roundness = 1` reduz **literalmente** à distância euclidiana de hoje. O
**Multiplane Scrape** e o **Clay Thumb** reusam a moldura; o **Blob** é o Crease
com o pinch invertido.

⚠️ **E o pré-requisito que nenhum dos três tem hoje: a DIREÇÃO DO TRAÇO não
chega ao dab.** O `Dab` carrega `center`/`eye`/`pull`/`amount` e mais nada, e a
moldura do strip precisa saber para onde a mão ia. A lição já está paga neste
repo — a `line/Painter` mediu **52,4° de atraso** num heading suavizado e a cura
foi *o eixo vem dos CENTROS dos dabs*: o `SculptStroke` vê a sequência, então é
ele que deriva, e o vetor entra no `Dab` para o espelho o tratar como trata o
`pull` (a lei das espécies geométricas do `dab()`).

### §7.19 — ✅ W6 (metade A): O DAB DEIXA DE SER UM DISCO (2026-08-14)

O item que a §7.18 apontou como o único da W6 cujo conteúdo não depende de
tabela ausente. `Verb::ClayStrips` — o **17º** verbo, e o primeiro cuja pegada
não é redonda.

**A espinha: a curva de falloff passa a receber uma COORDENADA LOCAL.** Até aqui
todo verbo media `dist / raio` contra o centro, e é por isso que o catálogo
inteiro tem a mesma silhueta. Agora existe uma [`Footprint`], hoisted uma vez por
dab (ao lado do `alpha_frame`), que responde `(t, portão)`:

- **`Disc`** devolve `(dist · inv_r, 1.0)` — o mundo que já shipa, **ao bit**;
- **`Strip`** devolve a **distância de caixa arredondada** no plano e a
  **parábola `z·(1−z)`** na profundidade.

⚠️ **A entrada da curva continua sendo UM número**, e é isso que faz uma forma
nova não pedir um segundo falloff: a dureza, as doze curvas e a curva própria da
máscara seguem lendo o mesmo `t`.

⚠️ **`roundness = 1` é a distância euclidiana, EXATAMENTE** (o miolo chato
colapsa num ponto e toda consulta cai no ramo da quina, cujo centro é a origem) —
a mesma âncora que o `rf = 1` da multi-resolução do Wet Paint e o `eye = 0` do
estêncil do alpha usam.

**A DIREÇÃO DO TRAÇO chega ao dab** (`Dab::path`), derivada pelo
`SculptStroke` da diferença entre CENTROS de dabs consecutivos — nunca de uma
tangente suavizada, que é a lição de 2D onde a `line/Painter` mediu **52,4° de
atraso**. Ela espelha como VETOR, junto do `eye` e do `pull`.

**Cinco correções que só a medição achou, e as três primeiras eram minhas:**

1. ⚠️ **O SINAL do plano.** A faixa deposita o que está ABAIXO do plano, então
   ele tem de estar ACIMA da superfície. Com o sinal trocado o portão fecha em
   todo vértice e sobra só o primeiro dab — dois gates mediam um disco contra
   outro e reportavam `0,6000 → 0,6000`.
2. ⚠️ **Sem caminho a faixa nasce REDONDA, não deixa de nascer.** A 1ª versão
   caía no `Footprint::Disc`, que **não tem portão de profundidade**: o toque
   depositava `0,039998` onde a lei manda **zero**. O caminho decide a
   ORIENTAÇÃO da caixa e mais nada; a profundidade é fato do PLANO, que existe
   desde o primeiro dab. Numa ponta redonda a moldura no plano é **irrelevante
   por construção**, então escolher um perpendicular qualquer não é inventar
   orientação — é escolher entre respostas iguais.
3. ⚠️ **A ferramenta nascia MORTA**, e quatro varreduras da suíte o disseram ao
   mesmo tempo (o alpha, o invert, os dois do aplicador), cada uma com *"dab
   inerte"*. Com o plano rente, `z = 0` em toda parte. ⇒ **`STRIP_PLANE_FRACTION
   = 0.5`**, e o número sai da própria lei: o pico da parábola está a meio raio
   abaixo do plano, então erguê-lo por meia fração põe o pico exatamente na
   superfície em repouso. O `plane_offset` do artista **soma** a este.
4. ⚠️ **Uma CAIXA não cabe no círculo que a inscreve.** O canto de uma faixa
   `1 × L` está a `√(1 + L²)` raios do centro ⇒ o `query_radius` cresce, senão a
   tira chega com as **quinas comidas** e o defeito é mudo.
5. ⚠️ **O shell tinha uma segunda cópia da contagem de verbos** (`[RefMode; 16]`
   literal em dois arquivos, onde a crate deriva de `Verb::ALL.len()`). Ela
   sobreviveu enquanto o catálogo não crescia e virou erro de tipo no dia em que
   cresceu.

**O `S` fica SILENCIOSO na faixa**, pela mesma frase que o `Sharpen` já
carregava: *o SculptGL não a tem*. O censo veio dizer o número novo — e ⚠️ **ele
não se moveu** (15, porque entraram um verbo e um `None` ao mesmo tempo), o que
é precisamente por que a coincidência ficou escrita ali; o `B` foi a **17**,
porque o `alpha = root_alpha²` é o funil de toda tool do Blender.

**Gates:** 8 na forma (`footprint_tests`) + 7 no produto (`verb_strip_tests`).
**7 mutações, 7 sangram** — ⚠️ e **duas delas escreveram gates**: *"sem portão de
profundidade"* passava por cinco gates de produto (nenhum falava de
profundidade), e *"o espelho não espelha o caminho"* passava pelos 213 testes
(nenhum traçava uma faixa **na diagonal** sob simetria — ao longo de um eixo o
espelho preserva a direção e a metade espelhada acerta por acidente).

**LOC por corte de RESPONSABILIDADE:** o catálogo de verbos sai do `brush.rs`
para `brush_verb.rs` (*que OPERAÇÃO* × *que pincel a carrega* — as duas crescem
por razões diferentes) e a expansão do espelho sai do `stroke.rs` para
`stroke_symmetry.rs` (*quantas vezes um dab acontece* × *o que ele faz a um
vértice*).

Sem schema, sem ADR, sem crate nova, sem dep nova; os ids dos verbos são
`hash_node_id` e o array cresceu de 16 para 17 com o gate a cobrar.

⛔ **1º SMOKE REPROVOU: *"parece redondo"* (Enio) — e ele estava certo, com o §0
a morder em casa.** O default que eu shipei era `tip_roundness = 1.0`, e nessa
ponta a caixa arredondada **É** a distância euclidiana: a faixa saía disco.

⚠️ **O número tinha fonte, e era a fonte ERRADA.** `DNA_brush_types.h:264` diz
`tip_roundness = 1.0` — e esse é o default do pincel **GENÉRICO**, não o desta
tool; o que declara por-tool é o `BKE_brush_sculpt_reset`, que **não está no
clone** (a §7.1 outra vez). Eu peguei o único número citável e deixei-o definir o
produto — *nunca deixe o fallback definir o produto*, na forma mais literal
possível.

⚠️ **E a byte-identidade nunca dependeu dele.** Quem a carrega é a
[`Footprint::Disc`], que é a rota dos outros dezasseis verbos; a faixa é nova e
não tem mundo anterior a preservar. Eu confundi *a âncora da MÁQUINA* com *o
default do PRODUTO*.

**A propriedade que decide foi MEDIDA** — um dab redondo arrastado deixa uma
LENTE (afina nas pontas), um de quina reta deixa uma tira de **lados
paralelos**. Largura do depósito em sete secções ao longo do caminho:

| roundness | larguras | ponta ÷ meio |
|---|---|---|
| 0,00 | `0,8` nas sete | **1,00** |
| 0,25 | `0,7` nas sete | **1,00** |
| 1,00 | `0,5 0,5 0,6 0,6 0,6 0,5 0,5` | **0,83** |

⇒ **`tip_roundness = 0.25`, declarado como NOSSO**: mesmo lado paralelo da quina
viva, o maior platô da varredura (**23,7 %** dos vértices movidos contra 16,3 %
em `0`), e a quina ainda arredondada o bastante para não virar degrau numa malha
grossa. O `strip_length` fica em **1,0** — e a medição diz que é o certo: *a tira
nasce do TRAÇO*, é a quina reta que faz os lados paralelos, e o esticão é um
segundo eixo de estilo.

⚠️ **E o default tinha passado por 214 gates.** Todas as fixtures deste arquivo
passavam a dureza **explícita**, então nenhuma o exercitava — *um default só é
testado por um teste que não o menciona*, a lei que a wave do Mirror já tinha
escrito noutro módulo. O gate novo
(`the_shipped_default_lays_a_strip_with_parallel_sides`) mede a LARGURA, não a
altura, e traz o CONTROLE da ponta redonda ao lado.

**E os dois knobs ganharam ROW** (`Tip roundness` · `Strip length`, Pro,
oferecidos pela MESMA porta que o motor pergunta). ⚠️ **Duas mutações
sobreviveram antes do gate delas existir** — tirar as rows da tabela e alargar o
`show` para `always` deixavam a suíte do painel inteira verde: um motor com knobs
inalcançáveis, e dois sliders mortos em dezasseis das dezassete ferramentas.
⚠️ **O piso do `strip_length` é `1` e não `0`**, porque abaixo de um a pegada
seria mais curta que larga — e porque o motor **recusa** `0`: *um slider que
alcança um valor que o motor recusa é um controle que mente*.

⚠️ **PENDENTE DE RE-SMOKE — `PH2D_SCULPT3D_SMOKE=29`.** As perguntas de olho: a
tira tem **topo chato e lados paralelos** onde o Draw faz domo · ela **acompanha
uma curva em S** · um **toque** sai redondo (a decisão, não o bug) · não há
**canto comido** no fim de uma tira · e os dois knobs novos aparecem em **Pro com
a faixa em mãos**, e em nenhum outro verbo.

**Aberto na W6:** o **Multiplane Scrape** e o **Clay Thumb** reusam esta moldura;
o **Blob** é o Crease com o pinch invertido; o **Draw Sharp** segue bloqueado
pela tabela ausente (§7.18).

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
