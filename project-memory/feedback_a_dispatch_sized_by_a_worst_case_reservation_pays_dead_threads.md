---
name: feedback_a_dispatch_sized_by_a_worst_case_reservation_pays_dead_threads
description: "Um despacho dimensionado pela RESERVA (o pior caso) paga um fio morto por cada lugar não escrito — as tracejadas pagavam 15,5× fios, não «mais arestas» (doc 121 §9.15)"
metadata:
  type: feedback
---

Doc 121 §9.15 (04/10, line/motion-value): as células das tracejadas custavam `0,63` contra `0,49` ms e o
briefing supunha «os traços têm mais arestas». O instrumento sem relógio (reservadas · escritas) mostrou o
contrário: as tracejadas ESCREVEM menos (`36 288` contra `42 624`), mas o `cs_deposita` corria um fio por
aresta RESERVADA (`561 600`, `15,5×`), e cada fio morto fazia a busca binária da cópia para sair. Cura:
um prefixo das escritas (`0,62 → 0,47`) e uma reserva mais justa (`561 600 → 285 120`; o buffer de arestas
cresce à potência de dois da reserva).

**Why:** um tecto de reserva é seguro para a MEMÓRIA e invisível na imagem; quando ele também dimensiona
um despacho, a folga vira trabalho.

**How to apply:** onde um despacho indirecto lê um total de reserva, meça reservado × escrito antes de
culpar a geometria; despache pelo escrito (prefixo depois da escrita). Ver
[[reference_topic_code_pattern_gotchas]].
