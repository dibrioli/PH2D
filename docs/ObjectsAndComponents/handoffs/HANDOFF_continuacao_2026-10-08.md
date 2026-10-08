# Continuação — line/ObjectsAndComponents (08/10, fim da onda 1)

- **Onda 1 fechada** (F0+F1+F2+F3, a Hierarchy nova): [handoff de integração](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_2026-10-08.md).
  O que entrou e as premissas que caíram: [doc 01 §11](../01_objeto_e_componente.md).
- ⚠️ Antes de abrir a onda 2: a linha espera a integração da onda 1 (ordem do dono) — `git cherry main HEAD`.

## Próxima onda (2): F4 — a Animação por objecto (doc 01 §6, escolha 9.1 = A)

- O componente **Animação** no objecto (catálogo `ph2d-component-desc`, família `Animation`); o
  documento por `StableId` (precedente: `ProjectFile.painted`).
- A Timeline segue a selecção — mostra a animação do objecto, ou do ancestral mais próximo que tenha
  uma; cabeçalho *«Animação de: Herói ▾»*. O `TimelineDoc` global de hoje vira a **Animação da Cena**
  (o ícone 🎞 da raiz já abre a Timeline: `ph2d-panel-hierarchy/src/rows.rs::SCENE_COMPONENTS`).
- Os clipes como CONTEÚDO do objecto: um `ModeFamily::content` (ou uma porta irmã, se a Timeline não
  for família de modo — medir) com o grupo *Clips (n)*.
- ⚠️ `PROJECT_SCHEMA` sobe aqui (documento novo por objecto): contar com `python3 scripts/schema-recount.py`.
- Conferir antes: como o `TimelineDoc` aponta objectos (`binding.rs:41`, por `wire_id`) e quem lê
  `ProjectFile.timeline` hoje.

Depois: onda 3 = F5 (Motion objecto) + F6 (busca por intenção + filtro por componente).
