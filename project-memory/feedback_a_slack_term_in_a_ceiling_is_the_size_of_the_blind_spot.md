---
name: feedback_a_slack_term_in_a_ceiling_is_the_size_of_the_blind_spot
description: Um gate `x <= a + folga` não vê nenhum defeito menor que a folga — e a folga posta "por segurança" é exactamente o tamanho do que ele deixa passar; meça a DIFERENÇA entre dois estados em vez de um teto absoluto.
metadata:
  type: feedback
---

Um gate afirmava *«o quadro compila no máximo `regiões + ladrilhos + 2` fitas»*. O termo
`ladrilhos` (**60**) estava lá pela rota que forka a árvore partilhada — legítima, mas **que naquela
fixtura nunca dispara**. A mutação que repunha o defeito (compilar uma fita por lote de pixels de
borda) acrescentava **27** fitas e **SOBREVIVEU**: 27 < 60.

**Why:** um teto absoluto tem de acomodar tudo o que *pode* acontecer, e cada termo dessa
acomodação é **cegueira comprada**. O defeito não precisa de ser pequeno — só precisa de ser menor
que a soma das desculpas. *A folga que se põe "por segurança" é exactamente o tamanho do defeito que
o gate deixa de ver.*

**How to apply:**
- Meça uma **DIFERENÇA entre dois estados do produto**, não um valor absoluto: aqui, o mesmo traçado
  **com** e **sem** a segunda passagem — a diferença é a passagem e mais nada, e o teto passou a ser
  `+1` em vez de `+62`. A mutação morreu na primeira corrida.
- Se o teto tiver mesmo de ser absoluto, **imprima os termos** e confira que o maior deles não é
  maior que o defeito que você quer apanhar.
- ⚠️ E torne a contagem **determinística antes de a afirmar**: em paralelo quem decidia quantos
  avaliadores nasciam era o escalonador da rayon, então o gate correu **serial** de propósito — *um
  gate sobre uma contagem só é um gate se a contagem for do produto.*

Irmã de [[feedback_an_inequality_accepts_a_whole_interval_only_an_oracle_accepts_an_answer]] (uma
desigualdade aceita um intervalo inteiro) e de [[feedback_a_bucket_nobody_fills_reads_as_perfect]].
