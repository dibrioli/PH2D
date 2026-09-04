---
name: an-opt-out-can-name-a-consumer-that-does-not-exist
description: "Uma isenção de gate traz um MOTIVO escrito à mão, e ninguém o verifica — a do pill_group nomeava um consumidor que nunca existiu"
metadata:
  type: feedback
---

Toda lista de opt-out deste repo é `(alvo, motivo)`, e **o gate só verifica o alvo**. O texto do
motivo é prosa: nasce plausível, envelhece em silêncio, e passa a proteger o nada.

Medido 2026-09-03: o `architecture_widget_showcase_coverage` isentava o `pill_group` com
*«compound: covered by the topbar Image Tools pill cluster on every paint»*. O topbar **nunca**
chamou `paint_pill_group` — ele pinta as próprias pílulas e só importava o **token**
`PILL_PADDING_PX`, que viajava por um `pub use` daquele widget. ⇒ um widget de 170 LOC com **zero**
consumidores em todo o repo, mantido vivo por uma frase que ninguém reconferiu.

**Why:** o custo não é o ficheiro morto — é a **decisão que ele bloqueia**. A frase fazia parecer
que apagar a pílula custava alguma coisa (*«mas o topbar usa»*), quando custava zero. Uma isenção
falsa transforma trabalho grátis em trabalho caro *na estimativa*, e ninguém o pega.

⭐⭐ **E há uma segunda espécie, medida em 2026-09-04: a isenção que CONTA.** O
`every_topbar_verb_has_a_door_that_is_not_the_legacy_key` isentava o `TOPBAR_SAVE` com *«as **duas**
linhas do `SaveMenu` estão no menu File»* — verdade no dia em que foi escrita. Em 02/09 outra linha
juntou uma terceira (*Export SVG…*), a frase passou a descrever mal o mundo, e o verbo ficou sem
porta com a suíte inteira verde. ⛔ **A raiz é que a isenção promete sobre uma população que o gate
NUNCA varre**: aquele censo lê os **ids de pill declarados**, e a promessa é sobre as **rows** que
cada pill abria. *Uma isenção só é honesta quando fala da mesma população que o gate conta.*

**How to apply:**
- ⛔ **Nunca acredite no motivo de um opt-out: grepe o consumidor que ele nomeia.** É um comando.
- ⛔ **E se o motivo traz um NÚMERO ou uma promessa («as duas linhas de X estão em Y»), ele já
  envelheceu** — troque-o por um gate que meça a promessa. Cura de 04/09:
  [[feedback_a_line_that_replaces_a_ui_surface_orphans_what_another_line_added_to_the_old_one]].
- Ao **escrever** uma isenção cujo motivo é *«tem consumidor noutro sítio»*, prefira uma que o gate
  possa VERIFICAR — «este símbolo tem ≥1 chamador fora da própria crate» é uma varredura, não uma
  frase.
- ⚠️ E o sintoma vizinho: um `let _ = X; // keep import alive` é um **cadáver de import** — alguém
  apagou o consumidor e manteve a linha para calar o compilador.
- ⭐ Antes de executar uma decisão de produto que parece cara, **CONTE a população**: o
  [[feedback_a_hit_rect_is_also_the_denominator_not_only_the_target]] veio da mesma jornada, e a
  contagem mudou a ordem do trabalho inteiro (o checkbox tinha 81 sítios, o slider 58 — e o estudo
  dava ao checkbox uma linha de tabela).
- ⛔ E conte quem **importa o símbolo**, não quem tem a palavra: três painéis tinham funções locais
  chamadas `paint_toggle` que desenham outra coisa (a contagem por nome dizia 29 sítios; os
  consumidores reais eram 3).

Relacionado: [[feedback_documented_decision_chesterton_fence]] ·
[[feedback_stale_comment_and_dead_code_lie]] ·
[[feedback_a_ratchet_without_a_staleness_census_only_ratchets_up]]
