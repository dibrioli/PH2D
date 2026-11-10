---
name: project_inputmap_three_versions_2026_11_10
description: "Dono 10/11 — Input Map vira aba central com as 3 versões da crítica (A lista, B teclado, C bancada) e seletor no topo; decisões de produto"
metadata:
  node_type: memory
  type: project
  originSessionId: 8054e732-a6a4-45aa-9b3f-a44dad6f65b8
  modified: 2026-11-10T10:14:22.030Z
---

Dono 10/11 (linha `line/InputMap`): implementar as TRÊS maquetes da seção InputMap de
`/home/enio/Documentos/GameEngineSamples/critica_ph2d.html#inputmap`, mantendo a boa aparência das três,
com um SELETOR de tipo de Input Map no topo do quadro central (aba no meio, molde do editor de animações).

Respostas do dono:
- Tela de controles do jogador (C·4) FUNCIONA no jogo (Play e exportado): o jogador troca a tecla e a
  troca fica guardada fora do projeto.
- Projeto novo continua nascendo com as 8 ações de fábrica; receitas ficam num «Start from…».
- A aba abre na última versão usada (1.ª vez: B).
- Luau ganha leitura pela AÇÃO (o jeito pela tecla continua).

**Why:** a crítica mediu 22 números mortos e 38 textos cortados na janela de hoje.
**How to apply:** qualquer agente da linha InputMap parte destas quatro respostas; não as pergunte de novo.
Ligado a [[reference_topic_owner_product_decisions]].

**Entregue 10/11** (linha `line/InputMap`, 82 commits; handoff em `docs/InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_2026-11-10.md`): aba central com as três vistas (List · Keyboard · Try it) e seletor no topo, a última usada guardada nas prefs; a janela flutuante saiu. O mapa entra no undo do projeto por diff; `PROJECT_SCHEMA` 231 → 232. A tela «Controls» do jogador funciona no Play (F1, só com «Players can change keys»), troca guardada em `~/.ph2d/players/<game_id>.txt`; tecla repetida avisa, nunca troca. Lei do Play: UM mapa efectivo para cinco leitores; no Stop o documento inteiro (mapa incluído) é restaurado. Não existe jogo exportado no app — o export herda a tela. Luau lê pela ação (`ph2d.action_*`).

**Dono 10/11, depois do smoke da onda 1 («Muito bom trabalho!»), ordem da ONDA 2:** (1) o teclado desenhado
das versões B e C passa a ser COMPLETO, no layout compacto de notebook da foto dele
(`docs/InputMap/maquetes/06_teclado_notebook_dono.png`: fila F, números, pontuação, modificadores, setas em T
e o teclado NUMÉRICO com Num Lock); (2) polimento pro em cada aspecto — hover, mouse down, foco nos campos
numéricos e todos os acabamentos. Prompt: `docs/InputMap/handoffs/PROMPT_MISSAO_InputMap_ONDA2_2026-11-10.md`.

**Onda 2 entregue 10/11:** o teclado das versões B e C é o de notebook da foto do dono (101 teclas, numérico com Numpad Enter próprio, Num Lock ⇭, teclas do sistema esmaecidas que nunca ligam; a 1280 a B empilha). Camada de polimento: registo único de controlos (premido nativo, Tab, a11y), campo numérico pro, caret que pisca; hover INSTANTÂNEO (D32). Handoff: `docs/InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA2.md`.

Dono 10/11 (depois da onda 2): a letra da tecla fica parada no centro, numa plaquinha tipo adesivo; os traços das acções passam por trás dela.
