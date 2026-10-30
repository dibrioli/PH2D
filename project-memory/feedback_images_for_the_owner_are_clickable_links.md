---
name: feedback_images_for_the_owner_are_clickable_links
description: Toda imagem mostrada ao dono (prancha, foto de smoke, simulação) vai como link CLICÁVEL em markdown, nunca como caminho em texto/código
metadata:
  type: feedback
---

Em 30/10 (J10, Painter) mandei ao dono as pranchas «hoje | proposta» como caminhos em `código`; ele
respondeu: «as imagens devem ter link clicável. coloque isso como regra».

**Why:** o dono decide olhando a imagem; caminho em texto obriga-o a procurá-la à mão no explorador.
No VSCode só o link markdown abre o ficheiro com um clique.

**How to apply:** em toda mensagem ao dono, cada imagem vai como `[nome curto](caminho/relativo/à/raiz.png)`
— relativo à raiz do VSCode do dono, que é SEMPRE a árvore primária `/home/enio/Documentos/Projetos/PH2D`,
mesmo quando a janela trabalha numa worktree: o ficheiro de uma linha linka-se como
`Worktrees/line-<módulo>/docs/…` (medido 30/10: o link relativo à worktree «não abre»). Sem crases em volta. Vale
para pranchas, fotos de smoke e simulações; ficheiro fora do repo (scratchpad) copia-se para `pranchas/`
antes de linkar. Ver [[feedback_a_smoke_for_the_owner_explains_what_each_thing_on_screen_is]].
