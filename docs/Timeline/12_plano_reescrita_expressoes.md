# PLANO — REESCRITA COMPLETA da feature de Expressões

> Ordem do Enio, 2026-07-29, depois de reprovar a implementação atual:
> *"Escrever um plano de reescrita completa da feature de Expressões. Eliminar as
> expressões similares umas às outras. Refazer todo o layout de forma profissional e bela.
> Reescrever todo algoritmo 1 a 1 com smoke de pequenos grupos (3 expressões por vez).
> Planejamento extensivo detalhado."*
>
> **Pré-requisito:** a auditoria de [`11_HANDOFF_AUDITORIA_EXPRESSOES.md`](11_HANDOFF_AUDITORIA_EXPRESSOES.md).
> Este plano foi escrito **antes** dela, a partir dos reports — a auditoria tem os números
> e **manda**. Onde ela contradisser este doc, ela ganha, e a contradição é registrada aqui
> em vez de apagada.
>
> ---
>
> ## ⚠️ A AUDITORIA RODOU (2026-07-29) — resultado em [`13_RESULTADO_AUDITORIA_EXPRESSOES.md`](13_RESULTADO_AUDITORIA_EXPRESSOES.md)
>
> Duas lentes independentes + medição do catálogo inteiro. **Este plano foi CORRIGIDO no
> lugar**; toda correção está marcada `⚠️ AUDITORIA` com o número que a produziu. As
> premissas que ela **falsificou**, e que este doc afirmava:
>
> | o plano dizia | a medição diz |
> |---|---|
> | *"sobra ~180 px para o nome"* (§5.1) e a cura é um card MAIOR (§5.3) | sobram **198 px**; o defeito nº 1 de layout é que **o card não é modal** — clicar a barra de fórmula edita o `Dur(s)`. Um card maior sobrepõe MAIS widgets |
> | *"nenhuma receita é inerte no seu default … excursão > 1%"* (§2.2) | **insatisfazível**: todo MODIFICADOR tem excursão 0 por construção. O critério cortaria 12 receitas que este plano MANTÉM |
> | *"11 valores por knob, incluindo extremos e default"* (§3.1) | **pula o −1** — e é o −1 que prova `speed ~> reverse-time`, uma das fusões que este plano afirma |
> | Ping-Pong/Pulse/Blink · Distance/Distance 1D · Freeze/Start são *"a mesma"* (§3.2) | **nenhuma contenção medida** entre elas. São decisões de PRODUTO, não cortes por redundância |
> | Shape tem 10 · Logic tem 7 (§3.2) | **9** e **6** |
>
> E **cinco defeitos que este plano não lista em D1..D9** entraram na §1.2 (D10..D14).
>
> Este plano **substitui** o [`10_plano_editor_de_expressoes.md`](10_plano_editor_de_expressoes.md)
> como plano ativo. O 10 fica como histórico: é a fonte das decisões que foram reprovadas, e
> saber que elas foram TENTADAS vale mais que apagá-las.

---

## §0 — O ESTADO DA ARTE, e as seis decisões que ele toma (2026-07-29)

> Ordem do Enio: *"Decida buscando o estado da arte e comece."* O que segue não é levantamento —
> é **decisão tomada**, com a fonte ao lado e o efeito sobre este plano. Onde o estado da arte
> contradiz uma premissa deste doc, ela cai aqui e a §correspondente é corrigida.

Quatro produtos respondem à MESMA pergunta que esta feature (*como um artista dirige uma
propriedade sem keyframes?*) e **os quatro convergiram na mesma arquitetura**:

| produto | o que é | quantos | o que ele tem que nós não |
|---|---|---|---|
| **Blender** — [F-Curve Modifiers](https://docs.blender.org/manual/en/3.3/editors/graph_editor/fcurves/modifiers.html) | pilha de modificadores **no canal** | **9** (Generator · FN Generator · Envelope · Cycles · Noise · Filter · Python · Limits · Stepped) | **Influence** (% de efeito) + **Restrict Frame Range** (start/end + blend in/out) por linha |
| **Cavalry** — [Behaviours](https://docs.cavalry.scenegroup.co/nodes/behaviours/) | drivers ligados a um **atributo**, empilháveis | **40+** | **[Falloff](https://cavalry.studio/docs/nodes/utilities/falloff/)** atenua qualquer behaviour · **[Common Attributes](https://docs.cavalry.scenegroup.co/nodes/behaviours/common-attributes-behaviours/)** (um cabeçalho igual para todas) · **[Behaviour Mixer](https://cavalry.studio/docs/nodes/behaviours/behaviour-mixer/)** |
| **Apple Motion** — [Parameter Behaviors](https://support.apple.com/guide/motion/intro-to-parameter-behaviors-motn49eb56eb/mac) | behaviour aplicado a **um parâmetro** | ~14 ([Oscillate](https://support.apple.com/guide/motion/oscillate-behavior-motn13745133/mac) · Randomize · [Wriggle](https://support.apple.com/guide/motion/wriggle-behavior-motn137475d5/mac) · [Rate](https://support.apple.com/guide/motion/rate-behavior-motn13747831/mac) · [Ramp](https://support.apple.com/guide/motion/ramp-behavior-motn137432ba/mac) · [Quantize](https://support.apple.com/guide/motion/quantize-behavior-motn1374610f/mac) · Reverse · Stop · Average · Negate · Track · MIDI · Audio · Custom) | nomes que são **frases de artista**, e a lista inteira cabe num menu |
| **After Effects** — [expressões](https://helpx.adobe.com/after-effects/using/using-expressions-editor.html) | **texto** + 4 botões | ∞ | o botão nº 2 **plota o resultado no GRAPH EDITOR de verdade** · o nº 3 é o **pick-whip** · o nº 4 é o menu da linguagem |

**D-0.1 — A arquitetura da pilha está CERTA; não há reescrita de modelo.** Blender empilha
F-Modifiers num canal, Cavalry empilha Behaviours num atributo (*"each layer adds to the result of
the layers below it so layer order is very important"* — a nossa lei de ordem, escrita por eles), e
Motion empilha Parameter Behaviors num parâmetro. O `RecipeStack` **é** o modelo da indústria.
⇒ A FASE E (*reescrever algoritmo por algoritmo*) fica; a **arquitetura sai do banco dos réus**.

**D-0.2 — O TAMANHO do catálogo não é o defeito, e a manchete deste plano estava errada.** Cavalry
shipa **40+** behaviours e ninguém reclama. ⇒ **A meta "50 → ~21" é ABANDONADA como meta.** O corte
continua, com o critério da auditoria e não um headcount: sai o que é **INERTE** (23 de 50 não fazem
nada escolhidas sozinhas) e o que é **PROGRAMAÇÃO** (as 6 de Logic). O alvo cai em **~27-30**, e o
número é o RESTO da regra, nunca a regra. *Um catálogo grande com tudo vivo é o estado da arte; um
catálogo pequeno com metade inerte é o que o Enio reprovou.*

**D-0.3 — `RowKind::Time` NÃO vira atributo do bloco.** A recomendação B3 deste plano (um cabeçalho
*"Clock: normal / stepped / delayed"*) **colapsaria em um relógio por pilha** o que Motion ship como
**behaviours por-parâmetro empilháveis** (`Rate`, `Reverse`, `Stop`) — dá para acelerar uma coisa e
reverter outra na mesma pilha, e o cabeçalho proíbe isso. ⇒ **B3 é REESCRITO**: a linha continua uma
LINHA, o que muda é que o **escopo dela aparece na tela** (*"afeta as linhas abaixo"*) e que uma
linha de Time **não é oferecida sozinha** (ela não tem o que retimar) — o critério por-KIND da §2.2,
não um modelo novo. ⚠️ Motion ship `Reverse`/`Stop` sabendo que só fazem sentido sobre um parâmetro
JÁ animado: o vão não é a existência delas, é **a UI não dizer isso**.

**D-0.4 — A FITA é o botão nº 2 do AE, e a resposta é a ESCALA do graph editor.** O AE plota o
resultado da expressão **no graph editor de verdade**, na escala real, contra a curva real. A nossa
normaliza por min/max ⇒ uma constante desenha uma reta no meio, indistinguível de *"não funciona"*
(D1 + §5.2.6). ⇒ a fita ganha **linha de base + escala em unidades**, e usa o **`__seed` do objeto**
(D11). Isto **confirma** a §5.3 e dá a ela uma fonte.

**D-0.5 — O knob que falta no catálogo é INFLUENCE por linha** (Blender) / **Falloff** (Cavalry).
É ele que torna um modificador **dosável** — e é a saída limpa para o `Flicker` multiplicativo numa
translação de base 0 (§8): não é escolher entre aditivo e multiplicativo, é poder aplicar **30% de
uma receita**. ⚠️ **Não é FASE 0**: entra como decisão da FASE E, com medição, porque muda a
aritmética de TODA linha e o neutro tem de sair byte-idêntico.

**D-0.6 — `Restrict Frame Range` (Blender) é a resposta ao item aberto *"expressão PURA extrapola a
strip"***, aberto por ordem do Enio desde 2026-07-27. O vínculo que aquele item pede *"autorado,
explícito"* é exatamente o start/end + blend do F-Modifier, **por linha**. ⚠️ **NOMEADO, não
construído** — é `DOC_VERSION` e decisão de produto; fica registrado aqui para ninguém re-derivar a
pergunta.

⚠️ **E uma coisa que o estado da arte NÃO nos dá:** nenhum dos quatro tem o problema do **`__seed`**
(D-J) nem do **card não-modal** (D10), porque em nenhum deles o preview é uma janela flutuante sobre
o transporte com um avaliador próprio. Esses dois são nossos, e são a **FASE 0**.

---

## §1 — O que foi reprovado, e o diagnóstico de causa raiz

Cinco rodadas de smoke, e a queixa mudou de forma a cada uma — o que é a assinatura de um
problema de **método**, não de bugs independentes.

### 1.1 — As três causas raiz

**CR-1 — O catálogo foi dimensionado por AMBIÇÃO, não por uso.** 55 receitas em 9 famílias,
derivadas de um levantamento do que AE/Cavalry/Motion oferecem. Nenhuma foi validada contra
*"um animador faria isto?"*. Consequências medidas: 5 eram identidade matemática de outra;
7 (Logic) são programação com limiar numérico; 7 (Time) são inertes sem uma linha
específica abaixo; 14 nasciam constantes por falta de alvo. **Mais da metade do catálogo era
ruído**, e o ruído é o que o Enio encontrou primeiro porque a galeria é ordenada por
família, não por utilidade.

**CR-2 — A UI foi orçada em SLOTS FIXOS e o conteúdo cresceu por baixo dela.** `BODY_SLOTS = 12`
foi *derivado* de "a maior família tem 10 receitas + busca + voltar". Depois a planilha
passou a gastar `1 + knobs` por linha, e três linhas de Turbulence já estouram — e a
resposta shipada foi **`+1 more rows`**, texto em vez de rolagem, num app que **tem**
primitivo de scroll com gate próprio. O doc-comment `"Nothing here scrolls, and the geometry
says why"` **deixou de ser verdade e ninguém reconferiu a nota**.

**CR-3 — O oráculo de aceitação era a FÓRMULA, e o produto é a TELA.** Eu media excursão de
fórmula, contagem de linhas, identidade de texto. O Enio olha um objeto se mexer e um
gráfico desenhar. Três rodadas de "medi e está bom" contra "não funciona" saem daí. **Todo
critério de aceitação deste plano é a tela ou um número que a descreve.**

### 1.2 — Os defeitos concretos que a reescrita tem de matar

| # | sintoma (palavras do Enio) | mecanismo |
|---|---|---|
| D1 | *"gráfico plano de flick"* | receita multiplicativa sobre `value = 0` dá exatamente 0 |
| D2 | *"quase tudo em Time não funciona"* | uma linha Time só age nas linhas ABAIXO; `wiggle` nem obedece |
| D3 | *"não vejo o menor sentido para artistas na seção logic"* | 7 condicionais com limiar numérico |
| D4 | *"mesmo deletando as expressões, elas ficam atuando"* | ver §3.2 do handoff: 3 candidatos, um confirmado por leitura (assimetria de escrita), um forte (prop sem keys congela) |
| D5 | *"layout absurdo, tudo apertado"* | 536 px de card, 190+320 de coluna, botões de 22 px, sem hierarquia visual |
| D6 | *"não tem scroll nem barra de scroll"* | slots fixos + `+N more rows` |
| D7 | `Detail = 0` na caixa, `1` na fórmula | clamp silencioso na emissão discorda do clamp do widget |
| D8 | *"não atualiza para o novo objeto"* (parcialmente corrigido) | o card não mostra NOME, então seguir a seleção é invisível |
| D9 | Link inusável | não existe pick-whip; nome digitado que não resolve é 0 silencioso |

⚠️ **AUDITORIA — os cinco que faltavam, todos medidos** (doc 13 §4/§4-bis):

| # | sintoma | mecanismo medido |
|---|---|---|
| **D10** | *"Layout absurdo"* — e a causa não é aperto | **o card NÃO É MODAL**: o fundo não registra hit rect ⇒ **18 widgets do transporte vivos sob a pegada**; clicar o centro da barra de fórmula dá `hit_at = TIMELINE_LENGTH_NUM` e **digitar edita o `Dur(s)` da composição**. A roda zooma a timeline atrás (`px_per_s` 120→326) |
| **D11** | *"não produz a curva do grafo de preview"* | **a FITA usa um `__seed` diferente do objeto** — três respostas para uma pergunta (cena `target*100` · fita **0** · censo `0,96`). O objeto #2 de um Jitter desloca **0,9 px**. O código declara a lei que a fita quebra: *"a preview with its own seed … which is the one thing it must never do"* |
| **D12** | *"mesmo deletando, ficam atuando"* — a **outra** metade | **esconder o painel deixa o preview dirigindo o objeto para sempre**, ANIMANDO (`x` 100→160→…), com `has_pending_restore() = false` ⇒ a pose nunca volta. Nenhuma UI na tela para parar |
| **D13** | idem — a metade **medida** | prop **SEM keys**: `value + 250` → DELETE + Apply ⇒ **fica em 250,0000**. Com keys volta a 7,0000. O `take_restore` cobre só o fim do PREVIEW |
| **D14** | (integração) | **`DOC_VERSION` desta branch é 16**, não 15 — v15 do `main` é RECUSADO no load. Se o B1 redesenhar o per-clip, **voltar a 15 quebra os v16 já salvos** |

⚠️ E **D2 tem um agravante medido**: uma row além da capacidade tem **ZERO widgets** enquanto
**dirige o objeto** (4 rows de Turbulence ⇒ rows 2-3 sem um pixel de UI, e a fórmula que roda
contém as quatro). O `+N more rows` não é só falta de scroll: é uma row viva inalcançável.

---

## §2 — Princípios da reescrita (os inegociáveis desta feature)

1. **Toda receita passa no teste do animador**: existe uma frase da forma *"eu quero que
   isto <verbo> "* que um animador diria, e a receita é a resposta. Se a frase precisa de um
   número que não é uma quantidade da cena (um limiar, uma tolerância, um índice de
   octave), a receita não entra.
2. **Nenhuma receita é inerte no seu default, em nenhuma propriedade.** ⚠️ **AUDITORIA — o
   critério que este plano escreveu é INSATISFAZÍVEL e o erro é o que a CR-3 dele nomeia**
   (*medir a fórmula e reportar a tela*): *"excursão em 4 s > 1%"* mede **animação**, e **todo
   MODIFICADOR tem excursão 0 por construção** — ele recebe `value`, e sobre um `value`
   constante a saída é constante. O critério original cortaria os 9 `Shape` e os 3 `if-*`, e
   este plano MANTÉM 4 de Shape.

   **O critério é por KIND, e a medição é diferente para cada um:**
   * **SOURCE** (`combine: Some`): *anima?* — variação no tempo em 4 s > 1% da faixa da prop,
     na base 0 **e** na base 1.
   * **MODIFICADOR** (`combine: None`): *muda o valor que entra?* — maior `|receita −
     identidade|` sobre uma grade `(time, value)` > 1% da faixa. **Nunca** amplitude do stack:
     o censo de hoje faz isso e por isso reporta *"defeitos de verdade: 0"* enquanto duas
     receitas são a identidade ao bit.
   * **`RowKind::Time`**: só é mensurável **com uma linha embaixo**; sozinha ela emite
     literalmente `value`.

   Medido com esse critério, os únicos reprovados são **`remap` e `multiply-add`** (delta
   **0,000000** — a identidade exata nos seus defaults), e **este plano mantém os dois** ⇒ ou
   eles ganham defaults não-identidade, ou a regra os isenta **por escrito**.

   Sem exceção quanto ao resto: se o modelo exige um contexto (uma linha acima, um link), a
   receita **não é oferecida** até o contexto existir (§5.4).
3. **O default nunca tira o objeto do quadro.** Canvas 4K a 100 px/m = 40,96 m. Default
   ≤ 1/50 do canvas para uma quantidade de posição.
4. **Duas receitas que se reproduzem uma à outra por ajuste de knob são UMA.** Provado por
   busca em grade, não por opinião (§3).
5. **Uma porta por pergunta.** A feature já pagou por três violações: dois mapas de nomes,
   dois escritores de expressão, dois clamps discordantes.
6. **O aceite é a TELA.** Cada wave fecha com um smoke que um humano roda e um número que
   descreve o que ele deveria ver.
7. **Grupos de TRÊS.** Nenhuma wave entrega mais de 3 receitas. O Enio pediu isso
   explicitamente, e é a cura direta da CR-3: 3 receitas é o que cabe num smoke que uma
   pessoa faz com atenção.
8. **`ph2d-expr` continua CONGELADO** (ADR-0039). Nada aqui pede `exp`, `atan2` ou `<=`. Se
   uma receita precisar, ela é **cortada** ou espera um ADR próprio.

---

## §3 — FASE A: o corte do catálogo (a eliminação das similares)

**Entrada:** a tabela de 50 linhas + a matriz de redundância da auditoria (Blocos 4.12/4.13
do handoff). **Sem elas, não comece.**

### 3.1 — O critério de similaridade, executável

Para cada par `(A, B)`: busca em grade sobre os knobs de A, avaliando ambas as fórmulas em 600
amostras de `(time, value)`. Classifique:

⚠️ **AUDITORIA — a grade que este plano especificou tem um buraco MEDIDO.** *"11 valores por
knob, incluindo os dois extremos e o default"* **não basta**: `speed` tem faixa `(−10, 10)`, e
11 passos uniformes dão `−10, −8, −6 …` — **o −1 nunca cai na grade**, e é exatamente o valor
que faz `speed` reproduzir `reverse-time`, uma das fusões que a §3.2 abaixo afirma. A grade tem
de incluir os **valores canônicos** (−1, 0, ½, 1, 2, quando na faixa) **e os pontos NEUTROS
declarados** pela receita (`Neutrality::Additive`). Efeito medido: **22 → 31 relações**.

⚠️ **E receitas INERTES saem da matriz.** A 1ª corrida da auditoria reportou **74 pares
"IDÊNTICOS"**, e 72 eram o clique das 9 receitas que produzem a identidade comparando-se entre
si — **duas coisas inertes são sempre idênticas**: razão entre dois doentes, não redundância.
Uma receita `RowKind::Time` entra na matriz **com uma linha embaixo**, senão ela é uma das
inertes.

* **IDÊNTICA** (pior delta < 1e-5): as duas são a mesma; **uma sai**, e a que fica é a que
  tem o knob que expressa a outra.
* **CONTIDA** (existe ajuste de A que reproduz B, mas não o contrário): B sai, e A herda as
  buscas de B.
* **PARENTE** (a diferença é só de FASE ou de SINAL): **uma sai**, e a diferença vira um
  knob (`Phase`, `Invert`) se ainda não existir.
* **DISTINTA**: as duas ficam, e a tabela registra **em uma frase** o que as separa — é essa
  frase que vai para o blurb, porque *"mais do mesmo"* é uma queixa sobre a EXPLICAÇÃO tanto
  quanto sobre o conteúdo.

⚠️ Já medido nesta jornada (5 IDÊNTICAS, já cortadas): `Sway (Cosine)` = Sway com Phase ·
`Ramp Loop` = Pulse (Decay 1, On/Off trocados) · `Mirror` = Opposite (Pivot 0) · `Midpoint` =
Blend Two (0.5) · `Negate` = Multiply/Add (−1). E já medido que **Sway ≠ Breathe** (delta
0,075) — o Enio os citou juntos, e a resposta a ele **não é cortar um**, é a frase de blurb
que os separa (`Breathe` nunca vai abaixo do valor).

### 3.2 — O corte por FAMÍLIA, com o alvo declarado

O catálogo alvo é **~20 receitas em 5 famílias**. Números por família são propostas a
confirmar pela tabela da auditoria:

| família hoje | receitas | proposta | racional |
|---|---|---|---|
| **Life** (6) | Shake, Turbulence, Drift, Jitter, Breathe, Flicker | **4** | Turbulence é Shake com octaves — `Detail`/`Roughness` são knobs de implementação (D7 nasceu de um deles) ⇒ fundir em Shake com um knob `Detail`. Flicker é multiplicativo ⇒ ou vira aditivo com uma faixa honesta, ou vira `Flicker (Opacity)` e só é oferecida em props 0..1 (§5.4) |
| **Wave** (7) | Sway, Bounce, Ping-Pong, Blink, Pulse, Orbit X/Y | **5** | Ping-Pong e Pulse e Blink são a MESMA pergunta (*que forma tem o ciclo?*) ⇒ candidata forte a UMA receita `Cycle` com um chip de forma (senoide / triângulo / dente / quadrada / pulso). **Isso é uma decisão de produto: um seletor de forma é mais ou menos descobrível que 4 cards?** — Enio decide |
| **Link** (7) | Follow, Opposite, Offset Copy, Distance, Distance 1D, Blend Two, Switch | **3** | Distance/Distance 1D é a mesma com menos eixos. Switch é Logic disfarçado. ⚠️ **Nenhuma delas é usável sem pick-whip** (D9) — a família só volta à galeria DEPOIS da §6 |
| **Shape** (10) | Limit, Floor At, Ceiling At, Remap, Remap Clamped, Multiply/Add, Invert Range, Absolute, Quantize, (Negate cortada) | **4** | Floor/Ceiling são Limit com um lado. Remap/Remap Clamped diferem por um clamp ⇒ checkbox. Estas são MODIFICADORES e sobrevivem ao teste do animador (*"não deixa passar de"*, *"em passos de"*) |
| **Time** (7) | Stepped, Delay, Speed, Reverse, Freeze After, Start At, Ping-Pong Time | **2** | D2. Reverse é Speed com −1. Freeze/Start são o mesmo clamp em lados opostos. E o modelo (*age nas linhas abaixo*) tem de aparecer na TELA (§5.3) |
| **Logic** (7) | If Greater/Less/Near, Gate And/Or, After Time, (Switch) | **0** | **D3 — a família SAI inteira.** É programação. O caso de uso legítimo (*"acontece a partir de tal segundo"*) é um KEYFRAME, e o app tem timeline |
| **Field** (3) | Fade by Distance, Scale by Proximity, Gradient by Value | **0-1** | as três são `Remap(Distance)` — ou seja composição de duas receitas que já existem. Uma delas pode sobreviver como atalho SE a auditoria mostrar que a composição é penosa |
| **Physics** (4) | Pendulum, Free Fall, Throw, Wave Along Chain | **2** | Free Fall é Throw com velocidade 0. Wave Along Chain precisa de link (§6). O decaimento é LINEAR porque não há `exp` — a nota atual admite; um Pendulum que não assenta como pêndulo **é uma promessa quebrada** e o blurb tem de dizer |
| **Raw** (1) | Custom Formula | **1** | fica. É a saída de emergência, e é o que o card usa para SEMEAR uma fórmula existente |

**Meta: 50 → ~21.** ⚠️ Este quadro é uma proposta; **a tabela da auditoria decide**. O que
NÃO é negociável é o critério (§2.1/§2.2/§3.1) e o corte da Logic.

⚠️ **ESTADO DA ARTE (§0, D-0.2) — a META "~21" está ABANDONADA.** Cavalry shipa **40+** behaviours;
o tamanho nunca foi o defeito. O corte fica, com o critério da auditoria (**inerte** ou
**programação**), e o alvo passa a ser o **RESTO da regra** — medido em **~27-30**, não um headcount.
Ler este quadro como uma cota é o erro que ele mesmo cometeu.

---

⚠️ **AUDITORIA — a tabela acima corrigida pela medição** (doc 13 §3/§7). Duas contagens estão
erradas e **três racionais afirmam redundância que não existe.**

**Contagens:** Shape tem **9**, não 10 (a tabela conta `Negate`, já cortada) · Logic tem **6**,
não 7 (ela lista `If Near` — o id é `if-equal` — e conta `Switch`, que está em **Link**). O
total é **50**.

**Confirmado por medição — pior delta 0,000000 em todos:**
`turbulence ~> shake` · `throw ~> free-fall` · `speed ~> reverse-time` · `limit ~> floor-at` ·
`limit ~> ceiling-at`.

**REFUTADO — nenhuma contenção medida; se forem cortadas, é decisão de PRODUTO e o doc tem de
dizer isso:**

| o racional afirma | a medição |
|---|---|
| *"Ping-Pong e Pulse e Blink são a MESMA pergunta"* | **sem contenção entre as três.** São formas distintas (triangular · quadrada · dente com decaimento). O `Cycle` com chip de forma continua defensável — mas como **produto**, não como corte por redundância |
| *"Distance / Distance 1D é a mesma com menos eixos"* | **sem contenção.** Leem 4 links contra 2 |
| *"Freeze / Start são o mesmo clamp em lados opostos"* | **sem contenção** |

**NÃO PREVISTO pela tabela — cinco relações medidas que mudam duas famílias:**

* `offset-copy ~> follow` e `blend-two ~> follow` (delta 0) ⇒ **`follow` é a SUBSUMIDA, não a
  espinha.** A proposta *"Link → 3: Follow · Offset Copy · Distance"* mantém a receita que sai
  de graça de outra (Offset 0).
* `follow ~> opposite` (delta 0) ⇒ `opposite` também sai (multiplicador −1).
* `limit ~> remap-clamped` **E** `remap-clamped ~> limit` — **MÚTUA** ⇒ são a mesma receita, e
  `remap ~> invert-range` + `multiply-add ~> invert-range` subsumem `invert-range` **por duas
  vias**. **Shape cai mais que os 4 propostos.**
* `if-greater ~> if-less` **MÚTUA** + `gate-and`/`gate-or ~> switch` ⇒ **5 das 6 de Logic
  colapsam em 2 formas**, o que **REFORÇA** o corte da família inteira.
* `wave-along-chain ~> sway` (delta 0, em Offset 0) ⇒ a tabela mantém as duas, em grupos
  diferentes (G1 e G6), sem notar que uma contém a outra.

**Sobre `Flicker`:** a medição confirma o mecanismo (`rng@v0 = 0,0000` · `rng@v1 = 0,3451`) ⇒ a
bifurcação que o racional propõe é a certa. ⚠️ Mas o §5.4 (a galeria conhece a prop) **não
basta sozinho**: enquanto a FITA desenhar plano no card vazio (D10/D11), o artista não
distingue *"esta receita é multiplicativa"* de *"a fita não funciona"*.

**Sobre `free-fall`/`throw`:** o racional só fala de fundir. A medição acrescenta que os
**defaults** são sadios e a **FAIXA** não é: o topo do knob `gravity` põe o objeto a **9,9
canvases** (396,7 m contra 40). São dez combinações receita·knob acima de 1 canvas — ver doc 13
§4 D-E.

### 3.3 — O que fazer com o que sai

⚠️ **AUDITORIA — o *"já feito para as 5 cortadas"* é 3 de 5.** Medido pela porta do produto
(`search`): `"ramp"` → **0 hits** · `"ramp loop"` → **0** · `"sway cosine"` → **0**. Os
SINÔNIMOS foram herdados (`"sawtooth"` → Pulse ✓, `"cosine"` → Sway ✓); os **rótulos** não.
Controles: `"mirror"` → 4 · `"midpoint"` → 1 · `"negate"` → 3 · `"time remap"` → 1 (a busca
aceita multi-palavra — `norm()` concatena, então o vão não é do buscador). **É exatamente o que
esta §3.3 proíbe** (*"cortar sem herdar é esconder capacidade"*), e o gate que falta é *"o
rótulo de toda receita aposentada ainda acha o sobrevivente"*.

* **Aliases herdados**: o sobrevivente responde às palavras do aposentado. Já feito para as
  5 cortadas; repetir para todas. Com a busca sendo a interface, cortar sem herdar é
  esconder capacidade.
* **`REFUSALS`**: a crate já tem um mecanismo de *recusa com roteamento* (digitar `loop`
  acha onde o loop mora). Toda receita cortada cuja palavra o artista pode digitar entra
  ali **apontando para o caminho certo** (ex.: `if` → *"condições viram keyframes; veja a
  timeline"*).
* **Nada é deletado do git**: o `git log` guarda; o doc registra o porquê com o número.

---

## §4 — FASE B: o motor, antes da UI

Não faz sentido embelezar um card que autora um documento com dois escritores. **Esta fase
vem primeiro.**

### B1 — UMA porta de escrita de expressão *(fecha D4)*
* Enumerar leitores/escritores (Bloco 2.4 da auditoria).
* **Decisão de projeto a tomar com o Enio:** a expressão é per-clip (ADR-0145) OU global
  (ADR-0144)? Hoje as duas existem, o snapshot lê `per-clip ?? global` e o Apply escreve só
  per-clip. Recomendação: **per-clip é a semântica certa** (fade com o strip, ADR-0146), e o
  global vira **legado com migração** — ou seja o load converte `binding.expr` para o clip 0
  e o campo morre. Isso mexe em `DOC_VERSION` ⇒ ADR.
* Gate: uma tabela `(escritor, leitor)` gerada por grep no fonte, com controle positivo.

### B2 — Apagar uma expressão DEVOLVE a propriedade *(fecha D4)*
* Uma prop SEM keys, ao perder a fórmula, hoje fica congelada no último valor. O
  `expr_pass::take_restore` já faz exactamente o hand-back certo para o fim do PREVIEW —
  **generalizar para o clear**, com o mesmo guard (`!composed.contains_key`, *"nada mais
  respondeu por este canal"*).
* Gate red-first: prop sem keys, fórmula `time*2`, apply-vazio ⇒ a prop volta ao `rest`.
  Medir o valor, não o estado.

### B3 — `ClockUse` visível *(fecha D2)*
* Medir as 7 de Time. As inertes-sozinhas **não podem ser oferecidas como card solto**.
* ~~Recomendação: uma linha Time deixa de ser uma LINHA e passa a ser um **atributo do bloco**
  — ou seja o card ganha um cabeçalho *"Clock: normal / stepped / delayed / …"* que vale
  para a planilha inteira.~~ ⚠️ **RECUSADO pelo ESTADO DA ARTE (§0, D-0.3):** o cabeçalho
  colapsa em UM relógio por pilha o que o Apple Motion ship como behaviours **por-parâmetro,
  empilháveis** (`Rate`/`Reverse`/`Stop`) — com ele, acelerar uma linha e reverter outra na mesma
  pilha fica **inexprimível**.
* **Recomendação nova:** a linha continua uma LINHA; o que muda é que **o escopo dela aparece na
  tela** (*"afeta as linhas abaixo"*) e que uma linha de Time **não é oferecida sozinha** (não há o
  que retimar) — o critério por-KIND da §2.2, sem modelo novo. ⚠️ Motion ship `Reverse`/`Stop`
  sabendo que só fazem sentido sobre um parâmetro **já animado**: o defeito nunca foi elas
  existirem, foi **a UI não dizer isso**.
* ⚠️ `wiggle` continua com relógio PRÓPRIO (fato do parser). Com o clock no cabeçalho, a UI
  pode dizer isso UMA vez (*"Shake tem relógio próprio"*) em vez de por-linha.

### B4 — O clamp é do WIDGET, não da emissão *(fecha D7)*
* `EmitCtx::lit` clampa em silêncio para produzir texto que parseia. O widget tem de recusar
  o valor inválido, e então a emissão não precisa mentir.
* Gate: para todo knob `Literal`, o valor que a caixa mostra e o que a fórmula usa são o
  MESMO — varrendo a faixa inteira mais os extremos e o zero digitado.

### B5 — Determinismo do `__seed`
* `target.get() * SEED_SPACING`. Se `target` é alocado por ordem de criação, **adicionar
  uma track re-rola o Jitter de todos**. Medir. Se for o caso: o seed passa a vir de
  `stable_name_id(Name)` — o mesmo id durável que a timeline e os joints já usam.

---

## §5 — FASE C: o layout, refeito *(fecha D5, D6, D8)*

### 5.1 — O diagnóstico do layout atual
Card de **536 × ~530 px**: galeria 190 px (uma coluna de texto), planilha 320 px, dentro da
qual cabem um olhinho (22), um chip de combine (22), o nome da receita, um readout (52) e um
X (22) — sobra ~180 px para o nome. Os knobs têm rótulo de 84 px e caixa de ~100. Nada rola.
Não há espaçamento de agrupamento: uma linha de receita e um knob dela têm o **mesmo peso
visual** (`ROW_H_PX`, mesma cor de fundo), então a planilha lê como uma lista plana de 12
itens em vez de 3 blocos.

### 5.2 — Os quatro problemas de desenho, nomeados
1. **Sem hierarquia**: receita e knob no mesmo peso.
2. **Sem respiro**: `Spacing::Xs` entre bandas de conteúdo diferente.
3. **Sem rolagem**: conteúdo de tamanho variável em container fixo.
4. **Sem identidade do alvo**: `#7294` em vez de `Ball · Position X`.

⚠️ **AUDITORIA — o diagnóstico da §5.1 está parcialmente errado, e falta o problema nº 1.**
Números medidos (doc 13 §4-bis): card **532 × 532**; header da row = olhinho 22 · chip 22 ·
**NOME 198** · readout 52 · X 22; row de knob = indent 8 · label 84 · caixa 96 · **MORTO 128**.

| a §5.1 diz | a medição diz |
|---|---|
| *"sobra ~180 px para o nome"* | **198 px** (~16 chars a 12 px). ⚠️ **A hipótese está REFUTADA**: o aperto do header é **gutter ZERO** entre nome │ readout │ X, não a largura |
| *"sem respiro"* (2) | ✅ certo, e pior: **128 px MORTOS (40% do sheet) em TODA row de knob** — o `ctrl_w` é computado como 168 e **descartado** no braço `Number\|Literal` (`expr_modal_columns.rs:440` vs `:444`) |
| *"sem rolagem"* (3) | ✅ certo — e a capacidade vertical é onde de fato aperta: `BODY_SLOTS = 12`, uma row gasta `1 + knobs`, então **`Fade by Distance` (8 knobs) come 9 dos 12 slots** e Turbulence deixa caber **2 rows**. Histograma: `{1:2, 2:11, 3:13, 4:10, 5:9, 6:1, 7:3, 9:1}` |
| *"sem identidade"* (4) | ✅ certo, e o custo é maior: **nada publica `Name` para o painel timeline** (grep) — o dope-sheet INTEIRO rotula por propriedade. Fechar isso é trabalho na shell |

**5. ⚠️ O PROBLEMA Nº 1, AUSENTE DESTA LISTA: o card não é MODAL.** O fundo dele não registra
hit rect ⇒ **18 widgets do transporte vivos sob a pegada**; clicar o centro da barra de fórmula
dá `hit_at = TIMELINE_LENGTH_NUM` e **digitar edita o `Dur(s)`**. A roda zooma a timeline atrás.
**Um card de 820 × 620 sobrepõe MAIS widgets, não menos** — a §5.3 abaixo, sem isto, piora o
defeito que ela tenta consertar.

**6. ⚠️ AUSENTE: a fita plana coincide com a linha de base.** `extent()` devolve
`(base−1, base+1)` numa curva plana ⇒ curva e baseline caem no **MESMO y** (0,5000 as duas),
**inclusive no card recém-aberto sem rows**. A cura da §5.3 (*"linha de base e escala em
unidades"*) é a certa; o que ela não sabe é que hoje a referência é apagada exactamente quando
é mais necessária.

### 5.3 — O desenho proposto

```
┌───────────────────────────────────────────────────────────────────────────┐
│  fx  Expression        Ball  ·  Position X                        [ X ]   │ ← título: NOME
├───────────────────────────────────────────────────────────────────────────┤
│  Clock   ( Normal ▾ )   Speed 1.00        ⓘ Shake has its own clock       │ ← B3: o relógio é do BLOCO
├──────────────────────────┬────────────────────────────────────────────────┤
│  ⌕ search…                │  ┌──────────────────────────────────────────┐ │
│                           │  │ 👁  +   SHAKE                    −0.30  ✕ │ │ ← cabeçalho da linha: peso ALTO
│  ▸ Life          4        │  │      Speed    [   2.00  ▴▾ ]             │ │ ← knobs: peso BAIXO, indentados
│  ▸ Wave          5        │  │      Amount   [   0.30  ▴▾ ]             │ │
│  ▸ Shape         4        │  └──────────────────────────────────────────┘ │
│  ▸ Time          2        │  ┌──────────────────────────────────────────┐ │
│  ▸ Link          3        │  │ 👁  ×   FLICKER                   1.00  ✕ │ │
│                           │  │      Speed    [   8.00  ▴▾ ]             │ │
│  ── recentes ──           │  │      Min      [   0.30  ▴▾ ]             │ │
│  Shake · Sway · Limit     │  └──────────────────────────────────────────┘ │
│                           │  [ + adicionar linha ]                    ▓  │ ← barra de scroll REAL
├──────────────────────────┴────────────────────────────────────────────────┤
│    ╭─╮        ╭─╮                                                         │
│  ──╯ ╰────────╯ ╰──   ← a fita, com a linha de base e a ESCALA em unidades │
├───────────────────────────────────────────────────────────────────────────┤
│  fx  value + wiggle(2, 0.3)*mix(0.3, 1, …)                                │
├───────────────────────────────────────────────────────────────────────────┤
│                                          [ Cancel ]        [ Apply ]      │
└───────────────────────────────────────────────────────────────────────────┘
```

**As decisões, e o motivo de cada:**

⚠️ **AUDITORIA — a ORDEM desta lista está errada.** Medido, o card maior é **o único item que
não conserta um defeito medido**, e sem o item novo (0) ele **piora** o pior deles. A ordem que
a medição sugere para a FASE C:

> **(0) o card ENGOLE o ponteiro na própria pegada + a roda** (D10) — sem isto todo o resto é
> cosmético, e um card maior sobrepõe mais transporte · **(1) gutters + knobs em 2 COLUNAS**
> (os 128 px mortos por row; isto mata o overflow **sem** introduzir um 2º eixo de scroll dentro
> de um painel que já rola) · **(2) a fita distinguível da baseline** (D11 + §5.2.6) · **(3) o
> nome do objeto**, que arrasta a shell · **(4)** o card maior/redimensionável, por último.

* **CARD MAIOR e redimensionável.** Alvo ~820 × 620. A galeria pode ser 220 e a planilha
  520 — o dobro de hoje. ⚠️ Medir contra o viewport mínimo suportado antes de fixar.
  ⚠️ **AUDITORIA:** o card é centrado no **viewport EXTERNO** (a janela), não no slot do painel
  — então crescer o card **aumenta** a sobreposição com o transporte. Faça o (0) primeiro.
* **A LINHA É UM CARTÃO**, não uma fileira: fundo próprio (`ColorToken` de superfície
  elevada), raio, e os knobs DENTRO dele, indentados. Isso resolve (1) e (2) de uma vez, e é
  o padrão que o `ph2d-panel-wet-tuning` e o `motion-params` já usam neste app — **copiar,
  não inventar**.
* **ROLAGEM REAL** na planilha. O app tem o primitivo (scrollbar id 837, gate
  `scrollable_panels_intercept_the_wheel` — **é obrigatório interceptar a roda**). O
  `+N more rows` morre.
* **O TÍTULO MOSTRA O NOME.** Exige `TrackView.name` (ou um mapa no snapshot) preenchido
  pela shell, que é a dona do `Name`. Isso conserta D8 **e** melhora o dope-sheet inteiro,
  que hoje rotula tracks com `#bits % 10000`.
* **A FITA GANHA ESCALA E LINHA DE BASE.** Hoje ela normaliza por min/max, então uma
  constante desenha uma reta no meio — **indistinguível de "não funciona"** (D1). Com a
  linha de base e a escala em unidades, uma constante lê como *uma constante*, e a fita
  passa a ser um instrumento em vez de uma decoração.
* **Vazio ≠ apertado.** Com poucas linhas, o card mostra um estado vazio com uma frase e o
  botão de adicionar — não 12 slots vazios.
* **RECENTES** na galeria: com ~21 receitas e busca, a lista cabe; os recentes cortam o
  caminho do uso repetido.

### 5.4 — Receita oferecida é receita que FAZ algo aqui
A galeria passa a saber a **propriedade**: uma receita cuja faixa não faz sentido na prop
(um `Flicker` multiplicativo numa translação de base 0; um `Blink` 0/1 numa posição em
metros) **não é oferecida**, ou é oferecida com o default já convertido para a prop. Isso
fecha D1 na raiz e responde ao report da rodada 1 (*"valores tão altos que o objeto some"*).
⚠️ Isso muda o contrato do catálogo, que hoje é deliberadamente **property-agnostic** — é
uma decisão de arquitetura com custo: ou o catálogo passa a conhecer `PropKind` (acoplamento
que o `10_plano` recusou de propósito), ou o **card** aplica uma tabela de conversão na
inserção da linha. **Recomendação: no CARD** — o catálogo continua leaf, e quem sabe a prop
é quem já sabe (o painel).

---

## §6 — FASE D: o pick-whip *(fecha D9 — sem ele a família Link não volta)*

A W4 do plano 10, nunca construída. Sem ela o artista **digita** `Ball.x` num campo de
texto, sem lista, sem autocomplete, e um nome errado é **silenciosamente 0**.

* **O gesto**: o knob de link é um **botão de pick** (o precedente exato existe: o
  eyedropper do `Body A/B` dos joints, `W-JointAuthoring`, e o `Pick Path` do Vector).
  Arma → o próximo clique no canvas (ou na Hierarquia) resolve o objeto → um segundo
  controle escolhe a propriedade, de uma lista.
* **Nunca digitar um nome**: mata o problema de nome com espaço e de caixa alta.
* **Um link que não resolve DIZ isso**: o `Row::waiting_for` hoje só cobre knob VAZIO. Ele
  passa a cobrir *nome que não resolve* — o que exige o card conhecer os nomes da cena
  (o mesmo insumo do título, §5.3).

---

## §7 — FASE E: a reescrita algoritmo por algoritmo, em grupos de TRÊS

O coração do pedido do Enio. **Cada grupo é uma wave completa**: reescrever, medir, gate,
smoke com o Enio, e só então o próximo. **Nenhum grupo começa antes do anterior ser
aprovado.**

### 7.1 — O protocolo de cada grupo (idêntico para todos)

1. **Escrever a frase do animador** para as 3 (§2.1). Se não sair, a receita não entra.
2. **Escrever o algoritmo** e medir, ANTES da UI:
   * ⚠️ **AUDITORIA — a medida é por KIND** (§2.2 corrigido): SOURCE ⇒ *anima* (variação no
     tempo, base 0 **e** base 1); MODIFICADOR ⇒ *muda o valor que entra* (delta contra a
     IDENTIDADE sobre uma grade `(time, value)`) — **nunca** amplitude do stack; `Time` ⇒ só
     mensurável com uma linha embaixo;
   * a fita: plana? qual a forma? ⚠️ **AUDITORIA — a fita NÃO SERVE como oráculo até D11 ser
     consertado**: ela alimenta `__seed = 0` enquanto a cena usa `target * 100`, então para toda
     receita que lê o seed (`shake`, `turbulence`, `jitter`) **ela desenha outra tinta**. Medir
     a fita antes disso é medir o instrumento;
   * sensibilidade de CADA knob: varrer 5 valores e reportar como a saída muda. ⚠️ Um knob
     cuja variação não muda a saída é um knob morto — foi assim que *"Shake: mudar os
     parâmetros não mudava a animação"* nasceu. ⚠️ **AUDITORIA — a fixture tem de conter o
     fenômeno**: um knob que só age através de um LINK parece morto se o link for constante no
     tempo (o outro objeto está **animado**, é o caso de uso inteiro), e dois links têm de ler
     números **diferentes** — indexe por hash do nome, nunca por comprimento (`"Ball.x"` e
     `"Cube.y"` têm 6 caracteres, e isso colapsa todo `mix(a, b, t)`);
   * o default tira o objeto do quadro? (≤ 1/50 de 40,96 m) ⚠️ **AUDITORIA — julgue a FAIXA
     também, não só o default**: dez combinações receita·knob de hoje passam de 1 canvas no topo
     do slider (`gravity` chega a **9,9 canvases**), e o gate existente só olha o default — numa
     janela de 2 s que faz o `free-fall` passar por **0,563 m**.
3. **Gates**: neutro (se houver) byte-idêntico · o knob acorda · composição com uma linha
   acima e uma abaixo · o default está na faixa.
4. **Mutação** por receita: quebrar o termo principal e provar que sangra.
5. **Smoke dedicado**: `PH2D_EXPR_SMOKE=<n>`, **3 objetos, um por receita**, lado a lado, e
   a cena **IMPRIME o que montou** (`[expr-smoke] grupo N: <3 nomes>`). ⚠️ A cena **abre o
   card** no primeiro objeto — a cena atual autora por código e por isso pula a costura que
   deveria provar.
6. **O Enio roda e aprova.** Reprovado ⇒ conserta no MESMO grupo; não avança.

### 7.2 — A ordem dos grupos, e por que ela é esta

A ordem é por **risco decrescente**: o que o Enio já usou primeiro, o que depende de infra
por último.

| grupo | receitas | por que juntas | risco |
|---|---|---|---|
| **G1** | Shake · Sway · Limit | o gerador orgânico, o gerador rítmico e o modificador. **Cobrem os três KINDS do modelo** (source-add, source-add, modifier) ⇒ o G1 valida a arquitetura inteira com 3 receitas | ALTO — se o G1 não convence, o modelo está errado |
| **G2** | Drift · Bounce · Multiply/Add | os irmãos: Drift é o Shake que obedece o relógio (valida B3), Bounce é a Sway retificada, Multiply/Add é o modificador de escala | MÉDIO |
| **G3** | Breathe · Cycle · Quantize | Breathe = a diferença de FORMA que o Enio questionou (o blurb tem de vender a distinção) · Cycle = a fusão candidata de Ping-Pong/Blink/Pulse/Ramp (§3.2) · Quantize = o stepped no VALOR | ALTO — a fusão do Cycle é uma decisão de produto |
| **G4** | Jitter · Flicker · Remap | as duas que o Enio disse não funcionar (B5 e D1) + o modificador de faixa. **Só depois de §5.4**, porque as duas dependem da prop | ALTO |
| **G5** | Clock: Stepped · Speed · Freeze | as 3 de Time que sobrevivem, **depois de B3** (o relógio no cabeçalho) | MÉDIO |
| **G6** | Pendulum · Throw · Orbit | physics-lite. Orbit é um PAR (X e Y) e é o único que insere 2 linhas | BAIXO |
| **G7** | Follow · Offset Copy · Distance | a família Link, **só depois da FASE D** (pick-whip). Sem ela, não é oferecida | BLOQUEADO por §6 |
| **G8** | Custom Formula + o que a auditoria salvar de Field | a saída de emergência e os atalhos | BAIXO |

**≈ 21 receitas em 8 grupos.** Um grupo por sessão de smoke.

### 7.3 — O que "reescrever o algoritmo" significa, concretamente

Não é re-digitar o `format!`. Para cada receita:

* **O modelo**: que função de `(time, value, knobs)` ela É, escrita em prosa e em uma linha
  de matemática, no doc-comment.
* **A unidade de cada knob**: `Speed` em Hz? em rad/s? em ciclos por segundo? ⚠️ O report
  *"a velocidade em shake nunca foi velocidade, parece mais com um seed"* é exatamente um
  knob cuja unidade nunca foi declarada. **Todo knob declara a unidade no rótulo ou no
  tooltip.**
* **A faixa**: derivada da unidade e do canvas, com a medição ao lado (§0 do CLAUDE.md —
  *meça antes de limitar*).
* **O neutro**: existe um ajuste que a torna identidade? Se sim, byte-idêntico, gateado.
* **O comportamento no default**: um número.

---

## §8 — Riscos, e o que este plano NÃO faz

* ⚠️ **A fusão do `Cycle`** (§3.2, G3) pode ser pior que 4 cards: um seletor de forma é
  menos descobrível que quatro nomes na galeria. **Decisão do Enio, com o desenho na mão.**
* ⚠️ **`Flicker`**: se ela virar aditiva, ela deixa de ser o que "flicker" significa
  (modular brilho). Se continuar multiplicativa, ela é inútil em translação. **A saída
  provável é §5.4** (oferecida só onde faz sentido) — e isso é uma mudança de contrato.
* ⚠️ **B1 (uma porta de escrita)** provavelmente mexe em `DOC_VERSION` ⇒ **ADR + ordem do
  Enio**, e um bump **recusa todo projeto já salvo**.
* ⚠️ **O nome do objeto no snapshot** melhora o dope-sheet inteiro e por isso toca um
  arquivo de outro dono. Escopo a combinar.
* **Não está aqui**: curvas de easing dentro de expressões · expressão em prop de VETOR
  (hoje é escalar por canal) · a expressão PURA extrapolar a strip (aberto por ordem do
  Enio desde 2026-07-27, exige vínculo autorado + provavelmente `DOC_VERSION`).

---

## §9 — Sequência de execução e critério de fim

⚠️ **AUDITORIA — a sequência ganhou uma FASE ZERO, e ela é o pré-requisito de TODAS.** Três
instrumentos que este plano usa como critério de aceitação estão **quebrados**, e medir com eles
é medir o instrumento: a **FITA** (mente sobre o seed, D11) · o **CARD** (não engole o ponteiro,
então "clicar no card" pode acertar o transporte, D10) · a **CENA de smoke** (exercita **zero**
receitas e o roteiro dela manda usar um widget deletado, doc 13 §4 D-F/D-L).

```
AUDITORIA (11 + 13)  →  ⚠️ FASE 0: consertar os INSTRUMENTOS
                            (D10 o card engole o ponteiro+roda ·
                             D11 a fita usa o seed da cena ·
                             D12/D13 a pose volta quando a formula sai ·
                             a cena de smoke abre o CARD e usa RECEITAS)
                             ↓
                        FASE A (corte)  →  FASE B (motor: B1..B5)
                                                    ↓
                        FASE C (layout)  ←──────────┘
                             ↓
                        G1 → smoke → G2 → smoke → G3 → … → G6
                             ↓
                        FASE D (pick-whip) → G7 → smoke → G8
```

* ⚠️ **NENHUMA fase começa antes da FASE 0.** A FASE A decide cortes lendo a fita e a excursão;
  a FASE C julga layout clicando no card; cada grupo G fecha com um smoke. Os três instrumentos
  têm de estar honestos primeiro — é a CR-3 deste plano aplicada aos próprios instrumentos.
* **FASE A e FASE B podem correr em paralelo** com a FASE C (crates diferentes), mas
  **nenhum grupo G começa antes de C**, porque o Enio julga na tela.
* **Critério de fim da feature**: as ~21 receitas aprovadas em smoke, em grupos de 3, com o
  card rolável e nomeando o objeto, e **os ~~9~~ 14 defeitos D1..D14 com um gate cada**.
* **Critério de fim de CADA wave**: o Enio disse "aprovado". Não "os gates estão verdes".
* ⚠️ **AUDITORIA — e um gate a mais no fim de tudo:** os 8 gates que a auditoria provou não
  provarem o que alegam (doc 13 §5) têm de ser **consertados ou mortos**, incluindo o
  `no_letter_is_used_as_an_icon`, que **fica verde com o `"O"` do report de volta**.

---

## §10 — Registro honesto

Este plano existe porque eu entreguei uma feature que o Enio chamou de *"grande trabalho de
bosta"*, depois de eu reportar três vezes que estava medido e funcionando. As três razões
estão na §1.1 e a lista das minhas falhas de método está na §9 do handoff irmão. O que
mudou de método, e vale para quem executar este plano:

1. **O oráculo é a tela.** Um número só conta se descreve o que se vê.
2. **Três por vez.** Uma feature de 55 peças validada em bloco não é validada.
3. **Nenhuma peça inerte é oferecida.** Se ela precisa de contexto, ou o contexto existe, ou
   ela não está na galeria.
4. **Toda nota é uma afirmação que expira.** O `"Nothing here scrolls, and the geometry says
   why"` era verdade quando foi escrito e virou a causa de D6.
