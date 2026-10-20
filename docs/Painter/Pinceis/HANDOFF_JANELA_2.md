# HANDOFF — JANELA 2 → JANELA 3 (2026-10-20)

> Leitor: a JANELA 3 (desenho, tinta, aerógrafo). Leia antes: `ESTADO.md`, `03_MOTORES.md` §4 (linhas ✅) e §7/§7.1,
> `06_LEIS_DAS_LACUNAS.md`, `04_BANCADA.md` §3.1. Árvore: `Worktrees/line-PainterWatercolor`, `line/PainterWatercolor`.

## O que a J2 entregou (motor — todos com portão red-first, mutação, custo e revisão cega de código)

| bloco | o que existe agora | SHAs principais |
|---|---|---|
| Leis medidas | `06_LEIS_DAS_LACUNAS.md` (+§10 fotos novas `fixtures/*_lacunas/`); 32 fotos velhas do Rebelle refeitas; guarda `foto_estavel.py` | `c3f27254d4` `2f7267cf01` |
| Leis sistémicas | pressão → CARGA em 20 presets · traço de 5 px por área · relevo `(ref÷D)^γ` · ponta recta e desfiada das cerdas · PAREDE entre traços (sem lente sobre a molhada) · grão do Wet nas pontas finas | `77da300cb1` `95c9597395` `bf832ce2c3` `2fc7a32d60` |
| L1+L2 | presets nos 4 meios (`wetpaint_preset/meios.rs`, navegador, miniatura, pastas `Digital/` `Watercolor/` `Impasto/`) · a mão por uma lei (`mao.rs`) + cartão Pen nos 4 meios · inclinação→opacidade | `b28136d620` `d35c87a54a` |
| L4+L11 | jitter: opacidade, contagem, espelho X/Y, redondeza, ângulo por traço, espalhar ATRAVÉS · cor secundária + pressão→cor | `d1489ae706` `9201d46716` `bb76fcf22c` |
| L7, L7-vazio | modos W3C em sRGB (provado) · «On empty» (fórmula geral sobre o transparente) | `f1260c541c` `c8e95a075d` |
| L12 | velocidade estreita: `(v/300)^−p` (Fineliner 0,058 · pen 0,136 · Marker 0,45) | `2045598fac` |
| L8+L5 | o PLANO do traço (Digital): tecto por traço, orla molhada/queimada · cauda re-pintada ao levantar · fall-off (4 meios) | `ed926fa0eb` `14915eeb77` |
| L18+L9 | ponta de píxel / pixel-perfect · grão: Multiply (o de hoje) + Subtract, Darken, Color Burn, Height, Linear Height; profundidade por dab | `0ef4275daf` `8298230687` |
| L19 | ponta de até 8 imagens: sorteio com peso · sequência · pressão · ângulo | `2efc5f3bde` |
| L6 | dual brush no Digital e no Wet (7 modos, paridade GPU, 0 quadros > 16,7 ms) | `b3c2379bdb` `7973c324de` `1dfd9c6670` `41a48df4b4` |
| L13 + memória | o Dabber redondo, uma lane por ponta; RSS a 1000 px 4,3 → 1,2 GiB | `8b4e2a1c21` |
| Limpeza | portas da macro decididas · controlos que mentiam esmaecidos · CERCA verde (8 isenções com prova) | `35f3629899` `68bd57f602` `f87844fb83` |

## O que a J3 tem de saber

1. **Um pincel novo é um FICHEIRO em qualquer meio.** Pinte-o, grave com «Save as New» ou pelo gerador
   (`tests/it/presets_de_fabrica.rs` + `receitas_de_fabrica.rs`); a bancada aceita o caminho. Exemplos:
   `fixtures/presets_exemplo/` (um por meio antigo), `fixtures/presets_teste/` (dual, pixel, grão, mangueira).
2. **A régua agora decide**: os ruídos de cada métrica foram MEDIDOS (`04_BANCADA` §3.1). A matriz foi refeita com as
   fotos certas e a fábrica de hoje (`matriz/vizinhos.md`). Mesmo assim: prancha OLHADA + revisão cega (05).
3. **Leis que mandam no desenho e na tinta** (06): o Rebelle NÃO afila o fim a pressão constante (cauda ≈ tampa) —
   a cauda é das canetas do Procreate; a velocidade só ESTREITA e só no Krita/MyPaint; o ruído de cor do Rebelle é
   POR TRAÇO; o efeito pousa a cor a α 0,056–0,216 por passagem.
4. **Recusas medidas** (não reconstruir): a lâmina que gira com inércia/memória (o Rebelle não gira; 06 §1, §10);
   pressão→FLUXO como linha nova (é o mesmo produto que a Strength, `dab.rs:451`); o piso de pressão; a quina
   do carimbo como ponta recta das cerdas na lei das cerdas (3 formas medidas e recusadas, 03 §7.1).
5. **Custo**: todas as lacunas ≤ 16,7 ms p95 a 200 px. O Dabber a 200 px leu 16,0 ms a load 18 — re-medir calmo.
6. **O fecho batched (suíte inteira + clippy) é da J5.** A J2 correu os portões POR NOME (lista no `03_MOTORES` e
   no relatório de fecho abaixo).

## Fecho da J2 (a bateria por nome, os achados da última revisão)

EM CURSO no momento desta escrita — a J2 actualiza esta secção antes do sinal.
