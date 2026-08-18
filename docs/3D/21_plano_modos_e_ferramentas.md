# Plano — OS TRÊS MODOS DE REFERÊNCIA, o Basic/Pro, e as ferramentas que faltam

> ⚠️ **Cortado em 2026-08-18.** A narrativa foi **verbatim** para
> [`21_plano_modos_e_ferramentas.md`](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md) (remontagem confere sha256 com o original).
>
> ⚠️ **Uma referência `§N` que você não encontrar aqui está LÁ** — o corte manteve a
> numeração original de propósito, para que os ponteiros internos continuem a resolver
> num `grep` sobre o arquivo. ⛔ E as **recusas medidas** têm índice no fim deste doc:
> consulte-o **antes** de propor qualquer otimização ou mudança de desenho aqui.

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

### §7.25 — 📊 O PLACAR: o que falta, medido contra a lista do §5.1 (2026-08-15)

**Waves** — **9 fechadas**, 4 por abrir, 1 sem cura em código.

⚠️ **Este cabeçalho dizia *"6 fechadas … 6 por abrir"* e já divergia da TABELA
LOGO ABAIXO dele** (que mostrava oito), e depois a **W7 fechou** (2026-08-17,
`23913321b`) sem ninguém voltar aqui. É a doença que o parágrafo do `faltam`,
quinze linhas adiante, nomeia para si mesmo — *duas contagens do mesmo fato
divergem no dia em que só uma é atualizada* — e ela reincidiu **duas vezes na
mesma seção**. O que não drifta é a tabela; o cabeçalho é derivado dela e foi
recontado.

| wave | estado |
|---|---|
| **W0** a espinha · **W1'** a UI · **W2** os knobs de Pro · **W3** os kernels divergentes · **W5** Kelvinlets | ✅ **fechadas** |
| ~~**W1**~~ os defaults do `B` | ⛔ **sem cura em código** — §7.0; vira decisão de produto |
| **W4** o Smooth que não encolhe | ✅ **FECHADA** — o `l-mode` (Taubin λ\|μ) · **Slide Relax** · o **Surface Smooth como pincel próprio** (HC) · e o **laplaciano por cotangentes**, que ⚠️ **não foi para onde esta tabela o mandava**: como direção do Inflate ele foi **RECUSADO por medição** (§7.28) e a casa dele é o operador sobre o qual o par λ\|μ corre — que é o que o §4 já dizia (*"o operador dos dois acima"*) |
| **W6** os dabs que não são discos | ✅ **FECHADA** — **Clay Strips** · **Blob** · **Clay Thumb** (§7.26) · **Multiplane Scrape** (§7.27). O **Draw Sharp**, o 5º da lista dela, saiu com motivo na §7.18 (ele é o item da W1) |
| **W8** a DEMÃO | ✅ **FECHADA** — o `layer.cc`: `disp += f·strength·(1,05 − |disp|)`, e ⚠️ **a lei tem conteúdo MEDIDO** (todo peso da pegada converge para `disp = 1` ⇒ a demão é um **PLATÔ**, e o falloff é uma TAXA e não um perfil). ⚠️ **E o custo estrutural que esta tabela previa NÃO existiu:** o `accum` **É** o `displacement_factor` da referência ⇒ zero plano por-vértice novo, zero rota de aplicador nova, zero campo no snapshot de undo — quem o removeu foi ler o tempo de vida do `ss.cache` (por-traço), não uma escolha de desenho. Cena `=33` |
| **W7** o plano MLS | ✅ **FECHADA** (2026-08-17, `23913321b`) — `stroke_surface.rs`, a projeção MLS de Alexa, Behr, Cohen-Or, Fleishman, Levin & Silva 2003, com sonda `measure_mls_plane` e os gates em `verb_surface_tests`. ⚠️ **E a wave achou um knob MORTO ao lado:** o `offset` do plano não chegava a lugar nenhum |
| **W9** Mesh Filter | 🔄 **W9a FECHADA** (2026-08-18) — a FIAÇÃO e os **4** tipos que reusam verbo: `Verb::filter_law()`/`filters_mesh()` (a porta única que o §6 já prescrevia: *o painel pergunta para OFERECER, o motor para HONRAR*) · o driver `stroke_filter.rs` · o interruptor no card TOOL · o gesto (arrasto horizontal = força e SINAL, a régua `0,001/px` de `sculpt_filter_mesh.cc:2301`) · **UM** passo de undo. ⚠️ **E o driver achou um defeito que o §6 não previa:** as três leis de ANEL leem a malha VIVA, o que num traço é correcto (o freio é o aplicador a interpolar de `base_pos`) e num filtro **não existe** — com `accum = 1` duas chamadas na mesma força COMPÕEM, e *o desenho passaria a depender de quantos eventos o rato mandou*. A cura é o `reset_translations_to_original` da referência: a pose congelada é REPOSTA antes de cada chamada, e as três leis são invocadas **verbatim**. ⚠️ **Consequência honesta e NOMEADA:** o `α` do `hc_shape` fica **INERTE** num filtro (com `q == o` ele interpola entre o mesmo ponto) — a degeneração correcta do knob, não um controle perdido. **5 mutações, 5 sangram** (o sinal do arrasto · o `filter_arm()` a esquecer a metade do VERBO · o `arm_filter` a não desarmar o transform · a row pintada sem o `filters_mesh()` · o `filter_at(dx)` em vez do `x` cru), e ⚠️ **a metade que faltava foi achada por um gate MEU sobre produto MEU:** a exclusão mútua estava escrita numa porta só, então **a ordem dos cliques decidia** o que o botão esquerdo faz — um `enum` de modo seria mais limpo e obrigaria a reescrever os cinco leitores do `transform_arm`; a exclusão por porta custa duas linhas em cada uma **e tem gate**. **LOC:** o `sculpt3d.rs` cruzou 600 (589 → 613) com os dois campos, o `Drag::Filter` e o `mod filter` ⇒ corte por ASSUNTO em **`sculpt3d_rulers.rs`** (*com que régua a mão fala com a cena* contra *o que a cena É*), 554 + 106 — ⚠️ e ele **absorve o `FILTER_DRAG_PER_PX`**, que é a mesma espécie dos vizinhos (pixel de arrasto → grandeza do gesto). ⚠️ **E o corte achou um doc-comment ORFANADO que já shipava:** o do `RADIUS_MIN_PX` tinha escorregado para cima do `MASK_OP_PASSES` (que abria descrevendo o piso do raio) e a const ficara NUA — *um `mod` novo entre uma doc e o item dela não dá erro: ela apenas passa a documentar o vizinho*, a família que o split do `paint.rs` do Painter já pagou em 2026-07-19. ⚠️ **E o `cargo fmt --all --check` por EXIT CODE achou a metade do motor da W9a fmt-VERMELHA no commit anterior** (`ph2d-sculpt3d/src/lib.rs`, o `pub use` re-embrulhado pelos dois tipos novos) — *um vermelho que só o ship vê é invisível entre integrações*. ⬜ **Falta a W9b** (Random · Sphere · Scale, os três kernels novos) **e a W9c** (Sharpen de dois passes · Enhance Details) |
| **W10** Cloth · **W11** handles · **W12** a geodésica | ⬜ **por abrir** |

**Ferramentas** — a lista do §5.1 tem 16 itens:

| | itens |
|---|---|
| ✅ **feitos (4)** | Clay Strips · Blob · Clay Thumb · **Multiplane Scrape** |
| ✅ **respondido SEM verbo novo (1)** | **Elastic Deform** — a §7.17 mediu que 3 dos 5 tipos dele são o mesmo verbo com outra família de escalas e os outros 2 já shipavam; o que faltava era o knob **Field width**. *Um sexto botão cujo conteúdo é um dropdown para verbos que a lista já tem é o item de menu morto que este plano recusa.* ⇒ **o alvo de 14 pincéis novos é de 13** |
| ⛔ **fora, com motivo (1)** | Draw Sharp — §7.18 mediu que o que o nome promete mora na **CURVA**, e a curva de fábrica por-tool está no mesmo `.blend` binário da §7.0 ⇒ ele **é** o item da W1 |
| ✅ **feitos na W4 (2)** | **Surface Smooth** · **Slide Relax** — ⚠️ esta linha os listava como pendentes enquanto a linha da W4, DUAS tabelas acima, já os dava por fechados: *duas contagens do mesmo fato divergem no dia em que só uma é atualizada* |
| ✅ **feito na W8 (1)** | **Layer** — a DEMÃO |
| 🔄 **em curso (1)** | **Mesh Filter** — **4 dos 9** tipos hoje (Smooth · Surface Smooth · Relax · Inflate, todos por reuso de verbo); faltam Random · Sphere · Scale (W9b) e Sharpen · Enhance Details (W9c) |
| ⬜ **faltam (6)** | Cloth · Pose · Boundary · Nudge · Thumb · Cloth Filter (5 tipos) |

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
Filter** (W9) era previsto aqui como *o mais barato da lista inteira, porque o
precedente do Filter Layer do Painter diz que não há kernel novo* — e a W9a
**mediu que a previsão estava certa e incompleta**: os quatro tipos que reusam
verbo custaram fiação, e o que ela NÃO previu foi a lei do anel a ler a malha
viva (acima), que é uma correcção de CORRECÇÃO e não de custo ·
o **Cloth** (W10) é o único que traz um SOLVER, com cadência e undo próprios · e
a **geodésica** (W12) troca o falloff da família inteira de uma vez.

**Se for para escolher onde parar** — a §7.1 já respondeu e a resposta continua
de pé: W0-W3 entregam o **pedido inteiro** (os três modos, o Basic/Pro, e o app
deixa de esculpir mal). O que corre agora é a metade que muda **o que o app
consegue fazer**.

---

---

## ⛔ Recusas MEDIDAS — 25, e nenhuma volta à fila

> ⚠️ **Este doc foi cortado em 2026-08-18** e a narrativa foi **verbatim** para
> [`21_plano_modos_e_ferramentas.md`](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md) — a remontagem das duas metades confere sha256 com o original.
>
> ⛔ **Uma recusa medida é o conteúdo mais caro do repo:** ela diz *o que foi tentado, medido
> e rejeitado, com o mecanismo* — e é a única coisa que impede alguém de refazer trabalho já
> pago. Estas ficaram no arquivo; este índice existe para que continuem a existir na prática.
>
> *Antes de propor qualquer otimização ou mudança de desenho aqui, procure-a nesta tabela.*
> Linhas marcadas `§` são o próprio título da seção — as mais duras, do tipo «não refaça».

| onde | a recusa |
|---|---|
| [(topo)](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L10) | ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma |
| [§7 — AS WAVES](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L25) | \| ~~**W1**~~ \| ⛔ **SEM CURA EM CÓDIGO** (trocou de lugar com a W3 — §7.1; o *porquê* medido está na **§7.0**) \| o perfil `B` de DEFAULTS não é construível: ⚠️ **não é o clone** — a partir do |
| [✅ DESBLOQUEADO — as nove curvas, que o doc 20 declarava INVERIFICÁVEIS](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L115) | ⛔ **E a frase *"o `B` pode declarar curva"* estava ERRADA — ler o arquivo a |
| [§](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L126) | ⛔ NÃO desbloqueado, e a razão é ESTRUTURAL — não é o trim |
| [§7.5 — ✅ W2 LANDOU: `Basic` × `Pro`, e a DUREZA ganha porta (2026-08-1](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L455) | ⚠️ **E uma substituição por âncora foi RECUSADA pelo próprio `assert count == 1`:** |
| [§7.6 — 📐 A W4 tem alvo, e o PAPER mudou: **Taubin, não HC** (medido em](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L509) | REFUTADO pela §7.7**: eu deduzi um teto do SINAL ter invertido, e a medição pela |
| [§7.10 — ✅ W5 (metade A): O AGARRE VIRA UM CAMPO ELÁSTICO (2026-08-13)](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L859) | ⛔ **E os 3,5 % deixaram de ser um resíduo ACEITÁVEL — ver §7.13.** Estas linhas |
| [§7.10 — ✅ W5 (metade A): O AGARRE VIRA UM CAMPO ELÁSTICO (2026-08-13)](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L865) | ⛔ **E a leitura que estas linhas faziam do `3` estava INVERTIDA — ver §7.11.** |
| [§7.11 — ✅ OS MODOS `B` E `L` DO GRAB ESTAVAM BIZARROS (2026-08-13)](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L970) | alternativa que o doc da §7.10 recusou foi recusada **por ESTIMATIVA** |
| [§7.13 — ✅ O CAMPO ATERRISSA NA BORDA DA PEGADA (2026-08-13, 4º report)](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L1126) | em `r/ε = 3` (**0,00011**) e declarei a tabela §7.10 refutada. O `rigid_profile` |
| [§7.13 — ✅ O CAMPO ATERRISSA NA BORDA DA PEGADA (2026-08-13, 4º report)](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L1137) | ⛔ **Esticar o alcance NÃO é a cura, e isto é medição:** `REACH` 4 → 1,19 % · |
| [§](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L1231) | ⛔ Três alavancas MEDIDAS e mortas |
| [§7.17 — ✅ A W5 FECHA NA LARGURA DO CAMPO, e o `Verb` novo era o item e](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L1408) | ⚠️ **E um segundo desenho meu foi REFUTADO por um doc que já estava no repo.** |
| [§7.17 — ✅ A W5 FECHA NA LARGURA DO CAMPO, e o `Verb` novo era o item e](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L1412) | tinha medido e recusado exatamente esse movimento — *"alargar o alcance NÃO é a |
| [§7.19 — ✅ W6 (metade A): O DAB DEIXA DE SER UM DISCO (2026-08-14)](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L1597) | ⛔ **1º SMOKE REPROVOU: *"parece redondo"* (Enio) — e ele estava certo, com o §0 |
| [§7.19 — ✅ W6 (metade A): O DAB DEIXA DE SER UM DISCO (2026-08-14)](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L1641) | inalcançáveis, e dois sliders mortos em dezasseis das dezassete ferramentas. |
| [§](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L1656) | §7.1 — ⛔ Por que a W1 trocou de lugar com a W3 (medido em 2026-08-12) |
| [O que a medição descartou pelo caminho](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L1864) | ⚠️ **Hipótese REFUTADA — o plano vivo NÃO produz crescimento sem limite.** O |
| [(A) O `l-mode` — 62 % do gesto caía FORA do anel do cursor](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L2157) | ⛔ **E não há corte honesto que o localize:** o perfil lateral é quase CHATO até o |
| [§](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L2560) | §7.28 — ✅ O LAPLACIANO POR COTANGENTES, e a célula do Inflate RECUSADA (2026-08-16) |
| [§](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L2584) | ⛔ A célula do Inflate: RECUSADA, com número |
| [Por que 0,50 e não o pico](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L2757) | ⛔ **O pico medido (0,65) NÃO é o ponto de operação:** ele fica a **93 %** de um |
| [Por que 0,50 e não o pico](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L2762) | ⛔ **O 0,60 foi recusado embora meça melhor:** ele iguala a força do `s-mode` |
| [⚠️ E a segunda sobrevivente derrubou uma justificativa MINHA](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L2895) | termos de quarta ordem em `2,5e-14`, abaixo do piso, o ajuste é **recusado**, e o |
| [§](../archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md#L3018) | O default, e o número da referência que foi RECUSADO |
