---
name: feedback-a-conservative-verdict-must-separate-unchanged-from-unmeasurable
description: "Num agregador conservador, colapsar \"não mudou\" com \"não sei medir\" no mesmo veredito faz a otimização nunca disparar — e ela fica VERDE em todos os gates"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 39ec3808-26ec-4cf4-b80e-b2291882bc64
  modified: 2026-08-02T02:02:39.125Z
---

Um veredito conservador (`Whole` · `Unknown` · `Bail`) é a resposta segura para *"não sei descrever
isto"*. ⚠️ **Mas ele não pode absorver também *"isto não mudou"*** — as duas frases têm o mesmo efeito
sobre a correção e efeitos OPOSTOS sobre o resultado: a primeira é um custo legítimo, a segunda mata a
otimização em silêncio, e nenhum gate de correção pisca (o produto continua certo, só lento).

**Caso medido (PH2D, o undo confinado, 2026-08-01):** o motor de delta manda para `StoredPlane::Whole`
todo plano que ele não sabe medir — e `fits()` recusa comprimento **zero**. As seis superfícies da
sessão de Sculpt são buffers VAZIOS num traço de pigmento comum, então cada entrada de undo trazia
`spre=WHOLE samt=WHOLE ssum=WHOLE …`, e o acumulador de confinamento azedava em **todo passo do
produto**. Dois buffers vazios não mudam figura nenhuma: instalar qualquer um dos lados devolve o mesmo
plano vazio. A pergunta certa (`is_empty()` nos DOIS) é exata e não custa uma varredura.

**Why:** quem escreveu o `Whole` estava respondendo a *"consigo guardar isto como janela?"*, e quem o LÊ
depois pergunta outra coisa — *"isto mudou?"*. O tipo carrega uma resposta e passa a ser usado para
duas perguntas; o caso degenerado (vazio) é onde elas divergem.

**How to apply:**
1. Ao construir um agregador conservador sobre um enum existente, **liste os casos degenerados do
   produtor** (vazio, tamanho zero, ambos os lados idênticos) e decida por escrito se cada um é
   *não-medível* ou *não-mudou*.
2. O sintoma é *"a otimização não dispara e a suíte está verde"* ⇒ o instrumento é um **relatório
   por-campo com destructure exaustivo**, nunca uma lista de campos escrita à mão: a lista esconde
   exatamente o campo que ninguém lembrou (ver [[reference_topic_gate_discipline]]).
3. Um gate que só exercita o caminho feliz não vê isso — a mutação que reverte a exceção tem de sangrar
   um gate de COMPORTAMENTO (a otimização disparou?), não só um de correção.
4. E o veredito mudo é o que atrasa o diagnóstico: quem devolve `None` deve saber dizer **por quê**, e
   esse instrumento mora dentro do gate que precisa dele — ver
   [[feedback_a_silenced_instrument_reads_as_a_result]].
