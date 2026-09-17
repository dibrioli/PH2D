---
name: feedback-a-wrong-id-in-a-list-is-not-a-visible-defect-until-its-reader-is-read
description: Um id errado numa estrutura só é defeito VISÍVEL depois de ler quem a consome — o seletor de cor «nunca vinha à frente» e sempre veio (pintado fora da ordem); passei-o ao dono como bug de ecrã sem ler o pintor (13/09)
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e1350c97-44b0-4738-9885-e8cbde973bb1
  modified: 2026-09-13T11:53:09.180Z
---

A auditoria de fecho da `line/editor-core` achou `INSP_BLENDER_PICKER = NodeId(380)` escrito à mão
num `bump_panel_z` e concluiu *«trazer o seletor de cor para a frente nunca funcionou»*. Eu repeti-o ao
dono como defeito visível, e ele autorizou a cura. Medido na integração de 2026-09-13: o seletor é
pintado FORA da ordem das janelas, depois de todo painel (`screens/hero/panel_walk.rs`), e o walk
salta ids sem painel — o `380` era uma entrada fantasma, sem efeito nenhum no ecrã.

**Why:** a régua leu o ESCRITOR (quem mete o id na lista) e nunca o LEITOR (quem pinta e encaminha o
clique). Um valor errado numa estrutura só vira pixel se alguém a consome para aquele objecto.

**How to apply:** antes de dizer ao dono que algo «não funciona» por um achado de código, siga o
valor até ao consumidor que o ecrã usa (pintor e hit-test) e escreva o sintoma que ELE veria. Se não
há sintoma, é defeito latente: cure-o e gateie-o, mas não o anuncie como produto nem o mande para o
smoke. Irmã de [[feedback-architecture-decisions-are-delegated-to-the-gold-standard]].
