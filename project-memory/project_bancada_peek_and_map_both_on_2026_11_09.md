---
name: project_bancada_peek_and_map_both_on_2026_11_09
description: "Dono 09/11 (editor de scripts, Onda B item 10): a aba do código esconde o canvas, então «pairar acende o contorno» vira ESPIAR a cena + MAPA na Bancada — os dois ligados por omissão, botões separados no menu View da aba"
metadata:
  type: project
---

O plano 13 dizia «pairar um objecto (grelha, balão, Console) acende o contorno dele no canvas», mas a aba do
código ocupa o lugar do canvas (são exclusivos). Perguntado, o dono escolheu **as duas** opções propostas,
**ligadas por omissão**, cada uma com o seu botão no menu da aba (View ▸ Peek at Scene · View ▸ Scene Map):
- **Espiar a cena:** o rato parado ~0,5 s num nome de objecto faz o código sumir e a cena aparecer, com o
  objecto contornado na cor dele; tirar o rato do nome traz o código de volta.
- **Mapa na Bancada:** cartão «Where» ao lado da grelha «Who has what», com o contorno de cada objecto que
  corre o script, na cor dele; o sob o rato acende.

**Why:** a premissa do plano (canvas à vista ao lado da aba) era falsa; o dono quer ver «qual é o Hero 2» sem
sair do código, e não quis escolher uma das formas.
**How to apply:** não troque nenhum dos dois por outro mecanismo nem os desligue por omissão sem perguntar;
registo em `docs/CodeEditor/00_decisoes_do_dono.md` linha 12. Ver [[reference_topic_owner_product_decisions]].
