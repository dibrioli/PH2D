---
name: project-add-text-entry-owner-decision-2026-11-02
description: Dono 02/11 — o menu Add ganha uma entrada «Text» própria que cria o objecto de texto já em modo Text (o Add ▸ Vector continua a nascer no 1.º traço)
metadata:
  type: project
---

Dono, 02/11 (smoke da GEF 21): ao montar o placar «Coins: N» ele não achou ferramenta Text — ela só existe dentro
de um objecto Vector, e **Add ▸ Vector não cria objecto** (só arma o pincel; o objecto nasce no 1.º traço,
`ph2d-app-vec/src/object_add.rs`). Perguntado entre «Vector nasce vazio», «entrada Add ▸ Text», «as duas» ou
«deixar», escolheu **uma entrada Add ▸ Text própria**, que cria o objecto de texto já em modo Text.

**Why:** o jogo monta-se com as peças ([[project-game-smoke-composes-pieces-never-ready-made-2026-11-02]]), e o
texto do HUD (Text dentro do HUD + componente Label ligado a um Counter) tem de ser alcançável sem truques.

**How to apply:** a entrada vive na família Vector (`ph2d_app_vec::object_add`), cria o `VecShape::Text` pela
porta da ferramenta (`vec_text_object::upsert_text_shape`) e entra em Draw ▸ Text; com o HUD seleccionado, nasce
filho dele. O Add ▸ Vector NÃO muda (não foi pedido).
