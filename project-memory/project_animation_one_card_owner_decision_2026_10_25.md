---
name: project_animation_one_card_owner_decision_2026_10_25
description: "Dono 25/10 — configurar animação estava espalhado em 3 sítios (Sprite Sheet, Auto Animate no Platform Player, Animation); aprovado UM cartão Animation + editor de animações próprio, estados por menu, velocidade em quadros/s"
metadata:
  node_type: memory
  type: project
  originSessionId: db96261a-ced1-40cb-b2b5-fa901a7d7a55
  modified: 2026-10-25T15:54:17.636Z
---

Depois do smoke OK da N1 + amostra de arte (GameEngineFuncional, 25/10), o dono achou confuso ter Sprite Sheet,
Platform Player ▸ Auto Animate e Animation para uma só coisa. Pesquisa (Construct 3, Godot, Unity, Unreal): todos
editam animação FORA do Inspector num editor que mostra os quadros, todos em quadros/s, nenhum liga o estado do
personagem à animação sozinho (a nossa vantagem). **Aprovado («sim. siga»):** no Inspector UM cartão «Animation»
(pré-visualização, lista com miniatura, «Editar animações…», «Quem escolhe: Eu · Script · O personagem» com
Parado/Correr/Saltar/Cair em MENUS das tags, nunca texto); editor próprio (lista · folha com grelha, clicar/arrastar
células · fila de quadros com tempo · quadros/s · Repete/Uma vez/Vai-e-volta · ▶); largar uma folha propõe colunas ×
linhas; Platform Player só movimento; Sprite Sheet deixa de ser secção à parte; ficheiros antigos abrem iguais.

**Why:** o dono é o utilizador-alvo (não engenheiro); três cartões para um conceito e nomes de tag em texto livre
(um rename deixa o estado a apontar para nada) — ver [[feedback_a_key_and_a_text_of_the_same_type_is_a_defect_waiting]].
**How to apply:** plano em `docs/GameEngineFuncional/07_animacao_num_cartao_plano.md` (line/GameEngineFuncional);
não acrescentar mais secções de animação ao Inspector fora deste cartão.

**Decisão 2 (25/10, depois do smoke do cartão):** os 4 estados fixos (Parado/Correr/Saltar/Cair) + o resto por
script DIVIDEM o mesmo conceito. Comparação (de memória): Unity/Godot = grafo de estados ou script; Construct 3 e
GDevelop (os mais intuitivos) = REGRAS «quando → animação» com condições dadas pelo comportamento. **Aprovado:**
uma LISTA DE REGRAS no cartão Animation — «Quando ___ → toca ___», os dois lados em menus; escolher «O personagem»
pré-preenche as 4 de hoje; «+ Regra» para o resto; a de cima ganha (arrastar reordena); o script continua para o
difícil. Condições da 1.ª versão: estados do personagem, teclas, mensagens de outros objectos (ex. «levou dano»).
«Abaixar» com corpo baixo é peça à parte. Plano 08 em `docs/GameEngineFuncional/` (a escrever na janela nova).
