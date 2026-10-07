# 48 — A aguada é um FILTRO: o estado da arte do vidrado sobre papel de cor (2026-10-07)

> **Pedido do dono** (2026-10-07, com foto, depois do #45): *«Descubra porque no modo watercolor o
> vermelho sobre fundo castanho fica parecendo fluorescente»* → (medido, BUGS #46) → *«eu quero o
> estado da arte. descubra qual é»*.

## 1. O defeito, medido (BUGS #46)

Na foto: papel `153,121,91`, miolo da aguada `230,95,85` — o vermelho `+80` ACIMA do papel. No código
(`papel_nasce_tests::diag_a_aguada_vermelha_no_castanho`, vermelho `255,0,0`): a óptica da aguada
calcula uma transparência por CANAL (o vermelho passa, o verde e o azul são absorvidos; no branco
`255,142,142`), mas a camada guarda UM alfa; o `un-premultiply` guarda `255,84,84` a alfa `0,66`, e
sobre o castanho «cobrir com transparência» soma luz vermelha: `220,97,86` (`+83` em `2 841` de
`2 881` texels). Uma aguada real só tira luz — nenhum canal fica acima do papel.

## 2. O estado da arte

| quem | lei do vidrado sobre o papel | na cena do dono (`255,142,142` no branco, papel `153,121,91`) |
|---|---|---|
| **Curtis, Anderson, Seims, Fleischer, Salesin — *Computer-Generated Watercolor*, SIGGRAPH 1997** (a referência que toda a literatura de aquarela digital cita) | **Kubelka–Munk**: cada vidrado tem, por canal, reflectância `R` e transmitância `T` (de `K`, `S` e da espessura `x`); o empilhamento é `R = R₁ + T₁²·R₂ / (1 − R₁R₂)`, `T = T₁T₂ / (1 − R₁R₂)`, com o papel como o `R₂` de baixo | vidrado transparente (`S ≈ 0`): `T₁² = o que se mostra no branco` ⇒ **`153,64,47`** — nunca acima do papel |
| Baxter et al. 2004 (IMPaSTo), e a linha GPU que se seguiu | o mesmo K–M, em tempo real no shader | — |
| Corel Painter (Digital/Real Watercolor) | composite **Gel** (tinge o que está por baixo) ou **Multiply** — a aproximação comercial do filtro | `153,67,51` (multiplicar em sRGB) |
| Rebelle 8 (medido, doc 47) | «cobrir com transparência», um alfa | soma luz — no azul do doc 47, `+63` acima do papel |
| PH2D hoje (#45) | o mesmo do Rebelle | `220,97,86` |

⇒ **O estado da arte é o vidrado de Kubelka–Munk**: a aguada é um filtro (o papel vê-se através dela
multiplicado pela transmitância, ida e volta) mais a luz que o próprio pigmento espalha (`R`). O
Rebelle não o faz; o Painter faz a aproximação. A ordem continua a não importar SE a camada guardar o
filtro em vez de uma cor já composta: mudar o papel recompõe `R + T²·P/(1 − R·P)`.

Fontes: [Curtis 1997](https://grail.cs.washington.edu/projects/watercolor/) (§5.2, as equações acima,
lidas do PDF) · [Baxter 2004](https://gamma.cs.unc.edu/IMPASTO/publications/Baxter-IMPaSTo_Print-NPAR04.pdf) ·
[Painter, Gel/Multiply](https://product.corel.com/help/Painter/540111155/Corel-Painter-en/Corel-Painter-Blend-layer-composite-method.html).
Algoritmo PUBLICADO: porta-se (DIRETIVA §1), não se lê fonte de app nenhum.

## 3. O que o PH2D já tem

A óptica da aguada já é a forma de Beer–Lambert por canal (`watercolor_render.rs`:
`optical = chão·T + pigmento·(1 − T)` em luz linear) — um K–M de um só passo. O que falha não é a
óptica, é o ARMAZENAMENTO: a camada tem um alfa só, e o `un-premultiply` contra o branco de
referência (#45) converte o filtro numa cor saturada.

## 4. O desenho (para construir — decisão técnica, padrão-ouro)

Cada texel de uma camada raster ganha um **filtro** `T` por canal (3 bytes; `255` = sem filtro, o
default ao byte de hoje). A lei de composição de UM texel sobre o que está por baixo `B`:

`mostrado = C·a + (B·T)·(1 − a)` (em tons de ecrã, como o compositor, ADR-0177)

— o par `(C, a)` é a tinta que COBRE (Digital, Impasto, Wet Paint: `T = 1`, a lei de hoje ao byte) e
`T` o filtro (a aguada: `a = R`, o que o pigmento espalha; `T` = a transmitância ida e volta). A forma
é **fechada sob empilhamento** no mesmo texel: filtro sobre tinta dá `(T·C, a, T)`; tinta sobre
filtro dá `(C₂a₂ + C₁a₁(1 − a₂), a₂ + a₁(1 − a₂), T₁)` — então pintar Digital por cima da aguada, ou
aguada por cima do Digital, cabe na mesma camada sem plano extra por traço.

A medir ANTES de construir (uma rodada): o par `(R, T)` que a óptica de hoje já produz por texel
(sem `un-premultiply`), e a cena do dono — o filtro tem de dar `≤` papel em todo canal (`153,64,47`
previsto), o branco tem de ficar igual ao de hoje (±1 ou o piso), e a ordem tem de continuar a não
importar (pior `0`, os 72 casos do #45).

Portas que mudam (todas no mesmo work item — DIRETIVA §1, o consumidor é parte dele): o plano da
camada (`LayerStack`/imagens, o canvas activo), o compositor CPU (`composite`, a região suja, o
«Use as», o Apply), o compositor GPU (WGSL, `GpuGlobals`, a paridade bit-a-bit com a CPU), o
desfazer (`ModelSnapshot`), o ficheiro (`PaintedDocument` — degrau do `PROJECT_SCHEMA`, contado por
`python3 scripts/schema-recount.py`), a escrita da aguada (`watercolor_render.rs` deixa de fazer o
`un-premultiply`) e a borracha (apaga o filtro também). Custo a medir: o quadro de um desenho novo a
2048² e 4096² (o #45 está em `0,25` ms Digital · `1,34` Aquarela por movimento).

## 5. Feito (2026-10-07, ordem do dono: *«sim. faça»*) — BUGS #46

- **O encaixe, medido antes** (`vidro_tests::diag_a_aguada_como_vidrado`, 12 casos): «cor + alfa + filtro
  em [0, 1]» não cabe em `~40 %` dos texels com corpo (erro no branco até `52` níveis, cortando a cor ou
  o filtro); o alfa por canal em `u8` com o píxel de sempre ao lado erra `≤ 1`. Foi esse.
- **A lei que o código executa** (`compositor::vidro`): `mostrado = W − t·(1 − papel)`, `W` a pilha sobre
  o branco, `t` o produto de `1 − alfa_c` camada a camada — em tons de ecrã (ADR-0177), na passada do
  compositor. Sem vidro, ou com o papel branco, é «cobrir com transparência», ao byte.
- **O SELO**: o vidro guarda o píxel que a aguada escreveu; qualquer outro pincel que o reescreva
  devolve o texel à lei de um alfa. Nenhum outro pincel precisou de conhecer o plano.
- **A óptica sobre dois chãos** é a forma de Curtis especificar um pigmento (sobre o branco e sobre o
  preto). Medido contra esse oráculo: pior `2` níveis (o piso), nos 24 casos do gate.
- ⚠️ **Não é linear no chão, e fica decidido sobre o branco de referência:** a presença de tinta que pesa
  o `Pigment` e a re-molhagem molhado sobre molhado (o oráculo do preto mede-a contra o preto: `8`
  níveis). E com corpo 0 só um pigmento de canais 0/255 é filtro puro: a óptica do PH2D devolve
  `pigmento·(1 − T)`, um azul `30,60,220` sobe o azul do papel castanho pelo que o próprio pigmento
  devolve (o oráculo concorda).
- Custo e gates: BUGS #46.
