---
name: feedback-a-condition-that-enumerates-its-readers-rots
description: Um guard escrito como lista dos consumidores de um dado (`if a.rake || b.rake`) apodrece quando nasce o terceiro consumidor — e o novo degrada em SILÊNCIO
metadata:
  type: feedback
---

O motor de traço só computava o **heading** (`Dab::dir`) quando o guard dizia
`stroke_method.rake_warmup_eligible() && (texture.rake || shape.rake)` — uma **enumeração dos leitores** de
`dir`, escrita quando os 2 slots de textura eram os únicos.

O **Chisel** do Sculpt nasceu como **terceiro leitor** (ele talha um V em torno do eixo do traço). Ninguém
mexeu no guard. Resultado: o dab do pen-down saía com `dir = [0,0]` ⇒ `perp = [0,0]` ⇒ o termo do V zerava
⇒ **o V colapsava em Scrape**. A ferramenta funcionava "quase": o sulco só começava cego.

**Why:** o degrade é **silencioso e plausível**. Nada quebra, nada loga, e o defeito parece "o pincel é
assim". Pior: o único jeito de o artista consertar era **marcar um checkbox sobre a rotação de uma IMAGEM de
silhueta** — duas portas pra mesma pergunta, e elas já tinham divergido
([[feedback_two_doors_to_the_same_question_diverge]]).

**How to apply:**
- Ao consumir um campo produzido condicionalmente, **grep quem liga a condição** antes de assumir que ela
  vale pra você. Se a condição é uma lista (`a || b`), você provavelmente precisa virar o `|| c`.
- Dê ao novo leitor **o seu próprio canal** (`BrushSpec::needs_heading`) em vez de fazê-lo pegar carona numa
  flag de outra feature. O canal pode até morar numa struct que não é "dele" — documente que é **canal**, não
  propriedade.
- **Gate de ausência + irmão de presença** ([[feedback_absence_gate_needs_a_presence_sibling]]): "nenhum dab
  sai sem heading" fica verde se o motor já desse heading de graça — aí o `needs_heading` seria código morto
  e o bug estaria noutro lugar. O irmão (`sem a flag, o 1º dab É cego`) é o que prova que a flag trabalha.
- E separe **default** de **mecanismo**: ligar o Rake por padrão é UX; o V não pode DEPENDER disso.
