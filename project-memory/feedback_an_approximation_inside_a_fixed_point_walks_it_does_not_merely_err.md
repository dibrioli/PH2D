---
name: feedback_an_approximation_inside_a_fixed_point_walks_it_does_not_merely_err
description: "Trocar função exata por tabela/aproximação dentro de laço de realimentação não só erra — CAMINHA; meça deriva sob iteração, não erro de chamada única"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: bac68944-e667-49ef-b3b4-f7b9e430eaca
  modified: 2026-07-24T00:49:15.381Z
---

Quando uma função exata é substituída por tabela/polinômio **dentro de um laço
que realimenta o próprio resultado** (mistura de cor re-aplicada, integrador,
filtro recursivo), o erro de chamada única **não descreve o risco**. O exato
costuma ser um **ponto fixo** — aplicar mil vezes devolve o mesmo valor; a
aproximação tem ponto fixo *próprio*, alguns passos adiante, e o estado
**caminha até lá** e fica.

Caso real (Wet Paint, K–M, 2026-07-23): erro de chamada única 1e-7 (invisível),
mas uma **lavagem parada** — tinta que ninguém tocou — derivava **meio nível de
byte** em 5000 re-misturas. O `libm` era ponto fixo exato (60,00000 continua
60,00000); a tabela fundida ia para 253,51 em c=254.

**Why:** erro de chamada única mede a distância ao valor certo UMA vez; num
laço o que importa é onde o estado *pousa*, e isso é determinado pelos zeros da
função de erro, não pela magnitude dela.

**How to apply:**
1. Gate obrigatório = **iterar milhares de vezes** e comparar com a cadeia exata
   iterada igual. ⚠️ O oráculo é a **cadeia exata**, nunca "o valor fica onde
   estava" — o modelo exato pode legitimamente mover (aqui, o piso de
   refletância move c=12 → 12,7 no primeiro passo, e um gate que exigisse
   imobilidade estaria medindo a física da referência e chamando de erro da
   tabela).
2. Preserve **endpoints exatos por atalho** (`w==0` devolve o destino ao bit):
   peso zero recorre na MESMA célula todo passe, então é o maior gerador de
   caminhada, e o atalho é a resposta matemática, não uma aproximação.
3. Onde a composição tiver **zero** ou derivada infinita (aqui `K/S → 0` no
   branco com `dR/dKS → −∞`), tabule o lado **bem-condicionado** e COMPUTE o
   resto. Uma tabela cujo erro **cancela** no round-trip (porque a volta é a
   inversa exata da ida) é segura; uma cujo erro não cancela contra nada, não.

Relacionado: [[reference_topic_oracle_discipline]] · [[reference_topic_fixture_discipline]] · [[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]]
