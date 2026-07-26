# Doc 76 — `value.reduce`: o reducer GERAL (reduce → broadcast) do domínio de valor (nota-ADR)

> Motion Nodes M2, domínio de VALOR (doc 12). Segue 68..75, e é irmão do
> 74 (`value.normalize`), que FUNDE uma redução min/max com um fit.

## O que é

Dobra o campo inteiro num número — **Sum · Mean · Min · Max** — e o **transmite de
volta como campo constante**. É o composável ao lado do `value.normalize`: onde o
normalize é o fit fundido (min/max → `[0,1]`), este **expõe o agregado em si** como
um campo que qualquer nó a jusante pode dobrar contra.

- **input** `in` : VALUE (`Instances, Scalar, Frame`, coluna `v`)
- **output** `out` : VALUE — **campo CONSTANTE** do mesmo comprimento (o agregado em
  cada elemento), não um stream de comprimento 1
- **params** `mode` (Sum | Mean | Min | Max)
- **Effect** `Pure` (sem clock, sem estado)

## Por que importa

É a única forma de tornar um valor **RELATIVO ao campo inteiro**:
`value.reduce(Mean)` + `value.math(Subtract)` **centra** um campo na própria média;
`value.reduce(Sum)` + `value.math(Divide)` o torna uma **distribuição** (cada
elemento como fração do total). Nada mais no domínio de valor alcança um número que
depende de TODOS os elementos — é a metade `reduce → broadcast` da forma dos
deformers, sobre uma coluna `v` (o `map` é o `value.math` que você compõe adiante).
É o *promote to detail* do Houdini / o Analyze CHOP do TouchDesigner.

## Decisões

1. **Saída = campo constante do MESMO comprimento**, não comprimento 1: assim ela se
   alinha elemento a elemento com a fonte quando um `value.math` dobra os dois (o
   `count_law` é `None`, N entra / N sai). Um stream de comprimento 1 não casaria com
   a geometria a jusante.

2. **O `count` do Mean é `Σ 1.0`, EXATO.** `Mean = Σ vᵢ / N`, e `N` vem de uma 4ª
   redução que dobra a constante `1.0` por elemento — inteiros somam exato em `f32`
   para `N < 2²⁴`, então o denominador da GPU casa com o `len()` da CPU **no byte**;
   só o numerador (`Sum`) carrega o ε. Auto-contido no canal de reduce, sem um
   uniforme de contagem.

3. **Min/Max são bit-exatos; Sum/Mean são ε** (a adição de float não é associativa — a
   árvore da GPU soma noutra ordem; é o mesmo ε dos deformers que usam `Sum`, e o gate
   documenta).

4. ⚠️ **A binding do kernel é `Write`, NÃO `ReadWrite`.** O kernel principal **nunca
   lê** o `v` original (as reduções o leem nos próprios passes); ele só ESCREVE o
   agregado. Um `ReadWrite` declara um `in_v` que o corpo nunca chama, e a **naga o
   remove do layout** enquanto o sequenciador ainda vincula o buffer dele — um
   descompasso 7-vs-6 no bind group. `Write` materializa uma coluna de saída fresca.
   (O `value.normalize` LÊ `v` para mapeá-lo, por isso ele é `ReadWrite` e não tropeça
   nisso.) Lição: **um kernel que só reduz e escreve usa `Write`.**

## Rejeitados

- **Range (max − min) / Product como modos** — Range é `value.reduce(Max)` menos
  `value.reduce(Min)` composto (e o normalize já o usa internamente); Product não é
  uma redução disponível (só Max/Min/Sum). Mantido no conjunto canônico Sum/Mean/Min/Max.
- **Uniforme de contagem para o Mean** — a redução `Σ 1.0` mantém CPU e GPU no MESMO
  mecanismo (ambos obtêm N) e é exata; não depende de um `count` do contexto que pode
  não existir.
- **Fundir com o `value.math`** — um nó que reduz E dobra seria uma 2ª resposta ao que
  o `value.math` já faz; a separação (reduce solta o agregado, math dobra) é o idioma
  composável (o `v` fonte alimenta o reduce E o math, um fork).

## Preço / cobertura

Kernel WGSL = um `switch` sobre `mode` lendo `reduce_sum()`/`reduce_count()`/
`reduce_min()`/`reduce_max()` e escrevendo o agregado a cada elemento (o broadcast),
binding `Write` na coluna `v`, `count_law: None`. As 4 reduções rodam ANTES do kernel
(passes próprios). Sem `applicable` ⇒ **sem fallback de CPU**. Paridade RTX pelo
caminho **Mean** (exercita o MAIS do canal: a `Sum` ε + o `count = Σ 1.0`); naga
valida.

**Gates:** cada modo dobra ao seu agregado (Sum totaliza, Mean promedia, Min/Max os
extremos) · a saída transmite a um campo constante do mesmo comprimento · subtrair o
Mean centra o campo (`Σ (vᵢ − mean) = 0`, a razão do nó) · cook end-to-end (`[2,4,6]`
por Mean = `[4,4,4]`) · registro · **paridade de dispositivo** (`#[ignore]`, RTX,
Mean sobre `[1,5]`).

## Demo — `PH2D_VALUE_REDUCE_SMOKE=1`

Duas fileiras de 24 instâncias: de baixo o driver `[1, 5]` direto em Y — uma **rampa**;
de cima o MESMO driver `→ value.reduce(Mean)` — a média `3` transmitida a todas, então
uma **linha PLANA** (a média da fileira de baixo). O nó marcado `>> EVALUATE <<` é o
reduce — selecione, troque **Mode** para Min (a linha desce a `1`), Max (`5`) ou Sum
(dispara para o total).
