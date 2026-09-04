---
name: feedback_a_flake_red_hides_the_rest_of_the_suite
description: "O nextest cancela no primeiro ✗ — um vermelho de flake deixa mil testes por correr, e a suíte parece medida quando foi só amostrada; `--no-fail-fast` é o que a torna uma medição"
metadata:
  type: feedback
---

Gate batched do fecho de uma linha, 2026-08-23. A primeira corrida parou assim:

> `Summary [21s] 10233/11240 tests run: 10232 passed, 1 failed`
> `warning: 1007/11240 tests were not run due to test failure`

O ✗ era uma **flake** de relógio, numa crate que a linha não tocava. Se eu tivesse
parado ali — «um só ✗, e é flake conhecida, verde» — teria fechado a linha sobre
**1.007 testes que nunca correram**. A re-corrida com `--no-fail-fast` deu **17.865**
testes e **outra** falha, em crate diferente, que a primeira nunca chegou a ver.

**Why:** a lista de flakes conhecidas cria a armadilha. Quando o único ✗ é um nome
que já está registado, o instinto é riscá-lo e seguir — e é exactamente aí que o
fail-fast cobra: o teste seguinte ao ✗ não é *"mais um"*, são **todos os que
faltavam**. ⚠️ Uma suíte com flakes conhecidas e fail-fast ligado **não é uma
medição, é uma amostra que pára no primeiro ruído**.

**How to apply:**
1. O gate batched de fecho corre **sempre** com `--no-fail-fast`:
   `CARGO_INCREMENTAL=0 cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast`
2. Antes de atribuir um ✗ a uma flake, **leia o `X/Y tests run`**. Se `X < Y`, a
   corrida não terminou e o veredito ainda não existe.
3. Só depois re-corra o suspeito sozinho (o protocolo do `CLAUDE.md` §5.0) — e
   registe ali toda flake nova, com o **número de corridas sozinho** ao lado.

*O `CLAUDE.md` §5.0 já mandava usar `--no-fail-fast` para os gates de GPU, e a razão
é a mesma; o que faltava era a frase valer para a suíte inteira.*

---

⭐⭐ **O «SOZINHO» DA ASSINATURA QUER DIZER *COM A CARGA MEDIDA*, NÃO *SEM FILTRO*** (2026-09-02).

A família de flakes de carga (CLAUDE.md §5.0) diagnostica-se assim: *verde sozinho 3–5 de 3–5,
zero linhas do diff naquela crate*. Corri `measure_normals_parallel_speedup` três vezes «sozinho»
e deu **3 de 3 VERMELHO** — o que li como *refuta a hipótese de flake, é defeito real*. A máquina
estava a **`load 82`**: a suíte de 20 316 testes ainda a esvaziar em segundo plano. Com
`load 3,2`, o mesmo comando dá **3 de 3 verde**.

**Why:** o passo que existe para DESMENTIR a flake foi executado dentro da condição que a CAUSA.
Um veredito assim manda alguém procurar um defeito que não existe — ou, pior, «curar» um gate
correcto baixando-lhe a barra.

**How to apply:** toda corrida de confirmação de flake imprime `cut -d' ' -f1 /proc/loadavg` **na
mesma linha do resultado**. Abaixo de ~5 o veredito vale; acima, espere (`while` até baixar) e
repita — e note que o próprio `cargo nextest` que acabou de correr deixa a máquina alta por
minutos. Ver [[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]].
