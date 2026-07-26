# Doc 79 — `value.wrap`: o MODO DE ENDEREÇAMENTO (Clamp/Repeat/Mirror) do domínio de valor (nota-ADR)

> Motion Nodes M2, domínio de VALOR (doc 12). Segue 68..78.

## O que é

O **modo de endereçamento** do domínio de valor — o que acontece quando um campo
passa das bordas de uma faixa `[min, max]`. Enquanto o `value.map_range` REESCALA
linearmente um intervalo sobre outro (e trava nas pontas), este decide *o que
acontece além da borda*: a mesma escolha que um sampler faz numa fronteira de
textura, ou uma animação faz depois da última chave.

- **input** `in` : o campo de valor (`v`)
- **output** `out` : `v` dobrado de volta em `[min, max]` — **não** um mask `[0,1]`
- **params** `lo` (Min) · `hi` (Max) · `mode` (Clamp/Repeat/Mirror)
- **Effect** `Pure` (sem clock, sem estado); mapa **unário**, comprimento preservado

Os três modos sobre a faixa `[min, max]` (largura `w = max − min`):

- **Clamp** — trava nas bordas: abaixo de `min` lê `min`, acima de `max` lê `max`.
  Um platô de cada lado (`ClampToEdge` / loopOut Continue).
- **Repeat** — ladrilha a faixa: dente de serra que salta de `max` de volta a
  `min` a cada `w` (`Repeat` / loopOut Cycle). `max` mapeia em `min` (ladrilho
  meio-aberto `[min, max)`, a convenção do sampler).
- **Mirror** — dobra ida e volta: triângulo que sobe até `max`, desce até `min` e
  volta, período `2w` (`MirroredRepeat` / loopOut PingPong).

## Decisões

1. **O padrão-ouro é o trio de endereçamento de textura** que TODO renderer traz —
   `ClampToEdge`/`Repeat`/`MirroredRepeat` (Vulkan/GL/WebGPU), o loopOut do After
   Effects (Continue/Cycle/PingPong), o `clamp`/`x%w`/triângulo do VEX. Os três são
   **transcendental-free** (HR-5): um `clamp`, ou uma dobra por `floor` — então o
   port de GPU é comparável em ε ao CPU e o nó é **device-resident** (sem fallback).

2. **Um COMPOSITOR, não um produtor.** Alimente um `value.instance_field` Ramp
   esticado além da faixa (por um `value.map_range`) e o Repeat ladrilha a grade em
   `N` cópias da rampa, o Mirror numa zig-zag — um **período espacial autorado** que
   nenhum produtor sozinho dá. `value.wrap` depois do `map_range` coloca a faixa
   onde você quer; `value.quantize` depois dele escadeia o ladrilho.

3. **A saída NÃO é um mask `[0,1]`** — cai em `[min, max]`, na escala que a faixa
   nomeia (uma dobra sem comparação é significativa em qualquer escala). É o oposto
   do `value.step`, que normaliza a saída de propósito.

4. **`lo`/`hi`, não `min`/`max`.** Os nomes de param viram os nomes dos campos do
   uniform do kernel, e `min`/`max` são **funções builtin** do WGSL — `params.min`
   é um acesso de campo (seguro), mas o desvio custa zero e lê limpo. As LABELS da
   UI dizem "Min"/"Max".

5. **Faixa degenerada (`max ≤ min`) prende em `min`.** Não há intervalo em que
   dobrar, e é isso que mantém todo caminho finito — nenhuma divisão por largura
   zero. Um `r / w` sem guarda seria `inf`/`NaN` envenenando o campo a jusante.

## Rejeitados

- **Um 4º modo "Extend"** (passa direto, identidade) — é um no-op; um modo que não
  faz nada é botão morto.
- **Normalizar a saída em `[0,1]`** — isso jogaria fora a escala da faixa, o oposto
  do ponto. Quem quer `[0,1]` põe `min=0, max=1`.
- **`value.wrap` é distinto do `pulse.compare`** (que existe): aquele é uma ponte
  valor→PULSO (Schmitt trigger, com estado, emite um EVENTO), este é um mapa
  valor→valor **stateless** que dobra na GPU. Domínios diferentes, membrana no meio.

## Preço / cobertura

Kernel WGSL = `vw_round(mode)` + a dobra: Clamp é um `clamp`; Repeat é
`r − w·floor(r/w)`; Mirror é a mesma dobra sobre período `2w` + o `select` do
triângulo. Binding `ReadWrite` na coluna `v` (lê `in_v`, escreve `out_v` — o corpo
CHAMA `read_v`, então nenhum `in_v` é removido pela naga; a lição do `value.reduce`
não recai aqui). `count_law: None` (unário). Sem `applicable` ⇒ **sem fallback de
CPU** (nenhum canal novo).

⚠️ **A única divergência de dispositivo é o `floor` numa FRONTEIRA de célula** —
onde `r/w` é exatamente inteiro, o `floor` de CPU e GPU poderia discordar por um
período INTEIRO. É medida-zero (nenhum valor autorado senta exatamente ali) e a
fixture de paridade usa params **un-round** para não tocá-la — o mesmo precedente
que o `value.quantize`/`field.remap` já pagam. Fora disso a paridade é o FMA que o
driver pode fundir em `lo + (…)`, ε bem abaixo do orçamento de `1e-4`.

**Gates:** Clamp trava nas bordas (identidade dentro, platô fora) · Repeat ladrilha
num dente de serra (`max` volta a `min`; falsificável contra Clamp) · Mirror dobra
num triângulo (`1.3w` lê a metade descendente, não a ascendente; falsificável contra
Repeat) · faixa degenerada prende em `min` e fica finita (`r/w` sem guarda seria
`inf`) · a saída é finita e dentro da faixa para todo valor/faixa/modo · cook
end-to-end (rampa `[0, 1.5]` sobre `[0,1]` Repeat = duas cópias do ladrilho) ·
registro · **paridade de dispositivo** (`#[ignore]`, RTX, rampa esticada a
`[−0.9, 4.7]` dobrada em `[0.1, 1.3]` por Mirror — o caminho mais cheio; `max|d| <
1e-4`).

## Demo — `PH2D_VALUE_WRAP_SMOKE=1`

Três fileiras de 24 instâncias, a MESMA rampa esticada a `[0, 3]` (três larguras da
faixa `[0,1]`) em cada uma: de cima **Repeat** (um dente de serra de três dentes,
marcada `>> EVALUATE <<`), do meio **Mirror** (um triângulo/zig-zag), de baixo
**Clamp** (um platô que trava no topo). Selecione a de cima, aperte a faixa (**Max**
menor) para MAIS dentes, ou troque o **Mode** e veja a mesma rampa virar serra,
triângulo ou platô.
