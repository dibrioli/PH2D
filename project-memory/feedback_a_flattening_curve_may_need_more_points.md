---
name: feedback-a-flattening-curve-may-need-more-points
description: Uma curva de 4 pontos que parece achatar pode ser uma de 6 que não achata — e as duas leituras mandam procurar coisas diferentes
metadata:
  type: feedback
---

Ao varrer um knob para saber **se ele é a causa ou só uma alavanca**, não conclua
pela forma da curva antes de a estender até ela deixar de mudar de forma.

**Why:** medido no quad remesher (2026-08-21). Varrendo o tamanho do lote do
rounding do campo cruzado, as singularidades de uma esfera deram:

| lote | 1/8 | 1/16 | 1/32 | 1/64 | 1/128 | 1/256 |
|---|---|---|---|---|---|---|
| singularidades | 194 | 132 | 84 | **72** | **40** | **24** |

Parando em `1/64` a leitura é *"desce e achata em ~72, que são 9× o chão de 8 ⇒
**há um segundo mecanismo**"* — e essa conclusão manda procurar um mecanismo que
**não existe**. Dois pontos a mais (custo: 65 s) mostram que ela continua a
descer, e a leitura vira *"o lote é a **causa** inteira"* — que manda tornar a
re-resolução barata. ⚠️ **As duas conclusões custam jornadas diferentes, e a
diferença entre elas eram dois pontos.**

⚠️ O sinal de alarme é a curva **achatar num valor que não é o chão teórico**. Se
o chão é conhecido (aqui, 8 por Poincaré–Hopf) e a curva pára acima dele, ou há um
segundo mecanismo **ou a varredura acabou cedo** — e a segunda hipótese é sempre a
mais barata de excluir.

**How to apply:** antes de escrever *"há um segundo mecanismo"*, acrescente pelo
menos dois pontos na direção que estava a melhorar, e só declare achatamento
quando **dois** pontos consecutivos não moverem a agulha. Se o ponto de referência
da literatura for caro demais para correr (aqui, um-de-cada-vez ≈ 1 h contra 11 s),
**orce-o e escreva o orçamento** em vez de o omitir — o orçamento já é meia
resposta. Irmã de [[feedback_scale_before_cause]] e de
[[project_m5_perf_validated]] (não otimize antes de medir); o inverso de
[[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]], que fala do limite
que se escreve sem medir — aqui o erro é a **conclusão** que se escreve com
medição a menos.
