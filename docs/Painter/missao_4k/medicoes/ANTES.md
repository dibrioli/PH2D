# Medição de BASE («antes») da missão Painter 4K — app real, antes de qualquer cura

- **HEAD:** `cd02ac77ef4f54ae5343646a7a6ae7dea5dfc582` (`line/PainterWatercolor`; árvore limpa no build, só `docs/Painter/missao_4k/` por rastrear).
- **Data:** 2026-11-10, 07:24–07:46 (-03).
- **Binário:** `CARGO_PROFILE_SMOKE_STRIP=none bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` (07:17, 1 min 07 s).
- **Carga** (`uptime`): antes `07:24:29 load 0,76 2,44 2,36` · depois `07:45:15 load 2,15 6,23 4,96 (picos de 8 durante o grupo 1080p/blocos: outros agentes; a carga de cada lançamento está no .meta)`. Fatia do `ph2d-run.sh`: CPU ≤ 1600 % de 32 núcleos, 24 G.
- **Placa: a Radeon 610M integrada (RADV), NUNCA a RTX.** Sob Xvfb o app pede `HighPerformance` e abriu a RTX
  (medido: 959 MiB no `nvidia-smi`, 1.ª tentativa, abortada). A matriz esconde a NVIDIA (`VK_DRIVER_FILES` = só o ICD
  da Radeon, GLVND no Mesa) e apresenta pelo WSI de software do Mesa (`MESA_VK_WSI_DEBUG=sw`, o Xvfb não tem DRI3).
  Conferido por lançamento: `.placa` = `libvulkan_radeon.so libgallium` (sem `nvidia`), e `nvidia-smi` só com o `kwin`.
  ⚠️ É a placa da missão, não a do dono: o «antes» e o «depois» TÊM de correr na mesma.

## Comando exato (repetir para o «depois»)

```
W=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
S=$W/target/mede-matriz; B=$W/target/smoke/ph2d-host-desktop; T=$W/docs/Painter/ferramentas/mede_pen_up_4k.sh
CARGO_PROFILE_SMOKE_STRIP=none bash $W/scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
PNG=1 SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T matriz 3840x2160 papel $B
SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T matriz 1920x1080 papel $B
SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T matriz 1024x1024 blocos $B
SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T oleo 2880 $B
SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T oleo 1440 $B     # não corrido no «antes» (prazo)
python3 $W/docs/Painter/ferramentas/mede_pen_up_4k.py --matriz $S/matriz <pasta-de-saída>
```

Um pincel só (para uma cura): acrescente `"<Pincel>:<painel>"` no fim da linha `matriz` (ex.: `"Pastel:digital"`).

## Método (o roteiro de cada lançamento)

Xvfb `:97` 1600×1000, rato e teclado reais (XTEST), um lançamento do app por pincel, **sem perf**; cada quadro escrito
pelo `PH2D_QUADRO_LENTO_MS=0` (`[quadro-lento] parede intervalo quadro`); CPU do processo por `/proc/<pid>/stat`
(utime+stime). Fases: espera a cena armada + 4 s → **parado** 3 s → **toque** (mousedown parado 0,4 s, e o 1.º movimento)
→ **traço** 125 eventos × 8 ms ≈ 1 s, 400 px de janela, numa onda → **soltar** → 3 s (Wet: taxa da água) → **Ctrl+Z** →
1,5 s → **parado** 3 s.

| coluna | o que é |
|---|---|
| base ms | p50 do intervalo de quadro no 1.º parado (o app redesenha SEMPRE: não há quadro ocioso) |
| toque ms | maior intervalo de quadro desde o mousedown até 150 ms depois do 1.º movimento (o 1.º dab) |
| traço p50/p95 | intervalo de quadro de 150 ms depois do 1.º movimento até o último evento |
| soltar ms | maior intervalo nos 200 ms depois do mouseup |
| +1 s máx ms | maior intervalo no segundo depois do mouseup (`soltar_1s_em_ms` no jsonl diz QUANDO) |
| Ctrl+Z ms | maior intervalo no segundo depois do Ctrl+Z (`ctrlz_em_ms` no jsonl) |
| parado CPU % | % de UM núcleo nos 3 s sem entrada, antes do traço / depois do Ctrl+Z |
| Wet img/s | `TAXA DA AGUA` do resumo `[frame]` (janela de 120 quadros) que acaba mais perto de Up + 3 s; só no painel wet |

**Alvos (✗):** traço p95 ≤ 16,6 ms · toque, soltar, +1 s e Ctrl+Z ≤ 33,3 ms (um quadro de 60 Hz a mais) ·
parado ≤ 5 % · Wet ≥ 10 img/s.

## O que NÃO coube no roteiro (sem mudar código de produto)

- **Fundo branco sem papel:** o roteador `PH2D_COMPOSITE_SMOKE` só monta o papel (`PH2D_COMPOSITE_PAPEL=1`) ou a imagem
  de blocos; não há tela branca. Próximo passo: `PH2D_COMPOSITE_FUNDO=branco` em `composite_smoke.rs` (código de cena).
- **Blocos de cor só a 1024²:** sem o papel a cena ignora `PH2D_COMPOSITE_TELA` (`imagem(LADO)`), por isso a linha
  `blocos` é 1024×1024, não 1080p/4K.
- **Real Oil:** é um interruptor do Wet Paint, não um preset, e a cena Composite não tem porta para ele. Medido pela cena
  do FPS (`PH2D_WETPAINT_SMOKE=3`, `PH2D_FPS_OLEO=1`, o app risca sozinho pela ponte, NÃO pelo XTEST) a lado 2880 (= os
  píxeis de 4K) e 1440 (= 1080p): secção «Real Oil» abaixo. Na matriz, o óleo de fábrica é o preset `Express Oil` (Wet).
- **Trocar de camada:** fora (o clique no painel Layers por coordenadas é frágil; fica para uma ronda própria).
- **Tamanho:** o de cada preset (o roteiro não mexe no Size); `Composite Brush` = a pilha da cena (6 camadas, Size 0,4).

## A matriz (45 lançamentos: 15 pincéis × 4K papel · 1080p papel · 1024² blocos)

| pincel | painel | tela | fundo | base ms | toque ms | traço p50/p95 ms | soltar ms | +1 s máx ms | Ctrl+Z ms | parado CPU % (antes/depois) | Wet img/s |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Chalk | digital | 3840x2160 | papel | 11.3 | **99.0 ✗** | 24.0 / **25.0 ✗** | 24.1 | **100.1 ✗** | **101.0 ✗** | **55.9 ✗** / **55.6 ✗** | — |
| Composite Brush | digital | 3840x2160 | papel | 11.0 | **97.1 ✗** | 12.8 / **25.8 ✗** | 25.0 | **99.1 ✗** | **102.4 ✗** | **52.6 ✗** / **53.0 ✗** | — |
| Crayon | digital | 3840x2160 | papel | 11.8 | **97.9 ✗** | 23.8 / **24.3 ✗** | 23.9 | **101.3 ✗** | **103.0 ✗** | **56.6 ✗** / **56.6 ✗** | — |
| Express Oil | wet | 3840x2160 | papel | 11.1 | **98.0 ✗** | 27.0 / **28.8 ✗** | 27.9 | **97.0 ✗** | **101.9 ✗** | **55.3 ✗** / **54.3 ✗** | 40.1 |
| Filbert | wet | 3840x2160 | papel | 11.1 | **101.1 ✗** | 34.0 / **37.2 ✗** | **37.2 ✗** | **101.1 ✗** | **100.9 ✗** | **54.3 ✗** / **55.3 ✗** | 39.9 |
| Finger Tip | digital | 3840x2160 | papel | 11.0 | **98.1 ✗** | 23.6 / **24.5 ✗** | 23.8 | **100.9 ✗** | **100.1 ✗** | **52.6 ✗** / **48.6 ✗** | — |
| Pastel | digital | 3840x2160 | papel | 11.3 | **99.9 ✗** | 24.0 / **25.0 ✗** | 24.8 | **98.0 ✗** | **99.2 ✗** | **55.6 ✗** / **55.0 ✗** | — |
| Round Mixing | digital | 3840x2160 | papel | 10.1 | **98.0 ✗** | 24.1 / **25.2 ✗** | 24.9 | **101.1 ✗** | **100.0 ✗** | **49.6 ✗** / **51.3 ✗** | — |
| Watercolor Basic | water | 3840x2160 | papel | 11.0 | **154.5 ✗** | 20.0 / **20.9 ✗** | 28.0 | **98.8 ✗** | **99.9 ✗** | **52.9 ✗** / **53.9 ✗** | — |
| Wet Liner | wet | 3840x2160 | papel | 11.2 | **97.1 ✗** | 24.1 / **25.1 ✗** | 24.0 | 31.0 | **105.4 ✗** | **55.9 ✗** / **56.3 ✗** | 35.9 |
| Wet Wash Mix | digital | 3840x2160 | papel | 11.8 | **101.8 ✗** | 25.0 / **25.9 ✗** | 24.9 | **102.8 ✗** | **99.0 ✗** | **56.6 ✗** / **55.6 ✗** | — |
| omissão | digital | 3840x2160 | papel | 10.2 | **97.9 ✗** | 23.1 / **23.9 ✗** | 23.9 | **101.9 ✗** | **98.9 ✗** | **50.6 ✗** / **51.3 ✗** | — |
| omissão | impasto | 3840x2160 | papel | 11.0 | **104.2 ✗** | 23.7 / **24.5 ✗** | 30.8 | **109.4 ✗** | **99.1 ✗** | **53.3 ✗** / **55.0 ✗** | — |
| omissão | water | 3840x2160 | papel | 11.1 | **155.2 ✗** | 19.9 / **20.9 ✗** | 27.9 | **101.1 ✗** | **100.1 ✗** | **54.6 ✗** / **53.9 ✗** | — |
| omissão | wet | 3840x2160 | papel | 11.1 | **97.2 ✗** | 23.9 / **24.3 ✗** | 24.0 | 28.6 | **104.0 ✗** | **54.9 ✗** / **54.6 ✗** | 38.5 |
| Chalk | digital | 1920x1080 | papel | 11.0 | **37.1 ✗** | 13.1 / 13.9 | 14.1 | 32.0 | **36.9 ✗** | **54.6 ✗** / **54.3 ✗** | — |
| Composite Brush | digital | 1920x1080 | papel | 12.0 | **35.2 ✗** | 12.1 / 16.0 | 15.8 | **36.1 ✗** | **34.2 ✗** | **59.6 ✗** / **56.6 ✗** | — |
| Crayon | digital | 1920x1080 | papel | 11.0 | **35.0 ✗** | 13.0 / 13.4 | 13.1 | 32.9 | **37.0 ✗** | **55.6 ✗** / **53.9 ✗** | — |
| Express Oil | wet | 1920x1080 | papel | 11.0 | **34.9 ✗** | 14.9 / **17.3 ✗** | 17.0 | 33.0 | **44.9 ✗** | **53.6 ✗** / **53.6 ✗** | 39.8 |
| Filbert | wet | 1920x1080 | papel | 11.0 | **38.3 ✗** | 20.1 / **23.8 ✗** | 23.9 | **33.9 ✗** | **47.8 ✗** | **54.6 ✗** / **55.0 ✗** | 39.8 |
| Finger Tip | digital | 1920x1080 | papel | 10.0 | **34.9 ✗** | 12.8 / 14.6 | 12.9 | 32.9 | **35.0 ✗** | **50.3 ✗** / **50.6 ✗** | — |
| Pastel | digital | 1920x1080 | papel | 12.1 | **35.8 ✗** | 13.7 / 14.1 | 13.7 | **35.9 ✗** | **37.9 ✗** | **59.6 ✗** / **57.6 ✗** | — |
| Round Mixing | digital | 1920x1080 | papel | 12.9 | **37.3 ✗** | 14.6 / 15.1 | 14.9 | **37.0 ✗** | **78.0 ✗** | **61.6 ✗** / **69.8 ✗** | — |
| Watercolor Basic | water | 1920x1080 | papel | 13.0 | **55.8 ✗** | 17.1 / **18.0 ✗** | 24.0 | **36.0 ✗** | **80.2 ✗** | **63.6 ✗** / **70.5 ✗** | — |
| Wet Liner | wet | 1920x1080 | papel | 13.8 | **39.0 ✗** | 15.0 / 16.0 | 17.0 | 18.8 | **49.4 ✗** | **65.3 ✗** / **66.3 ✗** | 39.8 |
| Wet Wash Mix | digital | 1920x1080 | papel | 13.9 | **38.1 ✗** | 15.9 / **16.8 ✗** | 16.6 | **35.9 ✗** | **34.0 ✗** | **65.3 ✗** / **63.9 ✗** | — |
| omissão | digital | 1920x1080 | papel | 10.9 | **34.9 ✗** | 13.0 / 14.8 | 13.9 | 31.7 | 32.2 | **53.3 ✗** / **51.6 ✗** | — |
| omissão | impasto | 1920x1080 | papel | 10.9 | **35.9 ✗** | 13.0 / 14.4 | 15.2 | **35.1 ✗** | 30.9 | **53.0 ✗** / **52.6 ✗** | — |
| omissão | water | 1920x1080 | papel | 11.0 | **52.1 ✗** | 14.4 / **16.9 ✗** | 21.2 | **34.8 ✗** | **36.0 ✗** | **53.6 ✗** / **52.6 ✗** | — |
| omissão | wet | 1920x1080 | papel | 11.0 | **34.9 ✗** | 13.0 / 14.0 | 13.6 | 13.8 | **40.2 ✗** | **54.3 ✗** / **64.3 ✗** | 40.3 |
| Chalk | digital | 1024x1024 | blocos | 11.0 | 15.1 | 11.0 / 11.2 | 12.7 | 12.7 | 12.1 | **59.9 ✗** / **54.3 ✗** | — |
| Composite Brush | digital | 1024x1024 | blocos | 10.0 | 18.9 | 10.9 / 14.0 | 14.0 | 14.0 | 11.5 | **52.0 ✗** / **51.0 ✗** | — |
| Crayon | digital | 1024x1024 | blocos | 12.8 | 15.2 | 12.0 / 12.9 | 12.1 | 12.1 | 22.7 | **65.9 ✗** / **64.9 ✗** | — |
| Express Oil | wet | 1024x1024 | blocos | 10.1 | 18.1 | 12.9 / 14.0 | 15.8 | 15.8 | 30.1 | **55.0 ✗** / **54.3 ✗** | 40.3 |
| Filbert | wet | 1024x1024 | blocos | 10.7 | 22.9 | 19.8 / **23.0 ✗** | 18.9 | 18.9 | **35.0 ✗** | **54.9 ✗** / **54.3 ✗** | 40.0 |
| Finger Tip | digital | 1024x1024 | blocos | 10.7 | 11.3 | 9.9 / 10.5 | 10.1 | 11.0 | 10.9 | **57.9 ✗** / **52.6 ✗** | — |
| Pastel | digital | 1024x1024 | blocos | 13.8 | 20.1 | 15.9 / **17.1 ✗** | 15.2 | 15.8 | 13.9 | **69.3 ✗** / **59.3 ✗** | — |
| Round Mixing | digital | 1024x1024 | blocos | 12.0 | 13.9 | 12.1 / 13.2 | 15.0 | 15.0 | 13.1 | **63.6 ✗** / **58.9 ✗** | — |
| Watercolor Basic | water | 1024x1024 | blocos | 12.0 | 21.9 | 14.0 / **17.3 ✗** | 22.2 | 23.3 | 16.8 | **62.9 ✗** / **67.9 ✗** | — |
| Wet Liner | wet | 1024x1024 | blocos | 11.2 | 18.9 | 12.4 / 13.2 | 16.8 | 17.8 | 30.8 | **60.6 ✗** / **58.9 ✗** | 39.7 |
| Wet Wash Mix | digital | 1024x1024 | blocos | 10.7 | 12.9 | 12.2 / 13.2 | 13.0 | 13.0 | 12.1 | **55.3 ✗** / **56.3 ✗** | — |
| omissão | digital | 1024x1024 | blocos | 13.2 | 13.9 | 11.0 / 12.3 | 10.9 | 11.7 | 14.0 | **66.6 ✗** / **56.0 ✗** | — |
| omissão | impasto | 1024x1024 | blocos | 12.1 | 26.4 | 14.5 / **18.9 ✗** | 14.8 | 16.9 | 14.1 | **65.6 ✗** / **56.6 ✗** | — |
| omissão | water | 1024x1024 | blocos | 10.0 | **33.9 ✗** | 14.9 / 16.0 | 21.0 | 21.0 | 15.0 | **54.3 ✗** / **68.9 ✗** | — |
| omissão | wet | 1024x1024 | blocos | 10.3 | 14.8 | 11.3 / 12.9 | 14.9 | 14.9 | 26.2 | **57.0 ✗** / **62.3 ✗** | 40.2 |

## Real Oil (cena do FPS, lado 2880 = os píxeis de 4K, Radeon integrada)

`PH2D_WETPAINT_SMOKE=3 PH2D_FPS_OLEO=1 PH2D_FPS_TELA=2880` — 6 rodadas de Dabber (Real Oil) · seca · Wet Drip · corre;
cru em [`antes/oleo_2880_fps.txt`](antes/oleo_2880_fps.txt). O lado 1440 não coube no prazo.

| rodada | dabber período p50/p95/máx ms | corre período p50/p95/máx ms | **corre imagens/s** | óptica ms por imagem |
|---|---|---|---|---|
| 0 | 14,4 / 16,2 / 145,2 | 10,0 / 20,9 / 29,3 | 11,9 | 61 |
| 1 | 13,7 / 15,5 / 64,6 | 11,9 / **31,1 ✗** / 83,9 | **3,2 ✗** | 241 |
| 2 | **17,0 ✗** / 24,3 / 74,1 | 11,7 / **29,9 ✗** / 108,5 | **1,6 ✗** | 463 |
| 3 | **16,9 ✗** / 30,2 / 86,2 | 12,0 / 19,0 / 134,2 | **1,0 ✗** | 725 |
| 4 | 15,9 / 19,4 / 111,5 | 11,9 / 18,4 / 139,2 | **0,8 ✗** | 851 |
| 5 | 16,1 / 20,1 / 87,7 | 12,4 / 17,4 / 174,4 | **0,6 ✗** | 1 112 |

## Leitura (o que a base diz)

1. **Um quadro de ~100 ms fixo a 4K, igual em TODOS os pincéis**, em três instantes: o 1.º movimento (toque), ~0,45–0,5 s
   depois do Up (`soltar_1s_em_ms`) e ~0,45–0,5 s depois do Ctrl+Z (`ctrlz_em_ms`). Escala com a tela: ~35 ms a 1080p,
   ausente a 1024² blocos (o papel e o tamanho mudam juntos nesta matriz — falta 4K blocos/branco para separar). O
   despacho do pintor nesse quadro é ≤ 2,6 ms (`[paint-perf] WORST split`): o custo é FORA do pincel. O atraso de ~0,5 s
   casa com a hipótese da §6.5 do `COMO_ACELERAR_UM_PINCEL.md` (`QUADROS_PARADOS = 30` do `follow_live`, ~30–40 quadros
   a ~11 ms) — por confirmar. O Wet omissão e o Wet Liner não pagam o do Up (28,6 / 31,0 ms); Express Oil e Filbert
   pagam; todos pagam o do Ctrl+Z.
2. **Water (omissão e Watercolor Basic) paga ~155 ms no toque a 4K** (52–56 ms a 1080p, 34 ms a 1024²): o pior toque.
3. **Traço a 4K: p95 21–26 ms em todos** (✗ contra 16,6), **Filbert 37,2 ms** (34,0 p50) e Express Oil 28,8 ms; a 1080p
   passam quase todos (Filbert 23,8 ✗). O quadro base parado é ~11 ms: o traço a 4K custa ~13 ms por quadro a mais.
4. **Parado nunca é ~0 %: 48,6–70,5 % de um núcleo** em todos os 90 períodos. O app redesenha SEM entrada (~90 quadros/s
   com o WSI de software; com Fifo seriam 60/s) — o `[frame]` parado mostra `cpu-encode ~5–6 ms` + `fora-do-encode ~6 ms`.
5. **Wet: a água corre a 36–40 img/s** em todos os pincéis Wet da matriz (passa). **Com Real Oil desaba rodada a rodada:
   11,9 → 0,6 img/s, a óptica de 61 → 1 112 ms por imagem** — cresce com a tinta acumulada.

