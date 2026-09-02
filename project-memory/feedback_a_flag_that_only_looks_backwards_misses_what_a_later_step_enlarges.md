---
name: feedback-a-flag-that-only-looks-backwards-misses-what-a-later-step-enlarges
description: "Bandeira «já passou X?» num pipeline vê o mundo de antes — se um passo POSTERIOR alarga a região avaliada, pergunte à lista inteira"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-01T00:20:16.134Z
---

Numa pipeline em que **o domínio de avaliação é o do FIM** (uma caixa de recorte, um envelope, um
orçamento global), uma bandeira do tipo *«já passou um X por aqui?»* acumulada **durante** o laço
está errada: ela vê o mundo de antes, e um `X` **posterior** alarga a região na mesma.

Caso medido (PH2D, `line/3DModeling`, 2026-08-31). A repetição (matriz/coroa) alarga a janela de
células vizinhas quando um deformador já passou. Os **mesmos** modificadores, só a ordem trocada:

| pilha | `‖∇f‖` a `40³` |
|---|---:|
| `[Shell, Array, Twist]` (deformador **depois**) | **`2 224,31`** |
| `[Shell, Twist, Array]` (deformador **antes**) | `0,38` |
| `[Radial, Bend, Radial]` | **`507,09`** |
| `[Bend, Radial, Radial]` | `0,28` |

⇒ **`5 000×`, e a única coisa que muda é a ordem.** A cura é uma linha: perguntar à **lista
inteira** (`mods.iter().any(...)`) em vez de acumular no laço.

⚠️ **E a mesma lei já estava escrita três linhas acima, no divisor** — *«o divisor de um passo
mede-se contra a caixa que a marcha percorre, e ela é a do FIM da pilha»*. Ela foi aplicada a um
consumidor e não ao vizinho. *Quando uma lei é escrita para um leitor, procure já os outros leitores
do mesmo facto* ([[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]).

**How to apply:**
- Ao ver uma bandeira mutável dentro de um laço de pipeline, pergunte *«o que esta bandeira protege
  é avaliado no domínio deste passo, ou no do fim?»*. Se for o do fim, ela tem de ser calculada
  **antes** do laço.
- ⭐ A sonda que o revela é **trocar a ordem** de uma pilha e comparar. Um gate de pares/trios
  derivado do enum (`ALL` nas N posições) faz isso de graça — e foi só o **trio** que o apanhou aqui,
  porque com dois membros o par simétrico existia e escondia
  ([[feedback_composing_two_things_can_cure_the_defect_of_one_so_a_pair_probe_goes_green]]).
- ⚠️ Verifique que o caso de omissão fica **byte-idêntico**: aqui, sem deformador nenhum a bandeira
  é `false` de ponta a ponta, e onde a lei estreita já bastava as células a mais entram por `min` e
  não movem a superfície.
