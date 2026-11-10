# B08.5 — as «faixas seguras» do PA §E medidas no app a 4K (o «antes»)

- **Binário:** árvore LIMPA `git archive cd02ac77ef` (a mesma régua do `ANTES.md` §B08b) + só o `composite_smoke.rs`
  deste commit (`PH2D_COMPOSITE_KNOBS`), `CARGO_TARGET_DIR=target/b08b-limpo`, `--profile smoke`, `STRIP=none`
  (19:08, incremental). HEAD da worktree no início: `68e20ae858`.
- **Placa:** a Radeon 610M (RADV, WSI sw): 20/20 `.placa` = `libvulkan_radeon.so libgallium`, nenhuma com `nvidia`.
- **Carga:** 12,7 no início · 33–44 no fim do 1.º lote · 45–52 no 2.º (`antes/b085/carga.txt` e a coluna `carga` de cada
  lançamento). ⚠️ **Muito acima de 5: o relógio é TECTO, não veredito.** Só vale a diferença A × B maior que o ruído da
  própria rodada (W16 e X1 deram o A PIOR que o B, ou seja, ruído de ~2×) e a CPU parada (trabalho do próprio app).
- **Régua:** o CONTROLO (SIGSTOP 250 ms) leu 260–349 ms em 20/20; o mesmo segundo sem trava reprova. Vale.
- **Roteiro:** o da matriz até ao CONTROLO (`CURTO=1`): parado 3 s · toque · traço 1 s (400 px) · soltar · +3 s · Ctrl+Z ·
  parado 3 s (CPU). Tela 3840×2160 com o papel Textured (relief 0,5). Uma rodada, A (dentro da faixa) e B (fora)
  INTERCALADOS por caso, o MESMO binário.

```
W=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor; B=$W/target/b08b-limpo/smoke/ph2d-host-desktop
SAIDA=$W/target/b085 bash $W/scripts/ph2d-run.sh bash $W/docs/Painter/ferramentas/mede_pen_up_4k.sh faixas 3840x2160 papel 1 $B
python3 $W/docs/Painter/ferramentas/mede_pen_up_4k.py --faixas $W/target/b085/matriz $W/target/b085/resumo
```

## A tabela (crua em [`antes/b085/faixas.md`](antes/b085/faixas.md), jsonl ao lado)

`]` × n = o raio × 1,15ⁿ (×4,0 com 10, ×8,1 com 15) sobre o tamanho de fábrica do preset.

| caso (PA) | A: dentro | B: fora | traço p50/p95 A → B ms | Ctrl+Z A → B ms | CPU parado depois A → B % | Wet img/s A → B | leitura (carga) |
|---|---|---|---|---|---|---|---|
| W3 secagem (PA-50) | Wet Wash `]`×10, fábrica | + Dry Speed 0,2 · Retention 1 · Water cap 30 · Base 0 | 26/28 → 34/49 | 128 → 190 | **69 → 459** | 39 → 38 | **confirmada**: a água que não seca deixa 4,6 núcleos a trabalhar parado (13–15) |
| W4 Max velocity (PA-51) | Wet Wash, 1 | 3 | 29/31 → 28/41 | 120 → 231 | 62 → 63 | 39 → 38 | sem sinal acima do ruído (16–21) |
| W5 lâmina chata (PA-52) | Flat Bristle fábrica | `]`×10 | **74/146 → 190/190** | 223 → 225 | 57 → 56 | **37 → 10** | **confirmada**: o traço chato largo cai a ~5 img/s (25–34) |
| W6 Size no Wet (PA-53) | Wet Paint fábrica | `]`×10 | 31/44 → 35/67 | 196 → 367 | **45 → 157** | 29 → 33 | **confirmada** no parado e no Ctrl+Z (33–34) |
| W11 Subsurface (PA-58) | Wet Paint, 0 | 0,5 | 30/57 → 47/71 | 356 → 225 | 74 → 51 | 26 → 34 | toque 50 → 120; o resto dentro do ruído (38–41) |
| W16 edgeSoftMass (PA-58) | Wet Paint, 0 | 4 | 67/90 → 29/41 | 332 → 179 | 71 → 61 | 12 → 36 | ruído (A pior que B) (36–38) |
| X1 Paper Relief (PA-63) | Digital, relief 0 | 1 | 20/27 → 34/66 | 97 → 46 | 60 → 53 | — | traço p95 ×2,4; Ctrl+Z ao contrário: inconclusivo (39–41) |
| D2 Size×Spacing (PA-60) | Digital fábrica | `]`×15 | 27/31 → 39/70 | 124 → 75 | 62 → 48 | — | traço p95 ×2,3 (39–43) |
| D1 Color mixing (PA-59) | Round Mixing fábrica | `]`×10 | 56/74 → 44/**400** | 59 → 82 | 37 → 40 | — | **confirmada**: p95 400 ms, +1 s 275 (45–47) |
| C2 Watercolor Size (PA-61) | Watercolor Basic fábrica | `]`×10 | 64/79 → 71/92 | 172 → 144 | 40 → 44 | — | toque 515/430 nos DOIS: o piso é o toque, não o raio (50–52) |

**O que isto diz:** as quatro faixas com sinal forte — secagem (W3), lâmina chata larga (W5), Size do Wet (W6) e Color
mixing largo (D1) — são as que custam a 4K na Radeon, todas de CPU. A CPU parada de 40–75 % EM TODOS os A (a fábrica)
é o piso da tela 4K já descrito no `ANTES.md` (o parado não é zero nem dentro da faixa). Sem avisos de painel
(Decisões, BACKLOG:714): estes números são o «antes» das curas que sobem o tecto.

## O que NÃO entrou nesta rodada, e porquê

| caso | porquê | onde está o número |
|---|---|---|
| W1 (K-M, Opaque, óleo… mantêm a água na CPU) | é contagem, não relógio | B08.4: 0/56 passam `placa_sabe` |
| W2 Grid 1 × Flow 1 | Grid 1 é lei do dono; a área molhada cresce com W6 | W6 acima |
| W7 Real Oil r10–r120 | a cena Composite não tem porta para a caixa Real Oil sem rolar o cartão (`OLEO_ROLA`, só no `wet6`) | `ANTES.md` §óleo 2880/1440 |
| W8 Smooth Edges, W1 Opaque/Body/Medium | linhas `Hidden` do Tuning: `PH2D_COMPOSITE_KNOBS` só chega às linhas com slider | próximo passo |
| W9 Bristle Size × Count, W17 Bristle relief | sem chave de Tuning com esse nome | próximo passo |
| W13 Dryer level, W14 Blender reach | só pagam com a ferramenta Dryer/Blender na mão | próximo passo |
| W15 Tilt | o XTEST do Xvfb não tem inclinação | — (precisa da caneta) |
| W12 Bloom edge, D3 Jitter, D4 camadas do Composite, D5 Blur, C1 Rewet, C3 Charge | o preset de fábrica não expõe a chave ao Tuning do Wet | Composite Brush e Round Mixing no `ANTES.md` §B08b |
| I1 Impasto (4 luzes) | o slider de luz já foi medido | `ANTES.md` §B08b cont. 2 (luz 4K papel pior 118 ms) |
