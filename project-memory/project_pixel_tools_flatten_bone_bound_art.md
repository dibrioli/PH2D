---
name: project-pixel-tools-flatten-bone-bound-art
description: "REGRA DO DONO (2026-09-16) — ferramenta que pinta/apaga/deforma pixels ou muda tamanho/margem trabalha na imagem SEM a dobra dos ossos (exceções: Liquify, filtros, cor, shaders); as de tamanho/margem SOLTAM a imagem dos ossos ao aplicar"
metadata: 
  node_type: memory
  type: project
  originSessionId: 288048cc-7a6f-40b2-8ea6-37cc673393d2
  modified: 2026-09-16T22:48:24.079Z
---

**Ordem do dono (2026-09-16), estendendo a de 2026-09-15 para o Painter:** *«nenhuma ferramenta de
edição de imagem deve trabalhar com arte dobrada»*. Numa imagem presa a ossos, enquanto uma ferramenta
que **pinta, apaga/remove ou deforma pixels** está em uso, a imagem é mostrada **sem a deformação**
(achatada); ao terminar a operação (sair da ferramenta) ela volta a dobrar-se para obedecer aos ossos.

- **Achatam:** o Painter (todos os modos que pintam/apagam/borram/clonam/curam/preenchem/esculpem) e a
  Remoção de fundo (Background Removal) — e toda ferramenta NOVA desta espécie.
- **Exceções (trabalham sobre a arte dobrada):** o **Liquify** (o modo `Deform` do Painter) — *«deverá
  ser capaz de fazer ajustes na imagem dobrada»* —; os **filtros** (contraste, blur, etc.) e os
  **shaders/efeitos** (sombras e afins), que não pintam nem apagam a imagem.
- **Resposta do dono no mesmo dia:** *«Color Equalization e qualquer outra do tipo que trata apenas
  cores, não endireita a imagem, pode ser aplicada dobrada. As que mudam tamanho ou padding devem
  endireitar e se aplicadas quebrar o binding com os ossos.»* ⇒ Padding · Upscale · Equalize Sizes
  endireitam a SELECÇÃO inteira; o Apply delas e dos botões de um clique que mudam tamanho/margem
  (Trim Transparency · Make Square · Rasterize · Real Size) SOLTA a imagem dos ossos (Ctrl+Z devolve).

**Why:** pintar sobre a malha dobrada tinha resíduos que nenhuma cura fechou por inteiro (a marca do
pincel, o chrome, o traço longo) — o dono preferiu a regra simples: editar pixels acontece na imagem
plana, e a dobra é da apresentação.

**How to apply:** a decisão mora numa porta só (`ph2d_app_painter::skin_suspend`, duas tabelas:
`FERRAMENTAS_QUE_ACHATAM` com o alcance, `FERRAMENTAS_QUE_MUDAM_A_MOLDURA`) — a ferramenta nova entra
na TABELA dela, nunca numa cerca própria; e todo Apply de imagem grava pela `commit_edit` da shell com
o id da ferramenta (`Edicao::new("<id>")`). Antes de construir «o pincel X segue a dobra», confira
se X não devia achatar. Ver [[project-skeleton-second-media-gold-standard]] e a fila
`docs/Skeleton/01_a_fila.md` (F6-s).
