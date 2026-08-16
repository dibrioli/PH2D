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
| **Inflate** | normal viva, int. 0,30 | normal viva, pressão assimétrica 0,25/0,125 | ⛔ **RECUSADO por medição** — a *normal de curvatura média por cotangentes* diverge **0,003°** da que já shipa na malha default, e onde diverge (p95 **87,9°**) ela deixa de ser uma normal; §7.28 | **2** |
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
| ~~**W1**~~ | ⛔ **SEM CURA EM CÓDIGO** (trocou de lugar com a W3 — §7.1; o *porquê* medido está na **§7.0**) | o perfil `B` de DEFAULTS não é construível: ⚠️ **não é o clone** — a partir do 4.3 o Blender tirou os defaults por-ferramenta do C e os pôs num **brush asset binário**. Vira decisão de produto, não pendência | — |
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

⚠️ **ADR escrito e aceito: [ADR-0159](../architecture/decisions/0159-sculpt3d-the-dab-vertex-loop-is-a-row-disjoint-map-rayon-exception.md)**
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

A **paralelização do laço de vértices** (1,6×) tem o [ADR-0159](../architecture/decisions/0159-sculpt3d-the-dab-vertex-loop-is-a-row-disjoint-map-rayon-exception.md)
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
exatamente aí que a §7.1 morde de novo — ⚠️ **mas NÃO pelo motivo que esta
frase dizia.** Ela foi escrita como *"o `BKE_brush_sculpt_reset` continua fora do
clone"*, e isso **contradiz a §7.0, escrita dois dias ANTES**: a função não está
fora do clone, ela **deixou de existir em C** (Blender 4.3+ guarda os defaults
por-tool num `.blend` de assets, binário — a medição inteira está lá, incluindo a
ausência do `DNA_brush_defaults.h`). *Uma frase corrigida num parágrafo não se
corrige sozinha no parágrafo seguinte.* A **conclusão** sobrevive intacta:
escrever *"o Draw Sharp nasce com a curva Sharp"* seria inventar um número e
shipá-lo com a autoridade de uma referência que não o declara — o que o §4 proíbe.

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
tool; o que declarava por-tool era o `BKE_brush_sculpt_reset`, que **não existe
mais em C** (a §7.0). Eu peguei o único número citável e deixei-o definir o
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

**Aberto na W6** *(fechado — §7.26 e §7.27)*: o **Multiplane Scrape** e o **Clay Thumb** reusam esta moldura;
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

---

### §7.20 — ✅ A FAIXA NIVELA: o lift do plano decide se ela FECHA ou EXAGERA o relevo (2026-08-15)

**Report do Enio, na 2ª rodada de smoke da W6:** *"A forma está correta,
retangular, mas o comportamento do tool não está idêntico. Num vale a tool
correta tende a fechar o vale, mas na nossa implementação tende a aumentar o
vale."*

**Reproduzido antes de qualquer hipótese** (`tests/measure_valley.rs`: um vale
liso de `0,40` de profundidade, pincel `r = 0,8`, nove dabs):

| | profundidade | veredito |
|---|---|---|
| repouso | 0,4000 | — |
| a faixa que shipava | **0,4269** | **AUMENTA (+0,0269)** |
| o Draw, no mesmo traço | 0,1572 | fecha |

E a crista subia **+0,0716** contra apenas **+0,0447** do chão: o depósito caía
nos ombros, não no fundo.

#### O mecanismo — e a lei do dab NÃO era o defeito

O `clay_strips.cc` calcula `offset = plane_normal · bstrength · radius` e
`translation = offset · factor`, com o `factor` a incluir `z·(1−z)` — **a mesma
forma que a nossa** (`add(live, n_area, reach·w)` com o portão da
[`Footprint::Strip`] dentro do `w`). O que difere é **onde o plano fica**.

A parábola `z·(1−z)` **sobe** de `z = 0` (o plano) até `z = 0,5` e **desce**
depois. Logo o depósito só cresce com a profundidade enquanto o ponto está a
menos de meio raio abaixo do plano. Como a superfície em repouso fica a `lift`
raios abaixo do plano, a faixa **enche** relevo até `(0,5 − lift)` raios abaixo
da média e **exagera** o que passa disso.

⚠️ **`STRIP_PLANE_FRACTION = 0,5` punha a superfície EXATAMENTE no pico — folga
de enchimento ZERO.** Era o único valor da família em que nenhum vale enche.

⚠️ **E o defeito era meu, pela mesma via da `tip_roundness` de ontem:** eu
derivei `0,5` de *"pôr o pico na superfície em repouso"*, que é conveniência
interna (o máximo depósito em chapa plana, e a cura do *"dab inerte"* que quatro
varreduras da suíte reportaram), **não** da propriedade que decide.

#### As duas propriedades puxam em sentidos opostos, e são a MESMA lei

| lift | vale (Δ) | miolo ÷ aro numa CÚPULA |
|---|---|---|
| 0,10 | −0,212 | **0,009** (o miolo não recebe nada) |
| 0,18 | −0,121 | 0,393 |
| **0,25** | **−0,073** | **0,649** |
| 0,30 | −0,048 | 0,756 |
| 0,50 | **+0,027** | 0,971 |

⚠️ **O "anel" da coluna direita NÃO é falha** — numa cúpula o miolo da pegada
está acima do plano ajustado e o aro abaixo, então nivelar **é** depositar mais
no aro: é o *"displaces vertices toward the brush plane"* do doc-header da
referência, e é o que faz a ferramenta cortar planos. O que o lift baixo produz
de errado é o miolo ficar **vazio** — o artista aponta e nada acontece ali.

#### O número, e por que ele não é citável

⚠️ **`0,25` é o MEIO da subida da parábola** (plano em `z = 0`, pico em
`z = 0,5`): folga igual para acrescentar num calombo e para encher um buraco. É
um marco da própria lei, e as duas medições o põem longe de qualquer extremo.

⚠️ **NÃO é citável da referência.** O `clay_strips.cc` lê `brush.plane_offset` e
o genérico do DNA é `0.0`; quem declara o valor por-tool é o
`BKE_brush_sculpt_reset`, que **não existe mais em C** (§7.0) — a lacuna que já
bloqueou a W1 e o Draw Sharp. O número é NOSSO, com a tabela ao lado (§4).

#### `STRIP_DEPTH_GAIN` — o lift é forma, não força

O portão vale `lift·(1 − lift)` na superfície em repouso, então baixar o lift
emagreceria a banda **junto** com a mudança de forma, e um smoke que muda duas
coisas ao mesmo tempo não é legível. O ganho `0,25 / (lift·(1 − lift))` repõe o
pico onde a superfície está: chapa plana **0,0876 → 0,0823** (94 %, o resto é o
re-ajuste vivo do plano). ⚠️ **DERIVADO da const, nunca um literal ao lado** —
escrito à mão apodrece no dia em que o lift se mover, e a ferramenta mudaria de
força em silêncio.

#### Gates — três propriedades para UMA constante

`the_strip_closes_a_valley_instead_of_deepening_it` (nasceu VERMELHO em
`0,4269`) · `the_band_still_lands_under_the_cursor_on_a_convex_form` (o
contra-peso, sem o qual "fecha o vale" é maximizado levando o lift a zero) ·
`moving_the_plane_lift_does_not_change_how_much_the_strip_lays_on_flat_clay`.
**3 mutações, 3 sangram**, cada uma no seu gate: lift `0,5` → o do vale · lift
`0,05` → o da cúpula · ganho `1,0` → o da magnitude. Um gate só deixaria as
outras duas livres para regredir em silêncio — foi assim que o `0,5` shipou.

⚠️ **O oráculo do vale é a PROFUNDIDADE, não a altura do chão:** uma passada que
levantasse a paisagem inteira subiria o chão sem nivelar nada, e é exatamente o
que o `0,5` fazia.

⚠️ **`measure_brush_kernel` é flake de CARGA** — falha na suíte cheia e passa
isolado (três corridas seguidas); ele mede uma RAZÃO de wall-clock com os 32
núcleos saturados. Re-rode sozinho antes de suspeitar de um merge.

⚠️ **PENDENTE DE RE-SMOKE — `PH2D_SCULPT3D_SMOKE=29`.** As perguntas de olho:
passe a faixa sobre um **vale** e ele tem de **fechar** · a banda continua a
aparecer **sob o cursor** numa forma convexa (mais fina no miolo é a ferramenta
a cortar plano, vazia é bug) · e a **espessura em barro liso não pode ter
mudado** — é o que o ganho existe para segurar, e é o controle que torna o
resto legível.

---

### §7.21 — ✅ A FAIXA É UMA TOOL DO BLENDER, e estava a correr a lei do SculptGL (2026-08-15)

**Report do Enio, com foto:** *"errado. estude os códigos referência na fonte"* —
lâminas finas a atravessar a silhueta de um membro.

⚠️ **O §7.20 (o lift do plano) estava certo e era INSUFICIENTE.** Esta é outra
causa, no mesmo verbo, e só a leitura da fonte a encontra.

#### O que a fonte diz, e o que ela REFUTA

`clay_strips.cc::calc_faces` monta o fator em treze passos; o **terceiro** é
`calc_front_face`, que faz `factors[i] *= max(dot(view_normal, n), 0)`.

E o `Brush.js:32-34` do SculptGL:

```js
var iVertsFront = this.getFrontVertices(iVertsInRadius, picking.getEyeDirection());
if (this._culling) iVertsInRadius = iVertsFront;   // _culling nasce DESLIGADO
```

⚠️ **A fonte REFUTOU a minha primeira suspeita**, e é bom que a tenha lido: eu ia
declarar que o nosso `RefMode::S` estava mal portado por ignorar o front-face no
depósito. **Não está** — no SculptGL o filtro só alcança o depósito com o
*culling* ligado, e alimenta **sempre** o `areaNormal`/`areaCenter`, que é
exatamente o que o `fit_plane_over` faz. O port do `S` é fiel.

#### A causa: uma referência a governar uma ferramenta que ela não tem

O `Brush::default()` shipa em `RefMode::S`, e o `declares()` devolvia `S => true`
para **todos** os verbos — inclusive o `ClayStrips`, que **o SculptGL não tem**.

⚠️ **As duas tabelas do `RefMode` discordavam, e só uma estava certa:** a de
DEFAULTS já devolvia `None` (o censo `the_census_of_offered_chips` escreve
*"todos menos o Sharpen e o Clay Strips"*), enquanto a da LEI dizia que o `S`
governava tudo. A faixa corria com `front_face: Ignored` — fiel ao `Brush.js`, e
simplesmente **não é a lei desta ferramenta**.

**MEDIDO** (`measure_whether_the_strip_deposits_on_back_facing_clay`, olho
RASANTE, que é a situação da foto):

| modo | depósito FRENTE | depósito COSTAS | costas ÷ frente |
|---|---|---|---|
| `S` (o que shipava) | 17,18 | **39,86** | **2,32** |
| `B` | 1,21 | 0,00 | 0,00 |

⇒ **a faixa punha mais do dobro do barro em geometria que o artista não vê**, e
perto da silhueta isso empurra a superfície de trás para FORA do contorno: é a
barbatana da foto. Depois do fix: **0,0000**.

#### A cura — a lei é perguntada por VERBO

`RefMode::S::declares(ClayStrips)` passa a ser `false` (as duas tabelas
concordam) e nasce **`RefMode::kernel_for(verb)`**: um verbo que o modo não
declara cai na lei da referência que o **TEM**.

⚠️ **`kernel()` continua público e responde outra pergunta** — *que lei este modo
declara* —, que é o que os gates comparam entre si. Quem esculpe pergunta por
verbo.

⚠️ **E o painel deixou de carimbar o default em todo verbo:** o
`mode_by_verb` nascia `[RefMode::default(); 17]`, o que deixaria a faixa com um
chip que o painel não oferece — **nenhum aceso**, a cicatriz da Faca do Painter.
Agora cada verbo abre no primeiro modo que o declara, **derivado** do
`offered_for`.

#### O que a medição descartou pelo caminho

⚠️ **Hipótese REFUTADA — o plano vivo NÃO produz crescimento sem limite.** O
`calc_area_normal_and_center_node_mesh` ramifica em `!ss.cache->accum` e lê o
pen-down congelado com o Accumulate desligado, e eu esperava runaway linear. A
sonda diz **1,47× para 3× os dabs** (sub-linear): a chapa em volta ancora o
ajuste. A divergência com a referência fica NOMEADA e **não** foi tocada.

⚠️ **E o perfil da banda é uma MESA de parede vertical** (`0,000 → 0,026 →
0,046` em duas células, platô, e desce igual), consequência do
`tip_roundness = 0,25` que a §7.19 pôs medindo *lados paralelos em planta* e
**nunca o perfil**. Numa laje de blocagem isso é o que a ferramenta é — a mesma
cadeia da referência produz a mesma mesa —, então fica **medido e não mexido**:
é decisão de LOOK, e o próximo smoke a julga sabendo que o número existe.

#### Gates

`the_strip_does_not_lay_clay_on_what_the_artist_cannot_see` (produto, olho
rasante, costas < 1% da frente) · `sculptgl_does_not_declare_the_strip_so_it_does
_not_govern_it` (as duas tabelas concordam, o chip oferecido é UM, e o
**CONTROLE**: o Draw é do SculptGL e a lei dele não pode ter mudado). **2
mutações, 2 sangram os DOIS gates** (o `S` voltar a declarar a faixa · o
`kernel_for` ignorar o verbo).

⚠️ **PENDENTE DE RE-SMOKE — `PH2D_SCULPT3D_SMOKE=29`.** A pergunta de olho é a da
foto: passe a faixa **perto da silhueta** de uma forma curva e nada pode aparecer
do outro lado do contorno.

---

### §7.22 — ✅ A FAIXA ESTAVA 7,5× MAIS FRACA: o `reach` também era do SculptGL (2026-08-15)

**Report do Enio, com três fotos:** o nosso traço sobre a esfera (estrias macias
que ACOMPANHAM a forma) e, ao lado, **o Blender** — uma fila de **placas chatas e
distintas** que CORTA a esfera em degraus.

⚠️ **É a §7.21 uma camada abaixo, e eu não a segui até ao fim.** O deslocamento
de um dab é `Brush::reach`:

```rust
radius * REACH_FRACTION * s      // REACH_FRACTION = 0,1
```

e o `0,1` é o `deform = intensidade · raio · 0,1` do **`Brush.js`** — do
SculptGL, que **não tem esta ferramenta**. O `clay_strips.cc:327` diz:

```cpp
const float3 offset = plane_normal * ss.cache->bstrength * ss.cache->radius;
```

**`raio · força`, fração `1,0`.** A faixa vinha **7,5× mais fraca por dab** que a
referência — daí estria macia em vez de placa.

#### O que a magnitude certa DESTRAVA (medido, e é mais do que velocidade)

| | com `0,1` | com `1,0` (a referência) |
|---|---|---|
| 1 / 3 / 9 / 27 / 81 dabs (chapa plana) | 0,006 → 0,116 | 0,047 → **0,592** |
| crescimento 27÷9 | 1,47× | **1,20×** (81÷27 = 1,04×) |
| vale de 0,40, `r = 0,5`, 9 dabs | 0,331 | **0,041** |
| razão altura ÷ largura da banda | 0,080 | **0,355** |

⚠️ **A saturação NASCEU com a magnitude certa.** Com o `reach` fraco a faixa
nunca alcançava o plano, então a parábola nunca fechava e a ferramenta parecia
crescer devagar para sempre; com `raio · força` o barro **sobe até o plano e
PARA** — o auto-limite que é a assinatura de um clay strip. Dez vezes o `reach`
dá só **2,4×** de altura, e essa não-linearidade É a ferramenta.

#### `STRIP_DEPTH_GAIN` MORREU

Ele existia (por umas horas, §7.20) para preservar a magnitude ao mover o lift —
**e a magnitude que ele preservava era ela própria errada**. Com a fração da
referência, a cadeia é a dela ponta a ponta: `offset = raio · força`, portão
`z·(1−z)` **cru**, sem termo de calibração. O gate
`moving_the_plane_lift_does_not_change_how_much_the_strip_lays_on_flat_clay` saiu
com a const, e o pico do portão voltou ao `0,25` da referência.

#### ⚠️ E a magnitude certa EXPÔS uma assimetria que o traço fraco escondia

O `the_mirrored_copy_lays_its_strip_along_its_own_path` media `0,63 %` e passou a
medir **5,16 %** contra uma barra de 5. **Não foi afrouxada — foi decomposta:**

| traço começa em | divergência |
|---|---|
| `x = 0,5` (as metades TOCAM-SE) | 5,28 % |
| `x = 1,1` (separadas) | **1,77 %** |

O raio de CONSULTA da faixa é `√(1 + L²)·r = 0,566`, então um traço a começar em
`x = 0,5` faz os dois passes da simetria tocarem os mesmos vértices perto do
eixo — e o segundo ajusta o plano sobre a superfície que o primeiro levantou.
**Dois terços da divergência eram a sobreposição, não o espelhamento do caminho,
que é o que o gate afirma** ⇒ a fixture saiu da zona de contacto e a barra ficou
onde estava.

⚠️ **E o residual tem DONO nomeado:** o nosso `fit_plane` lê a superfície VIVA,
enquanto o `sculpt.cc::calc_area_normal_and_center_node_mesh` ramifica em
`!ss.cache->accum` e lê o **pen-down congelado**. Sob o plano congelado os dois
passes de simetria seriam idênticos **por construção**. É a mesma divergência que
a §7.21 já tinha medido e deixado quieta (o crescimento sub-linear), agora com um
segundo sintoma — e é a candidata natural à próxima wave.

#### E a magnitude sozinha não bastava: o AUTO-LIMITE tem TRÊS metades

⚠️ **A mutação do `STRIP_REACH_FRACTION` SOBREVIVEU aos 219 gates** — devolver à
faixa o `0,1` deixava tudo verde. O número que fazia a ferramenta parecer certa
não era afirmado por ninguém, e escrever o gate expôs que faltavam mais duas
peças da referência:

| | nós (antes) | `clay_strips.cc` / `sculpt.cc` |
|---|---|---|
| deslocamento | `raio · 0,1` (`Brush.js`) | **`raio · força`** |
| posição do portão | CONGELADA (`from_live = accumulate`, a lei do *Stamp*) | **VIVA** (`position_data.eval`) |
| fonte do plano | VIVA | **CONGELADA** com `!accum` |

⚠️ **É a combinação que dá o auto-limite**, e nenhuma das três sozinha o dá:
posição viva contra plano congelado faz o `z` do portão `z·(1−z)` **encolher** à
medida que o barro sobe, até fechar. Medido, o pico pousa em **`0,1000` contra um
plano em `0,1000`** e lá fica (3 · 9 · 27 dabs: `0,1000` · `0,1002` · `0,1006`).
Com o Accumulate LIGADO ele constrói (`0,146` · `0,271` · `0,319`) — **o
interruptor passou a significar alguma coisa**: antes media `0,058` contra
`0,051`.

As duas portas novas espelham o `kernel_for`: **`Verb::grip_law`** (a lei que
governa este verbo, contra `Grip::law`, que é a lei do grip) e o ramo por-verbo
do `fit_plane`. Os quatro verbos de plano do SculptGL **não mudam** — o
`areaNormal`/`areaCenter` deles lê o vivo e é o que a fonte diz.

#### ⚠️ E uma PREMISSA não declarada invalidou uma sessão de medições minhas

`Brush::default().accumulate` está preso ao default do **Draw** (`true`), mas o
da faixa é **`false`**. Toda fixture `Brush { verb: ClayStrips, ..default() }`
— as minhas sondas e os gates — corria com o Accumulate **LIGADO**, que não é o
que a ferramenta shipa, e é justamente o interruptor que escolhe a fonte do
plano. A premissa passou a ser **escrita** no `strip_brush`.

⚠️ **E a fixture do gate de saturação variava DUAS coisas:** aumentar a contagem
de dabs aumentava junto o COMPRIMENTO do traço, e a 81 dabs ele media `4,8`
contra uma chapa de `1,5` — saía da malha, e o `1,67×` lia como *"não saturou"*.
O caminho é FIXO e o que muda é só quão fino ele é amostrado: é a lei que o
relevo do Painter já pagou quatro vezes.

**3 mutações, 3 sangram**, uma por metade da lei (o `reach` · o plano vivo · o
portão congelado).

⚠️ **PENDENTE DE RE-SMOKE — `PH2D_SCULPT3D_SMOKE=29`.** A pergunta de olho é a
comparação que o Enio mandou fazer: a faixa tem de deixar **placas** que cortam a
forma, como no Blender, e não estrias que a acompanham — e uma segunda passada
por cima da primeira **quase não pode subir mais** (é o barro a parar no plano);
ligar o Accumulate é que a faz construir.

✅ **SMOKE APROVADO (Enio, 2026-08-15): *"smoke OK"*.** A faixa fechou.

### §7.23 — ✅ W6 (metade B): O BLOB, o irmão do Crease com o aperto invertido (2026-08-15)

O 2º dos três itens que a §7.19 deixou nomeados. `Verb::Blob` — o **18º** verbo,
e o primeiro cuja definição inteira é *um vizinho com um sinal trocado*.

**A relação é a do Blender ao pé da letra:** o `crease.cc` tem UMA função
(`do_crease_or_blob_brush`) e um `bool invert_strength` que troca o sinal do
termo LATERAL e mais nada — o `offset` normal dos dois é o mesmo
`sculpt_normal · raio · força`.

#### Por que é um VERBO e não um slider negativo no `pinch`

⚠️ **O nosso próprio catálogo já decidiu esta pergunta uma vez, e não é gosto:**
[`Verb::Pinch`] e [`Verb::Magnify`] são exatamente o mesmo kernel com um sinal, e
são **dois chips**. Um `pinch` que alcançasse negativo seria a segunda resposta a
*"como o artista pede o oposto?"*.

#### ⚠️ A DIREÇÃO do depósito é NOSSA, e a §4 é o motivo

O nosso Crease **cava** por default porque herda o `_negative = true` do
`Crease.js`. O SculptGL **não tem** Blob, então não há `_negative` a herdar — e
inventar um com a autoridade de uma referência que não o declara é precisamente o
que a §4 proíbe. ⇒ a direção é escolha nossa, e é a que o NOME diz: um *blob* é
um monte, então ele **SOBE**. O `Ctrl` dá o oposto de cada verbo, como em toda a
família.

⚠️ **Os DOIS sinais mudam, e a simetria não é estética:** negar só o lateral daria
um monte que ainda cava; negar só o normal daria um Crease erguido, que é o que o
`Ctrl` no Crease já entrega. É a COMBINAÇÃO que nenhum ajuste do vizinho alcança,
e é ela que o torna um verbo em vez de um flag — o gate
`the_blob_is_not_the_inverted_crease` afirma exatamente isso, medindo o RADIAL
(`+2,616` contra `−2,688`) onde a ALTURA é idêntica nos dois (`+0,028`).

**O `S` fica silencioso** (a 3ª vez, depois do Sharpen e da faixa) e o `B`
governa — o `crease.cc` é do Blender. O `L` é oferecido: a `F` de traço zero do
[`crate::kelvinlet::pinch`] cobre os dois sinais, então o Blob é o **segundo
verbo COMPOSTO** e herda as três armadilhas do Crease inteiras.

**Gates:** 6 próprios (`verb_blob_tests.rs`) + 1 de forma do `l-mode`
(`the_blobs_dome_stays_narrow_while_the_push_reaches_out`). **4 mutações, 4
sangram.**

#### ⚠️ Um oráculo meu estava ERRADO, com o gate VERDE

O gate da largura comparava o domo com o **EMPURRÃO do próprio Blob**. O termo
lateral é `centro − posição`, que vale **zero no eixo**: o empurrão é um **ANEL**,
com o pico fora do centro. *Um domo é mais estreito que um anel para QUALQUER
expoente* — a mutação `shape⁴ → shape` passava (razão 2,911× → 1,881×, contra uma
barra de 1,5). O oráculo virou o **Draw como controle**, o espelho exato do gate
do Crease, e a mesma mutação sangra (2,128× → 1,375×).

⇒ **Duas grandezas de FORMA diferente não se comparam por meia-largura.**

#### ⚠️ E o censo do `S` mediu 15 antes e depois — pela SEGUNDA vez seguida

O comentário dele já registrava a coincidência da wave da faixa (*"um verbo e um
`None` ao mesmo tempo"*). Ela **reincidiu**: o Blob acrescenta um verbo E uma
exclusão. ⇒ *um censo de CONTAGEM não é um censo de CONTEÚDO* — quem se move é o
`B` (17 → 18) e quem diria o resto é a lista por NOME do gate da literatura.

#### ⚠️ E a wave achou TRÊS vermelhos-latentes da §7.22, todos da mesma causa

Nenhum é desta metade, e os três só apareceram porque um commit tocou os ids do
painel e a `ph2d-editor-core` entrou na varredura impactada — a mesma causa
estrutural que a `line/physics`, a `line/Vector` e a `line/motion-value` já
documentaram: **um fechamento por `cargo test -p <crate>` não alcança
`crates/ph2d-editor-core/tests/` nem `shells/desktop/tests/`.**

| vermelho | o que era |
|---|---|
| `verb_strip_tests.rs` **746 > 700** | o teto de LOC, e eu **reportei-o como verde** no fim da sessão anterior |
| `rows.rs:196` `max: 4.0` | um literal sem o marcador `LITERAL-PX-OK` |
| `every_verb_is_reachable_from_the_keyboard` | o gate lê `brush.rs` procurando `impl Verb {`, e a §7.19 **mudou o catálogo para `brush_verb.rs`** — ele morria no `expect` |
| `the_grab_holds_its_footprint_instead_of_re_picking` | o `63c856aa4` coalesceu o puxão por QUADRO, e o gate exigia `grab_at(` DENTRO do braço `Grip::Hold` |

⚠️ **E a PREMISSA do gate do teclado tinha EXPIRADO.** A mensagem dele dizia *"o
artista não consegue pegá-lo"* — verdade quando a cena 3D não tinha painel, e
**falsa desde a W10.7**, cujo `every_verb_has_a_chip_that_selects_it` garante um
chip por verbo. A ausência de tecla deixou de ser *inalcançável* e passou a ser
*sem atalho*; quem move o número que tornava algo inalcançável tem de reconferir
a nota, e ninguém reconferiu esta.

⇒ O gate passou a afirmar *tecla **OU** isenção NOMEADA*, com controle nos dois
lados (um nome que saiu do catálogo · um verbo que GANHOU tecla e ficou na lista).
⚠️ **E o teclado ACABOU, medido:** os dez dígitos estão tomados e das 26 letras só
`L` e `W` sobram — o `W` é a tecla do painel de física no app inteiro. Dar um
mnemônico fraco a um dos dois e deixar o outro sem seria pior que a ausência
nomeada. **A faixa e o Blob shipam chip-only, e a escolha de atalho é do Enio.**

**LOC:** o corte do `verb_strip_tests.rs` é por ASSUNTO — a **FORMA** da faixa
(silhueta, lados paralelos, quinas, toque redondo; grade PLANA) fica no pai, e a
**LEI** (o plano erguido, o vale, o auto-limite, a referência que governa;
superfície MOLDADA) vai para `verb_strip_law_tests.rs`. 423 + 343.

⚠️ **PENDENTE DE SMOKE.** A pergunta de olho: com o **Blob** em mãos e o `pinch`
alto, a passada tem de deixar um **monte REDONDO** onde o Crease deixa um sulco
afiado — e o `Ctrl` tem de dar o oposto de cada um (poço redondo · crista afiada).
Os dois chips ficam lado a lado no painel; **o `S` não é oferecido para o Blob**.

**Aberto na W6** *(fechado — §7.26 e §7.27)*: o **Multiplane Scrape** e o **Clay Thumb** reusam a moldura da
faixa; o **Draw Sharp** segue bloqueado pela tabela ausente (§7.18).

---

### §7.24 — ✅ O REPORT DA FAMÍLIA QUE APERTA: o `l-mode` sai, e o `B` do Pinch passa a ser o `pinch.cc` (2026-08-15)

Report do Enio, três frases, sobre o Blob que a §7.23 acabou de entregar e sobre
os dois vizinhos dele:

> *"Blob modo B bom! Blob modo L ruim. Pinch em B e S bons mas idênticos ou quase
> idênticos. Em L Pinch ruim. Crease OK."*

⚠️ **São DOIS defeitos com mecanismos independentes, e a medição achou os dois
antes de uma linha ser escrita** — sonda `measure_pinch_family_modes`, que dirige
`SculptStroke::dab` (a porta do artista), malha 64×96, `r = 0,30`, traço de oito
eventos a força `0,75`.

#### (A) O `l-mode` — 62 % do gesto caía FORA do anel do cursor

| verbo | modo | fora do anel | ΔV/V (10⁻⁴) | pico |
|---|---|---|---|---|
| Pinch | S | 0,0 % | −0,92 | 0,1027 |
| Pinch | B | 0,0 % | −1,41 | 0,0758 |
| **Pinch** | **L** | **62,4 %** | **−4,43** | 0,0514 |
| Crease | S | 0,0 % | −9,50 | 0,3309 |
| **Crease** | **L** | **43,7 %** | −11,48 | 0,3374 |
| Blob | B | 0,0 % | +10,52 | 0,2892 |
| **Blob** | **L** | **46,5 %** | +11,95 | 0,2906 |

O `KELVINLET_REACH = 3` é a **feature** do verbo que AGARRA — o doc dele nomeia o
preço em voz alta (*"o anel do cursor deixa de significar o que eu toco"*) — e é
o **defeito** de um verbo que aperta, que é local por definição.

⚠️ **E o campo PIORAVA justamente o que existia para curar.** A nota do
`Verb::Pinch` afirmava *"com campo ele deixa de REMOVER VOLUME … o que sai de
lado sai pela normal: aperta E espirra"*. Medido: ele remove **4,8× mais** volume
que o `s-mode`, e **dentro do anel o deslocamento normal é NEGATIVO** (−0,00078
na banda 0,5-0,75 r contra um lateral de +0,00761) — afunda, não espirra. O
mecanismo é geometria: o traço zero reparte `+s` na normal e `−s/2` no plano, mas
os vértices de uma MALHA vivem na superfície (`r · n ≈ 0`), então o termo normal
é ~zero. **Uma casca não tem material fora do plano para receber o que sai de
lado.**

⛔ **E não há corte honesto que o localize:** o perfil lateral é quase CHATO até o
anel (0,00304 · 0,00649 · 0,00761 · 0,00666) e ainda vale **88 % do pico** em
`1,0 r` — cortá-lo ali seria um degrau **trinta vezes** maior que os 2,90 % que o
`rim_landing` foi construído para curar.

⚠️ **A REFERÊNCIA fecha o argumento:** o `elastic_deform.cc` do Blender porta
**este mesmo paper** e declara cinco famílias — `GRAB`, `GRAB_BISCALE`,
`GRAB_TRISCALE`, `SCALE`, `TWIST`. **Nenhuma é o pinch.** O SculptGL não tem
Kelvinlets. O paper tem a família afim de traço zero como MATEMÁTICA e nenhum
escultor a shipa como PINCEL.

⇒ **O `Field::Pinch` foi retirado.** O `L` desaparece de Pinch/Crease/Blob **por
construção** (o `declares` pergunta `field(verb).is_some()`), e o censo da
literatura **encolheu pela primeira vez**: `["Smooth", "Magnify", "Move / Grab",
"Snake Hook", "Twist", "Local Scale"]`.

⚠️ **O Crease-L tinha a MESMA doença** (43,7 %) e o report diz *"Crease OK"* — o
Enio quase de certeza o testou em `S`/`B`, que são os modos limpos. **Ele sai
junto, e isto está aqui para o smoke poder discordar.**

#### (B) `B ≈ S` no Pinch — o chip `B` vestia a lei do `crease.cc`

Medido: a diferença é quase toda o `strength²`. Em força `1,00`, onde o `x²` é a
identidade, o que sobrava entre os dois era **0,0125 r no pior vértice (9 % do
pico)**, com a normal do `B` em `0,00000` exato contra `0,00129` do `S`. Dois
apertos radiais separados por um arredondamento.

⚠️ **A causa é uma leitura de fonte alheia feita pelo COMENTÁRIO e não pelo
código.** A nota do `LateralPull::Tangential` dizia *"isto NÃO é a lei do
Blender"* e descrevia o `pinch.cc` como *"a tangente ao longo do TRAÇO mais a
normal"* — direto do comentário de lá (*"the X vector (aligned to the stroke)"*),
**que é falso no próprio Blender**: o código monta `X = cross(area_no,
grab_delta)`, que é **perpendicular** ao traço. Lida a fonte, o mapa inverte-se:

| lei | remove | fonte |
|---|---|---|
| `Direct` | nada (3D cru) | `Pinch.js:52-58` · `Crease.js:59-61` |
| `Tangential` | a componente **NORMAL** | `crease.cc:112` — *"pinched towards a **line** instead of a single point"* |
| `AcrossStroke` | a componente **AO LONGO DO TRAÇO** | `pinch.cc:39-60` — *"the Y component is removed"* |

⇒ **Nós coincidíamos com o `crease.cc`** (e o `B` do Crease e do Blob estava
certo o tempo todo — é o que o Enio viu) e **faltava-nos o `pinch.cc`**.

**Com a lei certa em cada um:** o mesmo desvio em força `1,00` mede **`0,1342 r`
— 99 % do pico, 10,7× o que era**. Os dois chips deixaram de ser o mesmo aperto
com forças diferentes.

⚠️ **E a lei da referência responde de graça o que o `l-mode` tentava e
piorava.** Apertar para uma LINHA em vez de um PONTO não colapsa a vizinhança
radialmente, logo quase não remove volume: **`−0,0119` contra `−0,9066`** (10⁻⁴
de `V`), **76× menos** que o `s-mode`. O campo elástico existia para *"deixar de
remover volume"* e removia 4,8× mais; **a resposta estava na fonte o tempo
todo.**

⚠️ **E a nota que declarava isto bloqueado ENVELHECEU:** ela dizia *"fechar a
dele pede o frame do traço dentro do `Dab` — wave própria"*. O `Dab::path` chegou
na wave da FAIXA e ninguém reconferiu a nota. *Quem move o número que tornava
algo inalcançável tem de reconferir a nota* — a segunda vez nesta linha, depois
do gate do teclado da §7.23.

⚠️ **A lei lateral saiu do `KernelLaw` e virou `RefMode::lateral_for(verb)`**: os
outros dois eixos são fatos sobre o MODO, mas o Blender tem **duas ferramentas
nesta família e duas leis**, e um campo teria de responder duas coisas com um
valor. O `Magnify` fica no `Tangential` por AUSÊNCIA declarada — o Blender não
tem essa ferramenta.

#### ⚠️ Mudanças de comportamento, nomeadas

1. **O Pinch em `B` passa a ter componente normal** (o `z_disp` que o `pinch.cc`
   **guarda** de propósito). O nome do gate antigo — *"does not secretly
   flatten"* — era uma afirmação sobre a lei antiga; quem quer o aperto puro no
   plano tem o `S` ao lado no mesmo verbo.
2. **O Pinch em `B` recusa um dab sem direção.** É a referência
   (`pinch.cc:188-195`: *"delay the first daub because grab delta is not
   setup"*), e o preço é que um TAP solto neste modo não aperta. Inventar um eixo
   ali seria escolher uma direção que o artista não desenhou.
3. **O chip `L` some de Pinch, Crease e Blob.**

#### ⚠️ E um gate ficava VERDE sobre a afirmação falsa, por FIXTURE

O `the_elastic_pinch_gives_back_along_the_normal_what_it_takes_from_the_plane`
somava o deslocamento normal sobre a **esfera inteira** e lia `0,5043` contra
`0,1515` do `s-mode`. A decomposição por BANDA mostra que essa espirrada vive
**toda fora do anel** — dentro dele a normal é negativa. *Uma soma global disse o
contrário do que acontece sob o cursor*, a mesma doença que o Painter 2D pagou ao
medir a ondulação no EIXO do traço em vez do ombro.

⚠️ **E os outros dois EXIGIAM o defeito.** A mensagem de falha de um deles, com o
campo já removido, foi: *"o empurrão elástico não alcança além do anel — sem isso
o `l-mode` do Blob é um domo mais fraco e nada mais"*. Ele estava certo sobre o
mecanismo e errado sobre o veredito.

⇒ Os três foram substituídos por **dois**: `the_squeezing_family_stays_inside_the_cursor_ring`
(a propriedade que o artista vê) e `no_reference_declares_an_elastic_squeeze` (a
razão, que é de REFERÊNCIA e não de número — sem ela a primeira passaria no dia
em que alguém religasse o campo com um alcance menor).

#### ⚠️ E a minha fixture nova caiu no MESMO buraco, e o meu anti-vácuo pegou-a

A primeira versão do gate do anel usou o helper `stroke` do irmão, que carimba
sempre no mesmo centro — logo `path = 0` e o `B` do Pinch **recusa**. O
`assert!(inside > 1e-4)` disparou com *"a fixture não contém o fenômeno (0)"*
antes de eu ter olhado. **O anti-vácuo é a linha que separa um gate de um
carimbo**, e ele pagou-se no primeiro uso.

**5 mutações, 5 sangram**, cada gate com uma que só ele mata: religar o campo (4
gates) · o `B` voltar ao `crease.cc` (2) · o degenerado inventar um eixo (só o da
recusa) · o `remove_along` não remover (2) · o eixo ser o `path` cru (só o da
ortogonalização).

**Higiene:** `kelvinlet::pinch` e a cadeia afim inteira (`Mat3`, `mul`,
`raw_affine`, `affine_tip`, `affine_gain`, `affine`, `Affine`) ficam **congeladas
sob `cfg(test)`** — a `pinch` era a única consumidora de produção delas, e os
gates continuam a usá-las como ORÁCULO (o `twist` é verificado contra a forma
geral; o `affine_gain` é o que torna a degenerescência do `POISSON` uma asserção
em vez de prosa). O precedente é o `warp_axis` do Painter 2D: *um `pub fn` sem
chamador não é código morto silencioso, é uma segunda resposta à espera de que
alguém a chame.*

✅ **SMOKE OK (2026-08-15).** As três perguntas de olho, verificadas:
1. **Pinch em `B` contra `S`** — arrastado (não clicado), eles ficam
   VISIVELMENTE diferentes: o `B` faz um vinco ao longo do traço, o `S` um funil
   radial.
2. **Blob e Crease** — os chips `L` deles não existem mais; `B` e `S` (o Crease)
   continuam como estavam.
3. **O CONTROLE** — Grab, Snake Hook, Twist e Local Scale **mantêm** o `L`, e é
   ele que continua a alcançar além do anel, porque ali isso é a ferramenta.


---

### §7.25 — 📊 O PLACAR: o que falta, medido contra a lista do §5.1 (2026-08-15)

**Waves** — **6 fechadas**, 1 pela metade, 6 por abrir, 1 sem cura em código:

| wave | estado |
|---|---|
| **W0** a espinha · **W1'** a UI · **W2** os knobs de Pro · **W3** os kernels divergentes · **W5** Kelvinlets | ✅ **fechadas** |
| ~~**W1**~~ os defaults do `B` | ⛔ **sem cura em código** — §7.0; vira decisão de produto |
| **W4** o Smooth que não encolhe | ✅ **FECHADA** — o `l-mode` (Taubin λ\|μ) · **Slide Relax** · o **Surface Smooth como pincel próprio** (HC) · e o **laplaciano por cotangentes**, que ⚠️ **não foi para onde esta tabela o mandava**: como direção do Inflate ele foi **RECUSADO por medição** (§7.28) e a casa dele é o operador sobre o qual o par λ\|μ corre — que é o que o §4 já dizia (*"o operador dos dois acima"*) |
| **W6** os dabs que não são discos | ✅ **FECHADA** — **Clay Strips** · **Blob** · **Clay Thumb** (§7.26) · **Multiplane Scrape** (§7.27). O **Draw Sharp**, o 5º da lista dela, saiu com motivo na §7.18 (ele é o item da W1) |
| **W8** a DEMÃO | ✅ **FECHADA** — o `layer.cc`: `disp += f·strength·(1,05 − |disp|)`, e ⚠️ **a lei tem conteúdo MEDIDO** (todo peso da pegada converge para `disp = 1` ⇒ a demão é um **PLATÔ**, e o falloff é uma TAXA e não um perfil). ⚠️ **E o custo estrutural que esta tabela previa NÃO existiu:** o `accum` **É** o `displacement_factor` da referência ⇒ zero plano por-vértice novo, zero rota de aplicador nova, zero campo no snapshot de undo — quem o removeu foi ler o tempo de vida do `ss.cache` (por-traço), não uma escolha de desenho. Cena `=33` |
| **W7** o plano MLS · **W9** Mesh Filter · **W10** Cloth · **W11** handles · **W12** a geodésica | ⬜ **por abrir** |

**Ferramentas** — a lista do §5.1 tem 16 itens:

| | itens |
|---|---|
| ✅ **feitos (4)** | Clay Strips · Blob · Clay Thumb · **Multiplane Scrape** |
| ✅ **respondido SEM verbo novo (1)** | **Elastic Deform** — a §7.17 mediu que 3 dos 5 tipos dele são o mesmo verbo com outra família de escalas e os outros 2 já shipavam; o que faltava era o knob **Field width**. *Um sexto botão cujo conteúdo é um dropdown para verbos que a lista já tem é o item de menu morto que este plano recusa.* ⇒ **o alvo de 14 pincéis novos é de 13** |
| ⛔ **fora, com motivo (1)** | Draw Sharp — §7.18 mediu que o que o nome promete mora na **CURVA**, e a curva de fábrica por-tool está no mesmo `.blend` binário da §7.0 ⇒ ele **é** o item da W1 |
| ✅ **feitos na W4 (2)** | **Surface Smooth** · **Slide Relax** — ⚠️ esta linha os listava como pendentes enquanto a linha da W4, DUAS tabelas acima, já os dava por fechados: *duas contagens do mesmo fato divergem no dia em que só uma é atualizada* |
| ✅ **feito na W8 (1)** | **Layer** — a DEMÃO |
| ⬜ **faltam (7)** | Cloth · Pose · Boundary · Nudge · Thumb · Mesh Filter (9 tipos) · Cloth Filter (5 tipos) |

⇒ **23 verbos hoje** — `Verb::ALL.len()`, que é a fonte —, contra os 16 de que a
linha partiu. ⚠️ **Esta linha dizia 20, e o erro era a MESMA omissão da linha do
`faltam` acima:** ela foi escrita depois da W6 (16 + os 4 dela) e nunca contou os
**dois** que a W4 já tinha entregue. *Um número de contagem que não é derivado da
lista drifta na primeira wave que alguém esquece — e aqui ele driftou duas vezes
pelo mesmo esquecimento, em dois parágrafos vizinhos.* O placar do §10 dizia
**32**; com o Elastic Deform respondido sem verbo e o Draw Sharp fora, o alvo
honesto é **29** (16 + 11 pincéis + 2 filtros).

⚠️ **E os quatro que valem mais que a contagem, porque não são "mais um chip":**
⚠️ o **Layer** (W8) foi previsto aqui como *"traz um plano por-vértice novo, e
com ele a lei do repo — ao adicionar um plano, adicione-o ao snapshot de undo no
MESMO commit"*, e a §7.31 mediu que **ele não traz plano nenhum** (o `accum` já
é o `displacement_factor` da referência) — *a lei do repo continua de pé; o que
estava errado era supor que esta wave a acionaria* · o **Mesh
Filter** (W9) é o mais barato da lista inteira, porque o precedente do *Filter
Layer* do Painter diz que **não há kernel novo** (só `Sphere` e `Random` o são) ·
o **Cloth** (W10) é o único que traz um SOLVER, com cadência e undo próprios · e
a **geodésica** (W12) troca o falloff da família inteira de uma vez.

**Se for para escolher onde parar** — a §7.1 já respondeu e a resposta continua
de pé: W0-W3 entregam o **pedido inteiro** (os três modos, o Basic/Pro, e o app
deixa de esculpir mal). O que corre agora é a metade que muda **o que o app
consegue fazer**.

---

### §7.26 — ✅ O POLEGAR (W6): o primeiro verbo cujo alvo depende de QUANTOS dabs já passaram (2026-08-15)

`Verb::ClayThumb`, o **19º** — o `clay_thumb.cc`, o penúltimo item da W6.

**A lei, lida da fonte e não de memória.** A projeção é a do
[`Verb::Flatten`], **bilateral**, sem `comp` e sem teste de lado
(`calc_translations_to_plane`); a ferramenta inteira mora na construção do
plano:

1. ele passa pelo **centro do DAB** (`location_symm`), não pelo centro de área;
2. a normal é a de área **girada** em torno do eixo que ATRAVESSA o traço
   (`x = n × path`, o mesmo `X` que o `pinch.cc` monta);
3. o ângulo **ACUMULA** — `+0,8°` por dab, teto `60°` — *"simulate the clay
   accumulation by increasing the plane angle as more samples are added to the
   stroke"*.

⚠️ **DOIS erros de leitura foram evitados por ir ao código**, e os dois teriam
shipado silenciosos:

- os locais do `clay_thumb.cc` chamados `area_position` e `sculpt_plane_normal`
  estão com os **nomes trocados** — `calc_brush_plane(..., r_area_no, r_area_co)`
  devolve a **normal primeiro** (`sculpt.cc:3048-3053`). Lido pelos nomes, o
  verbo giraria uma POSIÇÃO como se fosse um vetor;
- o eixo chega ao `rotate_v3_v3v3fl` **escalado pelo raio** (`mat * scale`), e a
  função **normaliza sozinha** (`math_vector.cc:660`) — a armadilha de assumir
  que ela exige unitário é um erro que só aparece com `raio ≠ 1`.

**O que a wave NÃO precisou inventar.** O eixo de inclinação sai do **mesmo
door** que já responde *"este dab tem direção?"* — a referência monta `y = n × x`
e num frame ortonormal isso se inverte em `x = y × n`, então o `stroke_axis`
(que devolve o `y`) serve os dois, com **um** piso de degeneração. Um segundo
`cross` com um segundo piso seria a segunda resposta, e o dia em que um dos dois
mudasse o verbo depositaria onde o outro recusa.

**Sem direção não deposita**, e isso reproduz os DOIS `return` da referência (o
*"delay the first daub"* e o `is_zero(grab_delta)`) com **uma** pergunta: o
primeiro dab de todo traço tem `path = [0,0,0]` por construção.

**MEDIDO pela porta do artista** (`tests/measure_clay_thumb.rs`, esfera 96×144,
`R = 0,35`, passo `0,06 R`; o ângulo é lido **dos vértices**, por ajuste de plano
por mínimos quadrados):

| dabs | inclinação do corte | plano (a lei) |
|---|---|---|
| 2 | −0,65° | 0,80° |
| 5 | −2,83° | 3,20° |
| 10 | −5,75° | 7,20° |
| 20 | −15,18° | 15,20° |
| 40 | **−44,42°** | 31,20° |
| 76 | −79,97° | 60,00° |
| 120 · 200 | **−81,75° (idêntico)** | 60,00° |

⚠️ **As duas colunas medem grandezas DIFERENTES, e a segunda não é uma previsão
da primeira** — *plano* é quanto o plano de UM dab está inclinado contra a normal
de área DELE, *inclinação* é o corte que a SEQUÊNCIA deixou sobre uma esfera que
já curva sozinha. Elas coincidem por volta dos 20 dabs e divergem depois, porque
cada dab inclina contra o que os anteriores deixaram.

⚠️ **O CONTROLE é o Flatten no mesmo traço: −3,64°.** Sem ele o gate mediria a
curvatura da esfera e chamaria isso de inclinação.

⚠️ **E a mudança de ORIGEM tem SINAL, medido:** volume assinado **−10,68 no
Flatten contra +11,30 no polegar** — um REMOVE, o outro ACRESCENTA, e a causa é
só o plano passar pelo centro do dab (sobre a superfície) em vez do centro de
área (abaixo dela, numa calota curva). É o gate que morre se alguém "unificar" as
duas origens achando que a diferença é cosmética.

**O teto é ALCANÇÁVEL** (`60 / 0,8 = 75` dabs) e a ferramenta **satura**: 120 e
200 dabs deixam a malha **idêntica**, porque projetar num plano é auto-limitado.

⚠️ **O que é citável e o que é NOSSO:** `0,8°` e `60°` são literais do
`clay_thumb.cc`. Mas eles são **por DAB**, e quantos dabs cabem num centímetro de
traço é decisão de cada motor ⇒ *graus por comprimento de traço* é grandeza
NOSSA (medida: 26,67 · 13,33 · 6,67 · 3,20 °/raio nos espaçamentos 0,03 · 0,06 ·
0,12 · 0,25), e está escrita como nossa em vez de vestir a autoridade da fonte.

**A inclinação é do TRAÇO, nunca do espelho** — a referência a avança só em
`stroke_is_main_symmetry_pass`, e a nossa fronteira de chamada É essa passada.
Avançá-la por cópia faria a ferramenta mudar de lei ao ligar a simetria.

⚠️ **E o `!accum ⇒ orig` do estimador de plano vale para ele também** (o
`clay_thumb.cc` chama o mesmo `calc_brush_plane`): a base da inclinação é a
normal CONGELADA, senão ela persegue o barro que ela própria moveu. O `Blob`
ficou **FORA** dessa lista de propósito — ele também é do Blender e também ajusta
plano, mas hoje lê o vivo, e trocá-lo mudaria o desenho de um verbo que esta wave
não toca: quem o quiser dentro traz a medição junto.

**Gates:** 6 no verbo + 1 arch-gate novo na shell. **8 mutações, 8 sangram** — o
ângulo nunca avança (mata 2) · avanço por cópia de espelho · `begin` sem reset ·
sem o teto · a origem no centro de área · deposita sem eixo · o sinal da
inclinação · e a cena **muda** (o roteiro existe e ninguém o chama).

⚠️ **O arch-gate novo fecha uma classe que ninguém vigiava:**
`every_sculpt3d_scene_script_is_announced` — uma cena cujo `announce` não é
chamado compila, passa em toda a suíte e entrega ao Enio uma **janela sem
instruções**. É o irmão do `no_two_sculpt3d_scenes_claim_the_same_level` (lá a
cena é inalcançável, aqui ela é alcançável e não se apresenta).

⚠️ **DOIS defeitos de FIXTURE, e o primeiro reprovou o próprio CONTROLE:** o
ajuste de plano da sonda filtrava só em `xy` — um **cilindro** através de uma
esfera apanha as DUAS calotas, a maior dispersão passa a ser em `z`, e o ajuste
devolvia `±90°` para tudo, **inclusive para o Flatten**. *Uma sonda cujo controle
falha está a medir outra coisa.* E o gate `invert_changes_the_result_of_exactly_…`
reprovou no anti-vácuo apontando para o VERBO: ele dispara **um** dab em cada um,
e um dab só não tem caminho — a fixture passou a andar dois, que é o que a torna
honesta para os dezanove.

**Sem tecla, e a ausência é DELIBERADA** (`CHIP_ONLY`): sobra o `L`, e *"cLay
Thumb"* com `Clay`/`Clay Strips`/`Clay Thumb` no catálogo ensina uma regra que
não existe — o Blender também não lhe dá atalho de fábrica. **A escolha é do
Enio.**

**LOC:** o `stroke_target.rs` cruzou 700 (717) ⇒ corte por ASSUNTO em
`stroke_aim.rs` (648 + 94) — *a aritmética com que um alvo é escrito*, sete
funções que **não sabem que existe um verbo**, contra *para onde cada verbo
aponta*. ⚠️ O `stroke_axis` e o `lateral_pull` FICARAM no pai porque carregam LEI
e são citados de fora: descê-los obrigaria o `pub(super)` deles a virar
`pub(crate)`, e a visibilidade passaria a ser função do TAMANHO do arquivo.

`Verb::ALL` **18 → 19** · `SCULPT3D_VERB` **18 → 19** (`sculpt3d.verb.18`) ·
censo do `B` **18 → 19** (o `S` fica em 15: são 19 menos os quatro que o SculptGL
não tem). **Nenhum schema, nenhum ADR, nenhuma dep, nenhuma crate nova.**

✅ **SMOKE OK (2026-08-15).** O roteiro imprime os
números que ele manda contar (derivados das constantes, não escritos à mão). As
perguntas de olho: **o CONTROLE primeiro** (o Flatten não deita ao longo do
traço) · o polegar no mesmo gesto tem de ir **deitando** conforme a mão anda · um
**toque parado faz NADA** · passando dos 75 dabs a superfície **para de mudar** ·
o **traço seguinte nasce do zero** · e com o **espelho** os dois lados saem
iguais entre si e iguais ao lado único.

---

### §7.27 — ✅ A LÂMINA EM V (W6): o único verbo com DOIS planos, e a wave que FECHA a W6 (2026-08-15)

`Verb::MultiplaneScrape`, o **20º** — o `multiplane_scrape.cc`, o último item da
W6 e o que a §7.18, a §7.19 e a §7.23 vinham deixando em aberto (*"o Multiplane
Scrape e o Clay Thumb reusam esta moldura"*).

**A LEI.** Em vez de raspar contra uma superfície, ele raspa contra um
**TELHADO**: dois meios-planos partilham a origem (o centro do dab, como no
polegar) e as normais deles são a normal de área girada de `±ângulo/2` **em torno
do eixo que corre AO LONGO do caminho** — a rotação **ORTOGONAL** à do polegar,
que gira em torno do eixo que o atravessa. Os dois verbos inclinam o mesmo plano;
o que os separa é *em torno de quê*, e o que sobra do corte é um sulco de duas
facetas planas com uma **aresta viva** no meio.

**Qual dos dois um vértice consome sai do LADO em que ele caiu**
(`local_positions[i][0] <= 0`), e cada meio-plano tomba **para o lado que ele
serve** — é isso que abre o V em vez de o fechar. ⚠️ **UMA expressão para os dois
planos:** num frame ortonormal a rotação da normal em torno do eixo do traço é
exactamente `sin(θ/2)·across + cos(θ/2)·n`, então o *índice* da referência (que
monta dois `float4` e escolhe entre eles) vira o **SINAL** de um termo. A
representação a apagar o caso especial.

⚠️ **DOIS erros de leitura da fonte que teriam shipado em silêncio**, os dois
achados por ir ao código em vez de aos nomes: os locais do `clay_thumb`/`
multiplane_scrape` chamados `area_position` e `sculpt_plane_normal` estão com os
nomes **trocados** (o `calc_brush_plane(..., r_area_no, r_area_co)` devolve a
**normal primeiro**), e o eixo chega ao `rotate_v3_v3v3fl` **escalado pelo raio**,
com a função a **normalizar sozinha** — a segunda armadilha só apareceria com
`raio ≠ 1`.

**A PONTA NÃO É UM DISCO**, e a referência diz por quê no próprio comentário
(*"deform the local space along the Y axis to avoid artifacts on curved strokes;
this produces a not round brush tip"*). É a `Footprint::Blade`, e ⚠️ **ela é um
produto escalar e mais nada:** escalar UMA componente ortonormal por `k` dá
`|d'|² = |d|² + (k²−1)·(d·â)²`, então a decomposição inteira cancela — nem eixo
transversal, nem normal, nem as três projeções que a referência computa para
depois somar de volta.

**O MODO DINÂMICO** (`BRUSH_MULTIPLANE_SCRAPE_DYNAMIC`) amostra a normal média
dos **dois lados** da lâmina, mede o ângulo entre elas e usa isso como a abertura
— a ferramenta encontra a dobra que já existe em vez de impor um vinco próprio —,
com o knob a virar um **acréscimo** e o `0,2` do `interpolate` como única memória.
⚠️ **O Ctrl MUDA DE SIGNIFICADO entre os modos:** no fixo ele inverte (telhado →
vale), no dinâmico ele **zera** o ângulo, e a referência escreve o porquê — *"so
you can trim plane surfaces without changing the brush"*.

**A MEDIÇÃO, e é ela que decide o default** (`tests/measure_multiplane_scrape.rs`,
esfera unitária, pincel `0,35`, traço de 20 dabs sobre um arco de círculo máximo):

| autorado | diedro medido | fidelidade | crista (raios) | movidos |
|---|---|---|---|---|
| **0°** | 0,00° | — | **0,0000** | **0** |
| 15° | 4,65° | 0,31 | 0,0076 | 584 |
| 30° | 19,39° | 0,65 | 0,0677 | 837 |
| 45° | 34,18° | 0,76 | 0,1182 | 848 |
| **60°** | **46,85°** | **0,78** | **0,1739** | 855 |
| 90° | 63,01° | 0,70 | 0,2175 | 864 |
| 120° | 10,25° | 0,09 | 0,0908 | 871 |
| 160° | — | — | 0,0084 | 871 |

⚠️ **O `0` do `DNA_brush_types.h` é a ferramenta DESLIGADA, não *"um V
estreito"***, e o mecanismo é a ORIGEM: os dois meios-planos passam pelo plano
**TANGENTE** ao cursor, e acima de um plano tangente, num convexo, não há nada —
**zero vértices movidos**, contra os 994 que o `Verb::Scrape` (que projeta no
plano de ÁREA) move no mesmo traço. ⚠️ **Eu ia shipar a frase oposta** (*"`0` é o
Scrape ao bit"*): ela está corrigida no doc do campo, com o número ao lado. ⇒
`DEFAULT_MULTIPLANE_ANGLE_DEG = 60`, **NOSSO** (o valor de fábrica desta
ferramenta vive no `.blend` binário da §7.0), escolhido no pico da fidelidade com
uma crista que já vale **17% do raio**. O teto **fica onde a referência o pôs**
(`160`, `rna_brush.cc:3382`), agora com a tabela do que de facto acontece lá em
cima.

**11 gates · 12 mutações · 12 sangram.** ⚠️ **DUAS sobreviveram e as duas eram
achado, não ruído:**

1. **As duas amostras do modo dinâmico lendo o MESMO lado** passavam por nove
   gates — com o knob a somar 60°, *a ferramenta parece funcionar enquanto ignora
   a superfície inteira*. O gate que faltava põe o **knob em ZERO**: ali tudo o
   que sobra é a leitura, e o modo fixo é o CONTROLE que a torna visível (ele não
   move um vértice; o dinâmico move **680**).
2. **Tirar o verbo da lista de planos CONGELADOS** não sangra, e a razão é
   **GEOMETRIA**: o V é simétrico em torno da dobradiça, então a normal média das
   duas facetas que ele deixa é a MESMA que ele encontrou. Medido num traço que
   insiste no mesmo lugar — **0,08814 congelado × 0,09477 vivo** a 20 dabs, e a
   diferença **não compõe**. Fica **documentada em vez de gateada**: um gate sobre
   7% é um gate que alguém silencia.

⚠️ **E a segunda só foi mensurável depois de a FIXTURE ser corrigida:**
`..Brush::default()` carrega o `accumulate` do **Draw** (que é `true`), e a lei do
plano congelado é `!accumulate` ⇒ **o ramo do `pre` era inalcançável na suíte
inteira**, e as duas rotas saíam byte-idênticas. As fixtures passaram a derivar o
flag do VERBO, que é o que o painel faz ao trocar de ferramenta. ⚠️ **Os gates do
polegar e da faixa herdam a mesma cegueira** — nomeado aqui, não corrigido nesta
wave (mexer nas barras deles é wave de quem os fez).

⚠️ **QUATRO defeitos de fixture na sonda, e o primeiro reprovava o próprio
CONTROLE:** os dabs corriam numa RETA em `z = 1` (os de trás flutuavam **fora**
da esfera, e o corte medido virava função de quão longe o dab tinha flutuado) ·
o polo desta esfera é **+Y**, então `[0,0,1]` é um ponto do EQUADOR e a janela de
amostragem caía numa coluna de longitude só (**2 pontos por banda**, a sonda
inteira em `NaN`) · a malha de `96×144` **não resolve** as bandas do perfil (⇒
`160×240`) · e a crista era lida em ABSOLUTO, o que media a curvatura da esfera
(**0,0548 de crista num traço que moveu ZERO vértices**). ⚠️ **E o ajuste de
plano 3D foi DESCARTADO como oráculo:** a superfície cortada **não é um plano em
toda a pegada** (a projeção é ponderada pelo falloff), então ele media a mistura
— `14,5° · 17,2° · 12,4°` para ângulos autorados de `15° · 30° · 45°`. O oráculo
é o **PERFIL** da secção transversal, contra o repouso.

**Sem tecla, e a ausência é DELIBERADA** (`CHIP_ONLY`): o catálogo já tem
`Scrape` no `C`, e uma segunda tecla de *scrape* teria de ser sorteada — toda
escolha desse tipo é uma regra falsa que o artista aprende. O Blender também não
lhe dá atalho de fábrica. **A escolha é do Enio.**

**LOC: dois cortes por ASSUNTO.** O `brush_verb.rs` cruzou 700 (837) ⇒ as
constantes de magnitude saíram para `brush_magnitudes.rs` (610 + 250) — o arquivo
tinha virado **três** coisas (o catálogo, as portas, os números), e as três
crescem por razões diferentes: foi a terceira que o levou ao teto. O `stroke.rs`
cruzou (713) ⇒ o hoist da silhueta saiu para `stroke_shape.rs` (666 + 77), *que
FORMA este dab tem* contra *o que ele FAZ*.

`Verb::ALL` **19 → 20** · `SCULPT3D_VERB` **19 → 20** (`sculpt3d.verb.19`) ·
censo do `B` **19 → 20** (o `S` fica em 15: são 20 menos os cinco que o SculptGL
não tem) · ids novos `SCULPT3D_SCRAPE_ANGLE`/`_NUM`/`SCULPT3D_SCRAPE_DYNAMIC`
(hash de string) · 2 chaves i18n. **Nenhum schema, nenhum ADR, nenhuma dep,
nenhuma crate nova.** ⚠️ O `#[allow(clippy::large_enum_variant)]` no
`Sculpt3dIntent` nasceu de **crescer o `Brush`** — ele mede a largura do estado
autorado, não um defeito da fila (o precedente é o `Step` do `ph2d-ui-state`).

⚠️ **PENDENTE DE SMOKE: `PH2D_SCULPT3D_SMOKE=31`**, e o roteiro imprime os dois
números derivados das constantes. As perguntas de olho: **o CONTROLE primeiro**
(o Scrape deixa um canal de fundo chato) · a lâmina no mesmo gesto deixa uma
**crista** com uma faceta de cada lado · com o ângulo em **ZERO** ela faz
**NADA** (e isso não é bug) · num traço **VERTICAL** a crista corre na vertical ·
um **toque parado faz NADA** · o **Ctrl** vira o telhado em vale · e com **Read
the surface** marcado e o ângulo em zero ela **ainda corta**, porque leu a forma
que está debaixo do pincel.

---

### §7.28 — ✅ O LAPLACIANO POR COTANGENTES, e a célula do Inflate RECUSADA (2026-08-16)

O último item da **W4**. O §4 credita Meyer/Desbrun/Schröder/Barr 2003 com
*"laplaciano por **cotangentes**, normal de curvatura média"* para **"Inflate ·
o operador dos dois acima"** — e a wave descobriu, medindo, que as duas metades
dessa célula têm **vereditos opostos**.

#### O operador

`ph2d-mesh::cotangent` — o Laplace-Beltrami discreto:

```text
K(x_i) = (1 / (2·A_mixed)) · Σ_j (cot α_ij + cot β_ij) · (x_i − x_j) = 2·κ_H·n
```

Gateado contra o número do **paper**, não contra o que saiu: numa esfera de raio
`R` ele mede `2/R` com erro relativo **< 2 %** e desvio de direção **< 1e-3**,
nos três raios. Sem transcendental — `cot θ = (u·v)/|u × v|`, então nenhum
ângulo é materializado e a única raiz é a que a área do triângulo já precisa.

⚠️ **A BORDA devolve `None`, e é uma AFIRMAÇÃO:** a construção pede os **dois**
ângulos opostos a cada aresta e uma aresta de beira tem um só. Inventar o que
falta seria pôr um número onde a fonte não tem nenhum — a regra do §4.

#### ⛔ A célula do Inflate: RECUSADA, com número

`tests/measure_curvature_normal.rs`, o eixo separado do sinal:

| fixture | côncavos | eixo médio | eixo p95 |
|---|---|---|---|
| `sculpt_sphere` (a malha DEFAULT) | 0,0 % | **0,003°** | 0,020° |
| `uv_sphere` 24×32 | 0,0 % | 0,213° | 0,933° |
| depois de 4 traços | 3,3 % | **0,709°** | 2,030° |
| `uv_sphere_shuffled` | 12,6 % | 26,3° | **87,9°** |

Três razões, e cada uma bastaria:

1. **Na malha que o artista tem, o eixo diverge três milésimos de grau** da
   normal que já shipa. Um chip que não move um pixel.
2. **Onde ele diverge, diverge por deixar de ser uma normal:** `p95 = 87,9°` é
   quase TANGENTE — numa malha ruidosa o vetor de curvatura segue a ruga.
3. **`K = 2·κ_H·n` carrega o sinal da curvatura:** numa cova ele aponta para
   DENTRO. Caminhar por ele não é *inflar*, é **afiar** — outro verbo, que já
   tem `l-mode` próprio (o μ do Taubin).

⇒ O Inflate fica com **2 chips**, e a linha da matriz do §3 foi corrigida.

#### ✅ A casa dele: o operador do `l-mode` do Smooth

*"O operador dos dois acima"* são o Taubin 1995 e o Desbrun 1999 — ou seja, o
cotangente é o laplaciano **sobre o qual o par λ|μ deveria correr**, e o nosso
corria sobre o uniforme. A propriedade, medida porta contra porta
(`ph2d-mesh::measure_cotangent_smoothing`, `uv_sphere` 24×32, força cheia; a
*deriva tangencial* é quanto o vértice escorregou AO LONGO da superfície):

| passes | operador | raio médio | deriva tangencial |
|---|---|---|---|
| 1 | uniforme | 0,990715 | 0,003164 |
| 1 | **cotangente** | **0,992060** | **0,000014** |
| 4 | uniforme | 0,963384 | 0,012277 |
| 4 | **cotangente** | **0,968796** | **0,000322** |
| 16 | uniforme | 0,864052 | 0,043883 |
| 16 | **cotangente** | **0,882856** | **0,004974** |

**226× · 38× · 8,8× menos deriva**, e encolhe menos nos três. E o resíduo do
próprio par λ|μ encolheu **22 %**: a coluna `L` do `taubin_pair` era
`−0,0011 / −0,0104 / −0,0206 / −0,0409 %` e é agora
`−0,0008 / −0,0079 / −0,0159 / −0,0318 %`.

⚠️ **A chave do operador é a MESMA do par** (`(Smooth, L)`), e é o único jeito de
o chip não mentir: chaveados diferente, o Taubin rodaria sobre um operador e o
chip anunciaria outro. Escrita e não derivada de `passes().len() > 1`, pela mesma
razão que o `RefMode::declares` recusa a derivada.

#### As três lições de gate, todas minhas

1. **O primeiro gate de não-vazamento era AUTO-REFERENTE** — rodava o mesmo
   pincel duas vezes e comparava os resultados. Testa determinismo e **não pode
   falhar**. O oráculo honesto é `(Sharpen, B)` contra `(Sharpen, L)` **ao bit**:
   dois pincéis DIFERENTES obrigados ao mesmo resultado, porque o `L` não declara
   o Sharpen e `kernel_for` recua para o `B`.
2. **O segundo usava o `s-mode` como controle e REPROVOU produto correto**
   (`4,92×`): `S` contra `L` mede a lei de kernel **junto com** o operador. O
   controle que isola é o `B` — e os quatro números o mostram, com o `S/B` de
   ~4,8× a aparecer nos DOIS verbos.
3. **E o gate de propriedade ficava VERDE com o operador desligado**, porque sob
   o `L` entram DUAS coisas (o par e o operador) e o par sozinho já reduz a
   deriva. A isolação foi para `ph2d-mesh`, onde as duas portas são públicas e
   não há terceira variável.

#### A mutação que sobreviveu, e o teorema que a explica

**8 mutações, 7 sangram.** A que sobrevive apaga a guarda de `Σw ≤ 0` — e a
explicação não é *"falta fixture"*: varrida uma grade de razão de aspecto **1 a
1000**, a contagem de `Σw ≤ 0` foi **zero em todas**, porque

```text
cot q + cot r = sin(q + r) / (sin q · sin r) = sin p / (sin q · sin r) > 0
```

para todo triângulo não-degenerado — um triângulo tem no máximo **um** ângulo
obtuso, e o par que entra na soma nunca é dominado por ele. ⚠️ A instabilidade
real do operador mora nos pesos **individuais** por aresta, não na soma; o
doc-comment do `RingWeights::weight` dizia o contrário e foi corrigido. Gate:
`the_weight_sum_is_positive_by_identity_not_by_luck`.

#### Um doc que creditava a idempotência ao lugar errado

O `neighbour_average` afirmava *"a média das posições **CONGELADAS** … ler o
`pre` e não o vivo é o que torna o Smooth idempotente"*, e a leitura é
`mesh.positions()` — a viva, a mesma do `live` do `compute_target`. O que impede
a superfície de derreter sob um pincel parado mora no **aplicador**, que
interpola de `base_pos` para o alvo em vez de compor sobre o resultado anterior.
Corrigido nos dois lugares.

#### Superfície

`PROJECT_SCHEMA` intocado · contrato congelado intocado · **nenhuma crate nova** ·
**nenhuma dep nova** · nenhum ADR (o `rayon` da `ph2d-mesh` já é o das normais e
da curvatura, mesmo gather, mesmo CSR). Superfície pública nova: `RingWeights` ·
`ring_weights_at` · `mean_curvature_normal_at` · `curvature_normal_dir_at` ·
`curvature_normals_of` · `cotangent_ring_average_at` · `Face::tri_at` ·
`RingOperator` · `Brush::ring_operator`.

**Mudança de comportamento — UMA:** o `l-mode` do Smooth desenha diferente (é a
entrega). Todo outro pincel é **byte-idêntico**, e o gate que o afirma compara
dois pincéis distintos em vez da função consigo mesma.

---

### §7.29 — ✅ O `λ` DO TAUBIN ERA UM PALPITE, e o smoke o derrubou (2026-08-16)

**Report do Enio, no smoke da §7.28:** *"Smooth modo L é tão discreto que é quase
imperceptível."*

**Ele está certo, e o número é este** — sonda nova
`tests/measure_smoothing_power.rs`, esfera com ruga 0,03, força cheia, UM dab, a
régua sendo a **RUGOSIDADE** (`|p − média do anel|`, a grandeza que todo filtro
passa-baixa ataca):

| modo | queda de rugosidade | a malha andou |
|---|---|---|
| `S` | **44,9 %** | 0,016501 |
| `B` | 21,5 % | 0,003458 |
| `L` | **15,3 %** | **0,001958** |

O `l-mode` movia **8,4× menos** que o `s-mode`, e o slider de força **já estava
no topo** — não havia mais o que pedir.

⚠️ **A fixture tem de conter o fenómeno, e a irmã não continha.** A
`measure_smooth_shrinkage` mede uma esfera **LISA**: sobre ela não há alta
frequência para atenuar, todo passa-baixa mede zero por construção, e o que se
veria é o encolhimento. *Uma sonda que mede a coisa errada num caso onde a
resposta certa é zero não reporta defeito nenhum.*

#### De que recurso era o `0,33`? De nenhum.

O `TAUBIN_PASS_BAND` cita o paper (*"we have used k_PB = 0.1"*), o `TAUBIN_MU` é
**derivado** da relação — e o `λ` dizia, no próprio doc, *"o Smooth de sempre com
um terço do peso"*. **Uma descrição, não uma derivação.** É o §0 do `CLAUDE.md`
por inteiro: *um limite que não diz de que recurso é, é um palpite à espera de um
smoke*.

A varredura analítica (`tests/measure_taubin_lambda.rs`, a função de
transferência `f(k) = (1 − λk)(1 − μk)`) dá o mapa:

| λ | `f(2)` | veredito |
|---|---|---|
| 0,33 (o antigo) | **0,647** | estável, guarda 65 % da ruga |
| **0,50** | **0,000** | estável, **aniquila** a ruga de um vértice |
| 0,65 | −0,717 | estável |
| **0,699984** | — | **a FRONTEIRA** (acima, `|f| > 1` e o par AMPLIFICA) |

E a varredura pela **porta do produto** (a mesma sonda, editando a const e
re-medindo):

| λ | queda 1 dab | andou | queda 4 dabs |
|---|---|---|---|
| 0,33 | 15,3 % | 0,001958 | 41,5 % |
| 0,40 | 22,4 % | 0,002928 | 50,3 % |
| **0,50** | **34,1 %** | 0,004683 | 58,0 % |
| 0,60 | 44,5 % | 0,006881 | 62,4 % |
| 0,65 | **47,1 %** | 0,008148 | 63,9 % |
| 0,69 | 46,8 % ⬅ **CAI** | 0,009243 | 63,8 % ⬅ **CAI** |

⚠️ **A curva tem MÁXIMO e ele foi MEDIDO, não deduzido:** entre 0,65 e 0,69 o par
**anda mais e alisa menos** — a assinatura de já ter passado do ponto.

#### Por que 0,50 e não o pico

**`λ = 0,5` é o único candidato com DERIVAÇÃO:** `1/λ = 2` põe o zero do primeiro
fator exactamente no **topo do espectro** do laplaciano (tanto o uniforme como o
cotangente normalizado são médias de pesos que somam 1 ⇒ autovalores em `[0, 2]`),
e `k = 2` é o **padrão alternado — a ruga de UM vértice**, que é literalmente o
que o artista está a alisar. O zero é **exacto em `f32`** (`0.5 * 2.0` é `1.0` sem
arredondamento).

⛔ **O pico medido (0,65) NÃO é o ponto de operação:** ele fica a **93 %** de um
penhasco cuja posição sai do espectro **IDEAL**; num triângulo mal formado um peso
de cotangente pode ser negativo e o espectro efectivo passar de 2, movendo a
fronteira para baixo **de malha para malha**. λ = 0,5 fica a 71 % dela.

⛔ **O 0,60 foi recusado embora meça melhor:** ele iguala a força do `s-mode`
(44,5 % contra 44,9 %) e é **exactamente por isso** que não serve — é um número
escolhido por **casar uma coluna numa fixture**, que é o que este módulo já recusa
por escrito para o `μ` (*"um `μ` ajustado até a coluna zerar seria um número
ajustado a UMA fixture, que passa a mentir na malha seguinte"*).

#### O que a mudança compra, e o que ela custa

| | antes | depois |
|---|---|---|
| queda de rugosidade / dab | 15,3 % | **34,1 %** |
| contra o `b-mode` (21,5 %) | **mais fraco** | **mais forte** |
| deriva tangencial | 0,00002088 | 0,00004865 (**56,1×** menos que o `B`) |
| resíduo de raio em 20 dabs | −0,0159 % | −0,0373 % (**48,4×** menos que o `S`) |
| raio médio em 32 dabs | 1,000742 | 1,001644 (o `S` mede **0,930**) |

⚠️ **O `l-mode` era o mais FRACO dos três e passou a ser mais forte que o `B`** —
a ordem que nenhum artista espera do chip que anuncia literatura. E a razão contra
o `S` continua **plana em todo `n`** (47,8 · 48,9 · 48,8 · 48,4 · 47,9), que é a
propriedade que o gate afirma.

#### Os gates novos afirmam a DERIVAÇÃO, não o valor

`the_lambda_annihilates_the_one_vertex_ripple` (`f(2) == 0.0`, com o CONTROLE de
que a banda de passagem sobrevive) e
`the_pair_attenuates_the_whole_stop_band_and_amplifies_nothing` (o critério do
paper). ⚠️ **Escritos assim de propósito:** `assert_eq!(TAUBIN_LAMBDA, 0.5)` só
sabe dizer *"alguém mudou o número"*; estes dizem **por que ele é esse**.

**Mutações — 2, e elas DISCRIMINAM** (é o que prova que os dois gates não são
redundantes):

| mutação | derivação | estabilidade |
|---|---|---|
| λ = 0,33 (o antigo) | **RED** | ok |
| λ = 0,75 (além da fronteira) | **RED** | **RED** |

#### Duas notas que estavam ERRADAS e foram corrigidas

⚠️ **O doc do gate `the_literature_smooth_holds_the_radius…` citava
`L = −0,0206 % ⇒ 87,7×`** — a medição de **ANTES** do operador por cotangentes.
Eu actualizei a tabela do cabeçalho do módulo na §7.28 e **esqueci a medição
citada dentro do gate**, que atravessou uma wave inteira a mentir. *Um doc que
cita um número medido tem de ser re-medido por quem move o número*, e é por isso
que a sonda `the_drift_table_the_gate_cites` passou a reproduzir a fixture do
gate.

⚠️ **A sonda nova imprimia `0,01227699` onde o gate mede `0,01227704`** — ela
somava em `f64` com `mul_add` e o gate em `f32` simples: **dois números para a
mesma medição**, e o doc de um passaria a citar o do outro. A sonda foi alinhada à
aritmética do gate e agora os dois batem **dígito a dígito**.

**Suítes:** 588 verdes em release **e** em debug · fmt e clippy limpos · nenhum
schema, nenhum contrato congelado, nenhum id/token · nenhuma dep nova.

**Mudança de comportamento — UMA:** o `l-mode` do Smooth alisa **2,2× mais** por
dab. Todo outro pincel é byte-idêntico.

---

### §7.30 — ✅ W7: O PLANO VIRA UMA SUPERFÍCIE, e a medição decidiu a wave antes da primeira linha (2026-08-16)

Com a fila de P0 vazia, a próxima da tabela do §7 é a **W7 — o plano MLS**: o
`l-mode` de Flatten/Fill/Scrape/Clay, a **projeção MLS** de Alexa, Behr,
Cohen-Or, Fleishman, Levin & Silva 2003 (*Computing and Rendering Point Set
Surfaces*), que o §4 nomeia como *"a superfície local em vez de um plano"*.

**A MEDIÇÃO PRIMEIRO** (`tests/measure_mls_plane.rs`), porque *"tem conteúdo?"*
é uma afirmação sobre um número, e o §4 recusa chip sem conteúdo:

| esfera r=1 | dab 0,2 | 0,3 | 0,4 | 0,6 |
|---|---|---|---|---|
| desvio ao PLANO | 0,006024 | 0,012960 | 0,023390 | 0,052149 |
| resíduo do QUADRIC | 0,000017 | 0,000077 | **0,000256** | 0,001331 |
| o alvo move `\|g\|/raio` | 5,07% | 7,58% | **10,31%** | 15,81% |

O quadric explica **99% do desvio** e desloca o alvo em **10,3% do raio do
pincel** — a distância entre uma faceta e uma carícia.

⚠️ **E o CONTROLE que decidiu se a wave era segura de construir:** um Flatten
existe para REMOVER detalhe, e um alvo que seguisse o detalhe destruiria o verbo
com todos os números bonitos. Medido numa esfera com ruga `0,03`: **93,1% e
101,9% da ruga SOBRA** — o quadric é de grau 2 sobre a pegada inteira, captura a
curvatura e **deixa a ruga onde está**.

⚠️ **A minha primeira coluna de resumo lia o número AO CONTRÁRIO.** Ela dizia
*"o quadric capturou 41,6% do desvio"* e eu ia concluir que ele comia a ruga —
o desvio ao plano numa esfera RUGOSA é **curvatura + ruga**, e a curvatura é
exactamente o que ele deve capturar. *Só comparando o resíduo com a ruga MEDIDA
as duas se separam.*

#### O desenho, e o que ele apaga

O `PlaneFit` ganha `surface: Option<Quadric>` e **o `signed_distance` não
ramifica em modo nenhum**: ele subtrai uma altura que, sem quadric, é zero.
`S` e `B` ficam byte-idênticos por construção, e os quatro verbos não sabem que
existe um `l-mode` — a representação apaga o caso especial.

Só a metade **(2)** do paper é portada (o polinómio); a **(1)** — a otimização
não-linear do plano de referência — é substituída pelo `fit_plane` que os quatro
verbos já usam. ⚠️ **Não é atalho de custo:** re-derivá-la seria a **segunda
resposta** a *"que plano descreve esta pegada?"*, e o `l-mode` deixaria de ser
*outra lei sobre a mesma superfície*. O passo continua a ser ao longo da normal
do plano (`to_plane`, a porta única dos quatro) — **divergência declarada**.

#### ⚠️ Uma MUTAÇÃO SOBREVIVENTE expôs um CONTROLE MORTO

Ajustando o quadric contra o ponto **já deslocado**, o `c0` absorve o offset e o
`signed_distance` subtrai-o de volta: o alvo sai o MESMO e **o knob
`plane_offset` fica INERTE sob o `l-mode`** — que é um verbo inteiro a
desaparecer, porque o `Verb::Clay` *é* o Flatten contra um plano levantado.
Nenhuma fixture usava offset com o `L`, então o knob morto era invisível.

**A cura é uma assimetria:** a ALTURA é medida do ponto **pré-offset** e o
`(u, v)` do ponto final. O `(u, v)` não leva correção porque o offset corre ao
longo da NORMAL — ele não move a origem tangencialmente.

#### ⚠️ E a segunda sobrevivente derrubou uma justificativa MINHA

O doc do `Quadric` afirmava que a normalização por raio evita
mal-condicionamento *"exactamente onde o pincel é grande"*. Medido
(`where_the_unnormalised_fit_starts_to_lie`), o `f64` do solver absorve tudo até
um dab de raio **400 000**, com desvio de **2e-16**:

| escala | raio do dab | desvio relativo |
|---|---|---|
| 1 | 0,4 | 2,255e-16 |
| 100 | 40 | 3,331e-16 |
| 10 000 | 4 000 | 4,832e-16 |
| 1 000 000 | 400 000 | 3,274e-16 |

⇒ **o que ela compra não é precisão, é o PISO DE PIVÔ ser livre de escala de
cena**, e esse é o lado PEQUENO: sem normalizar, um pincel de raio `4e-4` põe os
termos de quarta ordem em `2,5e-14`, abaixo do piso, o ajuste é **recusado**, e o
`l-mode` colapsa no `s-mode` **em silêncio**. O gate foi re-apontado para BAIXO
(×0,001) — a versão que ampliava deixava a mutação passar.

#### Os gates, e os três que nasceram vermelhos sobre produto correto

**6 gates, 5 mutações, 5 sangram.** ⚠️ **Três oráculos meus estavam errados:**

1. *"a ruga tem de cair"* medida por `| |p| − raio |` **REPROVOU o `s-mode`**,
   que esta wave não toca (0,016941 → 0,033543): achatar uma calote **tira os
   vértices da esfera por construção**, então aquela régua soma a ruga com o
   próprio achatamento. ⇒ magnitude do **laplaciano**, que é local e cega à forma
   grande.
2. *"numa superfície PLANA os dois modos concordam"* nasceu vermelha por
   `0,007362` contra uma barra de `0,0016` que **eu** escolhera: uma esfera de
   raio 20 vista por um dab de 0,8 **não é plana** (sagita `0,016`), e `0,007362`
   é metade dela. *Não existe "plano o suficiente" que não seja um número
   escolhido a dedo* ⇒ a propriedade sem limiar é a RELAÇÃO — a altura de um
   quadric é `≈ κ·ρ²/2`, **linear na curvatura**, então quadruplicar o raio da
   esfera divide a divergência por ~4.
3. o gate do offset pousou **ZERO vértices**, e o controle é que o disse: um
   Flatten `OneSided` morde o lado `d > 0`, ou seja RASPA, e levantar o alvo
   acima da calote deixa-o sem nada para raspar ⇒ offset **negativo**, e a régua
   é a **INTERSEÇÃO** dos movidos (só quem pousou nos dois alvos tem os dois
   pousos a comparar).

⚠️ **E os dois gates da lei anti-chip-morto dispararam pela razão certa** — o doc
do `the_literature_mode_is_offered_exactly_where_it_declares_a_law` **previa este
dia**: *"um paper cuja lei não seja nenhum dos dois faz este gate falhar — ele
não sabe adivinhar o mecanismo do próximo"*. A superfície local é o **terceiro
mecanismo**, e o censo passa de 6 para **10 verbos** com `L`.

**Superfície de colisão:** `PROJECT_SCHEMA` **INTOCADO** (nada disto é
serializado) · contrato congelado intacto · **zero `Cargo.toml`** · **nenhuma dep
nova** · **nenhum ADR** (roda sob o ADR-0150) · registro do `ph2d-ecs`
**intocado** · uma crate-módulo nova (`stroke_surface.rs`, filho do `stroke`) e
uma cena, a **`=32`**.

**Aberto, com o preço ao lado:** o **`Draw`** é o quinto candidato que o §3
nomeia — *"a normal do ajuste MLS/PCA, o l-mode mais fraco da tabela; se medir
dentro do piso, o Draw shipa com 2"* —, e ele é uma pergunta sobre a **NORMAL** e
não sobre a altura: entra quando for medido · a projeção do paper caminha pela
normal LOCAL e a nossa pela normal do plano (segunda ordem na curvatura,
divergência declarada) · o peso é uniforme dentro da pegada, e o paper usa uma
gaussiana `θ(d)` — o efeito é suavizar a fronteira do ajuste, e não foi medido.

Smoke: **`env PH2D_SCULPT3D_SMOKE=32 cargo run -p ph2d-host-desktop --release`**
— ⚠️ **a cena NÃO arma o modo** (o chip é a costura que ela existe para provar),
e a pergunta de olho **não é *"mexeu?"***: os dois modos achatam, e o que os
separa é a faceta contra a curva que fica.

---

### §7.31 — ✅ W8: A DEMÃO, e o plano por-vértice que a medição REMOVEU (2026-08-16)

O §5.1 item 6 pede *"uma demão de **altura constante**, saturante e apagável"*, e
traz um custo escrito ao lado: *"ela introduz um plano por-vértice novo
(`displacement`), e a lei do repo para isso está escrita — ao adicionar um plano,
adicione-o ao snapshot de undo no MESMO commit"*.

⚠️ **O custo não existe, e quem o removeu foi a leitura da referência.** No
`layer.cc` o `layer_displacement_factor` mora no **`ss.cache`** — o Blender
constrói o `StrokeCache` no pen-down (`sculpt.cc:5148`, `MEM_new<StrokeCache>`) e
o **destrói no pen-up** (`sculpt.cc:6021`, `MEM_delete`) —, logo ele é estado de
**TRAÇO**, irmão do nosso `pre` congelado e não da máscara. Nada disto viaja num
documento, e nada disto entra num snapshot de undo.

⚠️ **E do nosso lado ele nem sequer é um plano.** O aplicador já anda
`lerp(pre, alvo, accum)`; pondo o alvo na altura **CHEIA**, o `accum` que o motor
guarda desde sempre passa a ser exactamente o `displacement_factor` da
referência. *A wave que o plano precificava como estrutural custou um `bool` numa
tabela.*

#### A lei, e a medição que a validou antes de uma linha ser escrita

```text
d ← clamp(d + w·força·(1,05 − |d|),  0, 1−máscara)   // satura
alvo = pre + normal_pre · sinal · altura              // a demão CHEIA
pos  = lerp(pre, alvo, d)                             // o aplicador de sempre
```

A sonda `measure_layer_law` fez três perguntas antes de a wave abrir.

**P1 — para onde cada peso converge?** É a pergunta que decide se o chip tem
conteúdo: se cada vértice parasse numa altura proporcional ao seu peso, o verbo
seria um Draw com teto. Medido, **todo peso converge para `d = 1,0000`** — a
demão é um **PLATÔ** e o falloff é uma **TAXA**:

| peso | d₁ | d₂ | d₄ | d₈ | d₁₆ | d₃₂ | d₂₅₆ | dabs até 99 % |
|---|---|---|---|---|---|---|---|---|
| 1,00 | 1,0000 | 1,0000 | 1,0000 | 1,0000 | 1,0000 | 1,0000 | 1,0000 | **1** |
| 0,50 | 0,5250 | 0,7875 | 0,9844 | 1,0000 | 1,0000 | 1,0000 | 1,0000 | 5 |
| 0,25 | 0,2625 | 0,4594 | 0,7178 | 0,9449 | 1,0000 | 1,0000 | 1,0000 | 10 |
| 0,10 | 0,1050 | 0,1995 | 0,3611 | 0,5980 | 0,8554 | 1,0000 | 1,0000 | 28 |
| 0,02 | 0,0210 | 0,0416 | 0,0815 | 0,1567 | 0,2900 | 0,4999 | 1,0000 | 142 |

⚠️ **O `1,05` é o que faz o platô FECHAR.** Fosse `1,0`, o incremento seria
`1 − d` e a demão aproximar-se-ia do teto por metades, para sempre; com `1,05` o
centro do pincel (peso 1) chega ao teto **num dab**, e o clamp corta o resto.

**P4 — a nossa recorrência contra a do Blender.** Lá a escrita é
`pos += (alvo − pos)·f` sobre a posição **VIVA**; o nosso aplicador anda do `pre`
**CONGELADO**. Medido com altura unitária:

| peso | vivo (Blender) | nosso (`accum = 1`) | nosso, **se** copiasse o `·f` |
|---|---|---|---|
| 1,00 | 1,000000 | 1,000000 | 1,000000 |
| 0,50 | 1,000000 | 1,000000 | **0,500000** |
| 0,25 | 1,000000 | 1,000000 | **0,250000** |
| 0,10 | 1,000000 | 1,000000 | **0,100000** |

⇒ **as duas recorrências pousam no mesmo lugar, e portar o segundo `f` para o
nosso aplicador QUEBRARIA o verbo** — o falloff vazaria para dentro da única
propriedade que a demão entrega. ⚠️ **E o `f` da referência ali não é um
amortecimento da demão:** é a atenuação genérica de borda que *todo* pincel do
Blender multiplica no `calc_translations`. No nosso motor essa atenuação **é** o
`accum`, e aplicá-la duas vezes é o perfil-em-dobro que o `Grip::Hold` já
documenta ter pago uma vez.

**Divergência declarada — o TRANSIENTE.** As duas recorrências separam-se no
meio do traço: pior separação **0,26–0,37** da altura, fechada em **8–55 dabs**
conforme o peso. A nossa chega ao platô mais depressa.

#### O default, e o número da referência que foi RECUSADO

O `rna_brush.cc:3230-3239` declara **três** números para o `height`: faixa dura
`[0, 1]`, faixa de UI `[0, 0,2]` e default **`0,5`**.

⚠️ **O terceiro cai FORA do segundo.** Copiá-lo shiparia um slider encostado no
máximo, com o artista só podendo descer — e o §7.0 desta linha já mediu que os
defaults **por-ferramenta** do Blender vivem num `.blend` binário desde o 4.3,
então esse `0,5` é o default do **campo**, não o da demão. ⇒ ficam as duas faixas
(que a fonte declara) e o default sai do **meio da que ela chama de
trabalhável**: `0,1`.

E ele tem um número no nosso mundo: a esfera de fábrica tem raio `1,0` (extensão
medida **2,0**), então `0,1` é **10 % do raio** — da ordem de **2,5 dabs de Draw
a raio 0,4** (`reach = 0,04` cada). Uma camada que se vê, não um bloco.

#### A cerca de Chesterton, e o que a medição fez com ela

O doc do `Verb` registava que Draw e Layer *"colapsavam"* sob a lei do envelope e
que a wave do accumulate (2026-08-11) os separou. ⚠️ **A minha primeira versão do
gate pediu que o Draw passasse MUITO do teto da demão, e a medição a derrubou:**
com Accumulate ele **para no RAIO** (`0,4000` num pincel de raio `0,4`), porque o
`from_live` mede a distância em 3-D da posição viva e o vértice **sai da pegada
ao subir**. Os dois verbos têm teto. O que difere é **de que grandeza cada teto é
função** — medido, varrendo o raio:

| raio | teto da DEMÃO | teto do Draw+Accum |
|---|---|---|
| 0,2 | 0,1000 | ~0,20 |
| 0,8 | 0,1000 | ~0,80 |

*Um teto maior era um número que eu escolhi; de que grandeza ele é função é o que
a lei diz.*

#### Os dois defeitos de GATE que a wave achou, os dois na mesma classe

⚠️ **Dois gates liam a porta ERRADA — a do GRIP em vez da do VERBO** — e os dois
eram verdes por acidente, porque até hoje nenhum verbo sobrescrevia a coluna que
eles consultavam:

* `stroke_apply_tests::unit_accum_verbs` derivava a lista de `Grip::law`. O
  doc-comment dele **já contava** a história de uma lista escrita à mão que ficou
  incompleta em silêncio — e ele então leu a tabela errada, um nível abaixo. (A
  faixa já sobrescrevia o `from_live` por `Verb::grip_law` desde 2026-08-13; o
  gate nunca perguntou por aquela coluna.)
* `seam::the_accumulate_switch_is_offered_only_where_it_does_something` derivava
  o esperado de `matches!(verb.grip(), Grip::Stamp)`, enquanto o painel pergunta
  a `Verb::accumulates()`. A demão é um carimbo que **não** oferece o
  interruptor, e foi ela que os separou.

*`Grip::law` responde qual é a lei deste GRIP; `Verb::grip_law` responde qual é a
lei deste VERBO. Um gate que julga o produto tem de perguntar pela porta que o
produto usa.*

#### O que ficou

**12 gates, 10 mutações, 10 sangram.** ⚠️ **Uma delas sobreviveu à primeira
rodada e o buraco era real:** trocar o teto do early-out (`accum >= keep`) por
`accum >= 1.0` deixa o resultado **byte-idêntico**, porque o `coat_step` já
clampa — o que muda é que todo vértice mascarado volta a ser reescrito, para
sempre, e vai ao refit do octree e ao upload por nada. O gate que a mata mede
**TRABALHO** (quantos vértices o dab moveu), não pixels, com o controle ao lado
para não estar a medir *"a máscara zerou o peso"*.

⚠️ **E três dos meus gates nasceram reprovando produto correto, os três por
fixture:** a idempotência entre traços (importei do Painter a frase *"um shape
editor re-carimba a figura a cada quadro"*, e este módulo não tem shape editors —
entre traços a demão **empilha**, que é a referência); o controle da chatice
(pedi ao Draw uma razão `< 0,5` que ele não tem depois de 512 dabs a saturar — a
régua certa é a **mesma** que a demão passa, 2 %); e o teto do Draw, acima.

Superfície: `Verb::ALL` **22 → 23** · `GripLaw` ganha a coluna `coat` (a terceira
lei de acumulação, com gate de exclusividade mútua sobre todo verbo × toda
combinação de flags) · `SCULPT3D_VERB` **22 → 23** · dois ids novos, os dois por
`hash_node_id` · uma chave i18n · **zero `PROJECT_SCHEMA`** · **zero ADR** ·
**zero `Cargo.toml`** · **zero dep**.

Smoke: **`env PH2D_SCULPT3D_SMOKE=33 cargo run -p ph2d-host-desktop --release`**
— ⚠️ **a cena NÃO arma o verbo nem a altura** (o chip e a row são a costura que
ela existe para provar), e a pergunta de olho **não é *"levantou barro?"***: o
Draw também levanta. É que a demão **PARA**, o topo dela é um **PLATÔ**, e o teto
não se move quando o pincel muda de tamanho.

---

### §7.32 — ✅ O SMOKE DA DEMÃO REPROVOU O FALLOFF, e o falloff estava certo: SETE verbos nasciam com a força da referência ERRADA (2026-08-16)

Report do Enio com dois screenshots, no smoke da `=33`: *"Falloff provavelmente
errado. resultado muito diferente e pior"* — a camada saía com **parede quase
vertical** e borda **escadeada**.

**O falloff foi inocentado por MEDIÇÃO, e a régua não é o kernel — é o perfil
radial do PRODUTO** (`measure_layer_law::the_radial_profile_of_a_coat`). A lei
satura, então todo peso converge para `disp = 1` e o falloff decide **quão
depressa**, nunca **até onde**: a 64 dabs a fração da altura em
`t = 0,5 … 0,95` mede `1,000 1,000 1,000 0,715 0,277`. *Nenhuma curva impede um
top-hat numa lei que satura.* O que decide se o artista vê ombro ou degrau é a
**TAXA** — e a taxa estava dobrada.

**A causa.** `Brush::weight` perguntava `verb.profile(self.mode)` **sem o recuo**
que o `kernel_for` e o `lateral_for` têm; o `S` não declara a demão, `profile`
devolve `None`, e o `map_or` caía no slider CRU onde o `layer.cc` eleva ao
quadrado. **`0,5000` contra `0,2500`.** E não era da demão: a **shell** nascia
com `[RefMode::default(); N]` enquanto o `Sculpt3dUi::default()` do painel **já**
derivava — **7 dos 23 verbos** num modo que não os declara.

**A cura tem duas metades, e cada uma cobre uma superfície diferente:**
`RefMode::birth_for` (o chip da faixa e o gesto de carimbar) e `RefMode::for_verb`
no `weight()` (a lei de força, que é a que muda a forma do barro).

⚠️ **E o destino do recuo continua o `Self::B` literal porque a MEDIÇÃO travou a
derivação:** o `Sharpen` é declarado pelos dois modos, então o derivado o manda
para o `S` e o gate `the_geometric_operator_does_not_leak_into_the_verb_next_door`
da W4 reprovou **sobre produto correto**.

⚠️ **Por que 277 gates ficaram verdes:** **64 das 86 fixtures** desta crate usam
`strength: 1.0`, e `1² == 1`. A suíte era cega **por escolha de fixture** — a
mesma lição que o `lateral_for` já tinha pago (BUGS #2). O gate novo usa `0,40` e
pina a cegueira num teste próprio.

**Superfície:** `RefMode` ganha `birth_for` + `for_verb` (as duas `const`, e o
`kernel_for`/`lateral_for` passam a delegar) · **zero componente, zero id, zero
`PROJECT_SCHEMA`, zero `Cargo.toml`, zero dep, zero ADR**. Detalhe e tabelas:
[`BUGS_sculpt3d.md` #4](BUGS_sculpt3d.md).

**Mudança de comportamento, nomeada:** os sete verbos do Blender passam a
depositar **na metade da taxa** no mesmo slider — é a correção, e ela é visível
em toda cena que os use.

---

### §7.33 — ✅ O SEGUNDO SMOKE DA DEMÃO: a curva não era ESCOLHÍVEL, e a premissa do `Basic` era emprestada (2026-08-16)

> Enio, re-smoke do `=33`: *"funciona corretamente mas não dá a opção de escolher
> o falloff e deveria dar"*.

**A demão passou** — a §7.32 curou a taxa e o ombro apareceu. O que sobrou é
outro eixo: o **seletor de curva** existia e o artista não o alcançava.

**Medido antes de qualquer hipótese.** O `paint_brush_tail` pintava a fileira
atrás de `ui_level.shows(UiLevel::Pro)`, e o `UiLevel::default()` é `Basic` ⇒ o
painel abre sem ela. Gate red-first
(`the_basic_level_never_hides_the_curve_that_shapes_the_dab`) nasceu **VERMELHO**
com a mensagem *"o Basic escondeu o seletor de curva com o Draw em mãos"*, nos
vinte verbos.

**A decisão não é de gosto, e a REFERÊNCIA a decide.** Lido o
`properties_paint_common.py` do Blender:

| fato medido | onde |
|---|---|
| `FalloffPanel` **não** é desenhado por `brush_settings_advanced` | classe própria, linha 675 |
| no cabeçalho de ferramenta ele é um **popover sempre visível** | `layout.popover("VIEW3D_PT_tools_brush_falloff")`, linha 1885 |
| no painel estreito ele é `DEFAULT_CLOSED` — **com o cabeçalho à vista** | linha 677 |
| e ali o widget é um **dropdown**, não a fileira expandida | `if region.type == 'TOOL_HEADER' … else col.prop(…, text="")` |

⇒ no Blender a curva é **dobrada, nunca ausente**: o artista sempre vê que ela
existe. O nosso `Pro` a tornava **invisível sem rastro**, e é a diferença entre
dobrar e amputar.

⚠️ **A regra do `UiLevel` não estava errada — ela é NECESSÁRIA e não suficiente.**
Ela diz *"só uma row cujo valor alguém ARMOU pode ser `Pro`"*, e a curva é armada
pelo `arm_verb_defaults`, logo ela **podia** ser `Pro`. Ser admissível não é ser
certo; quem decide a segunda metade é a referência, medida e não lembrada. As
duas metades estão escritas no doc do enum.

⚠️ **E o que estava de facto errado era a PREMISSA do `Basic`:** o doc dele dizia
*"o vocabulário do SculptGL"*, e o SculptGL **não tem** seletor de curva — a dele
é fixa. Herdar aquele vocabulário apagava do Basic um controle que a nossa malha
tem **doze** vezes. *Um vocabulário herdado descreve a ferramenta de onde veio,
não a que se está a construir.* Corrigido no `state.rs`.

⚠️ **Segue uma faixa que REFLUI, e não um dropdown**, pelo precedente que o
`paint/tool.rs` já mediu para os vinte verbos (*"um dropdown esconde quinze
ferramentas atrás de um clique para mostrar uma"*): quem escolhe uma curva a
escolhe COMPARANDO. O Blender troca para dropdown no painel estreito porque a
fileira dele **transborda**; o `seg` desta casa **reflui**, então a razão dele
não se aplica aqui.

**Gates:** o novo (`the_basic_level_never_hides_the_curve_that_shapes_the_dab`,
irmão exacto do `the_basic_level_never_hides_the_two_knobs_every_brush_has`,
varrendo os vinte verbos) — e a metade do falloff saiu do
`a_pro_row_is_reachable_in_pro_and_absent_in_basic`, que segue a varrer a TABELA
e continua verde. **1 mutação, 1 sangra** (reinstalar o `if shows(Pro)` ⇒ RED só
no gate novo, que é o par certo).

⚠️ **E um VERMELHO-LATENTE meu fechou junto, achado por rodar o gate que o
fechamento por crate não alcança:** o `rows.rs` estava em **603 > 600** desde a
própria wave da demão (§7.31) — o `architecture_panel_loc_cap` mora em
`ph2d-editor-core/tests/`, e um `cargo test -p ph2d-panel-sculpt3d` **não o
alcança** (a família estrutural que esta casa já registrou várias vezes). Cortado
por **ASSUNTO** para o irmão `rows_alpha.rs` (549 + 83): a tabela diz *o que um
knob É*, e o irmão responde as duas outras perguntas do PADRÃO — *quando uma
pista dele APARECE* e *como um valor de pista ATRAVESSA para o motor*. ⚠️ O
`always` **não veio junto**, e a ausência é o corte: ele é o predicado partilhado
pelas três tabelas e não fala do padrão.

**Superfície:** zero schema, zero id novo, zero `Cargo.toml`, zero dep, zero ADR,
contrato congelado intocado.

**Mudança de comportamento, nomeada:** o seletor de curva passa a ser pintado
**sempre**, em todo verbo — o `Pro` deixa de o esconder. Nenhuma curva muda de
valor (o `arm_verb_defaults` segue armando a da referência).

---

## §11 — O REPORT DO HARDNESS E DOS FALLOFFS: a atribuição (2026-08-16)

> Enio, com foto de uma esfera onde metade dos traços sai lisa e metade sai
> **rasgada**: *"tanto hardness como falloffs apresenta problemas graves. vá ao
> código original blender"*.

Fui ao Blender. **E a medição ABSOLVEU as duas leis que o report nomeia.**

### §11.1 — O que foi medido, contra a referência

| pergunta | veredito | número |
|---|---|---|
| a lei de UM dab reproduz a curva analítica? | **sim** | Smooth · Constant · Sphere · Pow4 batem a **três decimais**; o `Constant` mede `1.000` nas dezesseis faixas |
| o `hardness` é o do Blender? | **sim, verbatim** | `apply_hardness_to_distances` (`sculpt.cc:7549`) — mesma escada, mesmo ponto do pipeline, mesma faixa `0..1`, mesmo default `0` |
| o Blender tem uma guarda que nós não temos? | **não** | sem clamp de translação · `autosmooth` default **0** · o restore por-passo dele não cobre o Draw |
| o freio do `accumulate` funciona? | **sim, e é do SculptGL** | mas ele é carregado pelo **gradiente** do falloff ⇒ sob `Constant` (platô) ele é **inerte**: 16 dabs medem **16,00×** com o flag ligado e desligado |

⚠️ **O `accumulate` do SculptGL não multiplica nada** — ele troca de onde a
distância é medida (`crate::ref_kernels::Origin`), e o `stroke_accum_tests.rs`
já guardava isso num TIPO justamente para ninguém supor o contrário.

### §11.2 — O que RASGA a malha: a topologia dinâmica

Colapso + refino repetidos ao longo de um caminho, **sem um único dab**, deixam:

| | pior diedro | p99 | vértices |
|---|---|---|---|
| esfera intocada | 0,89° | — | 13.682 |
| um colapso sozinho | 4,66° | 0,34 | — |
| **a caminhada que o produto roda** | **179,91°** | 0,87 | **−4.782** |

179,91° é um triângulo **dobrado sobre si mesmo** — exatamente o que o matcap
desenha como estilhaço. E o alvo é calibrado contra o **raio do pincel**
(`edge_target = radius · sqrt((1.1 − detail) · 0.2)`), então em **todo** ajuste
de detalhe ele é mais grosso que a aresta da esfera de fábrica (3,7× a 12,3×):
*ligar a topologia dinâmica sempre decima a malha sob o pincel.*

⚠️ **Isto é uma wave própria e NÃO foi construída** — a atribuição está aqui
para que a próxima não recomece pelo hardness.

### §11.3 — O que foi construído: o `autosmooth_factor`

O knob do Blender que faltava (RNA `rna_brush.cc:3457`, sete linhas do
`hardness`): um **segundo passe de Smooth depois de cada dab**, dentro da
passada de simetria (`sculpt.cc:3635`), pulando SMOOTH e MASK.

Medido no traço (Draw, raio 0,30, força 0,5, `Constant`, 12 dabs):

| `auto_smooth` | crista | rugosidade |
|---|---|---|
| 0,00 | 0,11853 | 0,029113 |
| **0,25** | **0,10158** | **0,021050** |
| 1,00 | 0,04286 | 0,014097 |

E num toque único (`Constant`), o p99 do diedro cai **79,22° → 34,05°**.

⚠️ **Ele não cura o rasgo, e a razão é estrutural:** o passe herda a curva DURA
do artista, logo é cego exatamente na borda — como no Blender. O default é
**0,0**, e a faixa de trabalho é 0,1–0,3.
