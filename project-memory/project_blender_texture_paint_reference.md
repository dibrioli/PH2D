---
name: project-blender-texture-paint-reference
description: O Painter do PH2D é reimplementação CLEAN-ROOM do texture paint do Blender (GPL-2.0-or-later vs proprietário) — comportamento sim, expressão nunca; e a referência visual FICOU rastreada contra a própria política escrita
metadata: 
  node_type: memory
  type: project
  originSessionId: 946859c1-ddd7-4f30-9ba5-a6cc4547baac
---

**A decisão que não se re-discute (Enio, 2026-06-20):** o Enio pediu *"port idêntico do
Blender Texture Painter"*; a resposta é **reimplementação clean-room**, porque Blender é
**GPL-2.0-or-later** e o PH2D é **proprietário** (`LICENSE.md`). ⇒ porta-se **algoritmo e
comportamento**, nunca a escrita. O que existe hoje em `ph2d-painter-brush` +
`ph2d-tool-painter` nasceu assim.

⚠️ **Política escrita na mesma decisão:** material de referência **GPL/CC-BY-SA fica
`untracked`** no git.

⛔ **E ela foi quebrada por acidente — medido 2026-08-24:** as **17 capturas do manual do
Blender** (**CC-BY-SA 4.0**) em `docs/Painter/blender_ui_reference/` **estão rastreadas**,
apanhadas de carona por um commit de feature. O README delas ainda diz, em letra própria,
que a pasta é untracked *"até o Enio decidir versioná-la"* — ⇒ *a decisão foi tomada por um
`git add`, não por ele.* Sem impacto hoje (repositório privado; CC-BY-SA dispara na
**distribuição**), mas é escolha dele, não do índice do git.

⭐ **O recorte de FONTE do Blender não existe nesta máquina** (ficou no Mac), e a árvore
tem **zero** arquivos C/C++ não-rastreados. *A parede que importa segurou.*

⭐⭐ **A lição de fidelidade que custou uma UI inteira:** o motor da referência pode ter um
campo que a **UI dela não expõe** — eu expus dois sliders que existiam no modelo de dados e
**não** eram afordância da tela de pintura, e o corte foi refazer a seção inteira.
⇒ **Confira a imagem da UI de referência ANTES de expor um controlo**, não o modelo de dados.

⚠️ **A FORMA de citar proveniência é regulada, e o repo inteiro a violava:** ~460 notas
citam arquivo interno de alvo restrito pelo nome, 25 com transcrição. O fato é lícito
([SKILL_Cleanroom §4.1.2/§4.1.3](../docs/_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md));
o **endereço interno como forma de o citar** não é (§4.2). Achado e prioridades:
`docs/3D/cleanroom/ACHADO_proveniencia_por_nome_interno.md`.

**Onde ler o resto:** plano e algoritmos em `docs/Painter/` · estado vivo no `CLAUDE.md` §5 ·
a história de implementação está no git, e **não** aqui.

Irmãs: [[project-painting-removed-layers-effects-kept]] ·
[[project_painter_brush_came_back_cleanroom]] · [[feedback_documented_decision_chesterton_fence]]
