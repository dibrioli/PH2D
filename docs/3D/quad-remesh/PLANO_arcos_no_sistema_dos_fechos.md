# PLANO — a restrição dos arcos entra DENTRO do sistema dos fechos

> **Estado:** especificado e **medido**; a implementação está por fazer.
> **Origem:** [`ACHADO_ordem_das_fases.md`](ACHADO_ordem_das_fases.md) §23.13–§23.17.
> **Fonte lícita:** *papers* (Bommes 2009 · Kälberer 2007) + o código da casa. ⛔ Nenhuma
> linha do alvo GPL — a triagem e a vassoura valem como em toda esta linha.

## §1 — O defeito, numa frase, com o número

As **separatrizes** que o F3 traça não são **linhas de grade** do mapa que o G3 resolve.
Medido em 7 peças: `0`–`5 %` dos arcos concordam com o F4, e `44/47` … `90/91` **não são
isolinhas**, atravessando **~1 célula inteira** na mediana (§23.14).

⇒ Uma separatriz é, por definição, onde a grade **termina** num cone. Se ela não é uma
isolinha, as linhas de grade não terminam ali: passam ao lado e continuam. *É a descrição
exacta de um anel que não fecha.*

⚠️ **Não é defeito de código.** O G3 minimiza o desalinhamento do **gradiente** contra o
campo, e o layout do F3 serve **só para cortar**. Nada nunca pediu que os arcos fossem
isolinhas.

## §2 — A equação, e ela já está escrita e gateada

Um arco vai do canto `A` ao `B` na carta de um patch. Com `e` o eixo **atravessado** e
cada cópia a ser `z = R^rot·y + off`:

```text
    e·z_B − e·z_A = 0        ⇒        s_B·y_B[j_B] − s_A·y_A[j_A] = c
```

com `s ∈ {+1, −1}`, porque `e·(R^rot·y) = turn2(e, −rot)·y` e um quarto de volta leva um
**eixo** a outro **eixo com sinal**.

⭐ **Dois escalares, coeficientes `±1`** — a mesma forma que a costura elimina, com metade
da variável. Existe em [`arcline.rs`](../../../crates/ph2d-gridmap/src/arcline.rs), com
`the_axis_identity_holds_for_every_turn` a avaliar **os dois lados** em vez de reler a
minha álgebra.

## §3 — ⭐ O portão já passou (§23.16)

`0` conflitos de sinal nas 7 peças, e o grafo das restrições é **quase uma floresta**
(`0`–`3` ciclos contra `44`–`91` equações) ⇒ **96–100 %** eliminam um escalar
directamente. Os poucos ciclos discordam por **inteiros exactos** (`2`, `3` células).

⚠️ **A razão de ser floresta:** num canto onde quatro arcos se encontram, os horizontais
restringem o escalar `v` e os verticais o `u` — *o grafo parte-se por eixo*.

## §4 — ⛔ Por que a 1.ª implementação não serve, e é o coração deste plano

A wave foi construída como **segunda camada** de eliminação (`PH2D_GRIDMAP_ARCLINE=1`) e
nas peças reais entram **`0` grupos**. O contador dá a razão, **única**, em `100 %` dos
casos (§23.17):

| peça | entraram | recusados | ⭐ **por a classe SER incógnita LIVRE** |
|---|---|---|---|
| `sculpt_eared` | `0` | `11` | **`11`** |
| `sculpt_hooked` | `0` | `17` | **`17`** |
| `sphere_uv_96x144` | `0` | `12` | **`12`** |

⭐⭐⭐ **Os cantos dos arcos SÃO as incógnitas livres do sistema dos fechos** — os extremos
de uma separatriz são cones, e um cone é a variável que a soldadura da costura já possui.

⇒ **A restrição tem de entrar DENTRO do [`ClosureSystem`]**, como **terceira espécie de
fecho**, ao lado do plano e do que roda. ⛔ Não como camada por cima: a lei do
[`weld_flat`](../../../crates/ph2d-gridmap/src/weld_flat.rs) diz que *«duas eliminações
que leem o que a outra escreve não são duas eliminações»*, com a esfera a `NaN` e o toro a
`6,4e17` ao lado.

## §5 — Os passos, cada um com o seu controlo

| # | passo | o controlo que o mede |
|---|---|---|
| ✅ **A1** | **FEITO** (2026-08-27). O `ClosureSystem` ganhou `dep_axes` — por dependente, que **componentes** ele escreve. | ⭐ **Byte-idêntico nas 5 peças** (`sha256` antes/depois) + 2 gates da capacidade. |
| **A2** | As equações dos arcos entram no mesmo sistema, na **mesma ordem topológica** da substituição existente. | `FlatReport` ganha as contagens do arco: eliminadas · ciclo · `worst_det`. ⛔ Um `det ≠ ±1` é **meia célula** e tem de ser contado, não aceite. |
| **A3** | Os ciclos de arco (`0`–`3` por peça, desacordo inteiro) entram como **condição sobre as translações**, exactamente como o fecho plano. | O desacordo tem de ir a `0`; se não for, ele **nomeia** a peça. |
| **A4** | A escada gulosa e o endurecimento correm **com** o sistema novo. | ⛔ **A armadilha já medida:** impor no contínuo e **não** na escada dá saída *byte-idêntica ao controlo com todos os grupos a entrar*. |
| **A5** | Medir. | `measure_arc_quantization` (a coluna **atravessam**, que tem de cair) · `loop_census` (**voltas** e anéis fechados) · `quad_shape` (⚠️ a forma **não pode** regredir). |

### ⭐ A1 saiu MENOR do que esta espec pedia, e a razão é o código

A espec dizia *«`Var` ganha a componente, ou a eliminação passa a escrever uma linha em
vez de uma matriz `2×2`»* — as duas tocariam em quatro sítios. ⭐ **Nenhuma é precisa:**
uma `M2` com a linha não-escrita a **zeros** já dá o valor certo, e por isso o
[`ClosureSystem::bump`] fica correcto **sem uma linha mudar** (ele soma `mul_vec(a, Δ)`, e
um zero exacto soma zero). Quem precisa de saber é só o `apply`, que escreve o valor
**absoluto**.

⇒ A mudança inteira é **um campo (`dep_axes`) e um `if` no `apply`**. ⚠️ *Uma espec escrita
antes de ler o consumidor pede sempre mais do que o consumidor precisa* — e o preço de a
seguir à letra teria sido um refactor de quatro sítios no solver que fechou a casca.

⛔ **A máscara não é alcançável pela `build`** (ali toda eliminação é `2×2` por
construção), então os gates dela entram por `ClosureSystem::probe`, um construtor
`#[cfg(test)]`. *Um gate que só exercitasse o que o construtor produz nunca tocaria no
caminho que a A2 vai usar.*

## §6 — ⛔ O que NÃO reconstruir (medido e rejeitado)

| tentativa | onde parou |
|---|---|
| pregar também os **cantos do layout** no G5 | saída **byte-idêntica**: um canto regular não é variável livre, é escrito por substituição (§23.15) |
| culpar o **arredondamento** | o desvio já está **todo** no contínuo; em 2 peças o G5 até o **melhora** (§23.15) |
| a restrição como **segunda camada** | `0` grupos entram, `100 %` recusados pela mesma razão (§23.17) |
| a restrição como **termo de energia** | ⛔ a Obra A mediu: penalizar em vez de eliminar dá `NaN` e `6,4e17`, e amortecer não cura |
| trocar para a rota do *fill* (que tem F4) | ela é geometricamente **muito pior**: enviesamento `27°` contra `6,8°` (§23.13) |

## §7 — A segunda metade, e ela vem DEPOIS

A §23.14 parte a obra em duas, e a ordem é load-bearing:

1. ⭐ **«este arco é uma isolinha»** (atravessado `= 0`) — **não precisa do F4**, é este plano;
2. **«este arco leva `n` arestas»** (ao longo `= n`) — precisa do F4, que **não está na
   rota que shipa** (§23.13).

⚠️ Mesma razão pela qual a costura veio antes do vinco: a segunda é a mesma maquinaria com
um sujeito a mais, e herdaria qualquer defeito da primeira.

## §8 — A barra

O A/B da §23.13 (ligar o F4 na rota do *fill*) é a evidência de que fechar este desvio
ajuda: as **voltas** melhoram nas três peças medidas (`3,8→1,0` · `0,8→0,3` · `2,8→0,9`).

⛔ **E o que ele NÃO prova:** na `hooked` os anéis **fechados** caem de `8` para `4`. *A
régua das voltas e a contagem de anéis discordam nessa peça, e nenhuma das duas é a
errada* — quem fizer esta wave tem de reportar as duas.
