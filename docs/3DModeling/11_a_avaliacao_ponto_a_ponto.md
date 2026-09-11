# 11 — A avaliação ponto a ponto (W147)

> **O item aberto que esta wave fecha**, tal como estava escrito no §5 / handoff:
>
> > ⏳ **`Field::at` é 143× mais lento que o caminho em lote** — a cura publicada é o avaliador de
> > gradiente ANALÍTICO da `fidget` (já usado no `hybrid.rs`); é uma wave com espec própria porque
> > **move os números de todos os gates do módulo**.
>
> As duas metades foram medidas. A **primeira** estava certa quanto ao facto e **errada quanto ao
> mecanismo**; a **segunda** estava certa, e agora tem número — a cura publicada **fica recusada**.

---

## §11.1 — O que `Field::at` é, e quem o chama

O [`Field`](../../crates/ph2d-field-eval/src/lib.rs) é a régua ponto a ponto do módulo: **52** sítios
chamam `gradient_norm`, e ~70 ficheiros de teste constroem um `Field`. Só **um** caminho de produto o
usa — o [`field3d_pick.rs`](../../shells/desktop/src/field3d_pick.rs), que escolhe a forma debaixo do
rato. ⇒ *o preço desta função é, quase todo ele, o relógio da suíte*.

## §11.2 — ⛔ O mecanismo que eu nomeei primeiro, e que a medição REFUTOU

A `fidget` documenta o próprio `Context::eval`:

```rust
/// This is extremely inefficient; consider converting the node into a Shape […]
pub fn eval(&self, root, vars) -> Result<f64, EvalError> {
    let mut cache = vec![None; self.ops.len()].into();   // ⬅ O(CONTEXTO INTEIRO), por PONTO
    self.eval_inner(root, vars, &mut cache)
}
```

Daí eu escrevi que o custo era a **alocação por chamada**, proporcional ao documento inteiro. A
tabela desmentiu-o: o caminho novo **não aloca nada** por chamada e continua a crescer com o
documento, a `~2,4 ns por nó`.

⇒ **o que se paga por amostra é percorrer a fita**, e a alocação era uma parcela, não a causa.
*Uma citação do doc-comment de uma dependência é uma pista, não uma medição.*

⚠️ E a mesma leitura mostrou que eu tinha **reintroduzido o defeito pela porta das traseiras**: a 1.ª
fita fazia `clear()` + `resize(n, 0.0)`, ou seja **memsetava o buffer inteiro por chamada** (`98 KB`
num `gradient_norm` sobre uma peça de 2 048 nós). Hoje o scratch só **cresce**, e não se limpa — é
seguro por invariante da fita, não por sorte: em ordem topológica cada slot é escrito antes de
qualquer pai o ler.

## §11.2-bis — A tabela, e o instrumento que a torna legível sob carga

⛔ §5.0: *nenhuma leitura de relógio desta workstation vale acima de `load ~5`* — e esta correu com
outra linha a passar a suíte inteira. ⭐ A saída é que **a contenção só pode ATRASAR**: a sonda faz
`R = 9` corridas curtas e fica com o **MÍNIMO**, que é a estimativa do custo sem vizinhos.

⭐⭐ **E o instrumento tem controlo:** a coluna do caminho **velho** foi medida das duas maneiras —
passagem única com a máquina calma (`load 2,92`) e mínimo-de-9 com ela a arder (`load 11,35`) — e as
duas concordam a **~5 %** (`113,4 / 529,2 / 2 717,7 / 10 692,5` contra `102,3 / 528,7 / 2 617,3 /
11 081,6`). *Uma técnica que se propõe a desmentir a carga tem de ser confrontada com uma leitura
calma, senão é a própria carga a assinar o resultado.*

Mínimo de 9, `load 11,35`, `--release`:

| formas | nós | `at` velho | `at` novo | ganho | `∇` velho | `∇` novo | ganho |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 18 | `102,3` | `16,5` | **`6,2×`** | `644,3` | `53,8` | **`12,0×`** |
| 4 | 128 | `528,7` | `177,8` | `3,0×` | `3 113,3` | `575,8` | `5,4×` |
| 16 | 512 | `2 617,3` | `765,2` | `3,4×` | `15 779,6` | `2 409,6` | `6,5×` |
| 64 | 2 048 | `11 081,6` | `3 062,1` | `3,6×` | `66 641,0` | `10 502,7` | **`6,3×`** |

*(ns por amostra / por gradiente)*

Por nó, a 2 048 nós: **`5,4 ns` → `1,5 ns`** no valor, e **`0,85 ns`** por nó e por ponto no gradiente
— a `eval_many` amortiza a descodificação pelas seis amostras, e é essa a diferença entre `3,6×` e
`6,3×`. ⇒ **o que os 52 sítios chamam ficou `~6×` mais barato.**

## §11.3 — A cura que shipa: a fita `f64`, **bit-a-bit** a mesma resposta

[`point_tape.rs`](../../crates/ph2d-field-eval/src/point_tape.rs) achata o grafo do `Context` em
ordem topológica e avalia-o num passe para a frente, com o scratch `thread_local`.

⭐⭐⭐ **A propriedade que a torna barata de aceitar é a identidade de bits, e ela é POR CONSTRUÇÃO:**

1. percorre **o mesmo grafo** (`Context::get_op`) — a desduplicação e o dobramento de constantes que
   o `import` fez já lá estão;
2. cada valor sai de `BinaryOpcode::eval` / `UnaryOpcode::eval`, **as funções da própria `fidget`**;
3. a memoização do `eval_inner` avalia cada nó **uma vez**, tal como a ordem topológica ⇒ muda só a
   **ordem de visita**, e o valor de um nó só depende dos filhos.

E o `gradient_norm` passa a mandar as **seis** amostras numa passagem só (`eval_many`), porque
descodificar a fita seis vezes era trabalho repetido — a faixa `k` faz exactamente as operações que a
chamada escalar do ponto `k` faria.

⇒ **nenhuma régua do módulo se mexe.** Gates:
[`the_point_probe_is_the_same_answer.rs`](../../crates/ph2d-field-eval/tests/it/the_point_probe_is_the_same_answer.rs)
— `to_bits()`, sobre o valor **e** sobre o gradiente, num corpus de 1 a 40 formas.

## §11.4 — ⛔⛔ A cura PUBLICADA está RECUSADA, e o número é `9,4×` a folga

A nota mandava trocar a diferença central pelo **gradiente analítico** da `fidget`
(`GradSliceEval`, o que o `hybrid.rs` já usa). Medido
([`probe_the_analytic_gradient_blocker.rs`](../../crates/ph2d-field-eval/tests/it/probe_the_analytic_gradient_blocker.rs)):

| região | pontos | mediana | pior |
|---|---:|---:|---:|
| liso | 2 323 | `1,101e-13` | `1,567e-7` |
| vinco (**achado por grelha**) | 122 | `2,174e-6` | `3,058e-6` |
| vinco (**POSTO na aresta**) | 360 | **`1,876e-1`** | **`1,876e-1`** |

A folga típica do módulo é `SLACK = 1,02` sobre um `‖∇f‖` de `1,0` ⇒ **`2,0e-2`**. A discordância na
aresta é **`9,4×` essa folga inteira**.

⚠️ **O mecanismo não é precisão, é que são grandezas DIFERENTES.** Num vinco a derivada **não
existe**: a diferença central com `eps = 1e-4` põe um pé de cada lado e devolve a **média** dos dois
gradientes laterais; o analítico escolhe **um ramo** e devolve `1`. Passar a `f64` não cura — não há
nada para curar. ⇒ *a nota estava certa, e a wave que ela pedia continua fechada.*

## §11.5 — ⛔⛔⛔ E a primeira leitura desta tabela dizia o CONTRÁRIO

As duas primeiras linhas foram medidas sozinhas e o veredito escrito era **«o bloqueio dissolve-se»**:
`2,2e-6` contra uma folga de `2,0e-2` são quatro ordens de grandeza de margem.

**O corpus é que não produzia o fenómeno.** Um vinco é uma **superfície** — conjunto de medida nula —
e a chance de um ponto de uma grelha de passo `0,041` cair a menos de `eps = 1e-4` de uma aresta é
~`0,5 %`. A grelha reportou a população do vinco **60 000× mais limpa do que ela é**, e o detector de
vincos (segunda diferença) estava, na prática, a apanhar curvatura.

⇒ **os pontos do vinco PÕEM-SE, não se procuram**: a peça da 3.ª linha é escolhida para ter uma
aresta cuja equação se escreve (uma esfera a atravessar a face de cima de uma caixa ⇒ o vinco é a
circunferência da intersecção), e amostra-se **em cima** dela.

*Se eu tivesse parado na primeira tabela, tinha shipado uma troca que move toda leitura de aresta em
`0,19`, com a suíte inteira verde a dizer que estava tudo bem.*

## §11.6 — ⏳ O que fica aberto (e o tecto que sobra é honesto)

O caminho em lote do `hybrid` continua **`143×`** mais rápido por ponto, e continua **inalcançável
para esta régua**: ele é `f32` + SIMD + JIT, e §11.4 diz por que a `f32` não serve aqui. O que sobra
sem tocar na resposta:

- **fazer os 52 sítios pedirem N pontos de uma vez** — a `eval_many` já existe e é genérica em `N`;
  hoje só o `gradient_norm` a usa, com `N = 6`. Quem varre uma grelha podia pedir centenas, e a
  descodificação amortizava-se na mesma proporção;
- **`f64` SIMD por faixas** dentro da `eval_many` (4 faixas por registo): é bit-a-bit seguro pela
  mesma razão que a `eval_many` é — faixas independentes —, e não foi medido.

## §11.7 — A varredura em LOTE, e o `5 %` que quase a matou

`79 %` das avaliações de um `worst_gradient` não são gradientes: são o **teste de banda** da casca
(*este ponto está perto da superfície?*), um `at` por ponto de uma grelha de `78³`, cada um a
percorrer a fita sozinho. ⇒ [`Field::at_many`], que varre em faixas de `L = 8`.

Isolada, ela é **`2,4×`–`2,8×`** mais rápida que o `at` ponto a ponto (`1 386,9` contra `3 586,3`
ns/pt a 2 048 nós). Ligada ao censo, o teste melhorou… **`5 %`**.

⚠️ **A hipótese óbvia foi construída e REFUTADA:** *«uma vizinha paralela está a esfomear-lhe a
banda de memória»* — o teste corria ao lado de outro que é 32-way paralelo. Medido **sozinho**, a
razão é a mesma (`44,3 → 41,9`, `5,4 %`, contra `72,0 → 68,6`, `4,8 %`). *Isolar não mudou nada, logo
a contenção não explicava nada.*

## §11.8 — ⭐⭐⭐ O TECTO NÃO ESTAVA NO ALGORITMO: ESTAVA NO PERFIL DE BUILD

A sonda [`where_does_this_census_actually_spend_its_time`] mediu o mesmo trabalho — o
`worst_gradient` das 58 primitivas — nos dois perfis:

| perfil | tempo |
|---|---:|
| `--release` | `3,85 s` |
| `dev` (`opt-level = 0`) | **`44 s`** |

**`11,4×`.** E o `Cargo.toml` da raiz **já tinha a lista** de `[profile.dev.package.*]` com
`opt-level = 2`, criada para o Painter e para o DSP de áudio, com a justificação escrita ao lado:
*«at opt-0 they are 15-25x slower»*. As crates do campo implícito nunca lá entraram.

Acrescentadas (`ph2d-field-eval`, `ph2d-field`, `fidget`, `fidget-core`):

| | antes | depois |
|---|---:|---:|
| `every_primitive_honours_the_march` | `44,3 s` | **`5,7 s`** |
| suíte das 3 crates do campo (407 testes) | `372,3 s` | **`57,1 s`** |
| o teste mais longo da suíte | `307,2 s` | **`43,4 s`** |

⭐⭐ **E aí o lote passou a valer `14 %`** (`5,7 → 4,9 s`) em vez de `5 %`: *a `opt-0` a descodificação
que o lote amortiza está afogada em código não-optimizado, então a MESMA cura mede-se cinco vezes
menor no perfil errado.* ⛔ Se eu tivesse decidido pelo `5 %`, tinha deitado fora uma cura boa **e**
deixado o tecto de pé.

⚠️⚠️ **As duas lições, e custaram meia jornada:**
1. **Uma contagem de OPERAÇÕES não é um perfil.** Eu contei chamadas, acertei na fracção (`92,3 %` do
   tempo *é* a casca) e mesmo assim optimizei contra o tecto errado, porque nunca perguntei **quanto
   custa o total**. O modelo dizia `0,55 s` e o teste dizia `44 s` — *um desacordo de `80×` entre o
   modelo e o relógio é o achado, não um arredondamento*.
2. **O número que se mede depende do perfil em que se mede**, e uma cura pode ser rejeitada por ser
   medida no perfil errado.

⚠️ **Colisão a declarar:** isto edita o `Cargo.toml` da RAIZ, que toda linha toca. São quatro entradas
no fim do bloco `[profile.dev.package.*]` — apêndice, mas o integrador tem de o saber.

## §11.9 — ⛔⛔ DUAS afinações do perfil MEDIDAS e RECUSADAS

A pergunta do dono foi *«qual o melhor possível?»* — logo as duas opções óbvias acima do que shipa
foram medidas, e **as duas ficam fora**:

### (a) `opt-level = 3` em vez de `2` — **sem diferença real**

| | corridas (`every_primitive_honours_the_march`) | melhor |
|---|---|---:|
| nível `2` (1.ª vez) | `4,852` · `5,465` · `5,565` | `4,85 s` |
| nível `3` | `4,689` · `4,655` · `4,742` | `4,66 s` |
| nível `2` (**repetido no fim**) | `4,542` · `4,637` · `4,654` | **`4,54 s`** |

⭐⭐⭐ **O CONTROLO é que decide, e foi ele que salvou o veredito:** o nível `2` **repetido** saiu
**melhor** que o `3`. As duas medições do MESMO nível `2` diferem entre si (`4,85` e `4,54`) mais do
que o `2` difere do `3` ⇒ **a diferença é a máquina a acalmar, não o `opt-level`.**

⚠️ Uma 1.ª tabela sobre a suíte inteira dizia `61,1 s` (nível 2) contra `52,5 s` (nível 3) e parecia
decisiva — e estava **confundida**: as cargas foram `78`, `59` e `46`, sempre a descer. *Um A/B em que
a carga cai monotonicamente mede a carga.* ⇒ fica o `2`, que é a convenção das outras oito entradas
do ficheiro.

### (b) Alargar às outras três crates do campo — **PARTE UM TESTE**

Acrescentar `ph2d-field-render`, `-mesh` e `-profile` a `opt-2` fez reprovar o
`the_shape_constants_are_computed_once_per_shape_not_once_per_tile`:

```text
quadro MORNO tinha de pagar ZERO varreduras e pagou 4
```

⚠️ Ele passa nas duas configurações sem essas crates (`407/407` nas duas). **Não há mecanismo
nomeado** para o `opt-level` mudar uma CONTAGEM de varreduras, e ⛔ *não se liga o que não se
entende* — a fatia fica fora até alguém explicar aquele `4`.

⇒ **o que shipa é o óptimo dentro do que foi medido**: nível `2`, quatro crates.

⛔ **E há uma alavanca que foi vista e NÃO tomada, de propósito.** Dentro da `eval_many` o
`op.eval(…)` volta a fazer o `match` do opcode **por faixa**; içá-lo para fora do laço seria escrever
à mão as arms de `BinaryOpcode`/`UnaryOpcode`. Isso **quebra a propriedade do §11.3.2** — deixaríamos
de usar as funções da própria `fidget` — e troca uma garantia estrutural de identidade de bits por uma
tabela nossa que pode divergir na próxima subida da dependência. *A identidade vale mais que a última
fatia de relógio;* o compilador já tira a maior parte dela (o opcode é invariante do laço).
