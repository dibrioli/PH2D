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
[`the_point_probe_is_the_same_answer.rs`](../../crates/ph2d-field-eval/tests/the_point_probe_is_the_same_answer.rs)
— `to_bits()`, sobre o valor **e** sobre o gradiente, num corpus de 1 a 40 formas.

## §11.4 — ⛔⛔ A cura PUBLICADA está RECUSADA, e o número é `9,4×` a folga

A nota mandava trocar a diferença central pelo **gradiente analítico** da `fidget`
(`GradSliceEval`, o que o `hybrid.rs` já usa). Medido
([`probe_the_analytic_gradient_blocker.rs`](../../crates/ph2d-field-eval/tests/probe_the_analytic_gradient_blocker.rs)):

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

⛔ **E há uma alavanca que foi vista e NÃO tomada, de propósito.** Dentro da `eval_many` o
`op.eval(…)` volta a fazer o `match` do opcode **por faixa**; içá-lo para fora do laço seria escrever
à mão as arms de `BinaryOpcode`/`UnaryOpcode`. Isso **quebra a propriedade do §11.3.2** — deixaríamos
de usar as funções da própria `fidget` — e troca uma garantia estrutural de identidade de bits por uma
tabela nossa que pode divergir na próxima subida da dependência. *A identidade vale mais que a última
fatia de relógio;* o compilador já tira a maior parte dela (o opcode é invariante do laço).
