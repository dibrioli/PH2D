# Backup verbatim do módulo PAINTER completo — 2026-07-20

Snapshot tomado na branch `line/Painter` @ 04fd6c60, ANTES da integração do
módulo **Wet Paint** (física real estilo Rebelle — handoff
`docs/HANDOFF_line_Painter_wet_paint_2026-07-20.md`), por ordem do Enio.

Estado no snapshot: painter normal (port Blender + esteroides) + Watercolor
(render-path óptico completo) + Impasto (relevo/material/luz/sculpt/knife/AA)
+ AA de bordas (BUGS #16) + falloffs por tool + todos os smokes aprovados.

Conteúdo (paths verbatim da árvore):
- `crates/ph2d-painter-brush/`      — o engine (dab/stamp/falloff/film/height/smear/jitter/texture)
- `crates/ph2d-tool-painter/`       — a tool (paint modes, watercolor, impasto, sculpt, deform, …)
- `crates/ph2d-panel-painter-layers/` — o painel
- `crates/ph2d-painter-effects/`    — layers + efeitos (host)
- `shells-desktop-render_loop/`     — os `painter_*.rs` da bridge do shell
- `ph2d-render-src/`                — `impasto_light.rs` + `preview_premul.rs` (passes GPU)
- `docs/Painter/`                   — docs vivos do módulo (planos, BUGS, handoffs, ph2d_wet_paint)

NÃO é membro do workspace (não compila); `backups/**` está excluído do typos.
Restauração: copie os diretórios de volta aos paths originais.
