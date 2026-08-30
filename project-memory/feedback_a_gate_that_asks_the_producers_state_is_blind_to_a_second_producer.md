---
name: feedback-a-gate-that-asks-the-producers-state-is-blind-to-a-second-producer
description: Um gate que pergunta ao ESTADO do produtor («o handle que guardaste mudou?») fica verde quando alguém acrescenta um segundo produtor ao lado — pergunte ao que a SAÍDA emite
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-08-30T12:10:11.808Z
---

Ao gatear *«este produtor não cunha um recurso novo por quadro»*, a 1.ª versão perguntou ao
**estado**: um `probe_frame_id()` que lia o handle guardado no produtor. A prova de mutação
*«volta à porta cara»* foi aplicada como **um segundo desenho, acrescentado ao lado** do bom — e o
gate ficou **VERDE**: o handle guardado continuava o mesmo, e a saída passava a cunhar um recurso
novo por quadro na mesma.

⇒ A cura foi mover a pergunta para a **saída**: `VectorScene::probe_image_ids()`, que lê os ids de
toda imagem que a cena vai desenhar. Os dois `probe_*` de estado foram **apagados** (um deles era
API `pub` sem utilizador: *um id órfão é lixo, e a cura de lixo é apagar*).

**Why:** um gate sobre o estado do produtor mede uma condição **necessária e não suficiente**. Ele
prova *«o que eu guardo é estável»*, e a lei que interessa é *«o que sai é estável»* — as duas
divergem no instante em que existe **mais de um caminho de saída**, que é o caso normal em qualquer
módulo com mais de uma porta de desenho. É a mesma forma do terceiro passo do controlo morto
(`CLAUDE.md` §5.0): *o painel escreve onde · quem lê · o leitor DECIDE?* — aqui o «leitor» é o
recurso partilhado a jusante.

**How to apply:** ao gatear um produtor, pergunte **o que ele emite**, nunca o que ele guarda; se o
que ele emite não for observável, construa a sonda na camada da saída (`probe_*` que devolve tipo
primitivo, sem vazar o tipo da biblioteca) antes de escrever o gate. E teste a mutação na forma
**«acrescentar um segundo produtor»**, não só na forma «trocar o produtor» — é a que distingue as
duas leis. Relacionado: [[feedback_paint_and_dispatch_must_read_the_same_source]] ·
[[feedback_a_dead_knob_has_two_species_no_probe_catches]] ·
[[feedback_an_orphan_id_and_a_dead_knob_read_the_same_and_their_cures_are_opposite]] ·
[[reference_topic_mutation_proofs]]
