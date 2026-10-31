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
| D Rebelle 4K | aguarda AVAL do dono (ocupa o ecrã real) | — | — | — |

## Números

- C (poça 4096², smoke, mín/med): advect paralelo f64 5,21 / f32 5,17 ms (−1 %); série: f32 −3,9 %, v3 −8,5 % (só série; o rayon não herda v3).
  f32 após 12 passos: sett máx 8,1 (de 294) em 1,36 % das células; **cor infinita em 1–1,6 % (1/want estoura em f32)**.
  v3: **zero bits mudam** (Rust não funde a*b+c; mul_add já exacto). v3 exige `unsafe` (workspace proíbe).
