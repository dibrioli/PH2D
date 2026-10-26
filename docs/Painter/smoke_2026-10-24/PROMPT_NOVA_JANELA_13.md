# Prompt da janela seguinte — line/PainterWatercolor, 25/10 (a meio de N1–N4; a janela 12 trocou pelo tamanho)

```
Linha line/PainterWatercolor (Modo L), continuação de 25/10 — janela 13, a meio do bloco N1–N4.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md, docs/Painter/smoke_2026-10-24/PROMPT_NOVA_JANELA_12.md (os pedidos
do dono), PLANO_JANELA_12.md e este ficheiro inteiro. Delegue leitura e verificação.
```

⛔ Leis: sem bateria por crate nem suíte (dono 25/10) — testes por NOME + `clippy -D warnings`/`fmt` das crates
tocadas; handoff só DEPOIS do smoke do dono. ⚠️ Os agentes `sonnet` (verificador, documentador, mutacao) estão sem
cota até 27/10, 13 h: use `general-purpose`/fork ou faça à mão.

## O que a janela 12 fez (commit WIP no topo da linha)

- **Oráculo corrido** ([`rebelle_ferramentas/`](rebelle_ferramentas/README.md)): o painel de cada ferramenta do
  Rebelle — Smudge/Blend: Size · Opacity · Pressure (sem Water); Water: Size · Water; Dry: Size · Absorbency · Keep
  Layer Wet; Blow: Size · Strength. Presets: Smudge 6, Blend 8. Como escolher a ferramenta: Motion IO `SET_BRUSH`
  (o `xdotool` não entra na janela do Wine).
- **N1 feito**: fios `wet_paint/wet_smear/wet_blend/wet_water/wet_dry/wet_blow` (`WetTool::fio`, `set_paint_tool_mode`,
  `active_paint_mode_id`); trilho com as 5 ferramentas da água no lugar do Smear/Blur do Digital quando o meio é Wet
  (`no_trilho`, `WidgetStore::painter_rail_wet`, a ponte publica-o); ícones `wet-blend/water/dry/blow` (Lucide);
  o rádio «Wet paint tool» saiu do Inspector e os `PAINTER_WETPAINT_TOOL_IDS` foram apagados (testes →
  `pick_wet_tool`/`Gesto::Modo`).
- **N1b feito**: um slot de pincel por ferramenta da água (`WET_TOOL_SLOT0`, `slot_da_ferramenta(modo, borracha,
  wet)`); o Reset da secção já não troca a ferramenta. Gate: `trocar_de_ferramenta_da_agua_nao_seca_a_tinta`.
- **N2 feito (falta mutação)**: `Knob::SmearWater` (0,5 = a lei de antes ao bit; 0 = a seco; >0,5 molha até ao
  `WaterCap`), em `apply_smear`.
- **N3 a meio**: o cartão mostra a linha da ferramenta na mão (`knobs_da_ferramenta`: Smear → Strength + Water;
  Blend → Strength; Dry → Absorbency; Blow → Strength), knobs fora do Tuning (contagens em `rows.rs`).
  `FerramentaDoPincel` ganhou `SmearMolhado/BlendMolhado/Molhar/Secar/Soprar` e a linha `AguaDoVolume` (o Water do
  topo: só o Pincel e a Wet). As linhas `NAO_LE` delas são um PALPITE.

## Por fazer (por ordem)

1. **O censo `cada_ferramenta_da_agua_so_ve_o_que_le`** (`ph2d-panel-painter-layers`, `#[ignore]` «em construção»):
   corre (~6 min) e acusa (a) o chrome como «inerte» — secções, resets, Sync, docks, hex sem nome: filtre como o
   censo da Borracha (só o que o meio/ferramenta ACRESCENTA ao Pincel da água, ou uma lista de chrome com motivo);
   (b) na metade escondida, `PAINTER_WETPAINT_WATER`, `PIGMENT`, `DUST`, `BLEND`, `COLOR_JITTER_HUE` «mudam texels»
   do Smear — o gesto é aplicado ANTES das riscas, então muda as RISCAS; compare contra uma base com o gesto e as
   riscas sem a ferramenta, ou aplique o gesto só ao slot do Pincel depois das riscas sem encerrar a sessão.
   Depois: as linhas que cada ferramenta não lê (Shape, Grain, Stroke, Jitter… o motor das ferramentas usa
   `TOOL_HARDNESS` fixo), novas `LinhaDaFerramenta` se preciso, e tirar o `#[ignore]`.
2. **N4**: presets de fábrica Smear (Soft, Soft Fast, Spikey, Spikey 2, Flat, Shattered) e Blend (os 8; já há
   «Blender Soft» em `assets/brushes/WetPainter/Blend/Soft/`). Receitas em `tests/it/familias/mistura.rs` +
   `FERRAMENTAS_WET`; regerar com `PH2D_REGERA_PRESETS=1 … gera_os_presets_de_fabrica`. A gaveta
   (`wetpaint_preset/ferramentas.rs`): `ferramenta_dos_presets` ainda devolve `Pincel` no WetPaint — tem de devolver
   a ferramenta da água; `NOMES`/`Gaveta::chave` (o `_ => 4` junta tudo) ganham as cinco. Aplicar um preset de Smear
   só escreve o que o Smear lê (os knobs da água são partilhados entre as ferramentas). Medir contra o Rebelle pela
   bateria (`lote_bateria.sh`, BLEND/SMUDGE já mapeados), uma rodada.
3. Mutação: `SmearWater` (neutro ao bit; `arrasta_agua`; `molha`), `no_trilho`, `slot_da_ferramenta`.
4. `clippy -D warnings` + `fmt` das crates tocadas; o smoke ao dono em passos numerados (`PH2D_COMPOSITE_SMOKE=1`
   ou uma cena com tinta molhada; ensine onde está cada ferramenta e o Water do Smear).

## Abertos que dependem do dono (re-explicar em linguagem simples, do PROMPT_11)

(1) forma fechada no lugar do Newton do Rebelle (muda bits); (2) Borracha da Watercolor que mantém a sessão: buraco
limpo × mata-borrão; (3) arrasto do transfer por instantâneo × Grid Size 2. E os da janela 7/4 (PROMPT_12 §Abertos).
