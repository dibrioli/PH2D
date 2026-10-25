---
name: project_modes_share_names_and_switching_object_leaves_2026_10_24
description: Dono 24/10 — Image Paint→Draw, Vector Edit→Draw; escolher outro objecto num modo volta a Object (revoga o cadeado de 03/10); só o de cima no canvas; o Inspector vem à frente
metadata:
  type: project
---

Decisões do dono (24/10, abertura da `line/GameEngineFuncional`, antes de aproximar a engine da Godot):

1. **Nomes coerentes:** Image = Object · Draw · Edit; Flip = Object · Draw · Edit; Vector = Object · Draw. O
   `ObjectMode::Paint` saiu. `Draw` = criar o conteúdo; `Edit` = mexer no que existe.
2. **Num modo de criação, escolher OUTRO objecto (canvas, Hierarquia, laço), do mesmo tipo ou de outro,
   selecciona-o e volta a Object.** ⛔ Revoga a escolha 4 de 03/10 («não trocar», o cadeado) e o «Edit do TIPO»
   de 05/10 para o clique. Desseleccionar continua a não sair (05/10). Partes de dentro (ossos) não saem.
3. **No canvas, num modo, só o objecto de CIMA é escolhível** (sem o ciclo de sobrepostos do modo Object).
4. **Selecionar um objecto traz o Inspector à frente** (abre-o se fechado), e acabar um modo também.

**Why:** o dono quer a facilidade da Godot (tudo é objecto com as suas funções) e modos com o mesmo nome a
dizer o mesmo gesto em todos os tipos.

**How to apply:** não reconstrua o cadeado nem a recusa `object_mode.locked`; uma aba de cima pede o modo de UM
tipo (`CanvasOwner::Mode(kind, mode)`), porque `Draw` é de três. Lei:
`docs/GameEngineFuncional/01_W1_modos_selecao_inspector.md`. Ver [[project_code_editor_owner_decisions_2026_10_15]].
