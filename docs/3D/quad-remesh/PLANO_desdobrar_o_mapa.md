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

---

## §8 — ⭐⭐⭐ O DESEMARANHADOR EXISTE, E DESFAZ `62,4 %` DAS NOSSAS DOBRAS EM `193 ms`

**A crate:** [`ph2d-untangle`](../../../crates/ph2d-untangle/) — clean-room da literatura pública,
`8` gates. Os dois que decidem: *desemaranha a partir de um estado **já** dobrado* (com controlo de
que a fixtura contém o fenómeno) e *dois interiores que **trocam de lugar** desemaranham* — o
`desk-reject` que a literatura usa contra qualquer método que se diga injectivo.

⛔ **E um gate apanhou um defeito meu que teria sido lido como afinação:** o `two_loop` devolve
direcção de **subida**, e eu tinha a condição de recuo **invertida** *e* passava `+∇F` como
direcção — toda iteração subia, a busca linear recusava tudo, e o laço saía com a malha
**intacta**. ⭐ *O gate do gradiente ficou **verde** durante isso* — e é exactamente para o que ele
existe: **separar a matemática da descida**. Sem ele, o sintoma («não desemaranha») teria mandado
procurar o erro na derivada, que estava certa.

### A medição na peça do artista (`Detail 0,85`), retalho a retalho, fronteira presa

| | |
|---|---|
| retalhos com dobra | `28` |
| ⭐ **dobras** | **`149` → `56`** |
| retalhos que não fecharam | `16` |
| ⭐ **relógio** | **`193 ms`** |

⭐⭐⭐ **`62,4 %` das dobras desaparecem sem tocar na costura**, e o preço é `193 ms` contra uma
cadeia de `15`–`25 s` — **menos de `1,3 %`**.

⚠️ **A forma da prova:** desemaranhar **por retalho com a fronteira do retalho presa** deixa as
transições de carta **intactas** ⇒ a propriedade `GP` que a obra de 24/08 comprou fica preservada
**por construção**, não por promessa.

### ⚠️ E as que sobram estão NOMEADAS

As `56` que ficam vivem **na fronteira dos retalhos** — são precisamente as que a versão restrita
não pode mover. ⇒ **A wave seguinte é libertar as variáveis de costura**, e o caminho já está
escrito no §3: o `ClosureSystem` da obra A exprime a costura por **eliminação de variável**, então
o conjunto reduzido dele é exactamente o espaço em que o desemaranhador pode trabalhar **sem**
perder a `GP`.

### ⛔ DUAS RÉGUAS, DUAS POPULAÇÕES — a reconciliar antes de citar um número sozinho

| régua | conta o quê | dá |
|---|---|---|
| dobras por casca (§6) | a **minoria de sinal** sobre os triângulos do `corner_map` | `92` |
| o desemaranhador | `det J ≤ 0` sobre os triângulos **locais do corte** | `149` |

⚠️ **Os dois `A/B` são internamente consistentes** (cada um mede antes e depois com a *sua* régua),
e é por isso que o `62,4 %` vale. ⛔ **Mas os dois números absolutos não são comparáveis**, e
citá-los lado a lado sem esta nota seria a armadilha do denominador que este módulo já pagou.

---

## §9 — ⛔ O VEREDITO: o desemaranhador funciona, entra na cadeia, e SHIPA DESLIGADO

| coluna | desligado | ⭐ ligado |
|---|---|---|
| dobras da saída | `27` | ⭐ **`21`** |
| aspecto p99 · enviesamento p99 | `1,70` · `33,5°` | ⭐ `1,64` · `30,9°` |
| faces `>60°` · irregulares | `12` · `50` | ⭐ **`10`** · `48` |
| `χ` · bordo · não-manifold | `2` · `0` · `0` | `2` · `0` · `0` |
| ⚠️ gravatas · ponta pior | `1` · `−6,4 %` | ⚠️ `2` · `−6,6 %` |
| ⛔ relógio | `21,4 s` | ⛔ **`33,4 s`** |

⭐ **Todas as colunas de forma melhoram.** ⛔ **E mesmo assim ele não shipa ligado**, e as três
razões são medidas: `+56 %` do relógio do artista a cada clique · **não cura a foto** (a torção
`p99` do ombro anda `35,7° → 34,8°` e o máximo continua em `179°`) · duas colunas pioram.

### ⛔⛔ E o achado que vale mais que a feature: a ESCADA RE-DOBRA

A 1.ª colocação foi **antes** do arredondamento, com uma razão boa escrita ao lado. Medido:

| | dobras no mapa FINAL |
|---|---|
| sem o passe | `149` |
| ⛔ **com o passe antes da escada** | ⛔ **`169`** |

⇒ **A escada de arredondamento re-dobra o mapa — e re-dobra MAIS partindo de um mapa
desemaranhado.** *Uma restrição imposta numa fase e não na seguinte não é uma restrição; é um
ponto de partida* — e a mesma frase já estava escrita no `weld_round.rs` §23.18, sobre outro
assunto, desde Agosto. **A cura foi prender os inteiros, não escolher outra fase.**

### ⏳ A wave seguinte, com alvo

As dobras que sobram vivem **na fronteira dos retalhos** — o que a versão restrita não pode mover.
⇒ **libertar as variáveis de costura** pelo conjunto reduzido do `ClosureSystem`. ⭐ E a hipótese
mais forte, dado o achado acima: **a injectividade tem de entrar DENTRO da escada** (anti-flip por
prego), não como um passe antes ou depois dela.

---

## §10 — ⛔⛔⛔ ONDE NASCEM AS DOBRAS: a minha hipótese do §9 está REFUTADA

O §9 apostava que *«a injectividade tem de entrar DENTRO da escada»*. Medido, com a **mesma
régua** dos dois lados:

| | contínuo (G3) | final (pós-escada) | a escada acrescenta |
|---|---|---|---|
| `Detail 0,85` | ⛔ **`120`** | `149` | `+29` (**`24 %`**) |
| `Detail 0,50` | ⛔ **`73`** | `88` | `+15` (**`21 %`**) |

⭐⭐⭐ **Quatro em cada cinco dobras já nascem no solver CONTÍNUO.** A escada é a minoria.

### E as três curas possíveis estão TODAS medidas — nenhuma sozinha chega

| cura | onde | veredito |
|---|---|---|
| **endurecimento local** (pesar o triângulo virado) | dentro do G3 | ⛔ **construído, medido e REJEITADO em 2026-08-25** (`STIFFEN_PASSES = 0`, tabela no doc dele). A literatura concorda: sem garantia |
| **desemaranhar ANTES da escada** | entre G3 e a escada | ⛔ **pior**: a escada re-dobra e re-dobra MAIS (`149 → 169`) |
| **desemaranhar DEPOIS da escada** | no fim | ⚠️ **funciona e shipa desligado** (§9): `+56 %` de relógio para um ganho que o dono não vê |

⇒ ⭐⭐⭐ **A conclusão é aritmética, e é arquitectural:** curar só a escada deixa `120`; curar só o
contínuo deixa `29` — **e curar o contínuo e depois arredondar dá `169`, pior que não curar nada**.
*Nenhum passe colocado em sítio nenhum resolve isto.*

⇒ **A injectividade tem de ser uma propriedade do OBJECTIVO que a fase inteira optimiza**, e não um
passe antes, dentro ou depois dela. Hoje o G3 minimiza `‖∇f − R/h‖²` — uma Poisson por mínimos
quadrados **sem termo nenhum de injectividade** —, e é exactamente isso que a literatura substitui.

⚠️ **E a objecção que o doc do endurecimento levanta NÃO se aplica a esta rota:** ele conclui
*«no domínio não se cura pesando triângulos: é uma propriedade do CAMPO»*, e tem razão **sobre
pesar triângulos**. A energia com barreira não pesa nada — ela **muda o objectivo**, e traz a
garantia que o peso não tinha. *Uma recusa medida responde UMA pergunta; esta é outra.*

### ⏳ A wave seguinte, com alvo e com o preço à vista

**Substituir o objectivo do G3** pela energia regularizada do §3, sobre o **mesmo conjunto reduzido
de variáveis** do `ClosureSystem` (que é o que preserva a `GP`). ⚠️ **É a obra grande desta linha:**
troca uma relaxação linear por uma descida não-linear na fase mais central da cadeia. ⛔ E o preço
tem de ser medido contra o relógio do artista **antes** de shipar — o passe de `§9` já mostrou que
`+56 %` é fácil de gastar.

---

## §11 — ⚠️ A VIABILIDADE DA OBRA GRANDE: INCONCLUSIVA, e o instrumento é o culpado

O §10 concluiu que a injectividade tem de ser propriedade do **objectivo**, sobre as variáveis que
a costura deixa livres. ⭐ **Antes de reescrever o G3, a pergunta era se essa liberdade CHEGA** —
a lei de *medir se a cura tem sujeito antes de a construir*.

**A sonda** (`seam_free_probe`): resolver o contínuo, depois alternar *descida sem NADA preso* com
`derive` de todas as classes — a costura por **projecção**. Peça do artista, `Detail 0,85`:

```
dobras 120 -> 66   |   curva [90, 65, 68, 68, 65, 66, 69, 68, 76, 71, 68, 66]   |   102 ms
```

⭐ **Cai `45 %` em DUAS rondas e depois OSCILA** — `65`, `76`, `71`, `68`, `66`… Mais iterações não
compram nada.

### ⛔⛔ E o veredito que eu tinha escrito na sonda EXCEDIA o que ela mede

A 1.ª redacção lia só o último número contra um limiar de metade e imprimia *«a obra grande está
condenada»*. ⛔ **Isso é mais do que esta sonda pode dizer**, e a curva prova-o: um planalto **com
oscilação** não é um limite da liberdade — é o **instrumento a lutar consigo próprio**.

⚠️ **O mecanismo, e é da construção da sonda:** a costura entra por **projecção**, e o `derive`
empurra o valor da cópia **RAIZ** para todas as outras ⇒ **todo o trabalho que a descida fez nas
cópias não-raiz é deitado fora a cada ronda.** *A descida desfaz e a projecção refaz, para sempre.*

⇒ ⭐⭐ **A obra grande não é isto.** Ela exprime a energia **nas variáveis livres** do
`ClosureSystem` — a costura por **eliminação**, não por projecção —, e aí a descida **nunca produz
um estado que a restrição tenha de desfazer**. *A sonda mediu um espantalho da obra, não a obra.*

⇒ ⏳ **A viabilidade continua por decidir**, e o instrumento que a decide é a própria obra. ⚠️ O
que esta medição compra é o **limite inferior**: mesmo com a costura a lutar, `45 %` das dobras do
contínuo cedem em `102 ms`. *Não é uma promessa; é um piso.*

---

## §12 — ⭐⭐⭐ A OBRA GRANDE EXISTE, E O §11 ESTAVA CERTO SOBRE O ESPANTALHO

`ph2d_gridmap::make_injective` desce a energia de barreira regularizada **nas raízes das classes
de costura**. A costura é a **variável**, não uma restrição imposta por cima: cada cópia é
`uv = R^k · raiz + t`, com `k` e `t` extraídos **uma vez** do mapa consistente que entra. ⇒ todo
estado que a descida visita satisfaz a costura **exactamente**, e não há projecção nenhuma a
desfazer o trabalho.

Peça do artista, `Detail 0,85`, a mesma régua dos dois lados:

| | dobras no mapa contínuo | relógio |
|---|---|---|
| §11, a costura por **projecção** (o espantalho) | `120 → 66`, e depois **oscila** | `102 ms` |
| ⭐⭐⭐ **§12, a costura por ELIMINAÇÃO (a obra)** | **`120 → 33`** | `4,6 s` |

**`−72,5 %`**, sem oscilação, e o pior determinante melhora **duas ordens de grandeza**
(`−1,977e3 → −1,581e1`). ⇒ **o §11 estava certo ao recusar o veredito da sonda**: o planalto com
oscilação era o instrumento a lutar consigo próprio, e não um limite da liberdade que a costura
deixa.

### ⭐⭐ E ela CONVERGIU — a varredura do orçamento decide, e o eixo que manda é o `ε`

⛔ A 1.ª medição gastou o orçamento **todo** (`64` externas / `2048` internas = exactamente os
tectos). *`33` truncado e `33` convergido são a mesma impressão e duas conclusões opostas* — daí a
varredura, pela env (`PH2D_INJ_OUTER` / `PH2D_INJ_INNER`):

| externas | internas | **dobras** | `min det` final | relógio |
|---|---|---|---|---|
| ⭐ **`64`** | **`32`** | `33` | `−1,581e1` | ⭐ **`4,6 s`** |
| `256` | `32` | `32` | `−6,544e0` | `20,3 s` |
| ⛔⛔ `64` | `128` | ⛔ **`40`** | `−1,138e1` | `16,9 s` |
| `256` | `128` | `31` | `−8,259e0` | `72,0 s` |

⭐⭐⭐ **`16×` o orçamento compra DUAS dobras e paga `15,7×` o relógio.** O método assentou em
`31`–`33`; ele não estava truncado em nenhum sentido útil.

⛔⛔ **E a linha `64 × 128` é o achado desta varredura: mais trabalho no eixo ERRADO piora o
resultado** (`40` contra `33`, com `4×` as iterações internas). *A célula que discorda é a que
escolhe* — as internas gastam-se a resolver com precisão uma barreira ainda **esborratada**,
porque quem faz o `ε` encolher é o laço **externo**. ⇒ ⭐ **o motor desta obra é o calendário do
`ε`, não a descida**, e o ponto de operação de omissão (`Settings::default`) está certo por
medição e não por herança.

### ⚠️ O que a obra NÃO entrega

`31`–`33` dobras **não é zero**, e a propriedade que a literatura promete é `det+` em todo o
elemento. ⇒ **a obra reduz o defeito; ela não o elimina**, e a diferença tem de estar escrita onde
alguém a leia antes de prometer uma malha injectiva a jusante.

⚠️ **Correcção de método, e vale para toda a varredura acima:** o binário foi **reconstruído a
meio** dela (as duas primeiras linhas correram num binário e as duas últimas noutro, com o ramo
novo do produto pelo meio). O ramo nasce atrás de `PH2D_GRIDMAP_INJECTIVE` e a varredura não a
põe, logo ele é **inerte** — mas *«inerte» é uma afirmação, e por isso a linha `64 × 32` foi
**re-corrida no binário final** como controlo.

---

## §13 — ⛔⛔⛔ CORRECÇÃO AO §12: os números dele saíam de um DEFEITO DE UNIDADES

⚠️ **Tudo o que o §12 mede está certo sobre o código que ele correu, e esse código tinha um bug
de escala.** O referencial de repouso de cada elemento era construído a partir do triângulo 3D
**em unidades do mundo**, e o termo `g(J) = (det²J + 1)/det J` da energia é minimizado em
**`det J = 1`** ⇒ ele pedia *uma célula por unidade de área do mundo*, quando o alvo do G3 é *uma
célula por `h`*, com `h ≈ 0,038`. **A barreira lutava contra a densidade pedida, e a varredura de
orçamento do §12 mediu o empate dessa luta.**

⭐ Com o repouso **em unidades de célula** (dividido pelo passo do triângulo, pela porta
`Step::at`), a mesma peça, o mesmo dia:

| | dobras no contínuo | `min det` | iterações | relógio |
|---|---|---|---|---|
| §12 (repouso em unidades do mundo) | `120 → 33` | `−1,977e3 → −1,581e1` | `64` externas (**o tecto**) | `4,6 s` |
| ⭐⭐⭐ **§13 (repouso em CÉLULAS)** | **`120 → 0`** | ⭐ **`−2,870 → +1,245e−4`** | **`5`** de `64` | ⭐ **`352 ms`** |

⭐⭐⭐ **O mapa contínuo fica LOCALMENTE INJECTIVO** — a propriedade `det+` que a literatura
promete —, converge (gasta `5` das `64` externas: *não é truncagem*), e custa **`13×` menos**
relógio, porque chegar é mais barato que bater no tecto.

⇒ ⚠️ **A varredura de orçamento do §12 fica REGISTADA e SEM VALOR PREDITIVO**: ela mediu o
planalto de um problema mal posto. *Uma varredura de afinação sobre um objectivo errado escolhe o
melhor ponto de um mapa que não se quer.* A lição que sobrevive é a forma da célula `64 × 128`
(mais trabalho no eixo errado piora), que é sobre o método e não sobre as unidades.

⇒ ⭐⭐ **E é o §0.0 do `CLAUDE.md` outra vez:** *um limite legítimo diz de que recurso ele é.* O
`33` foi lido como *«o limite do método»* e era **o limite de uma unidade errada**.

## §14 — ⛔⛔⛔ E COM O MAPA PERFEITO O PRODUTO SAI PIOR — a escada é a culpada, por aritmética

A/B ponta a ponta pelo botão, peça do artista, `Detail 0,85` (a tabela viva mora no doc de
[`ph2d_gridmap::injective_solve::enabled`], que é onde alguém a lê antes de ligar isto):

| coluna | controlo | com a obra |
|---|---|---|
| quads | `9 598` | ⛔ `14 521` |
| enviesamento p50 | `6,4°` | ⛔ **`21,3°`** |
| faces `>60°` | `2` | ⛔ **`1 191`** |
| defeitos locais | `0,48 %` | ⛔ **`4,83 %`** |
| `χ` · bordo | `1` · `4` | ⛔ `0` · `12` |
| faces dobradas na extracção | `22` | ⛔ **`415`** |
| ⭐ pontas cortadas | `2` de `12` | ⭐ **`1` de `12`** |
| ⭐ cobertura p50 | `0,271 %` | ⭐ **`0,061 %`** |

⭐⭐⭐ **A localização do defeito não precisa de mais nenhuma sonda:** o mapa que entra na escada
tem **zero** dobras e o que sai da extracção tem **`415`** faces dobradas. *Um input impecável a
produzir um output pior põe o dano a jusante, sem margem para interpretação.* A escada gulosa
prega os inteiros um a um e re-relaxa entre pregos — a partir de outro ponto de partida ela faz
outras escolhas, e piores.

### ⚠️ E há um SEGUNDO mecanismo, este de desenho

O G3 minimiza `‖∇f − R/h‖²`, que fixa **a escala e a ORIENTAÇÃO contra o campo cruzado**. A
energia de barreira fixa a escala (termo `g`) e a conformidade (termo `f`), e **não tem termo
nenhum a amarrar o mapa ao campo** ⇒ as linhas de grade podem rodar em relação a ele. *É isso que
o enviesamento lê.*

⇒ ⭐⭐ **A obra seguinte está NOMEADA, e não é esta:** a barreira entra **somada** ao objectivo do
G3 — `‖∇f − R/h‖² + w · barreira` —, nunca a **substituí-lo**. É literalmente o que o §10 pedia
(*propriedade do objectivo que a fase optimiza*), e o que esta jornada construiu e provou é a
**maquinaria** dela, toda gateada: a costura como **variável** (a costura sai exacta, com gate
que verifica cópia a cópia), o **calendário de `ε`**, o **repouso em células** (com gate de
invariância de escala) e a prova de que, a partir de emaranhado, ela **chega a zero**.

⚠️ **Duas colunas MELHORARAM e não se apagam:** a peça perde **menos** pontas (`1` contra `2` —
que é a queixa do dono) e a fidelidade à escultura fica **`4,4×`** melhor. *A direcção está
certa; o que falta é não pagar por ela na forma dos quads.*

⇒ ⛔ **Shipa DESLIGADO** (`PH2D_GRIDMAP_INJECTIVE=1` liga), e o caminho de omissão é
byte-idêntico — há gate a afirmar as duas metades (o interruptor **e** que o produto o consulta).
