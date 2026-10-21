# Fixtures — a cena «sobre» do Rebelle 8 para as ferramentas que NÃO pintam (J4, família `mistura`)

```
# cabecalho
#   motor     = Rebelle 8 Pro (Trial) sob Wine, porta Motion IO (ws_bateria.mjs), tela virtual — 2026-10-21
#   bateria   = docs/Painter/Pinceis/ferramentas/bateria.json, cena «sobre» (as faixas vermelhas pela BASE)
#   base      = OIL_AND_ACRYLIC:Round/Round Hard (a mesma das fotos de rebelle_bateria/{SMUDGE,ERASER,BLEND})
#   eventos   = bateria_rebelle.py com PH2D_REBELLE_REPETE_TROCA=1: na TROCA de ferramenta o SET_BRUSH vai duas vezes
#   lista     = SMUDGE Default/Soft · ERASER Default/Square Hard, Round Soft, Round Medium · BLEND Default/Soft
#               2.ª passagem (13:38): BLEND Default/Textured · BLEND Default/Soft _aq · WATER Default/Soft ·
#               DRY Default/Sponge, Default/Paper (WATER, DRY e as _aq sobre WATERCOLOR:Watercolor/Round)
#   ⚠️ a foto A do WATER e do DRY é a da água à VISTA (o Rebelle pinta a área molhada a ciano com essas ferramentas):
#               compara-se a B (seca)
#   comando   = PH2D_REBELLE_REPETE_TROCA=1 bash ferramentas/vez_do_app.sh rebelle bash ferramentas/lote_bateria.sh <lista> fixtures/rebelle_mistura
```

**Porquê** (medido 21/10, `exames/mistura.md`): nas fotos de `rebelle_bateria/` o 1.º traço depois de trocar da base
para a ferramenta sai ao tamanho da BASE — o `cruza_1` do Smudge Soft com 68 px contra 15 no `cruza_2`, o do Square
Hard 75 × 39, o do Round Medium 8 × 38 (e o `resto` 0,27 × 0). Com o SET_BRUSH repetido na troca, os três cruzamentos
saem iguais (15 · 15 · 15; 39 · 39 · 39; 38 · 38 · 38). Fora do `cruza_1` (x > 350) as borrachas são as mesmas ao
píxel (0 píxeis com |Δ| > 8), o Blend difere em 859 (máx 20) e o Smudge em ~23 mil: o traço «papel» arrasta para a
direita o rasto do `cruza_1`, que agora é o fino.
