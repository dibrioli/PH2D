# As ferramentas do Wet Paint no Rebelle (oráculo corrido, 25/10)

- **Fonte:** Rebelle 8.3.4 (teste, sob Wine) numa `kwin_wayland --virtual` (`:2`), arrancado por
  `docs/Painter/Pinceis/ferramentas/rebelle_pronto.sh`. A ferramenta escolhe-se pelo Motion IO
  (`{"event_type":"SET_BRUSH","tool":"SMUDGE","preset":"Default/Soft"}` + `BOOKMARK`, enviado por
  `ws_bateria.mjs`; ver `sel_smudge.json`). Os cliques do `xdotool` NÃO entram na janela do Wine.
  Foto: `magick import -window <id> -crop 245x1020+0+28`.
- `smudge_blend_water_dry.png`: Smudge · Blend · Water · Dry, da esquerda para a direita. `blow.png`: Blow
  (`"tool":"BLOW"`).

| Ferramenta | Controlos do painel | Presets |
|---|---|---|
| Smudge & Liquify | Size · Opacity · Pressure (+ os modos do Liquify) — **sem Water** | Soft · Soft Fast · Spikey · Spikey 2 · Flat · Shattered |
| Blend | Size · Opacity · Pressure — **sem Water** | Soft · Soft 2 · Soft Noisy · Soft Noisy 2 · Dots · Textured · Lines · Hard |
| Water | Size · Water | 10 (Soft … Line) |
| Dry | Size · Absorbency · Keep Layer Wet | 7 (Spatter … Sponge) |
| Blow | Size · Strength | nenhum |

Barra de ferramentas (2 filas): Blend · Smudge · Clone · Eraser · Fill · Pick / Water · Dry · Blow · Select ·
Transform · Crop. Cada ferramenta tem o SEU tamanho (Smudge 15, Blend 25, Water 65, Dry 18 nesta corrida).

⚠️ O **Water do Smear** do PH2D não vem do Rebelle: é pedido do dono (smoke 25/10, N2). O Rebelle decide o
resto do painel de cada ferramenta.
