---
name: project-isolation-by-use-and-atlas-strategy-stays-on-object-2026-10-26
description: Dono 26/10 — o problema do Inspector é falta de ISOLAMENTO POR USO (não o desenho); a estratégia Atlas/Individual FICA no objeto; o modelo Godot (componentes como nós) fica em aberto.
metadata:
  node_type: memory
  type: project
  originSessionId: f763a089-165e-460a-bd55-6a2ef47fec9f
  modified: 2026-10-26T19:53:45.587Z
---

Depois de testar o mesmo nível em Godot, Unity e GDevelop (`/home/enio/Documentos/Projetos/Comparacao_Engines/`,
os três funcionam), o dono disse: *«Talvez não seja o desenho da UI o problema. Mas sim a falta de clareza em cada
seção, a falta de isolamento por uso.»* Exemplos dele: Platform High mostra «H Frames 3» (truque interno do
azulejo de fábrica); arrastar a mesma imagem para o canvas dá Individual 1×1 e outra aparência; trocar a Image de
um bloco por Flag mantém as proporções do bloco e não deixa configurar a animação.

Decisões: ⛔ **a estratégia Atlas/Individual fica no objeto como está** — não a esconder nem mudar para a imagem
(26/10: *«quero que o usuário tenha a capacidade de escolher a estratégia»*; *«por enquanto a estratégia Atlas
fica como está»*). O modelo Godot fica adiado; o dono diz que a escolha do modelo Unity (24/08, ADR-0166) foi
induzida pela LLM com o argumento de que o Bevy está mais perto da Unity — não use esse argumento de novo.

**Why:** medido na Unity (doc `docs/GameEngineFuncional/13_isolamento_por_uso_medido.md` da line/GameEngineFuncional):
o corte em quadros, os pixels por metro e as bordas são da IMAGEM; o objeto guarda imagem/modo/tamanho; a
animação manda no campo Sprite.

**How to apply:** a onda 13 da line/GameEngineFuncional isola por uso (imagem parada · repetida · animada), corte
na imagem, uma porta só para meter imagem, aviso ao pôr imagem animada num bloco — sem tocar na estratégia. Liga
com [[project-image-is-a-project-asset-reference-2026-10-26]].
