# Doc 77 — `value.smooth`: o FILTRO (box blur sobre o índice) do domínio de valor (nota-ADR)

> Motion Nodes M2, domínio de VALOR (doc 12). Segue 68..76.

## O que é

O **filtro** do domínio de valor — suaviza um campo pela média de cada elemento
com os vizinhos de índice (um box blur sobre a ordem das instâncias). É o
de-ruído: um driver JAGGED (um `value.noise`, um `instance_field` Random) virando
um gradual. O *Filter/Lag CHOP* do TouchDesigner, o Smooth do Cavalry.

- **input** `in` : VALUE (`Instances, Scalar, Frame`, coluna `v`)
- **output** `out` : VALUE — **unário**, comprimento preservado
- **params** `radius` (a meia-janela; default **0** = passthrough)
- **Effect** `Pure` (sem clock, sem estado)

## A novidade: o elemento lê os VIZINHOS

Ao contrário de todos os outros nós de valor até aqui, a resposta do elemento `i`
lê `v[i−r] … v[i+r]`, não só `v[i]`. É a forma do Filter CHOP — uma média móvel
sobre o campo ordenado. A ordem faz sentido quando as instâncias estão em
sequência (uma linha, uma grade), o caso comum de um driver por-instância.

`out[i] = média( v[clamp(i−r, 0, N−1)] … v[clamp(i+r, 0, N−1)] )` — as bordas
**estendem** (um índice clampado repete o valor da fronteira), então a janela é
sempre `2r+1` amostras e o divisor é fixo.

## Decisões

1. **A soma da janela é de ORDEM FIXA (esquerda→direita) nos dois lados**, então a
   paridade é **BIT-EXATA** — é uma soma por-elemento de janela fixa, NÃO a
   redução em árvore cuja ordem o canal de reduce documenta um ε. (A pequena
   diferença medida — `4.77e-7` — vem da fonte Random do fixture, não da soma.)

2. **`radius = 0` é um passthrough bit-exato** (o default neutro): a janela é só
   `[i]`, então `out[i] = v[i]`. Um caminho rápido no WGSL evita o laço.

3. **Bordas por EXTENSÃO (clamp), divisor fixo `2r+1`.** O box blur padrão: um
   índice fora repete a fronteira, e a janela mantém `2r+1` termos, então o divisor
   é uniforme (nunca a contagem de amostras in-bounds). Simples e bit-exato.

4. **A binding é `ReadWrite`.** O kernel LÊ o campo (os vizinhos, de `in_v`) e
   ESCREVE um `out_v` fresco — buffers separados, então uma escrita nunca corrompe
   uma leitura de vizinho. (Contraste com o `value.reduce`, que só escreve e por
   isso usa `Write` — aqui o `in_v` É lido, então não é removido pela naga.)

## Rejeitados

- **Múltiplas iterações / Gaussiana** — N passadas de box blur convergem para uma
  Gaussiana, mas é um 2º param e uma decisão de gosto; um box blur de uma passada é
  o primitivo, e encadear nós dá as passadas. Um box blur **achata um pico num
  PLATÔ** (não um pico arredondado — isso é a Gaussiana), e o gate pina isso.
- **Suavização TEMPORAL (lag no tempo)** — seria estado por-frame (o `motion.delay`
  é o análogo temporal); este é espacial, sobre o índice. Domínios diferentes.
- **Divisor = contagem in-bounds** — a extensão de borda com divisor fixo é o box
  blur canônico e mantém a paridade simples; um divisor variável nas bordas seria
  outra política sem ganho.

## Preço / cobertura

Kernel WGSL = o laço de vizinhos (`clamp(i±k)`, soma esquerda→direita, divide por
`2r+1`) + o fast-path de `radius 0`, binding `ReadWrite` na coluna `v`,
`count_law: None` (unário). Custo `O(radius)` por elemento (barato para um campo de
valor). Sem `applicable` ⇒ **sem fallback de CPU** (o canal de kernel existente lê
os vizinhos do buffer — **nenhum canal novo**). Paridade RTX sobre um campo Random
(o smooth faz trabalho real; um ramp seria quase-identidade); naga valida.

**Gates:** `radius 0` é identidade bit-exata · um pico se espalha num PLATÔ e a
massa se conserva (bordas zero) · campo constante é intocado em qualquer raio ·
saída finita e comprimento preservado (raio maior que o campo clampa) · cook
end-to-end (`[0,3,0]` raio 1 = `[1,1,1]`) · registro · **paridade de dispositivo**
(`#[ignore]`, RTX, Random raio 3 — o laço de vizinhos real).

## Demo — `PH2D_VALUE_SMOOTH_SMOKE=1`

Duas fileiras de 24 instâncias, o MESMO campo Random (mesma seed): de baixo o campo
direto em Y — **espinhado**, cada instância numa altura aleatória; de cima o MESMO
campo `→ value.smooth(Radius 4)` — **gradual**, cada uma a média das vizinhas. O nó
marcado `>> EVALUATE <<` é o smooth — selecione, arraste **Radius** de `0` (volta a
espinhar) até `8` (quase plano na média).
