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
| 3 memória (P1) | FEITA b4119a7af0: undo COW por blocos 64×64 — histórico 330 → 4,8 B/px; processo 428 → **177 B/px** (Rebelle ~230–260) ✓ | general-purpose | b4119a7af0 | [MEDIDA_31_10_RODADA2.md](MEDIDA_31_10_RODADA2.md) |
| 4 advecção (P2) + fluxo | FEITA: outflow bt, take_scale, dst 3 planos, bt⊂prepare, commit por TROCA (−19..33 ms; c6ea88ad3a cura transbordo i32); secagem SEM (Newton pára sem correcção, −36 % na sonda, 46199a6a7c); tolerâncias da secagem RECUSADAS (erro acumula sob Glaze) | general-purpose | ver git log | sob carga (load 50–110) as réguas não resolvem cortes < 5 ms; falta rodada calma |
| 5 pincéis | CAP-01 inércia (329564e0ed, 12a87ecfbb) + 6 de fábrica (48848924ec, afinados 5d29d7d8ec, regerar); prancha [cap01](../Pinceis/pranchas/cap01_seis_pinceis.png). R2: Charcoal/Wax Scrape/Square Hard ✓; Knife Stain/Grainy/Spray/Dry Marker/Blender Grainy mecanismo registado. R1: Wet Wash/Liner ✓ por valores; Smudge tecto recusado; edgeSoftMass em curso | vários | ramos r1/r2 por juntar | fábrica 72 |

## Janelas de reserva (dono 31/10) — ramos próprios a partir de 6ba5d06493; o coordenador junta-os à linha no fim
| janela | worktree / ramo | missão | relatório |
|---|---|---|---|
| ph2d-be (reserva 01) | line-PainterWatercolor-r1 | borda molhada (Wet Liner/Wash/Mop), fio do arrasto (Smudge Finger), Water/Fan/Gouache | docs/Painter/Pinceis/R1_RELATORIO.md |
| ph2d-56 (reserva 02) | line-PainterWatercolor-r2 | pontas (Charcoal/Wax Scrape/Square Hard), grão no óleo, Spray/Splatter, Dry Marker, Rake | docs/Painter/Pinceis/R2_RELATORIO.md |
| ph2d-6a (reserva 03) | line-PainterWatercolor-r3 | velocidade de PINTAR (dab serial ~4,7 núcleos), óleo composite | docs/Painter/49_realshader/R3_PINTAR_RELATORIO.md |
Subagentes na linha principal: P1 undo (painter/*, grid/snapshot.rs), P2 advecção (advect_jacobi, scratch, par, sedimenta), CAP-01 + 6 pincéis (painter-brush spec/footprint/pen, mao.rs, inercia/).

## Próxima alavanca (estudo 31/10, explorador): a água na PLACA = 3–4 janelas (ADR-0146 PROPOSTA; paridade passa à forma Jacobi do ADR-0147, épsilon por placa). 1.º degrau numa janela: advecção na placa com leitura de volta, ms RTX/Radeon. CPU restante: secagem ~46 % (aritmética K–M por avaliação), advecção ~20 %, fluxo ~10 %.
