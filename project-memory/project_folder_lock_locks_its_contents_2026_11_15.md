---
name: project_folder_lock_locks_its_contents_2026_11_15
description: "Dono 15/11 — o cadeado de uma pasta COMUM na Hierarchy tranca tudo o que está dentro (como Figma/Photoshop), não só a pasta"
metadata:
  node_type: memory
  type: project
  originSessionId: 2f3480c1-0ba6-4e1c-8629-2cd6e29e41e4
  modified: 2026-11-16T01:41:39.421Z
---

Dono 15/11/2026 (pergunta do §2.4 do handoff W4l do Joint Editor): trancar o cadeado de uma pasta COMUM na
Hierarchy tranca todos os objetos dentro dela — nenhum se move nem se seleciona no canvas até destrancar.
Antes só a montagem do Joint Editor trancava o conteúdo (`locked_by_a_folder` olhava o `Locked` da montagem).

**Why:** o idioma do Figma/Photoshop; uma pasta trancada que deixa mexer no que tem dentro mente.

**How to apply:** generalizar a lei da montagem (W4i, `locked_by_a_folder`) para qualquer pai com `Locked`;
o gate `locking_the_assembly_locks_the_whole_machine` ganha o irmão para a pasta comum. Linha JointEditor
(onda W4o) — ver [[project_joint_editor_version_b_2026_11_12]].
