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
SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T oleo 1440 $B     # corrido no B08b (abaixo)
python3 $W/docs/Painter/ferramentas/mede_pen_up_4k.py --matriz $S/matriz <pasta-de-saída>
# B08b (o complemento): fundo branco e blocos a WxH (PH2D_COMPOSITE_FUNDO, B08.2)
PNG=1 SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T matriz 3840x2160 branco $B
PNG=1 SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T matriz 3840x2160 blocos $B
SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T matriz 1920x1080 branco $B "omissão:digital" "omissão:water" \
  "omissão:impasto" "omissão:wet" "Watercolor Basic:water" "Filbert:wet" "Express Oil:wet"
# B08b continuação (o roteiro já traz o CONTROLO e trocar de camada; as coordenadas de omissão são as da janela 1600x1000)
PNG=1 SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T matriz 3840x2160 papel $B "omissão:digital" "omissão:water" \
  "omissão:impasto" "omissão:wet"
SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T matriz 1920x1080 branco $B "Round Mixing:digital" "Wet Wash Mix:digital" \
  "Wet Liner:wet" "Pastel:digital" "Chalk:digital" "Crayon:digital" "Finger Tip:digital" "Composite Brush:digital"
PNG=1 SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T matriz 1920x1080 blocos $B
PNG=1 SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T matriz 3840x2160 blocos $B
# B08b continuação 2: o roteiro traz também o GRUPO (todos), o slider de luz (só no painel impasto: LUZ_ROLA cliques
# da roda + arrastar o Angle) e abrir doc WxH (ABRIR = a linha «Composite 2» da Hierarquia; PH2D_COMPOSITE_SEGUNDA)
SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T matriz 1920x1080 branco $B "Round Mixing:digital" "Wet Wash Mix:digital" \
  "Composite Brush:digital"
for tf in "3840x2160 papel" "3840x2160 blocos" "1920x1080 blocos"; do read -r TELA FUNDO <<< "$tf"   # bash: o zsh não parte
  PNG=1 LUZ="1345 753 1435" LUZ_ROLA=30 ABRIR="117 301" SAIDA=$S bash $W/scripts/ph2d-run.sh bash $T matriz $TELA $FUNDO $B \
    "omissão:impasto" "omissão:digital"; done
```

⚠️ O «depois» só compara com o B08b num binário que TENHA o `PH2D_COMPOSITE_FUNDO` (o `cd02ac77ef` puro não o tem:
o B08b correu num binário de árvore limpa = `git archive cd02ac77ef` + só o `composite_smoke.rs` deste item).

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

- ~~**Fundo branco sem papel**~~ e ~~**blocos de cor só a 1024²**~~: feitos no B08b (`PH2D_COMPOSITE_FUNDO=branco|blocos|papel`,
  todos a `PH2D_COMPOSITE_TELA`) — secção «B08b» abaixo.
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


## B08b — o complemento do «antes»: fundo branco, blocos a 4K, Real Oil 1440

- **Binário:** árvore LIMPA = `git archive cd02ac77ef` + só o `composite_smoke.rs` do B08b (`PH2D_COMPOSITE_FUNDO`,
  blocos a `PH2D_COMPOSITE_TELA`), `CARGO_TARGET_DIR` próprio (`target/b08b-limpo`, deps por reflink do `target/smoke`),
  `--profile smoke`, `STRIP=none` (08:15, 1 min 41 s). A árvore da linha tinha B10.3/B23 a meio: não entram.
- **Placa:** a mesma Radeon 610M (RADV, WSI sw); cada `.placa` = `libvulkan_radeon.so libgallium` (nenhuma `nvidia`).
- **Carga (⚠️ outros agentes a compilar; acima de `load ~5` o relógio vale pouco — ler como TECTO, não como base):**
  4K branco 08:18–08:25, load 1 min por lançamento mín 3,0 · mediana 5,6 · máx 9,0 · 4K blocos 08:25–08:32, 6,8 · 11,1 · 16,2 · Real Oil 1440 08:32–08:33, 14–18 · 1080p branco (7 pincéis, o prazo) 08:33–08:37, 2,5 · 5,5 · 17,7. A carga de cada lançamento está no `.meta` (campo `carga` do jsonl).
- Cru: [`antes/b08b/`](antes/b08b/) (`matriz.jsonl`, `quadros/`, `oleo_1440_fps.txt`, `carga.txt`, `foto_4k_branco.png`, `foto_4k_blocos.png`).
- ~~**Trocar de camada: continua fora**~~ — feito na continuação (abaixo): cliques XTEST no cartão Layers, foto por passo.

| pincel | painel | tela | fundo | base ms | toque ms | traço p50/p95 ms | soltar ms | +1 s máx ms | Ctrl+Z ms | parado CPU % (antes/depois) | Wet img/s |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Chalk | digital | 3840x2160 | blocos | 15.8 | **38.6 ✗** | 13.9 / 15.9 | 16.2 | 19.9 | 15.8 | **66.6 ✗** / **52.9 ✗** | — |
| Chalk | digital | 3840x2160 | branco | 11.0 | **56.0 ✗** | 12.8 / 13.8 | 12.1 | **52.0 ✗** | **52.2 ✗** | **54.3 ✗** / **58.0 ✗** | — |
| Composite Brush | digital | 3840x2160 | blocos | 16.1 | **60.1 ✗** | 15.0 / **20.7 ✗** | 29.5 | **76.3 ✗** | **58.4 ✗** | **69.6 ✗** / **49.6 ✗** | — |
| Composite Brush | digital | 3840x2160 | branco | 10.0 | **49.9 ✗** | 11.0 / 14.7 | 14.8 | **53.9 ✗** | **55.0 ✗** | **51.6 ✗** / **53.0 ✗** | — |
| Crayon | digital | 3840x2160 | blocos | 12.1 | 29.1 | 14.8 / **25.2 ✗** | **71.5 ✗** | **75.3 ✗** | 15.0 | **60.9 ✗** / **61.0 ✗** | — |
| Crayon | digital | 3840x2160 | branco | 11.0 | **51.0 ✗** | 11.3 / 12.4 | 11.3 | **49.1 ✗** | **54.0 ✗** | **52.9 ✗** / **58.6 ✗** | — |
| Express Oil | wet | 1920x1080 | branco | 11.0 | **35.0 ✗** | 13.4 / 15.1 | 16.1 | 18.1 | **41.8 ✗** | **53.6 ✗** / **53.3 ✗** | 39.9 |
| Express Oil | wet | 3840x2160 | blocos | 15.8 | **51.0 ✗** | 21.1 / **32.1 ✗** | **34.0 ✗** | **34.0 ✗** | **186.9 ✗** | **69.3 ✗** / **52.5 ✗** | 37.6 |
| Express Oil | wet | 3840x2160 | branco | 12.9 | **105.0 ✗** | 14.9 / **16.9 ✗** | 17.2 | **52.9 ✗** | **103.9 ✗** | **52.6 ✗** / **65.6 ✗** | 38.1 |
| Filbert | wet | 1920x1080 | branco | 11.0 | **38.0 ✗** | 20.9 / **24.1 ✗** | 23.0 | 23.0 | **43.1 ✗** | **52.6 ✗** / **53.3 ✗** | 39.3 |
| Filbert | wet | 3840x2160 | blocos | 17.2 | **53.9 ✗** | 22.1 / **25.0 ✗** | 20.2 | 22.8 | **70.1 ✗** | **44.3 ✗** / **62.9 ✗** | 40.0 |
| Filbert | wet | 3840x2160 | branco | 12.0 | **217.2 ✗** | 65.1 / **168.5 ✗** | **87.0 ✗** | **87.0 ✗** | **107.2 ✗** | **58.6 ✗** / **57.0 ✗** | 38.2 |
| Finger Tip | digital | 3840x2160 | blocos | 15.0 | **41.8 ✗** | 19.9 / **43.7 ✗** | **48.8 ✗** | **51.9 ✗** | 16.1 | **58.6 ✗** / **65.3 ✗** | — |
| Finger Tip | digital | 3840x2160 | branco | 14.1 | **85.2 ✗** | 16.0 / **20.6 ✗** | 16.0 | **60.0 ✗** | **53.9 ✗** | **67.9 ✗** / **55.6 ✗** | — |
| Pastel | digital | 3840x2160 | blocos | 23.1 | **112.5 ✗** | 18.1 / **38.1 ✗** | 32.2 | **42.9 ✗** | 32.0 | **56.6 ✗** / **62.6 ✗** | — |
| Pastel | digital | 3840x2160 | branco | 11.9 | **53.0 ✗** | 12.3 / 13.1 | 12.3 | **54.2 ✗** | **53.0 ✗** | **58.6 ✗** / **56.9 ✗** | — |
| Round Mixing | digital | 3840x2160 | blocos | 18.8 | **59.2 ✗** | 21.7 / **62.7 ✗** | **51.2 ✗** | **62.1 ✗** | **56.0 ✗** | **49.6 ✗** / **40.9 ✗** | — |
| Round Mixing | digital | 3840x2160 | branco | 11.1 | **50.0 ✗** | 13.1 / 14.9 | 13.2 | **51.1 ✗** | **53.7 ✗** | **56.3 ✗** / **58.9 ✗** | — |
| Watercolor Basic | water | 1920x1080 | branco | 10.8 | 27.1 | 14.1 / 15.0 | 22.0 | 22.0 | 25.1 | **52.3 ✗** / **51.6 ✗** | — |
| Watercolor Basic | water | 3840x2160 | blocos | 14.9 | **62.9 ✗** | 26.2 / **45.3 ✗** | **55.1 ✗** | **55.1 ✗** | **50.3 ✗** | **65.2 ✗** / **61.3 ✗** | — |
| Watercolor Basic | water | 3840x2160 | branco | 11.1 | **41.8 ✗** | 20.8 / **22.0 ✗** | 27.8 | **54.0 ✗** | **77.9 ✗** | **57.6 ✗** / **53.6 ✗** | — |
| Wet Liner | wet | 3840x2160 | blocos | 12.1 | **44.9 ✗** | 11.9 / 13.4 | 14.0 | 19.9 | **67.9 ✗** | **60.3 ✗** / **57.3 ✗** | 39.4 |
| Wet Liner | wet | 3840x2160 | branco | 11.8 | **106.8 ✗** | 13.0 / 13.9 | 14.9 | 21.1 | **106.9 ✗** | **57.3 ✗** / **60.0 ✗** | 38.2 |
| Wet Wash Mix | digital | 3840x2160 | blocos | 15.9 | 32.9 | 19.1 / **35.8 ✗** | 17.1 | 28.1 | 22.8 | **59.2 ✗** / **61.6 ✗** | — |
| Wet Wash Mix | digital | 3840x2160 | branco | 11.0 | **50.0 ✗** | 12.4 / 13.6 | 13.0 | **53.2 ✗** | **50.8 ✗** | **53.6 ✗** / **53.3 ✗** | — |
| omissão | digital | 1920x1080 | branco | 10.2 | 20.0 | 9.9 / 11.8 | 11.3 | 20.1 | 17.8 | **53.3 ✗** / **52.3 ✗** | — |
| omissão | digital | 3840x2160 | blocos | 13.0 | **47.4 ✗** | 15.9 / **37.1 ✗** | 27.1 | **47.3 ✗** | **77.0 ✗** | **60.3 ✗** / **43.3 ✗** | — |
| omissão | digital | 3840x2160 | branco | 14.1 | **60.1 ✗** | 13.8 / **19.6 ✗** | 17.1 | **66.4 ✗** | **55.9 ✗** | **67.3 ✗** / **59.3 ✗** | — |
| omissão | impasto | 1920x1080 | branco | 10.0 | 26.6 | 11.0 / 11.8 | 13.2 | 27.0 | 23.3 | **52.9 ✗** / **49.6 ✗** | — |
| omissão | impasto | 3840x2160 | blocos | 15.3 | **162.1 ✗** | 22.9 / **66.8 ✗** | **46.1 ✗** | **91.6 ✗** | **83.0 ✗** | **42.9 ✗** / **33.4 ✗** | — |
| omissão | impasto | 3840x2160 | branco | 14.9 | **101.6 ✗** | 20.1 / **56.5 ✗** | **46.8 ✗** | **134.5 ✗** | **135.3 ✗** | **68.9 ✗** / **58.6 ✗** | — |
| omissão | water | 1920x1080 | branco | 10.7 | 26.7 | 13.4 / 14.0 | 20.5 | 21.1 | 24.9 | **53.0 ✗** / **51.3 ✗** | — |
| omissão | water | 3840x2160 | blocos | 12.9 | **44.0 ✗** | 26.0 / **121.3 ✗** | 32.0 | **71.9 ✗** | **50.9 ✗** | **61.9 ✗** / **68.9 ✗** | — |
| omissão | water | 3840x2160 | branco | 10.7 | **39.9 ✗** | 19.5 / **20.2 ✗** | 27.8 | **53.1 ✗** | **72.6 ✗** | **53.3 ✗** / **51.6 ✗** | — |
| omissão | wet | 1920x1080 | branco | 12.0 | **47.0 ✗** | 17.0 / **25.0 ✗** | 26.7 | 27.0 | **56.0 ✗** | **59.9 ✗** / **65.6 ✗** | 40.3 |
| omissão | wet | 3840x2160 | blocos | 17.2 | **89.1 ✗** | 24.3 / **70.9 ✗** | **67.4 ✗** | **67.8 ✗** | **193.5 ✗** | **40.6 ✗** / **43.3 ✗** | 34.9 |
| omissão | wet | 3840x2160 | branco | 13.1 | **97.9 ✗** | 17.9 / **30.1 ✗** | 31.1 | 32.8 | **198.2 ✗** | **65.3 ✗** / **68.9 ✗** | 25.5 |

### Real Oil a 1440 (= os píxeis de 1080p; cena do FPS, `PH2D_FPS_TELA=1440`, load 1 min 14–18 ⚠️)

| rodada | dabber período p50/p95/máx ms | corre período p50/p95/máx ms | **corre imagens/s** | óptica ms por imagem |
|---|---|---|---|---|
| 0 | **16,8 ✗** / 40,3 / 98,1 | **17,0 ✗** / 65,2 / 390,2 | **7,2 ✗** | 95 |
| 1 | 15,8 / 28,0 / 45,0 | 13,0 / 61,2 / 82,1 | **4,6 ✗** | 158 |
| 2 | **23,0 ✗** / 56,1 / 166,3 | 15,9 / 71,3 / 184,0 | **2,2 ✗** | 311 |
| 3 | **25,8 ✗** / 75,2 / 120,4 | 13,8 / 59,2 / 196,3 | **1,4 ✗** | 455 |
| 4 | **22,5 ✗** / 61,5 / 128,2 | 14,5 / 59,5 / 248,1 | **1,2 ✗** | 570 |
| 5 | **26,4 ✗** / 67,5 / 172,2 | 14,1 / 59,2 / 279,2 | **1,0 ✗** | 642 |

O mesmo desabamento da 2880, mais lento: 7,2 → 1,0 img/s, a óptica 95 → 642 ms por imagem (2880: 11,9 → 0,6 img/s,
61 → 1 112 ms). A 1440 a óptica da rodada 0 já é MAIOR que a da 2880 (95 × 61 ms): a carga de 14–18 durante esta corrida.

### Leitura (o que o complemento separa)

1. **O quadro fixo de ~100 ms a 4K é metade TELA, metade PAPEL.** Sem papel (4K branco) o toque, o +1 s e o Ctrl+Z dos
   Digital caem para **~50–60 ms** (com papel 97–103); a 1080p branco **17–27 ms** (omissão Digital e Impasto; com papel 31–37). O papel (o passe de
   luz do relevo) põe ~50 ms a 4K; os outros ~50 ms escalam com a tela sem papel nenhum (folha ao vivo, B10.3;
   recomposição inteira, B10.5).
2. **Water: o toque de ~155 ms a 4K é o PAPEL** — 40–42 ms sem ele (1080p: 52–56 → 27).
3. **Traço a 4K: sem papel os Digital passam** (p95 12–15 ms; Finger Tip 20,6, omissão 19,6 ✗); com papel 24–26. O custo
   do traço a 4K é o papel. O Water fica em 20–22 ms (✗) com ou sem papel: é dele.
4. **O Ctrl+Z do Wet NÃO é o papel:** 104–107 ms (Express Oil, Wet Liner) e **198 ms** o Wet omissão a 4K branco
   (1080p branco 42–56): é o Wet (B15.6). **Filbert a 4K branco: toque 217, traço p95 168,5** (com papel 37,2), load 5,1.
5. **Impasto a 4K sem papel é PIOR que com papel:** +1 s 134,5 · Ctrl+Z 135,3 · traço p95 56,5 (papel: 109,4 · 99,1 ·
   24,5; load 6,0) — o «outro custo» que o B10.3 deixou por perfilar não é a folha nem o papel.
6. **4K blocos: load mediana 11 (até 16) — leia como TECTO.** A base parada sobe a 12–23 ms (contra 10–11): o relógio
   está sujo. Mesmo assim o Wet paga o Ctrl+Z (187–194 ms) e o Impasto o toque (162 ms).
7. **Parado: 33–69 % de um núcleo em TODOS os fundos e tamanhos** — não depende do papel nem da tela (B23).
8. **Real Oil a 1440 desaba como a 2880:** 7,2 → 1,0 img/s (óptica 95 → 642 ms por imagem).

## B08b continuação — controlo da régua, trocar de camada, 1080p blocos, 1080p branco completo, 4K blocos repetido

- **Binário:** o mesmo de árvore LIMPA, reconstruído (o do B08b fora apagado): `git archive cd02ac77ef` + só o
  `composite_smoke.rs` de `cf80f3574d`, `CARGO_TARGET_DIR=target/b08b-limpo`, `--profile smoke`, `STRIP=none` (08:42,
  2 min 26 s). Placa: a Radeon 610M (RADV, WSI sw); nenhuma `.placa` com `nvidia`.
- **Roteiro novo** (no fim de cada lançamento, depois do parado 3 s):
  - **CONTROLO da régua:** o app parado 250 ms por `SIGSTOP`/`SIGCONT` (o processo é do roteiro). O maior intervalo
    de quadro do segundo seguinte TEM de ler 250 a 250 + 2 quadros (283 ms); se não, a régua não vê uma trava e a linha
    vale zero (`mede_pen_up_4k.py` escreve `✗ régua`).
  - **Trocar de camada** (cartão BRUSH, janela 1600×1000 em 0,0; coordenadas por `CAMADA_*`): aba `Layers` (1510,159) ·
    `+` New layer (1332,233; a nova fica ativa) · a de baixo, `Layer 1` (1430,397) · a de cima, `Layer 2` (1430,306) ·
    Duplicate layer (1389,233) · Delete layer (1417,233). Cada coluna = o maior intervalo de quadro no segundo depois do
    clique. Foto por passo conferida (as de `omissão` Digital 4K papel e 1080p blocos em `antes/b08b/continuacao/`):
    o `+` cria `Layer 2`, o clique ativa a camada certa, Duplicate cria `Layer 2 copy`, Delete apaga-a.
    (Duplicar/apagar entraram a meio do bloco: o grupo 4K papel não os tem.)
- **Carga** (`carga.txt`; a de cada lançamento no jsonl): 4K papel 08:44–08:47, load 1 min 8,5–47 ⚠️ · 1080p branco
  08:47–08:52, 9–50 ⚠️ · 1080p blocos 08:52–09:02, mediana 1,6 (0,8–9,5) · 4K blocos 09:02–09:12, mediana 1,0 (0,7–1,8) — a base LIMPA.
  Acima de `load ~5` leia como TECTO.
- Cru: [`antes/b08b/continuacao/`](antes/b08b/continuacao/) (`matriz.jsonl`, `quadros/`, `carga.txt`, fotos dos passos).

| pincel | painel | tela | fundo | base ms | toque ms | traço p50/p95 ms | soltar ms | +1 s máx ms | Ctrl+Z ms | parado CPU % (antes/depois) | Wet img/s |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Chalk | digital | 1920x1080 | blocos | 11.0 | 17.2 | 11.3 / 11.9 | 11.5 | 12.2 | 12.0 | **51.6 ✗** / **51.3 ✗** | — |
| Chalk | digital | 1920x1080 | branco | 11.2 | 21.9 | 11.4 / 12.1 | 12.7 | 21.1 | 18.9 | **56.6 ✗** / **53.3 ✗** | — |
| Chalk | digital | 3840x2160 | blocos | 11.0 | 29.0 | 11.3 / 12.8 | 11.9 | 17.0 | 13.9 | **51.6 ✗** / **51.6 ✗** | — |
| Composite Brush | digital | 1920x1080 | blocos | 10.1 | 23.9 | 11.3 / 14.7 | 14.1 | 14.1 | 13.0 | **50.6 ✗** / **50.3 ✗** | — |
| Composite Brush | digital | 1920x1080 | branco | 13.9 | **33.9 ✗** | 18.2 / **34.8 ✗** | 26.9 | 28.9 | 30.1 | **58.6 ✗** / **58.0 ✗** | — |
| Composite Brush | digital | 3840x2160 | blocos | 10.0 | **41.8 ✗** | 11.2 / 14.7 | 15.0 | 15.0 | 17.9 | **50.3 ✗** / **50.3 ✗** | — |
| Crayon | digital | 1920x1080 | blocos | 11.0 | 18.0 | 11.2 / 11.7 | 11.7 | 12.2 | 12.0 | **51.3 ✗** / **51.3 ✗** | — |
| Crayon | digital | 1920x1080 | branco | 12.0 | 23.0 | 13.0 / 15.2 | 12.9 | 23.0 | 22.0 | **61.6 ✗** / **60.0 ✗** | — |
| Crayon | digital | 3840x2160 | blocos | 11.0 | 29.0 | 10.9 / 11.8 | 12.4 | 13.9 | 17.9 | **52.6 ✗** / **51.0 ✗** | — |
| Express Oil | wet | 1920x1080 | blocos | 10.9 | 22.9 | 13.1 / 14.8 | 16.0 | 16.0 | **40.8 ✗** | **50.6 ✗** / **50.6 ✗** | 39.7 |
| Express Oil | wet | 3840x2160 | blocos | 11.0 | **46.0 ✗** | 13.4 / 14.9 | 15.7 | 20.8 | **69.8 ✗** | **51.3 ✗** / **51.3 ✗** | 39.3 |
| Filbert | wet | 1920x1080 | blocos | 11.0 | 27.0 | 20.0 / **23.1 ✗** | 23.1 | 23.1 | **40.9 ✗** | **52.3 ✗** / **51.6 ✗** | 40.1 |
| Filbert | wet | 3840x2160 | blocos | 11.0 | **53.0 ✗** | 20.8 / **23.2 ✗** | 23.2 | 23.2 | **70.2 ✗** | **54.9 ✗** / **53.6 ✗** | 39.9 |
| Finger Tip | digital | 1920x1080 | blocos | 11.1 | 13.1 | 10.0 / 11.1 | 10.8 | 11.9 | 12.1 | **58.6 ✗** / **51.3 ✗** | — |
| Finger Tip | digital | 1920x1080 | branco | 10.0 | 22.0 | 11.9 / 13.1 | 12.1 | 21.0 | 20.0 | **51.3 ✗** / **58.3 ✗** | — |
| Finger Tip | digital | 3840x2160 | blocos | 10.0 | 18.7 | 10.2 / 10.7 | 10.8 | 16.9 | 12.4 | **49.6 ✗** / **52.3 ✗** | — |
| Pastel | digital | 1920x1080 | blocos | 11.0 | 18.1 | 11.2 / 12.0 | 11.9 | 12.1 | 12.0 | **50.9 ✗** / **51.3 ✗** | — |
| Pastel | digital | 1920x1080 | branco | 15.1 | 21.9 | 12.3 / **19.8 ✗** | 13.1 | 23.0 | 22.0 | **67.6 ✗** / **58.0 ✗** | — |
| Pastel | digital | 3840x2160 | blocos | 10.9 | 29.1 | 11.0 / 12.1 | 11.5 | 13.2 | 17.8 | **50.6 ✗** / **50.3 ✗** | — |
| Round Mixing | digital | 1920x1080 | blocos | 11.0 | 14.9 | 10.4 / 11.2 | 10.8 | 12.1 | 12.2 | **53.6 ✗** / **54.9 ✗** | — |
| Round Mixing | digital | 1920x1080 | branco | 18.6 | **36.3 ✗** | 18.8 / **84.1 ✗** | **177.4 ✗** | **177.4 ✗** | 31.4 | **59.8 ✗** / **59.5 ✗** | — |
| Round Mixing | digital | 3840x2160 | blocos | 11.0 | 20.9 | 10.3 / 10.9 | 10.9 | 17.0 | 14.1 | **54.9 ✗** / **54.6 ✗** | — |
| Watercolor Basic | water | 1920x1080 | blocos | 10.4 | 25.8 | 13.9 / 14.3 | 20.5 | 20.5 | 17.8 | **51.0 ✗** / **50.3 ✗** | — |
| Watercolor Basic | water | 3840x2160 | blocos | 10.2 | **41.2 ✗** | 19.8 / **21.0 ✗** | 26.1 | 26.1 | **38.0 ✗** | **51.0 ✗** / **50.6 ✗** | — |
| Wet Liner | wet | 1920x1080 | blocos | 11.0 | 23.1 | 11.0 / 12.1 | 14.9 | 14.9 | **40.0 ✗** | **51.0 ✗** / **50.3 ✗** | 39.9 |
| Wet Liner | wet | 1920x1080 | branco | 12.0 | **35.0 ✗** | 13.0 / 14.1 | 15.2 | 21.2 | **78.2 ✗** | **59.3 ✗** / **59.3 ✗** | 39.4 |
| Wet Liner | wet | 3840x2160 | blocos | 10.9 | **47.1 ✗** | 11.8 / 12.5 | 13.2 | 19.9 | **63.7 ✗** | **50.6 ✗** / **50.3 ✗** | 39.4 |
| Wet Wash Mix | digital | 1920x1080 | blocos | 11.0 | 15.0 | 12.1 / 12.8 | 12.9 | 12.9 | 12.1 | **51.3 ✗** / **50.6 ✗** | — |
| Wet Wash Mix | digital | 1920x1080 | branco | 22.1 | **85.9 ✗** | 36.1 / **48.1 ✗** | **39.0 ✗** | **55.3 ✗** | 24.9 | **59.1 ✗** / **66.0 ✗** | — |
| Wet Wash Mix | digital | 3840x2160 | blocos | 11.0 | 20.8 | 12.0 / 12.9 | 12.2 | 13.0 | 14.0 | **50.9 ✗** / **50.9 ✗** | — |
| omissão | digital | 1920x1080 | blocos | 12.0 | 19.1 | 13.2 / 14.9 | 13.9 | 15.3 | 18.0 | **57.6 ✗** / **58.3 ✗** | — |
| omissão | digital | 3840x2160 | blocos | 11.0 | 18.2 | 10.1 / 10.4 | 10.8 | 17.7 | 14.9 | **54.6 ✗** / **50.3 ✗** | — |
| omissão | digital | 3840x2160 | papel | 10.9 | **98.7 ✗** | 23.1 / **24.1 ✗** | 23.5 | **100.1 ✗** | **109.4 ✗** | **52.6 ✗** / **51.9 ✗** | — |
| omissão | impasto | 1920x1080 | blocos | 11.0 | 24.9 | 11.6 / 12.0 | 15.7 | 19.9 | 19.8 | **56.9 ✗** / **63.6 ✗** | — |
| omissão | impasto | 3840x2160 | blocos | 11.1 | **62.4 ✗** | 18.0 / **18.5 ✗** | 24.3 | **43.0 ✗** | **33.5 ✗** | **56.6 ✗** / **49.3 ✗** | — |
| omissão | impasto | 3840x2160 | papel | 10.8 | **101.3 ✗** | 23.0 / **23.9 ✗** | 30.1 | **103.9 ✗** | **99.1 ✗** | **51.0 ✗** / **59.6 ✗** | — |
| omissão | water | 1920x1080 | blocos | 11.8 | 26.9 | 14.1 / 14.9 | 21.2 | 21.2 | 20.0 | **56.9 ✗** / **59.3 ✗** | — |
| omissão | water | 3840x2160 | blocos | 10.8 | **41.9 ✗** | 20.2 / **21.9 ✗** | 28.0 | 28.0 | **38.4 ✗** | **54.3 ✗** / **57.0 ✗** | — |
| omissão | water | 3840x2160 | papel | 15.0 | **321.4 ✗** | 28.1 / **56.7 ✗** | **47.2 ✗** | **197.6 ✗** | **193.4 ✗** | **55.9 ✗** / **51.3 ✗** | — |
| omissão | wet | 1920x1080 | blocos | 12.0 | 19.0 | 14.0 / 15.0 | 15.7 | 15.7 | **37.9 ✗** | **57.6 ✗** / **65.9 ✗** | 39.6 |
| omissão | wet | 3840x2160 | blocos | 10.9 | 26.3 | 10.9 / 12.3 | 17.9 | 17.9 | **64.1 ✗** | **50.6 ✗** / **50.6 ✗** | 38.9 |
| omissão | wet | 3840x2160 | papel | 19.1 | **55.4 ✗** | 36.2 / **54.8 ✗** | **38.2 ✗** | **46.3 ✗** | **352.0 ✗** | **67.7 ✗** / **84.7 ✗** | 25.9 |

| pincel | painel | tela | fundo | controlo sem/com trava de 250 ms | aba Layers ms | + camada ms | trocar ms | voltar ms | duplicar ms | apagar ms |
|---|---|---|---|---|---|---|---|---|---|---|
| Chalk | digital | 1920x1080 | blocos | **11.2 ✗ régua** / 262.5 | 11.0 | 18.7 | 17.1 | 17.8 | 16.1 | 15.9 |
| Chalk | digital | 1920x1080 | branco | **13.0 ✗ régua** / 258.2 | 11.1 | 18.0 | 19.8 | 20.9 | 21.2 | 19.0 |
| Chalk | digital | 3840x2160 | blocos | **11.1 ✗ régua** / 265.4 | 10.6 | **44.0 ✗** | **41.9 ✗** | **41.2 ✗** | **49.9 ✗** | **42.9 ✗** |
| Composite Brush | digital | 1920x1080 | blocos | **10.9 ✗ régua** / 257.4 | 10.1 | 18.8 | 17.0 | 16.9 | 19.9 | 15.0 |
| Composite Brush | digital | 1920x1080 | branco | **13.0 ✗ régua** / 257.1 | 10.8 | 17.1 | 20.0 | 19.2 | 19.9 | 17.9 |
| Composite Brush | digital | 3840x2160 | blocos | **12.0 ✗ régua** / 265.1 | 10.9 | **45.2 ✗** | **47.0 ✗** | **46.0 ✗** | **53.1 ✗** | **46.0 ✗** |
| Crayon | digital | 1920x1080 | blocos | **13.0 ✗ régua** / 263.3 | 11.2 | 18.0 | 17.8 | 17.1 | 17.8 | 16.0 |
| Crayon | digital | 1920x1080 | branco | **13.7 ✗ régua** / 263.1 | 23.1 | 21.0 | 24.5 | 20.8 | 20.8 | 28.6 |
| Crayon | digital | 3840x2160 | blocos | **11.9 ✗ régua** / 259.3 | 12.0 | **43.9 ✗** | **47.0 ✗** | **43.9 ✗** | **50.0 ✗** | **44.2 ✗** |
| Express Oil | wet | 1920x1080 | blocos | **11.2 ✗ régua** / 259.4 | 11.1 | **37.7 ✗** | 18.4 | 23.5 | 17.8 | 15.9 |
| Express Oil | wet | 3840x2160 | blocos | **11.9 ✗ régua** / 265.5 | 12.0 | **85.8 ✗** | **44.8 ✗** | **50.1 ✗** | **47.9 ✗** | **43.2 ✗** |
| Filbert | wet | 1920x1080 | blocos | **11.9 ✗ régua** / 259.3 | 12.1 | **35.3 ✗** | 16.3 | 23.3 | 16.9 | 16.0 |
| Filbert | wet | 3840x2160 | blocos | **12.9 ✗ régua** / 261.4 | 12.7 | **93.7 ✗** | **43.8 ✗** | **52.3 ✗** | **48.0 ✗** | **46.0 ✗** |
| Finger Tip | digital | 1920x1080 | blocos | **12.1 ✗ régua** / 259.7 | 11.0 | 18.0 | 17.2 | 17.9 | 16.1 | 16.8 |
| Finger Tip | digital | 1920x1080 | branco | **49.8 ✗ régua** / 266.3 | 14.0 | 26.8 | 26.0 | **154.0 ✗** | **40.0 ✗** | 23.1 |
| Finger Tip | digital | 3840x2160 | blocos | **11.9 ✗ régua** / 257.4 | 11.0 | **45.9 ✗** | **44.0 ✗** | **43.9 ✗** | **46.8 ✗** | **43.8 ✗** |
| Pastel | digital | 1920x1080 | blocos | **11.1 ✗ régua** / 264.4 | 11.0 | 18.0 | 17.0 | 17.1 | 16.9 | 15.8 |
| Pastel | digital | 1920x1080 | branco | **13.0 ✗ régua** / 263.4 | 13.1 | 19.4 | 21.0 | 20.0 | 23.1 | 20.2 |
| Pastel | digital | 3840x2160 | blocos | **12.0 ✗ régua** / 265.4 | 10.7 | **44.0 ✗** | **43.0 ✗** | **43.9 ✗** | **51.9 ✗** | **44.0 ✗** |
| Round Mixing | digital | 1920x1080 | blocos | **11.7 ✗ régua** / 265.5 | 10.0 | 18.9 | 18.0 | 17.1 | 17.9 | 15.9 |
| Round Mixing | digital | 1920x1080 | branco | **32.0 ✗ régua** / 272.1 | 24.8 | **36.8 ✗** | **57.6 ✗** | **63.9 ✗** | **48.5 ✗** | **37.9 ✗** |
| Round Mixing | digital | 3840x2160 | blocos | **11.1 ✗ régua** / 264.3 | 10.8 | **44.8 ✗** | **42.9 ✗** | **43.0 ✗** | **49.2 ✗** | **45.9 ✗** |
| Watercolor Basic | water | 1920x1080 | blocos | **11.1 ✗ régua** / 260.5 | 10.9 | 18.9 | 18.0 | 15.0 | 17.0 | 16.0 |
| Watercolor Basic | water | 3840x2160 | blocos | **11.0 ✗ régua** / 263.2 | 11.2 | **43.1 ✗** | **45.9 ✗** | **44.0 ✗** | **47.9 ✗** | **44.9 ✗** |
| Wet Liner | wet | 1920x1080 | blocos | **11.1 ✗ régua** / 258.9 | 10.8 | 32.1 | 17.9 | 22.6 | 21.8 | 18.0 |
| Wet Liner | wet | 1920x1080 | branco | **14.9 ✗ régua** / 261.9 | 11.2 | 32.0 | 22.1 | 23.9 | 23.1 | 19.9 |
| Wet Liner | wet | 3840x2160 | blocos | **11.1 ✗ régua** / 265.4 | 11.0 | **83.6 ✗** | **44.0 ✗** | **56.4 ✗** | **46.9 ✗** | **42.8 ✗** |
| Wet Wash Mix | digital | 1920x1080 | blocos | **11.1 ✗ régua** / 260.5 | 12.0 | 18.7 | 18.1 | 17.9 | 17.0 | 14.9 |
| Wet Wash Mix | digital | 1920x1080 | branco | **15.0 ✗ régua** / 259.5 | 14.1 | **42.8 ✗** | 19.9 | 18.1 | 22.9 | **36.5 ✗** |
| Wet Wash Mix | digital | 3840x2160 | blocos | **11.2 ✗ régua** / 264.5 | 11.0 | **42.2 ✗** | **43.9 ✗** | **42.9 ✗** | **47.1 ✗** | **42.1 ✗** |
| omissão | digital | 1920x1080 | blocos | **33.8 ✗ régua** / 264.6 | 32.0 | **35.9 ✗** | **43.1 ✗** | 23.9 | 30.8 | 31.3 |
| omissão | digital | 3840x2160 | blocos | **10.1 ✗ régua** / 257.2 | 11.0 | **44.9 ✗** | **41.0 ✗** | **42.3 ✗** | **49.8 ✗** | **44.0 ✗** |
| omissão | digital | 3840x2160 | papel | **13.0 ✗ régua** / 259.4 | 13.1 | **104.2 ✗** | **108.9 ✗** | **102.2 ✗** | — | — |
| omissão | impasto | 1920x1080 | blocos | **23.9 ✗ régua** / 267.9 | 12.0 | 18.8 | 18.0 | 17.1 | 16.9 | 14.9 |
| omissão | impasto | 3840x2160 | blocos | **11.0 ✗ régua** / 257.3 | 11.0 | **44.9 ✗** | **44.0 ✗** | **44.1 ✗** | **51.0 ✗** | **45.9 ✗** |
| omissão | impasto | 3840x2160 | papel | **14.8 ✗ régua** / 265.5 | 13.0 | **173.4 ✗** | **212.0 ✗** | **220.8 ✗** | — | — |
| omissão | water | 1920x1080 | blocos | **14.2 ✗ régua** / 260.2 | 13.0 | **45.9 ✗** | **52.9 ✗** | 17.9 | 23.0 | 26.2 |
| omissão | water | 3840x2160 | blocos | **11.9 ✗ régua** / 258.0 | 11.1 | **45.0 ✗** | **49.0 ✗** | **47.8 ✗** | **54.9 ✗** | **47.0 ✗** |
| omissão | water | 3840x2160 | papel | **66.0 ✗ régua** / 270.4 | **49.7 ✗** | **209.9 ✗** | **203.2 ✗** | **213.2 ✗** | — | — |
| omissão | wet | 1920x1080 | blocos | **24.5 ✗ régua** / 261.8 | 26.8 | **33.6 ✗** | 17.3 | 24.0 | 18.0 | 16.0 |
| omissão | wet | 3840x2160 | blocos | **11.1 ✗ régua** / 261.5 | 12.1 | **65.0 ✗** | **45.9 ✗** | **44.6 ✗** | **34.9 ✗** | 27.0 |
| omissão | wet | 3840x2160 | papel | **24.3 ✗ régua** / 267.1 | 21.0 | **241.0 ✗** | **422.5 ✗** | **207.8 ✗** | — | — |

### Leitura (o que a continuação acrescenta)

1. **A régua vê uma trava conhecida:** os 250 ms de `SIGSTOP` leem 257–272 ms em TODOS os 42 lançamentos (≤ 283);
   o mesmo critério no segundo antes, sem trava, lê 10–66 ms e reprova (`✗ régua`): a régua distingue.
2. **Trocar de camada a 4K paga o mesmo quadro fixo do toque/+1 s/Ctrl+Z:** com papel ~100–110 ms (Digital, load
   8–13; `+`, trocar, voltar); sem papel (4K blocos, load ~1) **41–50 ms** em todos os pincéis e em todos os cliques
   (`+`, trocar, voltar, duplicar, apagar) — a metade «tela» do B08b, sem a do papel. O Wet paga mais no `+` (65–94).
   A 1080p (branco ou blocos) 15–27 ms: passa. Impasto 4K papel 173–221 e Water/Wet 200–420 com load 30–47: TECTO.
3. **4K blocos repetido com load ~1 (contra 11 no B08b):** traço Digital passa (p95 10–15 ms); toque 18–29 ms nos
   Digital; Composite 41,8, Express Oil 46, Wet Liner 47, Filbert 53, Water 41–42 e Impasto **62** reprovam; o
   **Ctrl+Z do Wet 64–70 ms** (o 187–194 do B08b era a carga); o do Water 38 e o do Impasto 33,5. Os 100+ ms do B08b
   4K blocos eram carga.
4. **1080p blocos (load ~1,6): só o Wet reprova** — Ctrl+Z 38–41 ms (Express Oil, Filbert, Wet Liner, Wet omissão) e o
   traço do Filbert (p95 23,1). O resto passa em toque, traço, soltar e +1 s.
5. **1080p branco, os 8 restantes:** Chalk, Crayon, Finger Tip, Pastel passam (toque 22–23); Round Mixing (traço p95
   84, soltar 177), Wet Wash Mix (toque 86) e Composite reprovam, mas com load 9–50: TECTO, por repetir.
6. **Parado: 50–67 % de um núcleo** em todos os 42 — o mesmo do B08b (B23).
7. Fora ainda: slider de luz, abrir doc 4K, grupo de camadas, os 10 mais lentos (dependem do B08.3) e o Oil Wash (B36).
   (Os três primeiros: continuação 2, abaixo.)

## B08b continuação 2 — grupo, slider de luz, abrir doc; 1080p branco repetido com load < 1

- **Binário:** o de árvore LIMPA (`git archive cd02ac77ef`), com o `composite_smoke.rs` DESTE commit (+ a 2.ª tela de
  `PH2D_COMPOSITE_SEGUNDA`; o binário foi compilado antes de uma linha de doc-comment e do `rustfmt`, código igual),
  `CARGO_TARGET_DIR=target/b08b-limpo`, `--profile smoke`, `STRIP=none`. Radeon 610M em todos os 9 (`.placa` sem `nvidia`).
- **Roteiro novo**, depois de apagar a camada: **grupo** (`New group`, 1361,233: a camada ativa entra num `Group`, foto
  `camada7_grupo`) · **slider de luz** (só no painel impasto: aba Brush, 80 cliques da roda = o fundo do cartão, arrastar
  a PEGA «I» do `Intensity` da luz 90 px em 30 passos × 16 ms: 1.00 → 1.67 nas fotos `luz1`/`luz2`; coluna = o pior
  quadro do arrasto até 1 s depois / a mediana do arrasto) · **abrir doc WxH** (a 2.ª tela igual, `Composite 2`, na
  Hierarquia, 117,301: o Painter liga um doc NOVO; depois volta à `Composite`, o doc guardado; 2 s cada).
  ⚠️ O roteiro com `ABRIR` tem 2 telas na cena (a 2.ª fora da vista): as fases de antes leem-se com isso.
- **Régua:** o controlo de 250 ms lê 260–265 nos 9; sem trava 11–15: reprova (distingue).
- **Carga** (`carga.txt`, por lançamento no jsonl): 1080p branco 09:13–09:15, load 1 min **0,5–0,8** · 4K papel
  09:26–09:28, 2,6–5,8 · 4K blocos 09:28–09:30, 1,9–2,8 · 1080p blocos 09:30–09:32, 1,1–2,8.
- Cru: [`antes/b08b/continuacao2/`](antes/b08b/continuacao2/) (`matriz.jsonl`, `quadros/`, `branco1080/`, 4 fotos).

| pincel | painel | tela | fundo | base ms | toque ms | traço p50/p95 ms | soltar ms | +1 s máx ms | Ctrl+Z ms | parado CPU % (antes/depois) |
|---|---|---|---|---|---|---|---|---|---|---|
| Composite Brush | digital | 1920x1080 | branco | 10.2 | 26.1 | 11.9 / **16.8 ✗** | 17.0 | 19.8 | 18.8 | **53.0 ✗** / **53.3 ✗** |
| Round Mixing | digital | 1920x1080 | branco | 10.0 | 21.8 | 12.0 / 12.9 | 10.9 | 20.1 | 23.0 | **51.0 ✗** / **52.6 ✗** |
| Wet Wash Mix | digital | 1920x1080 | branco | 11.0 | 21.0 | 12.8 / 13.9 | 13.0 | 23.1 | 20.9 | **52.3 ✗** / **51.6 ✗** |
| omissão | digital | 1920x1080 | blocos | 10.1 | 12.9 | 10.4 / 11.1 | 11.8 | 12.2 | 12.8 | **51.3 ✗** / **55.0 ✗** |
| omissão | digital | 3840x2160 | blocos | 10.8 | 17.8 | 9.9 / 10.2 | 10.1 | 13.0 | 17.7 | **52.3 ✗** / **53.0 ✗** |
| omissão | digital | 3840x2160 | papel | 10.9 | **96.9 ✗** | 23.0 / **23.2 ✗** | 23.0 | **105.2 ✗** | **98.9 ✗** | **51.9 ✗** / **50.6 ✗** |
| omissão | impasto | 1920x1080 | blocos | 10.0 | 23.7 | 11.0 / 11.2 | 15.4 | 18.9 | 15.9 | **50.6 ✗** / **49.6 ✗** |
| omissão | impasto | 3840x2160 | blocos | 10.0 | **60.1 ✗** | 17.2 / **18.0 ✗** | 24.0 | **41.0 ✗** | **34.1 ✗** | **51.3 ✗** / **52.6 ✗** |
| omissão | impasto | 3840x2160 | papel | 10.2 | **98.1 ✗** | 23.1 / **24.0 ✗** | 30.1 | **111.4 ✗** | **95.8 ✗** | **50.0 ✗** / **50.6 ✗** |

| pincel | painel | tela | fundo | controlo sem/com | + camada | trocar | voltar | duplicar | apagar | grupo ms | luz máx / p50 ms | abrir doc / voltar ms |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| omissão | digital | 1920x1080 | blocos | 11.8 / 265.2 | 19.1 | 17.9 | 18.0 | 17.0 | 15.9 | 19.0 | — | 24.4 / 21.9 |
| omissão | digital | 3840x2160 | blocos | 12.9 / 260.4 | **43.0 ✗** | **44.2 ✗** | **46.0 ✗** | **46.9 ✗** | **46.9 ✗** | **67.1 ✗** | — | **96.0 ✗** / 25.1 |
| omissão | digital | 3840x2160 | papel | 12.8 / 260.1 | **109.1 ✗** | **103.2 ✗** | **104.1 ✗** | **111.0 ✗** | **102.1 ✗** | **126.1 ✗** | — | **154.8 ✗** / 23.9 |
| omissão | impasto | 1920x1080 | blocos | 11.0 / 261.7 | 18.9 | 17.2 | 17.9 | 16.9 | 15.8 | 19.0 | 19.1 / 13.9 | 27.7 / 22.0 |
| omissão | impasto | 3840x2160 | blocos | 15.1 / 265.5 | **44.1 ✗** | **58.9 ✗** | **42.8 ✗** | **49.2 ✗** | **43.8 ✗** | **60.9 ✗** | **55.8 ✗** / 27.2 | **97.4 ✗** / 23.8 |
| omissão | impasto | 3840x2160 | papel | 12.0 / 265.4 | **100.9 ✗** | **100.1 ✗** | **102.9 ✗** | **104.9 ✗** | **101.1 ✗** | **122.9 ✗** | **118.3 ✗** / **69.3 ✗** | **148.1 ✗** / 22.1 |

### Leitura (o que a continuação 2 acrescenta)

1. **1080p branco com load < 1: Round Mixing e Wet Wash Mix PASSAM** (traço p95 84 → 12,9; soltar 177 → 10,9; toque
   86 → 21): os ✗ da continuação eram a carga 9–50. O Composite fica a 0,1 ms do quadro no traço p95 (16,8).
2. **Grupo** custa o mesmo quadro fixo de trocar de camada e um pouco mais: 1080p 19 ms; 4K sem papel 61–67; 4K papel
   123–126.
3. **Slider de luz do Impasto:** a 1080p passa (pior 19, mediana 14); a 4K CADA passo do arrasto paga a recomposição —
   sem papel pior 56 / mediana 27, com papel pior 118 / **mediana 69 ms** (≈ 14 img/s enquanto se arrasta).
4. **Abrir doc:** um doc NOVO custa 1080p 24–28 ms, 4K sem papel **96–97**, 4K papel **148–155**; VOLTAR ao doc guardado
   22–25 ms em todos (o guardado não reabre a tela inteira).
5. O resto repete a continuação: 4K papel ~100 ms por clique/toque/Ctrl+Z; 4K blocos 43–59; 1080p 16–19.
6. Fora ainda: os 10 mais lentos (dependem do B08.3) e o Oil Wash (B36).

## B08b continuação 3 — a placa nos 56 Wet (B08.4), os 100 pincéis à velocidade do dono (B08.3), os 10 mais lentos no app (B08.2)

- **Árvore:** LIMPA = `git archive cd02ac77ef` em `target/b08b-fonte` + só o `tests/it/sonda_custo_no_app.rs` deste commit
  (a sonda nova `sonda_placa_sabe_nos_wet`; a `sonda_custo_no_app` é a de antes, igual ao bit), `CARGO_TARGET_DIR=target/b08b-limpo`,
  `--profile smoke`. Os presets de fábrica da base = os de hoje (`git diff cd02ac77ef HEAD -- assets/brushes` vazio).
- Cru: [`antes/b08b/continuacao3/`](antes/b08b/continuacao3/).

### B08.4 — os 56 Wet de fábrica e a placa: **0/56** passam `placa_sabe`

```
CARGO_TARGET_DIR=$W/target/b08b-limpo bash $W/scripts/ph2d-run.sh cargo test --manifest-path $W/target/b08b-fonte/Cargo.toml \
  -p ph2d-tool-painter --features gpu --profile smoke --test it sonda_placa_sabe_nos_wet -- --ignored --nocapture
```

Cada Wet de fábrica num `PainterTool` 256², um traço curto (6 quadros), e `ph2d_wet_gpu::passo::placa_sabe` sobre o motor da
sessão (a função REAL, `crates/ph2d-wet-gpu/src/passo.rs:19-45`, igual na base e hoje); ao lado, cada condição dela que reprova.
Determinístico (não é relógio). Sem a RTX não importa (decisão 2): fica o registo.

| reprova por | Wet |
|---|---|
| K–M (`pigmentMixing`) | **53** (todos menos Wet Liner, Wet Wash e Gouache Opaque) |
| lei/óleo da grade (`LeiDaTinta::Velha` com `g.oleo` não vazio depois do traço) | 21 (os 14 Real Oil, Express Oil, Oil Pastel, Wax Crayon, Wax Scrape, Wet Liner, Wet Wash, Gouache Opaque) |
| Real Oil (`p.oleo ≠ 0`) | 14 (Dabber, Dabber 4, Filbert, Filbert Dry, Flat Bristle, Thick Impasto, Tilted Shade, os 6 Knife, Metallic Flat) |
| Opaque / opaca na grade | 3 (Fan Brush, Gouache Flat, Gouache Opaque) |
| fingering/backrun/difusão | 1 (Ink Brush) |
| ordem · passo pendente · ecrã ≠ Cobre · gota | 0 |

Os 3 sem K–M (Wet Liner, Wet Wash, Gouache Opaque) caem só no óleo da grade (e o Gouache Opaque também na opaca): nenhum
Wet de fábrica chega à placa hoje. Lista por pincel: [`placa_sabe_56_wet.txt`](antes/b08b/continuacao3/placa_sabe_56_wet.txt).

### B08.3 — os 100 pincéis à velocidade do dono: **49/100 passam de 16,7 ms/quadro**

```
CARGO_TARGET_DIR=$W/target/b08b-limpo PH2D_SONDA_PRESETS=todos PH2D_SONDA_EVQ=9 PH2D_SONDA_VELS=40 PH2D_SONDA_TELA=1024 \
  PH2D_SONDA_RODADAS=2 bash $W/scripts/ph2d-run.sh cargo test --manifest-path $W/target/b08b-fonte/Cargo.toml \
  -p ph2d-tool-painter --profile smoke --test it sonda_custo_no_app -- --ignored --nocapture     # 455 s
```

Tela 1024² branca, o tamanho do preset, 9 eventos × 40 px por quadro (360 px/quadro = a caneta do dono num traço rápido,
`Pinceis/COMO_ACELERAR_UM_PINCEL.md` §1), 40 quadros por bloco, 2 rodadas intercaladas; ms = o MÍNIMO das medianas.
⚠️ **load 13,4 → 18,0** (outros agentes a compilar; o build do B08.4 nos 2 primeiros minutos): leia como TECTO — a
ordem dos pincéis vale, o número absoluto pode descer com a máquina calma. Tabela inteira (100 linhas, depósito ×
composite × tique): [`sonda_100_pinceis.md`](antes/b08b/continuacao3/sonda_100_pinceis.md).

| # | pincel | pasta | diâm | depósito | composite | TOTAL ms/quadro |
|---|---|---|---|---|---|---|
| 1 | Filbert | Wet | 64 | 151,6 | 55,3 | **209,2** |
| 2 | Knife Oily | Wet | 51 | 99,2 | 63,9 | **164,3** |
| 3 | Filbert Dry | Wet | 34 | 105,4 | 44,3 | **149,9** |
| 4 | Flat Bristle | Wet | 46 | 74,5 | 67,9 | **142,5** |
| 5 | Tilted Shade | Wet | 40 | 85,2 | 56,3 | **141,0** |
| 6 | Knife Dry | Wet | 51 | 87,2 | 48,6 | **136,9** |
| 7 | Scratch | Wet | 248 | 125,9 | 4,0 | **130,9** |
| 8 | Metallic Flat | Wet | 40 | 81,6 | 43,9 | **127,7** |
| 9 | Thick Impasto | Wet | 64 | 88,9 | 31,5 | **121,2** |
| 10 | Express Oil | Wet | 64 | 93,6 | 4,2 | **99,5** |
| 11–20 | Dabber 96 · Knife Smooth 3 92 · Knife Stain 85 · **Oil Paint 84** · **Gouache 83** · Knife Textured 80 · **Dry Gouache 80** · Knife Grainy 77 · Wax Scrape 70 · Dabber 4 61 | | | | | |
| 21–34 | Water Brush 51 · Pastel Side 49 · Ink Brush 46 · **Round Mixing 43** · Charcoal 39 · Square Hard 38 · Soft Pastel 36 · Fan Brush 35 · **Paint And Apply 34** · Oil Pastel 34 · **Thick Oil Paint 34** · Broad Nib 31 · Marker Chisel 29 · Compressed Charcoal 26 | | | | | |
| 35–49 | **Wet Wash Mix 22** · Wet Dry Brush 22 · Wet Drip 21 · **Brush Pen 20** · Wax Crayon 20 · Wet Round 20 · Graphite HB 20 · Marker Bullet 20 · Graphite 2B 20 · **Gouache Blender 19** · Wet Liner 19 · Graphite 2H 19 · **Finger Tip 18** · Wet Wash 18 · Ink Bullet 18 | | | | | |

(**negrito** = Digital; o resto é Wet.) Leitura:
1. **Os 10 mais lentos são todos Wet**: 8 dos 14 Real Oil (fora Dabber, Dabber 4 e 4 Knife), o Scratch e o Express Oil;
   99–209 ms/quadro = 5–10 fps à velocidade do dono, numa tela de 1024². Nos Real Oil ~metade é o **composite**
   (31–68 ms: a óptica do Real Oil), metade o **depósito**; o Scratch (248 px) e o Express Oil são quase só depósito.
2. **39 dos 56 Wet** passam de 16,7; **10 dos 43 Digital** (Oil Paint, Gouache, Dry Gouache, Round Mixing, Paint And
   Apply, Thick Oil Paint, Wet Wash Mix, Brush Pen, Gouache Blender, Finger Tip: os de mistura/véu = os candidatos da
   meia definição, NA-3 passo 2). O Watercolor Basic (7,6) passa.
3. O Wet Wash Mix já tem a meia definição na base (`31e59443d1`): 22 ms aqui sob load ~15 (24 no doc da cura 6).

### B08.2 — os 10 mais lentos no app: 4K papel, 1080p papel, 4K branco (30 lançamentos)

Binário `target/b08b-limpo/smoke/ph2d-host-desktop` (o da continuação 2, árvore limpa), Radeon 610M nos 30 (`.placa` sem
`nvidia`), roteiro da matriz inteiro (`mede_pen_up_4k.sh matriz <WxH> <fundo> $B "Filbert:wet" "Knife Oily:wet" …`, sem
fotos). ⚠️ **Carga** (`carga.txt`): 4K papel 13:18–13:25 load 14 · 1080p papel 13:25–13:32 load 10–18 · 4K branco
13:32–13:40 load **28** — TECTO, não base. Falharam sem janela: Filbert 1080p papel, Express Oil 4K branco (cortado pelo
prazo). Fora (o prazo do agente): 1080p branco, 4K e 1080p blocos — o mesmo comando, `branco`/`blocos`. Tabelas
inteiras (fases + camadas + régua): [`tabela_matriz.md`](antes/b08b/continuacao3/tabela_matriz.md); cru `matriz.jsonl`, `quadros/`.

| fase (ms; mín–máx dos 10) | 4K papel | 1080p papel | 4K branco |
|---|---|---|---|
| toque | 74–127 | 38–173 | 113–942 |
| traço p50 | 14–242 | 17–114 | 39–218 |
| traço p95 | 33–313 | 21–439 | 56–495 |
| soltar / +1 s máx | 31–317 / 99–317 | 19–122 / 34–122 | 37–227 / 46–227 |
| Ctrl+Z | 112–288 | 48–205 | 116–676 |
| + camada · trocar | 153–419 · 108–424 | — (ver tabela) | — (ver tabela) |
| parado CPU % | 59–73 | 42–72 | 33–79 |
| água img/s | 21–40 | — | — |

Leitura: com os pincéis lentos o quadro fixo de ~100 ms a 4K (o da tela + papel, §B08b) vira **o piso**, não o
tecto: Tilted Shade traça a 242 ms/quadro (4 fps), Knife Oily 114, e o Ctrl+Z chega a 288 (papel) / 676 (branco, sob
load 28). A 1080p o traço p50 dos Real Oil fica 22–50 ms (Scratch 114, Metallic Flat 80 sob carga). Parado a CPU fica
em 60–70 % em todos (o Wet a correr). A régua: o controlo de 250 ms lê 258–284 com trava, 14–37 sem (distingue).
**Oil Wash:** não existe na fábrica (`assets/brushes` sem ele): o «antes» é do B36.
