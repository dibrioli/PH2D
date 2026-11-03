---
name: project_vector_draw_stays_on_vector_selection_2026_11_02
description: "Dono 02/11 — EXCEÇÃO do Vector à regra de 24/10: no Draw do vetor, escolher OUTRO objecto vetor (ou vários) NÃO volta a Object; fica em Draw (as booleanas precisam de várias formas)"
metadata:
  type: project
---

Dono, 02/11 (linha UIUX, depois do smoke OK da ferramenta Offset Path): *«se estou num objeto vector modo draw e
seleciono outro objeto vetor ou vários objetos vector, não deve sair do modo draw para o modo Object. Deve permanecer
em draw. Diferente de outros tipos de objetos, temos boolean com vector e não é legal sair do modo draw ao selecionar
mais de um objeto»*.

**Exceção** à escolha 2 de [[project_modes_share_names_and_switching_object_leaves_2026_10_24]]: só para Vector →
Vector (um ou vários, canvas / Hierarchy / laço). Escolher um objecto de OUTRO tipo continua a voltar a Object;
os outros tipos (Image, Flip…) continuam com a regra de 24/10.

**Why:** as booleanas, o Shape Builder e as outras ferramentas do Draw do vetor trabalham sobre VÁRIAS formas;
sair para Object a cada seleção corta o gesto.
**How to apply:** a decisão de sair do modo ao trocar de seleção tem de perguntar «o novo alvo é todo Vector e o
modo é o Draw do Vector?» antes de voltar a Object. Gate vermelho primeiro (1 forma, várias formas, forma + imagem).
