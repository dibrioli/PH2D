# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 36: a cor do Shape e o Grain (2026-10-14)

## §1 — Coordenadas

Linha sobre o `main` `0aab2c1cb6` (14/10). Sem push. Prompt:
[36](../NewFeaturesWetPaint/36_PROMPT_janela_cor_do_shape_e_do_grain.md) · avaliação das rampas:
[doc 37](../NewFeaturesWetPaint/37_rampas_de_cor_shape_grain_papel.md) · lista:
[26](../NewFeaturesWetPaint/26_ABERTOS.md) (A5, B8–B12, C15, C21, C22). Commits:

1. `e03302cb34` — a lei da rampa + portões + corte de `stamp_cache.rs`.
2. `41369b2a3d` — a régua do preço (por meio e por N grãos) + a prancha antes/depois.
3. O commit de docs (26, 36, 37, este handoff, fixtures Krita).

## §2 — Mecanismo (D1 e D2 eram um)

O núcleo por píxel (`dab/bands.rs`) e `stamp_color.rs::render_ramp_color_stamp` faziam `cor = rampa[grão]` e saltavam
`grain_coverage` sempre que havia rampa, fosse quem fosse o dono. Dono Shape = `shape_color_ramp_enabled && !bw`
(`ramp_lut.rs::color_ramp_owner`). Efeito: a cor colava-se à tela (o xadrez do grão, D1) e a cobertura enchia (D2).
A guarda `on_grain_assigned` (B&W automático ao atribuir um Grain) escondia-o até o artista desmarcar B&W; o papel
como Grain (`papel_como_grain`, Digital com Relief+Tooth) nunca dispara essa guarda.

## §3 — A lei e a porta

`ph2d-painter-brush/src/ramp_alpha.rs`: `RampIndex { Grain, Silhouette }`, `RampPaint { lut, alpha_mode, index }`,
`ramp_texel(...)`. `stamp_dab_ramped` e `render_ramp_color_stamp` recebem `RampPaint`; no lado da ferramenta
`RampLutOwner::index()` (`ramp_lut.rs`); `RampColorStampKey` ganhou `ramp_index`. A Watercolor já seguia a lei da
silhueta (`rampa_da_aguada`); o Wet Paint nunca lê a rampa de cor.

**Corte (C15):** `stamp_cache.rs` 700 → 500; as rotas `stamp_dabs_cached_ramped`/`stamp_dabs_ramped` foram para
`stamp_ramp_routes.rs`. O censo `plane_fork_tests::the_pigment_deposit_forks_the_canvas_in_parallel` lia só
`stamp_cache.rs` (exige ≥ 5 `fork_canvas(`) — agora lê os dois ficheiros (`concat!`).

## §4 — Portões e mutações

- `a_rampa_do_shape_em_cor_nao_desliga_o_grain`: 3 rotas × com/sem Shape, α igual ao byte; controlo: o grão morde sem rampa.
- `a_cor_da_rampa_do_shape_anda_com_o_pincel`: Shape View, grão xadrez Tiled a profundidade 0,5, queda suave, k = 48 px
  = 1,5 períodos; controlos Δα > 0,02, texels partilhados > 50, gama de matiz > 0,2; Δ matiz ≤ 0,03.
- Mutações 5/5 sangram: M1 lei antiga inteira → ambos vermelhos · M2 só o índice → D1 · M3 só a mordida → ambos ·
  M4 rota assada forçada ao índice Grain → só D2 (D1 usa Tiled, logo por píxel, como desenhado) · M5 rota por píxel
  forçada → ambos.

## §5 — Medições

Régua `tests/cor_do_shape.rs` (sondas `#[ignore]`: `regua_cor_do_shape_e_o_grao`, `prancha_cor_do_shape_e_o_grao`
com `PH2D_PRANCHA_DIR`). D2, antes → depois (média α / buracos):

| Caso | Off | Cor, antes | Cor, depois |
|---|---|---|---|
| Shape none + Noise Tiled | 0,578 / 0,181 | 1,000 / 0,000 | 0,578 / 0,181 |
| Shape View + Noise View | 0,241 / 0,769 | 0,500 / 0,459 (corr 0,862) | 0,241 / 0,769 (corr 1,000) |
| Papel como Grain | α 0,264 | α 1,000 | α 0,264 |

D1 vê-se na [prancha](../NewFeaturesWetPaint/36_prancha_cor_do_shape_antes_depois.png) (linha Shape View + grão xadrez
pequeno Tiled: as cores eram o xadrez do grão, fixo à tela) e no portão (matiz no mesmo ponto relativo ao dab em dois
sítios). Preço (doc 37 §3, não se copia): rampa de cor +0,5 ms/traço no Digital, +1,1 ms no Composite, ruído nos
outros; ~55 ns por amostra de grão por texel, linear em N.
**Lição:** neste motor o Size MULTIPLICA a coordenada da textura (maior = mais fino); a 1.ª régua com Size 0,05
mediu um recorte plano e leu «0 = segue» — o controlo (variância > 0) apanhou-o.

## §6 — Oráculo

Krita 6.0.4 sob Xvfb :97, config descartável: [`fixtures/krita_rampas/README.md`](../NewFeaturesWetPaint/fixtures/krita_rampas/README.md).
Armadilhas: o modo da ponta `brushApplication` no XML do preset é ignorado ao carregar (põe-se no combo do editor);
as definições vazam entre casos se não se recarrega de um preset limpo; o diálogo de gravar PNG da camada pendura o
Krita em lote (usar pixel cru).

## §7 — ABERTO

- **Smoke do dono** (Digital: Shape Color com cores + Grain), passos numerados a escrever com a prancha.
- Decisões B8–B12 (26; recomendações no doc 37 §5).
- C21 (Grain Color no Smear, controlo morto por confirmar) e C22 (riscos da cor baixa da rampa nas bordas AA).
- O Rebelle não foi corrido para as rampas (D2 da lista 26 continua: a bancada não verifica o `DISPLAY`).
