---
name: project_macro_library_owner_decisions_2026_10_23
description: Biblioteca de macros (W18) — decisões do dono 23/10 (1–8, o objecto Macro): a macro leva o objeto e a sua criação; biblioteca + ficheiro para partilhar; aplicar pergunta na hora; lista no painel Macros + asset
metadata:
  type: project
---

Dono, 23/10 (pedido: «criação, salvamento, exportação, importação de macros»):
1. Serve para **as duas coisas**: receitas pessoais e ficheiro para enviar a outros (o mesmo ficheiro).
2. **A macro envolve o objeto inicial e a sua criação** — uma pintura de 1024×1024 tem na macro a criação
   dessa imagem de 1024×1024. A macro é autossuficiente; nada de «esticar à tela de outro tamanho».
3. Aplicar onde já há desenho: **escolho na hora** (criar o objeto da macro ou tocar por cima de um existente).
4. Onde: **lista no painel Macros**, com «guardar como asset» no painel de Assets.
5. **A macro é um OBJECTO** com transformada própria (pos/rot/escala), criado pelo «+» da Hierarchy (Add ▸ Macro),
   gizmo de objecto vazio. Por cima de imagem de outro tamanho: tamanho original, **não no centro** — onde o gizmo está.
6. As pinceladas por cima **entram na imagem de baixo** (o objecto Macro diz onde, em que ângulo e tamanho).
7. **«Tudo pode ser macro»** (ex.: pintar um player, juntar collider, simular física): a macro é um objecto completo e
   independente. A família «mundo» (collider, física) = **W19**; na W18 a porta declara-se recusada.
8. O **Motion também**, igual ao Painter.

W19 (24/10), o dono delegou («tome as melhores decisões sem olhar custos e siga»), decidido pela linha:
9. Ao aplicar, a física da macro cai sobre o **chão do projecto de destino** e sobre o que veio dentro dela.
10. As **teclas do jogador** durante «Simular» entram na macro (o troço da fita), para ela ser independente.

**Why:** até 23/10 a macro só vivia como componente dentro do projeto (sem ficheiro, sem biblioteca).
**How to apply:** prompt `docs/Macros/handoffs/PROMPT_W18_biblioteca_de_macros_2026-10-23.md` na
`line/Macros-Automação`; plano + ADR antes de código. Relacionado: [[project_code_editor_owner_decisions_2026_10_15]].
