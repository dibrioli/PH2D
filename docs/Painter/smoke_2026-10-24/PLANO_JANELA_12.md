# Plano da janela 12 — as ferramentas do Wet Paint (N1–N4, um bloco)

Lei: o Rebelle ([`rebelle_ferramentas/`](rebelle_ferramentas/README.md)), salvo o Water do Smear (pedido do dono).

## N1 — as ferramentas na barra da esquerda

- **Fios** (`ph2d-tool-painter`): `wet_paint` · `wet_smear` · `wet_blend` · `wet_water` · `wet_dry` · `wet_blow`
  (`WetTool::fio`/`do_fio`). `set_paint_tool_mode` aceita-os (põe a ferramenta e entra no WetPaint se armado);
  `active_paint_mode_id` devolve-os (o Pincel da água continua `"brush"`). `pick_wet_tool` fica como porta
  (macros) e passa pelo fio.
- **Trilho** (`ph2d-editor-core`): ids `PAINTER_RAIL_WET_*` (5); o store sabe se o meio é o Wet Paint
  (`painter_rail_wet`, publicado pela ponte a cada quadro). Com ele: saem o Smear e o Blur do Digital, entram
  Blend · Smear · Wet · Dry · Blow (a ordem do Rebelle), no grupo «mexer na tinta» da coluna. O Pincel do
  trilho, com a água, manda `wet_paint`. `sync_from_mode` acende o botão certo. Ícones novos (SVG Lucide, ISC):
  `wet-blend`, `wet-water`, `wet-dry`, `wet-blow`; o Smear usa o `smear`.
- **Inspector**: sai a fila «Wet paint tool» e os `PAINTER_WETPAINT_TOOL_IDS` (id órfão ⇒ apaga-se); os
  testes passam a `pick_wet_tool`.

## N1b — cada ferramenta tem o seu pincel

`slot_da_ferramenta` ganha um slot por ferramenta da água (o Rebelle guarda Size por ferramenta). Sem isto um
preset do Smear mudaria o pincel de pintar.

## N2 — o Water do Smear

`Knob::SmearWater` (`0..1`, fábrica `0,5` = a lei de hoje ao bit): abaixo do meio o Smear arrasta menos água
(`0` = esfrega a seco); acima, molha o rasto até ao `WaterCap` (e então dissolve o pigmento assente, pela porta
do lift). Gate: neutro ao bit + mutação.

## N3 — o Inspector por ferramenta

- `FerramentaDoPincel` ganha `SmearMolhado` · `BlendMolhado` · `Molhar` · `Secar` · `Soprar`; as linhas que
  cada uma NÃO lê vêm do censo (`o_painel_mostra_o_que_a_ferramenta_le`, as ferramentas da água entram nele).
- No cartão do Wet Paint, a linha da ferramenta na mão (o knob que o leitor dela lê, fora do Tuning — R3):
  Smear → Strength (`Smear`) + Water (`SmearWater`); Blend → Strength (`BlendForce`); Dry → Absorbency
  (`Dryer`); Blow → Strength (`Blow`); Wet → o Water do topo.

## N4 — presets de fábrica

Smear: os 6 do Rebelle; Blend: os 8. Parâmetros de mão a partir de `rebelle_presets_281.json` (pressão →
tamanho/opacidade, opacidade do carimbo, jitter); pontas NOSSAS. Foto contra o Rebelle pela bateria
(`lote_bateria.sh`), uma rodada só.

## Fecho

Testes por nome + mutação das leis novas; `clippy -D warnings` e `fmt` das crates tocadas; sem suíte. Smoke ao
dono; só depois handoff e prompt.
