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

⚠️ **Ela foi quebrada por acidente e a decisão foi RETOMADA de propósito — 2026-08-24.**
As **17 capturas do manual do Blender** (**CC-BY-SA 4.0**) em
`docs/Painter/blender_ui_reference/` estavam rastreadas por engano (carona num commit de
feature) contra o que o README delas dizia. ⭐ **Decidido: FICAM.** CC-BY-SA permite
redistribuir com atribuição, a proveniência completa já está no README, e remover quebraria
referências vivas e tiraria as imagens das outras máquinas — ⚠️ *esta casa já mediu o que
acontece à referência «que se rebusca depois»: o recorte de FONTE evaporou ao mudar de
máquina.* ⛔ **Duas cercas:** nenhuma delas alcança a UI do produto ou um artefato público, e
se o repositório for publicado a pasta viaja com o README de atribuição.

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
