# Estado — missão «O que o Rebelle faz melhor, medido» (30/10)

Prompt: [PROMPT_JANELA_REBELLE_MEDIDO.md](PROMPT_JANELA_REBELLE_MEDIDO.md). Backup: ramo
`backup/line-PainterWatercolor-antes-do-rebelle-medido-2026-10-30` + tag `painter-antes-do-rebelle-medido-2026-10-30`
(conferidos). Base: `341341b233`.

⚠️ O doc do dono (artifact `QUaFiJcbRj1YCed8ttaU1L`) não está acessível a esta janela (acesso negado, 30/10):
os factos do Rebelle usados são os que o prompt transcreve.

## Itens

| item | estado | agente | commits | número |
|---|---|---|---|---|
| Fase 1 conferência | FEITA: composite à res. do documento (ratio 1), zoom só no sprite (`painter_bridge.rs:83`), composite não conhece a câmara ⇒ W1 GPU na vista > 1 janela | explorador | — | — |
| A NanoPixel | A.1 CPU FEITA r2 ([A1_oleo_na_vista.md](A1_oleo_na_vista.md)): B-spline interpolante C2, s=1 ao byte, 3,27× a bilinear; listras restantes = brilhos por fio já no documento. GPU = escolha do dono | general-purpose | bdbb414984 6c690ae097 | 1 fio: hoje 1066 ns/px; nova s=4 778 ns/px ⇒ 4K inteira ≈ 6,4 s/1 fio: tempo real exige placa |
| B núcleos | FEITA r2 ([B_nucleos.md](B_nucleos.md)) | general-purpose | 95cd9830c8 182dfc661e | 7,5 passos/s (128–133 ms), 28,9 núcleos ÚTEIS (espera 0,99 %), FP 37,5/62,5/0 %, 428 B/px. Passo: secagem/assenta ~46–55 % (`assenta.rs:334` 11 %), advecção 34 % (~43 ms) |
| C.1 f32 / C.2 v3 | FEITA ([SONDAS_C_f32_v3.md](SONDAS_C_f32_v3.md)) | general-purpose | b393430f66 | ver abaixo |
| A ronda 3 | mutação: M2/M6/M6a/M7 sobreviveram ⇒ 3 testes (escala da curvatura, meio pixel, borda) | mutacao | 28b374e391 | 10 mutantes, 6 sangraram de início |
| Fecho | FEITO: handoff [HANDOFF_REBELLE_MEDIDO.md](HANDOFF_REBELLE_MEDIDO.md); fmt/clippy ✓; testes por nome ✓; 5 vermelhos anteriores à missão anotados; mutação 10/10 após r3 | — | ver git log | — |
| D Rebelle 4K | FEITA ([D_rebelle_4k.md](D_rebelle_4k.md)); régua «parado» de custo_facas.sh bate no tecto de 90 s em 4K (parado 0,08–0,11 núcleo) — números do traço por segundo; traços em coords 1200×800 (canto da tela) | general-purpose | 4738e697a0 | criação 4K 1–2 s, 3,2 s CPU, pico ~3 núcleos · 20 pinceladas ~8 s, 9–11 s CPU, ~2 núcleos; placa ~4 % |

## Números

- C (poça 4096², smoke, mín/med): advect paralelo f64 5,21 / f32 5,17 ms (−1 %); série: f32 −3,9 %, v3 −8,5 % (só série; o rayon não herda v3).
  f32 após 12 passos: sett máx 8,1 (de 294) em 1,36 % das células; **cor infinita em 1–1,6 % (1/want estoura em f32)**.
  v3: **zero bits mudam** (Rust não funde a*b+c; mul_add já exacto). v3 exige `unsafe` (workspace proíbe).

## Onda 2 (dono 30/10 «sim»): mais rápido que hoje
Ordem aprovada: (1) régua comparável — a MESMA aguada no PH2D e no Rebelle («somos X× mais lentos»); (2) secagem (≈ metade
do passo) mais leve sem mudar a cor; (3) memória por ponto (428 → ~220 B/px). Rebelle no ecrã real = pedir de novo ao dono.
| item | estado | agente | commits | número |
|---|---|---|---|---|
| 1 régua comparável | FEITA ([D2_rebelle_passo_agua.md](D2_rebelle_passo_agua.md)) | general-purpose | ver git log | Rebelle 4K 37,3 ms/passo (21 núcleos, 75 ns·núcleo/px), 1K 3,9 ms; PH2D 4K 100,5 ms (~278 ns·núcleo/px) ⇒ META < 37 ms |
| 2 secagem | FEITA no produto, ao bit (05b3830f34 3a931fccb8 6b152bd6ad 85c0a42c5b) | general-purpose | ver acima | passo 4K 128 → 100,5 ms (+27 % passos/s) |
| 3 memória (P1) | undo só do que o traço muda (78 % do RSS é o histórico 6×56 B/px) — em construção | general-purpose | — | 428 B/px hoje |
| 4 advecção (P2) | cortes ao bit: outflow 2 linhas, take_scale 1×, prepare⊂momentum, dst SoA — em construção | general-purpose | — | advecção ~43 ms |
| 5 pincéis | catálogo FEITO: CAP-01 ponta que gira destrava Tilted Shade, Compressed Charcoal, Marker Chisel, Filbert Dry, Flat Bristle, Broad Nib; depois pontas a direito, borda molhada, fio do arrasto | — | — | depois da performance |
