---
name: feedback_a_premultiplied_source_breaks_the_blend_whose_identity_is_one
description: "Uma fonte pré-multiplicada codifica «não contribui» como ZERO — o que dá a alfa de graça a todo modo cujo neutro é 0 e INVERTE o único cujo neutro é 1 (o Multiply); e um gate que só mede alfa=1 mede o único ponto em que os modos concordam"
metadata:
  type: feedback
---

O Enio reportou, a olho, no smoke `=84`: *"shadow multiply parece não obedecer o alpha
da cor"*. Medido na GPU (fundo 55, frente 128, byte do centro):

| modo | α=0,00 | α=0,25 | α=0,50 | α=0,75 | α=1,00 |
|---|---|---|---|---|---|
| Add · Subtract · Screen · Mix | **55** | … | … | … | … |
| **Multiply (antes)** | **0** | 3 | 6 | 9 | 12 |
| **Multiply (depois)** | **55** | 44 | 34 | 23 | 12 |

Não era *"não obedece"*: era **invertido**. Alfa 0 pintava PRETO, e subir a alfa
CLAREAVA. Não havia valor nenhum em que a sombra desaparecesse.

**Why:** o shader emite `vec4(rgb·α, α)` — uma fonte **pré-multiplicada**, que codifica
*"não contribui"* como **zero**. Todo modo que acumula a partir do zero (`Add`,
`Subtract`, `Screen`, o `over`) ganha a resposta à alfa **de graça**. O elemento neutro
do `Multiply` é **`1`**, não `0` ⇒ com `dst_factor: Zero` a pré-multiplicação leva o
produto para **preto** em vez de para *nada*. *A alfa deixa de dizer «quão presente» e
passa a dizer «quão escuro», ao contrário.* Cura: `src: Dst`, `dst: OneMinusSrcAlpha`
⇒ `dst·(α·src + 1 − α)`, a lei de opacidade de camada do Photoshop.

⚠️ **E o defeito viveu anos dentro de um gate VERDE e honesto** (`blend_mode_regression`,
com tabela e ordenação por modo): ele media tudo a **`alpha = 1`** — o **único ponto em
que os seis modos concordam sobre o que a alfa quer dizer**, e o único em que as duas
colunas acima coincidem (12 = 12). *Uma suíte de modos que varia só o MODO não tem eixo
nenhum para expor uma lei sobre a ALFA.*

**How to apply:**
1. Ao escrever um modo de mistura em **função fixa**, pergunte qual é o **elemento
   neutro** dele. Se for `1` (multiply, e os seus primos color-burn/darken), a
   pré-multiplicação trabalha contra si e o par de fatores tem de trazer o
   `OneMinusSrcAlpha` de volta.
2. **A régua de um modo é «α = 0 devolve o destino, exactamente»**, e a barra é o fundo
   **medido no mesmo passe**, nunca um número escrito à mão. Vale para todos os modos,
   e é uma linha por modo.
3. Uma tabela de modos precisa do **segundo eixo**. Se o parâmetro que o produto oferece
   (a alfa) não é um eixo da suíte, ele não está medido — por mais rica que a suíte seja
   na outra direcção. [[feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless]]
4. ⚠️ **Um par de fatores fixos não exprime a fórmula W3C** (`Cs' = (1−αb)·Cs + αb·B`),
   que precisa da alfa do DESTINO como termo — a versão inteira só existe num passe
   programável (`layer_composite.wgsl` tem-na). Registe a fronteira onde ela vive, senão
   o próximo tenta e não consegue.

*O relatório do Enio é sobre APARÊNCIA e chegou antes de qualquer instrumento:
[[feedback_ergonomics_verdict_is_a_design_bug]]. Irmã de
[[feedback_a_parameter_that_changes_nothing_is_discarded_downstream]] — só que aqui o
parâmetro mudava alguma coisa, e no sentido errado.*
