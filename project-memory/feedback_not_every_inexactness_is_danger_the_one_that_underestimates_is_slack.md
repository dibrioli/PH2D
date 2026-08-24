---
name: feedback-not-every-inexactness-is-danger-the-one-that-underestimates-is-slack
description: Um doc que avisa "esta operação não devolve uma distância exacta" não diz de que LADO — meça o sinal do erro antes de a tratar como perigo
metadata:
  type: feedback
---

O `Unary::Taper` do módulo 3D traz escrito *"é o primeiro modificador que NÃO devolve uma distância
exata"*. Eu classifiquei-o como suspeito de inflar o gradiente sem medir. **Medido: ele DESCE** —
`‖∇f‖` vai de `1,000` a `0,844` com o declive. Ele **subestima** a distância.

**Why:** para uma marcha de esferas, subestimar é **folga** (o raio anda menos que podia) e
sobrestimar é **furo** (atravessa a superfície). O aviso do doc é sobre exactidão, não sobre
segurança, e tratar os dois como o mesmo faz o caminho mais lento definir o teto do mais rápido.

**How to apply:** ao herdar um limite que existe "porque X não é exacto", meça **o sinal do erro** de
X antes de o honrar. E varra o parâmetro: a `Difference Exact` do mesmo módulo lia `1,000` **exacto**
com `r = 0,1` e `1,143` com `r = 0,6` — [[feedback-a-cure-measured-on-a-fixture-that-lacks-the-phenomenon-reads-as-useless]].
