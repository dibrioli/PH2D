# PLANO — DESDOBRAR O MAPA: o que a literatura diz sobre o defeito que medimos em 2026-08-30

> **Estado:** pesquisa fechada, obra por abrir. **Origem:** ordem do Enio (30/08) — *«tente uma
> pesquisa em fóruns técnicos ou literatura para descobrir dicas de implementação»*, depois de
> decidir **ficar fechado** (a via de licença encerra — ver
> [`TRIAGEM_licenca_e_distribuicao.md`](../cleanroom/TRIAGEM_licenca_e_distribuicao.md), sem
> consequência para o produto).
>
> ⚠️ **Fontes: literatura pública apenas.** Nenhuma linha veio de fonte de alvo restrito, e as
> superfícies que renderizam fonte foram **bloqueadas na busca**. Este documento é o que a
> disciplina de clean-room autoriza como fonte (*espec + papers + PH2D*).

---

## §1 — O defeito, no vocabulário da literatura

Medido em 30/08 na peça do artista (`Detail 0,85`, passo uniforme) — handoff §8-terdecies e
§8-quaterdecies:

```
campo ACORDA (4 singularidades na casca da ponta)  ->  traçado PARTE (31 arestas de parede)
   ->  ⛔ o MAPA DOBRA: 3,12 % de triângulos invertidos no OMBRO contra 0,14 % no corpo (23×)
   ->  extracção emite face a 169°–177° e uma gravata  ->  a foto do zoom
```

A literatura tem nome para as três propriedades que um mapa destes deve ter
([Ray 2025](https://arxiv.org/abs/2507.15404)):

| propriedade | o que é | nós |
|---|---|---|
| **GP** (*grid preserving*) | costura por rotações de 90° **e translações inteiras** | ⭐ **temos por construção** (campo + quantização) |
| **det+** | `det ∇F > 0` em toda parte — sem dobras | ⛔ **NÃO temos** — `3,12 %` no ombro |
| **SoG** (*singularity on grid*) | as singularidades caem em pontos **inteiros** da grade | ⏳ **nunca medido aqui** |

⚠️ **E o autor diz porquê:** o **GP** é fácil (é imposto pelo campo e pela quantização); o **det+**
é difícil *«porque se exprime como uma restrição de desigualdade quadrática»*; e o **SoG** é
**ainda mais difícil**, porque o mapa pode ganhar singularidades **que nunca foram prescritas** —
elas *«não existiam no campo, apareceram durante a optimização numérica que produziu o mapa»*.

### ⭐⭐⭐ E isso nomeia o NOSSO defeito de 29/08, palavra por palavra

O mesmo texto descreve o mecanismo: para **reduzir a distorção**, a optimização converte um índice
numa combinação (`−1/2` vira `−1` mais `1/2`) — e acrescenta que esses índices *«nem sequer são
válidos para produzir uma malha de quads (geram vértices de valência 1 e 2)»*.

⇒ ⛔⛔ **É exactamente o defeito dos *doublets*** que esta linha curou em 29/08 (`19` vértices de
valência `2`, **todos em pontas finas**). *Nós tratámos o sintoma — dissolver o doublet; a
literatura nomeia a causa — o optimizador a inventar singularidades onde a distorção aperta.* E o
sítio onde a distorção aperta é, medido, **o ombro do espinho**.

---

## §2 — As três famílias de cura, com o veredito da literatura

| família | o que faz | veredito publicado |
|---|---|---|
| **Local stiffening** (a heurística de 2009) | itera, pesando mais os triângulos distorcidos | ⚠️ *«often leads to sufficient results»* mas **sem garantia**, e falha *«especially for large target edge lengths»* — ⛔ **que é exactamente o nosso `Detail` de fábrica** |
| **Restrições anti-flip lineares** (2013) | desigualdades que proíbem a inversão | ⛔ *«just a linearization of the full non-linear functional»*, restringem a orientação e podem tornar o espaço **inviável**; o *branch-and-bound* que as acompanha *«is known to take even days»* |
| ⭐⭐⭐ **Barreira / regularização** ([LIM 2013](https://igl.ethz.ch/projects/LIM/), [TLC 2020](https://duxingyi-charles.github.io/publication/lifting-simplices-to-find-injectivity/), [Garanzha et al. 2021](https://arxiv.org/abs/2102.03069)) | energia finita enquanto `ε > 0` e `→ ∞` sobre elementos invertidos quando `ε → 0` | ⭐ **passa 100 % de um benchmark público de `10 743` casos 2D + `904` 3D**, e ⭐⭐ **funciona a partir de um estado JÁ EMARANHADO** |

⛔⛔ **CORRECÇÃO AO NOSSO PRÓPRIO REGISTO:** o doc do botão e o `CLAUDE.md` §5 dizem que a cura
publicada para a regressão das faces `>60°` é o **local stiffening**. A literatura diz que ele é a
heurística **antiga**, sem garantia, e que falha precisamente com alvo de aresta grande. *Ele não é
«a cura publicada»; é a primeira tentativa histórica.* ⇒ nota a corrigir quando se lhe tocar.

---

## §3 — ⭐⭐⭐ A RECEITA, ao nível de implementação

De [Garanzha, Kaporin, Kudryavtseva, Protais, Ray, Sokolov (2021)](https://arxiv.org/abs/2102.03069).
⚠️ **Escolhida porque é a única da família que não precisa de partida válida** — e a nossa partida
tem dobras.

**A energia por elemento**, pesada pelo volume do elemento original, com `λ` a trocar ângulo por área:

```
f(J) = tr(JᵀJ) / (det J)^(2/d)      g(J) = det J + 1/det J      (ambas +∞ se det J ≤ 0)
F(U) = Σ_t ( f(J_t) + λ·g(J_t) ) · vol(T_t)
```

**O truque, e é uma linha:** todo `det J` que aparece num **denominador** é substituído por

```
χ(D, ε) = ( D + √(ε² + D²) ) / 2
```

`χ` é suave e **positiva** para qualquer `D`; quando `ε → 0⁺` ela tende a `D` para `D > 0` e a `0⁺`
para `D < 0`. ⇒ *a energia é finita e derivável sobre uma malha emaranhada, e só se torna infinita
sobre a dobra à medida que `ε` encolhe.* **É isso que permite partir de onde estamos.**

**O calendário de `ε`.** Existe um conservador com teorema de desemaranhamento em número finito de
passos; e existe o **empírico, que os autores dizem funcionar melhor na esmagadora maioria dos
casos** e que faz a base inteira passar no teste de injectividade:

```
ε^k = √( 1e−12 + 4e−2 · [ min(0, min_t det J_t^k) ]² )
```

**O laço:**

```
k ← 0
repetir
    calcular ε^k
    U^{k+1} ← L-BFGS-B( U^k, ε^k )          // basta F e ∇F
                                             // (Hessiana modificada + CG + line search
                                             //  só nos casos mais duros)
    k ← k+1
até   min_t det J_t^k > 0   E   F(U^k, ε^k) > (1 − 1e−3)·F(U^{k−1}, ε^{k−1})
```

⭐ **Aplica-se a malhas de quads também** — os autores avaliam `J` em cada triângulo que forma um
canto do quad (regra do trapézio). ⚠️ Custo relatado: de uma fracção de segundo a alguns minutos
para as malhas maiores, num 12 núcleos.

### ⭐⭐ E o encaixe na NOSSA cadeia já está pago

O mapa não é livre: ele tem de continuar **GP** (costura por rotação de 90° + translação inteira).
Um desemaranhador genérico destruiria isso.

⭐⭐⭐ **Mas a obra A de 24/08 já resolveu essa metade:** a costura deixou de ser **penalizada** e
passou a entrar por **eliminação de variável** (`94 %` das ligações eliminam uma; as que fecham
ciclo entram num sistema linear só). ⇒ **as variáveis livres do nosso G3 são exactamente as que o
desemaranhador pode mover, e a costura fica preservada por construção.** *A parte difícil de
aplicar isto a um mapa com costuras é a única que já está construída aqui.*

---

## §4 — A ordem de trabalho, e porquê nesta ordem

1. ⭐ **MEDIR o SoG primeiro** (barato, e nunca foi feito). Já temos a lista de singularidades — é
   a que o `round_welded` recebe — e temos o mapa. A pergunta é: *elas caem em pontos inteiros?*
   ⚠️ Se **não** caírem, há uma segunda causa **independente** das dobras, com cura mais barata:
   *fixar explicitamente as coordenadas das singularidades conhecidas no problema de optimização* —
   é o que a literatura prescreve, textualmente.
2. ⭐⭐ **O desemaranhador**, sobre o conjunto reduzido de variáveis do G3. É a obra grande, e é a
   que tem `10 743 + 904` casos de prova públicos atrás dela.
3. ⛔ **NÃO começar pelo *local stiffening***. A literatura diz que falha com alvo de aresta
   grande, e o alvo de fábrica é grande.

### A régua já existe, e a barra sai da medição

| régua | onde | hoje | alvo |
|---|---|---|---|
| dobras do mapa na casca `[0,75 · 0,90)` | `does_the_field_wake_up_at_a_thin_tip` | ⛔ `3,12 %` | `≤ 0,14 %` (o nível do corpo) |
| torção `p99` da mesma casca | a sonda do zoom (§8-duodecies) | ⛔ `169°` | `~35°` (o nível do corpo) |
| cobertura da casca exterior | `ph2d_quadfill::coverage` | `0,095 %` | não regredir |

⛔ **E não a percentagem GLOBAL de dobras** — ela já é `0,3 %` e não se move com isto. *É a mesma
cegueira de mediana que este módulo pagou quatro vezes.*

---

## §5 — ⛔ O que esta pesquisa NÃO promete

- ⚠️ **Nenhum destes trabalhos é sobre pontas finas.** Eles são sobre injectividade em geral; o
  nosso ombro é apenas onde a distorção aperta mais. *A cura é da classe certa; a medição na nossa
  peça é que decide.*
- ⚠️ **O desemaranhador move geometria no domínio, não muda conectividade.** Se a combinatória
  que o traçado emitiu já for má na ponta, ele desdobra e mantém a má combinatória — e é por isso
  que a medição do **SoG** vem primeiro: ela responde se a combinatória está sã.
- ⛔ **Nada aqui vem de fonte de alvo restrito**, e nada aqui foi transcrito de código externo.

---

## Fontes

- Garanzha, Kaporin, Kudryavtseva, Protais, Ray, Sokolov — *Foldover-free maps in 50 lines of code*
  (2021) — <https://arxiv.org/abs/2102.03069>
- Ray — *On Quad Mesh Extraction From Messy Grid Preserving Maps* (2025) —
  <https://arxiv.org/abs/2507.15404>
- Schüller, Kavan, Panozzo, Sorkine-Hornung — *Locally Injective Mappings* (SGP 2013) —
  <https://igl.ethz.ch/projects/LIM/>
- Du, Aigerman, Zhou, Kovalsky, Yan, Kaufman, Ju — *Lifting Simplices to Find Injectivity*
  (SIGGRAPH 2020) —
  <https://duxingyi-charles.github.io/publication/lifting-simplices-to-find-injectivity/>
- Bommes, Campen, Ebke, Alliez, Kobbelt — *Integer-Grid Maps for Reliable Quad Meshing* (2013) —
  <https://www.graphics.rwth-aachen.de/publication/03197/>
- Ebke, Bommes, Campen, Kobbelt — *QEx: Robust Quad Mesh Extraction* (2013) —
  <https://www.graphics.rwth-aachen.de/media/papers/ebck2013_1.pdf>
- Bracci, Tarini, Pietroni, Livesu, Cignoni — *Towards a robust and portable pipeline for quad
  meshing* (2023) — <https://www.sciencedirect.com/science/article/pii/S0097849323000341>

---

## §6 — ⭐⭐⭐ O PASSO 1 FOI FEITO: o SoG falha, e falha SÓ no ombro

Medido em 30/08 na peça do artista (`Detail 0,85`), distância de cada canto singular ao ponto
inteiro mais próximo:

| `r / Rmax` | cantos singulares | **p50** | máx | a meia célula |
|---|---|---|---|---|
| `[0,00 · 0,50)` | `635` | ⭐ **`0,0000`** | `0,4745` | `20` |
| `[0,50 · 0,75)` | `327` | ⭐ **`0,0000`** | `0,4951` | `24` |
| ⛔⛔ **`[0,75 · 0,90)`** | `33` | ⛔ **`0,1040`** | `0,3152` | `0` |
| `[0,90 · 1,00]` | `23` | ⭐ **`0,0000`** | ⭐ **`0,0000`** | `0` |

⭐⭐⭐ **O ombro é a ÚNICA casca cujas singularidades saem sistematicamente FORA da grade.** Todas
as outras têm mediana **exactamente zero**, e a casca da **ponta** está **perfeita** (`máx 0,0000`).

⇒ **O mesmo bocado de `577` triângulos falha as DUAS propriedades independentes:**

| propriedade | ombro `[0,75 · 0,90)` | resto da peça |
|---|---|---|
| **det+** (sem dobras) | ⛔ `3,12 %` | `0,14 %`–`0,75 %` |
| **SoG** (singularidade na grade) | ⛔ `p50 0,1040` | `p50 0,0000` |

### ⭐⭐ E as duas são SEPARÁVEIS — há evidência, não suposição

A casca `[0,00 · 0,50)` tem `54` dobras (`0,754 %`) e mediana de SoG **`0`**. ⇒ **dobra não implica
singularidade fora da grade.** *São dois defeitos, e o ombro tem os dois.*

⚠️ **O que isto NÃO diz:** qual vem primeiro. As singularidades podem sair da grade **porque** o
mapa ali está dobrado, ou o mapa pode dobrar **porque** elas não assentaram. A ordem só se decide
curando uma e re-medindo a outra — e é por isso que a cura do SoG (fixar as coordenadas das
singularidades conhecidas no problema) vale a pena **antes** do desemaranhador: ela é mais barata e
o resultado dela **discrimina**.

⚠️ **Nota lateral, e é da literatura:** os `44` cantos a **meia célula** nas duas cascas do corpo
são o modo de falha que o *paper* nomeia explicitamente (uma singularidade a `+½` não produz quad
nenhum). Eles são minoria ali — mas existem, e ninguém os tinha visto.

---

## §7 — ⛔⛔ CORRECÇÃO AO §4: a cura do SoG JÁ EXISTE, JÁ ESTÁ LIGADA, e está ESGOTADA

⚠️ **O §4 mandava construir a cura do SoG. O código já a tinha.**
[`RoundOptions::pin_singularities`] e [`RoundOptions::pin_lone_singularities`] são as duas `true`
por omissão **desde 2026-08-25**, com a cadeia causal medida ponta a ponta no doc-comment delas —
*singular não pregado ⇒ imagem fraccionária ⇒ transições fraccionárias ⇒ isolinha cai ao lado ⇒
órfã ⇒ **aresta de bordo, na ponta***.

⇒ *A lei da casa — «confira o CÓDIGO antes de acreditar numa ausência» — poupou uma obra inteira.*
**A minha leitura da literatura estava certa e a minha leitura da nossa árvore estava errada.**

### E a medição do RESÍDUO discrimina, com um zero que decide

| `r / Rmax` | singulares | com UMA cópia | ⛔ **fora da grade** | **dos quais com uma cópia** |
|---|---|---|---|---|
| `[0,00 · 0,50)` | `104` | `55` | `15` | ⭐ **`0`** |
| `[0,50 · 0,75)` | `53` | `14` | `12` | ⭐ **`0`** |
| ⛔ `[0,75 · 0,90)` | `6` | `1` | **`3`** (metade) | ⭐ **`0`** |
| `[0,90 · 1,00]` | `4` | `3` | ⭐ **`0`** | ⭐ **`0`** |

⭐⭐⭐ **NENHUMA das singularidades fora da grade é «solitária».** As `73` que o corte não duplicou
— o alvo exacto da cura de 25/08 — estão **todas** na grade, nas quatro cascas. *A cura funcionou
a 100 % para o que ela mirava, e não há mais nada a ganhar por ali.*

⇒ **O que sobra é outra população:** singularidades **com** fecho, que são pregadas e ainda assim
derivam — `30` na peça inteira, e no ombro **metade das que lá existem**.

### ⇒ A ordem inverte-se, e agora com prova

O §4 punha o desemaranhador em segundo. Ele passa a **primeiro**, porque:

1. o ramo barato está **esgotado** — não há «pregar mais» a fazer;
2. o resíduo do SoG vive onde o mapa está **dobrado**, e num sítio dobrado o sistema linear do
   fecho põe o vértice onde calha ⇒ **o `det+` é o suspeito de causar o resto**;
3. e isso é uma **previsão testável**: curar o `det+` e **re-medir o SoG**. Se o resíduo cair
   junto, a ordem causal fica provada; se ficar, é um terceiro mecanismo e tem de ser nomeado.

⛔ **Sem essa previsão escrita antes, curar o `det+` e ver o SoG melhorar não provaria nada** — é a
diferença entre uma medição e uma coincidência lida a favor.
