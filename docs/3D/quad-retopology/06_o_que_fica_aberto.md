# 06 — O que fica ABERTO, com endereço

> ⚠️ **Uma lista de aberto sem endereço é uma lista de desejos.** Cada item aqui diz *onde* o
> defeito vive, *o que já foi medido* e *qual é a próxima medição* — e nenhum deles bloqueia o
> uso do botão hoje.

## §1 — A agulha que ainda é cortada

Na `sculpt_antes.obj` a ponta `4849` sai com `gap 2,57` (barra `0,5`), e o remate **recusa-se
por desenho**: ali o que falta é **célula**, não fase.

- **O número:** a agulha tem raio local `0,037` e o quad pedido mede `0,0399` — *a grade não
  tem resolução para representar o último centímetro*.
- **A cura publicada:** o **factor de escala conforme por construção** (`Δ log h` contra a
  curvatura de Gauss, `h = h₀ · e^{−s}`), que é integrável por definição — ao contrário do campo
  de tamanho de hoje, cuja parte não-integrável o G3 **projecta fora**.
- **A próxima medição:** com o campo conforme, a razão `ALVO/F1` no bico daquela agulha e o
  `gap` dela, contra as `4` pontas da mesma peça.

## §2 — O mapa ainda DOBRA

O `ph2d-gridmap` entrega `134` triângulos dobrados (`4,5 %`) na peça de referência. A extracção
descarta as **almofadas** que isso produz, o que é uma rede, não uma cura.

- **A cura de fundo** é um solver **injectivo** (a família *«locally injective integer-grid
  maps»*), e é **recusa medida por âmbito**: ela substitui o G3 inteiro.
- **O que existe hoje** contra isso: o descarte de almofadas, o `dissolve_doublets`, o
  `catch_unwind` por tentativa, e o selector a preferir a candidata sem furos.

## §3 — O SORTEIO dos últimos bits

A mesma escultura, nos mesmos knobs, noutra **escala** dá outra realização — e o destino da
ponta mais longa muda com ela (medido: `0,18` · `0,47` · comida, em três realizações).

- **O que NÃO é:** aleatoriedade. A cadeia é determinista para a mesma entrada; o que muda são
  os últimos bits da entrada.
- **O que o remate fez:** fechou a **fase** (a parte do sorteio que era sub-célula). O que
  sobra é a candidata que vence, que ainda pode mudar.
- **A próxima medição:** o critério do plano §104 — o mesmo veredito nas **cinco** realizações
  — tem hoje **três** medidas.

## §4 — O PÓLO: a grade atravessa o bico em vez de fechar nele

Na malha que o dono aprovou **todo** espinho fecha com um pólo `+1` (quatro valência-`3` a
`≤ 2 h`). Nas nossas, a coluna `pole` da tabela lê `(0, 4)` na ponta mais longa — *a grade passa
por cima do bico*. Isso já **não** produz ponta amputada (a calota e o remate resolvem o que se
vê), mas é a diferença estrutural que resta contra a referência.

- **Já medido e recusado:** reforçar o alinhamento na calota (`PH2D_TIP_ALIGN`), sozinho **e**
  com a calota — ver [`04_recusas_medidas.md`](04_recusas_medidas.md) §2.
- **O endereço:** as separatrizes do F3 **não são** linhas de grade do mapa (`0`–`5 %` dos arcos
  concordam), e o desvio já está todo no G3 contínuo. Plano em
  [`PLANO_arcos_no_sistema_dos_fechos.md`](../quad-remesh/PLANO_arcos_no_sistema_dos_fechos.md).

## §5 — O motor `Fast` do dropdown

Ele é o motor **local**, e devolve na peça do dono `437` quads com **`150` não-quads** contra
`1 494` e `100 %` do de omissão — *a um clique, com o nome que um artista alcança depois de
ouvir que o bom é lento*. Decisão de produto: renomear, esconder, ou curar.

## §6 — O que o `Detail` alto custa

`123 s` na escultura do dono (era `337 s` antes do remate). O tecto do `MAX_QUADS` está em
`25 000` por **dois** recursos medidos — o relógio (`35 s` a `24 190` na cadeia de então) e a
**topologia**, que rebenta acima disso. ⇒ subir o tecto exige medir os dois de novo, não um.

## §7 — As feature lines autoradas

O traçado não recebe arestas marcadas pelo artista. A terceira tentativa do botão usa linhas de
feição **derivadas**, e em algumas peças ela é a melhor candidata — mas o artista não tem como
dizer *«esta aresta é uma quina»*. É feature de produto, e nunca foi pedida.
