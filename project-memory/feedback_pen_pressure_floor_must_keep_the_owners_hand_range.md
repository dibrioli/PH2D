---
name: feedback_pen_pressure_floor_must_keep_the_owners_hand_range
description: "Não subir o piso de pressão de um pincel para casar as pontas do Rebelle — a mão do dono (0,24–0,86) perde a resposta; smoke 18/10 revogou"
metadata:
  node_type: memory
  type: feedback
  originSessionId: f3bd0dc0-02fa-4592-844a-b7eab48c811e
  modified: 2026-10-18T13:17:42.095Z
---

Na W6 (18/10) subi o `pressureSize` de 9 pincéis de fábrica para as PONTAS da rampa sintética (0,15→1) baterem as da
foto do Rebelle. No smoke o dono reprovou: «apertar muito e pouco variam pouco a espessura; antes funcionava melhor».
Revertido (`12d7792980`, line/PainterWatercolor).

**Why:** a mão real do dono vai de pressão 0,24 a 0,86 (p5–p95, os 180 s gravados a 17/10). Com o raio mínimo a 0,35–0,8
essa faixa só varia a espessura ~1,3× (contra ~2,7× com o de sempre, 0,9). A rampa sintética de 0,15 a 1 não é a mão.

**How to apply:** antes de mudar qualquer lei de pressão, meça a resposta na FAIXA DA MÃO DO DONO
(`docs/Painter/ferramentas/caneta/medicoes_2026-10-17/pressao_por_toque.txt`), não só na rampa do oráculo; se as pontas
do Rebelle voltarem, é por outra via (curva do tamanho, tecto), nunca pelo piso. Liga a [[feedback_the_owners_reference_product_decides_the_law]].
