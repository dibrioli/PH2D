---
name: feedback_anchor_must_be_invariant_under_user_transforms
description: "Âncora de controle/geometria assada tem de ser INVARIANTE sob o que o usuário mexe (zoom/escala/DPI) — 'o que o usuário vê' muda com a câmera; só geometria pura sobrevive"
metadata:
  type: feedback
---

**A âncora tem de ser invariante sob as transformações que o usuário controla.** O balde do
Flip quebrou TRÊS vezes no mesmo dia (BUGS #12→#13→#14) trocando de âncora: silhueta externa →
borda interna → **eixo da polilinha**. As duas primeiras eram "o que o usuário vê" — mas o que
ele vê é *aparência*, e aparência é função da câmera: com espessura absoluta em px de TELA e
geometria assada em unidades de DOCUMENTO, qualquer âncora derivada da espessura fica congelada
no zoom do clique e transborda `(w/2)·(zoom−1)` px ao aproximar depois. Só o eixo é geometria
pura. Corolário: **duas semânticas de âncora num mesmo controle** (uma pra 0, outra pra <0)
criam uma DESCONTINUIDADE que o usuário sente como "o slider não funciona" (saltava w+1 px
entre grow 0 e −1).

**Why:** é a mesma família do teto-na-unidade-errada ([[feedback_test_with_product_numbers_not_convenient_ones]],
BUGS #11) e do seed≠sample ([[feedback_derived_coordinate_seed_must_match_sample]]): valor
correto num frame de referência, errado sob a transformação que o usuário controla. O harness
só pega varrendo a FAIXA (espessura × zoom), nunca um ponto.

**How to apply:** ao assar geometria derivada de algo que vive noutro espaço (px de tela vs
doc), pergunte "isto muda se o usuário der zoom/escalar DEPOIS?" — se sim, re-ancore em
geometria pura (eixo/centro/ponto autorado) e deixe a aparência para o render. Escreva o gate
que assa num zoom e MEDE noutro (ex.: `the_baked_fill_stays_under_the_line_at_any_later_zoom`
em `ph2d-flip-fill/src/tests.rs`) e um de continuidade do controle através do 0. Ver
[[project_flip_module_grease_pencil_2d]].
