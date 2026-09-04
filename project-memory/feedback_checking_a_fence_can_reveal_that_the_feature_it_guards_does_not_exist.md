---
name: checking-a-fence-can-reveal-that-the-feature-it-guards-does-not-exist
description: "Ao verificar a cerca de uma constante, pergunte se o GESTO que ela protege existe — a do scrollbar protegia a única via de rolagem táctil, e não havia outra"
metadata:
  type: feedback
---

Antes de afinar uma constante, lê-se a cerca dela. O passo que falta quase sempre é o **seguinte**:
*e o que a cerca protege, existe de facto?*

Medido 2026-09-03. A proposta era encolher o `SCROLLBAR_W` de `10` para `2 px` em repouso
(«devolve 8 px a todos os painéis»). A cerca tinha as palavras do dono — *"comfortable drag target
on iPad/tablet"* — o que já bastava para recusar, porque **num tablet não há hover**.

⭐ Mas ao verificar *como* é que um dedo rola um painel, o censo completo deu: **a roda do rato e o
polegar da barra. Mais nada.** Sem `kinetic`, sem `fling`, sem arrasto de corpo — e
`PointerSource::Touch` com **zero** usos fora do host, logo nem «gordo no toque, fino no rato» era
possível.

⇒ Os painéis eram **não-roláveis num tablet**, que é o pré-requisito declarado do trabalho inteiro.
*Oito pixels de largura era a pergunta errada.*

**Why:** uma constante com cerca parece um problema de afinação, e o instinto é negociar o número.
A cerca, porém, é um **ponteiro para um requisito** — e um requisito só vale se a coisa que o
satisfaz existir. Verificar a cerca custa um `grep`; verificar o requisito custa outro, e é o que
transforma uma poupança de 8 px numa feature em falta.

**How to apply:**
- Ao ler uma cerca que nomeia um utilizador ou uma plataforma, **faça o censo do gesto**: *quem, no
  código, satisfaz este requisito?* Sem `head` — [[feedback_a_swallowed_panic_silently_shrinks_the_candidate_set]].
- ⚠️ Um censo por PALAVRA mente: `kinetic|fling|drag_to_scroll` deu 23 acertos e **todos** eram
  outra coisa (*reshuffling*, *fling* de animação). Leia-os.
- ⭐ Quando o gesto novo precisa de saber *«em que painel estou?»*, **derive das tabelas que o
  pintor publica**, nunca de uma lista escrita à mão — a lista irmã (`cursor_over_hero_panel`) já
  tinha deixado um painel mudo na mesma jornada.
- ⛔ E arme o gesto novo só onde **não havia gesto nenhum** (`hit.is_none()`), com um gate de
  controlo a provar que uma pressão reclamada por um widget nunca vira o gesto novo.

Relacionado: [[feedback_documented_decision_chesterton_fence]] ·
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]] ·
[[feedback_an_opt_out_can_name_a_consumer_that_does_not_exist]] ·
[[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]]
