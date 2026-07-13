---
name: feedback-a-sentinel-needs-a-gate-on-its-reader
description: Valor-sentinel ("vazio", "destacado", 0, -1) só é seguro se TODO leitor o honra — gate o LEITOR, não o escritor
metadata:
  type: feedback
---

O `resolve_entities` sempre escreveu **`entity = 0`** como sentinel de "binding destacada". Era
contrato documentado. Mas **`Entity::from_bits(0)` não devolve uma entidade morta — ele entra em
PÂNICO** (o índice do bevy é `NonZero<u32>`), e o apply do frame decodificava *toda* binding (é
ele quem DECIDE o `missing`, então não pode pular nenhuma para evitar decodificar).

A mina ficou armada por semanas porque o único caminho que escrevia o sentinel era **código
morto**. No dia em que o loader passou a usá-lo, o Ctrl+O de qualquer projeto com animação
derrubava o app — e **o gate que eu tinha escrito afirmava `entity == 0` como condição de
SUCESSO**, enshrinando o estado do crash.

**Why:** um sentinel é um contrato entre um escritor e N leitores. Testar o escritor prova que o
valor é escrito; não prova que alguém sabe lê-lo. E a biblioteca por baixo pode ter uma opinião
própria e violenta sobre aquele valor — `0`, `-1`, `""`, `NaN` e `usize::MAX` são os candidatos
clássicos, e nenhum deles é "obviamente" um nulo.

**How to apply:** ao introduzir (ou passar a USAR) um valor-sentinel: (1) `grep` por TODO leitor
do campo e prove que cada um o honra — em Rust, `try_from_bits`/`checked_*`/`Option` em vez do
construtor infalível; (2) escreva o gate que **dirige o leitor real** com o sentinel dentro (aqui:
o `apply` do frame sobre o documento que o loader instala), não só o escritor; (3) desconfie de
qualquer gate cuja asserção de sucesso seja *"o campo está no valor sentinel"* — ele descreve o
estado, não o que acontece com ele. [[feedback_harness_reproduces_mechanism_not_context]] ·
[[feedback_zero_valued_fixture_is_a_gate_that_cannot_fail]]
