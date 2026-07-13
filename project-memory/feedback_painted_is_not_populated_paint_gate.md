---
name: feedback-painted-is-not-populated-paint-gate
description: Nenhum gate do projeto testava PINTURA — um widget pode estar wirado, testado e contract-limpo e simplesmente não existir na tela
metadata:
  type: feedback
---

**"Esse botão não existe"** é um relato de usuário que passa em TODOS os gates do projeto.

Havia duas provas, e nenhuma cobria a pergunta certa:

| prova | o que responde |
|---|---|
| `tests/seam.rs` (blindagem Fase 1.2) | "o clique CHEGA na tool?" — roda `populate → apply_event → bus → tool` |
| `architecture_panel_wiring_parity` | lê o **texto-fonte**: o id é hit-indexado e registrado no populate? |

**Nenhuma das duas roda o `Panel::paint`.** Então um widget pode estar registrado, wirado,
unit-testado e contract-limpo enquanto a chamada que o desenha mora atrás de um `return` (seção
modal), ou nunca foi escrita — e o app entrega um controle que não existe, com tudo verde.

**Why:** o que o usuário pode clicar é o que a **pintura** registrou no hit index. Registro no
`populate` é permissão para existir; `paint` é existência.

**How to apply:** use `MockPanelHost::paint::<P>(&mut state, viewport)` (em `ph2d-ui-testkit`) —
roda o `Panel::paint` REAL headless (cena Vello sem GPU + `TextSystem::without_system_fonts()`) e
devolve os `(id, rect)` clicáveis. Todo painel novo ganha um teste "todo controle é pintado com
área > 0", e seções modais ganham o par "aparece SÓ no modo X". No primeiro uso, esse gate achou um
bug real: a barra da tira do Flip **descartava em silêncio** todo controle que não coubesse na
linha — num viewport de 1280px, NOVE dos dezoito sumiam (`docs/Flip/BUGS_flip.md` #8-#9).

**Corolário de layout:** esconder um controle é pior que deixá-lo transbordar. Um layout que "nunca
quebra" porque descarta o que não cabe é mentiroso — reporta sucesso e entrega um app mutilado. Se
precisa ceder, ceda em **espaço** (mais linhas, scroll), nunca em **existência**.

Relacionadas: [[feedback_tool_unit_green_integration_dead]] · [[feedback_panel_populate_register]] ·
[[feedback_disabled_button_still_dispatches]]
