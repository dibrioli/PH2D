# Handoff de integração — line/PainterWatercolor, missão Painter 4K (10–11/11)

Leitor: o agente integrador. O mecanismo de cada item está na linha do [BACKLOG](../missao_4k/BACKLOG.md) (id `B<nn>.<n>`) e no commit;
aqui só o que evita colisão e regressão. **Nada integrado nem enviado.** A missão PAROU no fecho por ordem de custo do dono
(11/11, «consumindo muitos créditos»), com o BACKLOG longe de fechado (§7).

## 1. Identidade

- Branch `line/PainterWatercolor`, merge-base com o main `bc4a6ce930`, 285 commits de código/docs (10/11 07:10 a 11/11), 1 403 ficheiros.
- HEAD do código `e7728c2ee2`; **os commits deste fecho vêm por cima** (só docs: BUGS, guia, ADR-0203, handoff, prompt, `CLAUDE.md` §5.1).
- Anterior: [pen-up 4K e mistura, 10/11](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-10_PEN_UP_E_MISTURA.md) (já no main).
  Missão: [prompt 10g](PROMPT_MISSAO_2026-11-10g_painter_4k_autonomo.md) · [regras](../missao_4k/REGRAS_DO_AGENTE.md) ·
  [despacho](../missao_4k/DESPACHO.md) (ondas 1–14, o que cada agente devolveu). Continuação: [10h](PROMPT_CONTINUACAO_2026-11-10h_pos_missao_4k.md).
- Tocado, por crate (linhas + / −, ficheiros): `ph2d-tool-painter` 32 312/7 103 (346) · `ph2d-wet-paint` 5 981/1 460 (83) ·
  `ph2d-painter-brush` 3 803/947 (46) · `ph2d-app-painter` 1 790/799 (26) · `ph2d-painter-effects` 1 036/129 (8) · `ph2d-render`
  423/17 (14) · `ph2d-panel-wet-tuning` 228/21 · `ph2d-inpaint` 214/58 · `ph2d-anim-sheets` 179/12 · `ph2d-editor-core` 175/0 ·
  `shells/desktop` 173/155 (20) · pequenos: `ph2d-app-motion`, `-audio`, `-board`, `-sprite-anim`, `ph2d-wet-gpu`, `ph2d-gpu`,
  `ph2d-panel-painter-layers` · **`ph2d-app-host` 62 797/0 (220) = o `winit` 0.30.13 vendorizado** (ADR-0203) · `docs/Painter` 585.

## 2. Superfície de colisão (`bash scripts/collision-surface.sh`, 11/11)

```
merge-base bc4a6ce930 · 285 commit(s) · 1403 arquivo(s)
⚠ PROJECT_SCHEMA 232 (base 231) · ⚠ tripla (232, 13, 29) (base (231, 13, 29)) · VEC 29 · FLIP 13 · DOC_VERSION 25
ph2d-ecs 121 (base 121) · ✗ SONDA CEGA ph2d-render/ph2d-script = defeito conhecido do SCRIPT, a linha não toca componentes
Contrato congelado §6: node.rs intocado · tool.rs intocado
ADR: último no disco 0202 · esta linha CRIA 0202 (e 0203 neste fecho) — reconte contra o main do dia
Cargo.lock: nenhum '+name' novo · marcadores de conflito: nenhum
TETOS DE LOC: 25 ✗ todos em crates/ph2d-app-host/vendor/winit/** (código de terceiros; o gate varre só crates/*/src)
```

- O degrau do schema é UM (`9c76d12521`, B50.1: os enums do pincel gravam-se pelo NOME; SCHEMA do preset 1→2). Na fusão:
  `python3 scripts/schema-recount.py <valor do main + 1>` DENTRO do conflito (escada + tripla, ficheiros irmãos `project_schema*.rs`).
- `Cargo.toml` da raiz: `members` ganha `crates/ph2d-app-host/vendor/winit` excluído e `[patch.crates-io] winit = { path = … }`.
  Outra linha que mexa no `[patch]` ou suba o `winit` reconcilia aqui (VENDOR.md diz os 3 hunks).
- Ficheiros de ponte com outras linhas: `shells/desktop/src/main.rs` e `present_title.rs` (laço a pedido, B23), `ph2d-render`
  (mipgen/premul por retângulo, B20a), `ph2d-app-motion`/`-board`/`-audio` (declaram o seu `quer_quadro`).

## 3. Números que se CONTAM (DELTA)

- **`PROJECT_SCHEMA`: +1** (231 → 232; tripla `(232, 13, 29)`). Sem migração (CLAUDE §0.11).
- **ADR: +2** (0202 rayon no compositor e kernels espaciais; 0203 quadro a pedido + `winit` vendorizado). **Registo de componentes: 0.**
  **Pacotes externos novos: 0** (o `winit` é o mesmo 0.30.13, agora por caminho). Dep interna nova: `rayon` em `ph2d-painter-effects`.
- **Tectos:** `ph2d-app-painter` 22 427 linhas contadas por `find|wc` (catraca `the_shell_only_shrinks` ≤ 24 429; o gate conta à sua
  maneira, medido 24 382 em `5a5aac2f52`) · shell 189 015 (`aa64a921ee`) · ficheiros `.rs` ≤ 700 fora do vendor (o fecho partiu
  `composite_smoke.rs` 706→700, `solid_deposit/banda.rs`, `stamp_banded_capped_tests.rs`).
- **BUGS:** #86–#113 (28 defeitos curados) em [BUGS_painter.md](../BUGS_painter.md).

## 4. O que entra, por tema

| Tema | Itens (BACKLOG) | Número que mudou (4K, Radeon integrada) |
|---|---|---|
| Perda de dados e undo | B01, B02, B03, B04.1, B05, B06a–c, B48 | #86–#96; ajustes com passo de undo; Wet/tinta por assar presos ao documento |
| Pen-up / quadro 4K | B10.3, B10.5, B10.6–8, B20a (`PH2D_PREVIA_INTEIRA=1` volta) | quadro p50 4096² 38,8→10,2 ms (4 camadas 47,4→10,6); toque/traço/Ctrl+Z 32–38→15–17 ms |
| Compositor CPU | B19a–b, B19.7, B19.11, B52.R3 (ADR-0202) | arrasto de ajuste 1 149→134 ms; 1 044 fios do SO→0; camadas vazias 38,1→19,2 ms |
| Memória do undo | B17a–c, B12b | Impasto diagonal 265→32 MB; Wet dab r30 897→26 MB (motor); cópias de tela 2→1 |
| Wet | B11 (agenda, espelho estreito, worker morto), B12.1 anel, B13 span_runs, B14 (óptica, luz simples), B02.A3 | espelho 30,35→3,89 ms; Ctrl+Z Wet Round 54→30 ms; secagem longe 65,1→22,1 ms/12 q |
| Watercolor | B16.1–4, B16.9, B16.12, B16b/c/d, B04 | pen-up aguada 48,5→1,95 ms; toque no papel 186→32 ms; Rewet 201→3,1 MB |
| Impasto | B15 (settle, planos, lâmpada), B10a | planos Gaussiano/Nitidez 434→3 ms; Ctrl+Z ciclos −52 % |
| Carimbo / pincéis | B24, B25, B26.A2, B29a, B49.1, B14c | pó Pastel pen-down 6,65→2,62 ms; dente 0,149→0,045 ms/dab; mesmo byte a 1/2/32 fios |
| Quadro a pedido | B23 (ADR-0203) | janela oculta 97–100 %→44–46 % CPU; parado: pedido 3 % × contínuo 57–70 % |
| Formas, seleção, macros, pânicos | B40a–b, B41a–b, B16.7, B22.2 | Feather r64 11 121→16,8 ms; arrasto da seleção sem recompor |
| Preset | B35.A1, B50.1–2, B35a.1/.4/.6a, B35.A3 | enums pelo nome (schema +1); Tuning sem as 19 linhas do Inspector |
| Fecho | `14a2efbbae` `88f867ac68` `1eb4854b25` `3e3572bd18` `e7728c2ee2` | clippy workspace verde; tectos; provas de mutação |

Pranchas antes/depois das curas que mudam a aparência: [pranchas/INDEX.md](../missao_4k/pranchas/INDEX.md) (B06a, B02-cont, B24,
B14b, B16.2, B25.2, B40b, B22.2, B49-cont). Técnicas e recusas: [COMO_ACELERAR_UM_PINCEL.md](../Pinceis/COMO_ACELERAR_UM_PINCEL.md) §7–§8.

## 5. Prova de fecho (11/11)

Nada de suíte completa nem `nextest-impacted` (dono 23/10). Cada item foi red-first e verde por nome no seu commit (a verificação de
cada onda no [despacho](../missao_4k/DESPACHO.md)). No fecho: `clippy --workspace --all-targets -D warnings` rc=0 (`88f867ac68`; antes
38+61+5+2 erros) · `fmt --check` verde (idem) · `workspace_src_files_under_loc_cap` verde por nome (`1eb4854b25`) · censo HR-15
`every_word_this_engine_shows_comes_from_the_string_table` red→green (`14a2efbbae`) · `architecture_claude_md` por nome (este fecho, §5.1).

**Mutação:** [mutacao_A](../missao_4k/mutacao_A.md) 8 mutações, 7 sangram · [mutacao_B](../missao_4k/mutacao_B.md) 9, 8 sangram.
Sobreviventes (gates em falta, §7): `funde_as_caixas` acima de `MAX_CAIXAS`; rejeição por banda com folga de 1 linha (B25.1). Os
dois vermelhos do B49.1 são pânico de índice, não a comparação de bits.

**Alarmes de pincel:** vermelhos desde `3f843b81b1` (B49.1, 46 presets) — **não regravados** (regra do dono); relatório
`target/relatorio-pinceis/index.html` vai ao smoke.

**Medição «depois» no app (B08c): NÃO feita** — a placa esteve com a `line/InputMap` ([DEPOIS.md](../missao_4k/medicoes/DEPOIS.md));
os números do §4 são dos A×B por item. `ANTES_DEPOIS.md` não existe.

`agent-loop-profile` (20 sessões):
```
✗ paralelismo de ferramenta              1.15/passo   alvo: >= 1,5  (10% dos passos com 2+ chamadas)
✓ respostas por sessao (mediana)                174   alvo: <= 800
✗ cargo test : cargo check                430 : 132   alvo: <= 1,0  razao 3.3x (baseline: 4,3x)
✓ edicoes SEM conferir que casaram             1.8%   alvo: <= 2%  (13 de 741)
✗ contexto relido por passo (media)         260 mil   alvo: <= 250 mil
✓ contexto no inicio da sessao               68 mil   alvo: <= 80 mil
```

Binário do smoke (2.ª corrida de `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`):
```
▸ linha line_painterwatercolor · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.25s        (1.ª corrida: 57,37 s; zero «Compiling» na 2.ª)
```

**NA INTEGRAÇÃO, suíte inteira:** código movido entre ficheiros e crates (dezenas de partições), deps (`rayon` no painter-effects,
`[patch]` do `winit`), formato gravado (schema +1), e os gates de custo sob carga (§8).

## 6. Smokes do dono

Binário: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop` (depois da integração, o do main).
Tela nova 3840×2160 com papel Textured (o caso que travava). Nenhum destes foi visto pelo dono.

1. **Pintar em 4K** (B10, B20a): Round/Water/Impasto/Wet — traço, soltar, esperar 1 s, Ctrl+Z. Tem de ficar sem soluço; errado = trava visível ao soltar ou no Ctrl+Z.
2. **Ajustes de camada** (B19b, B15.5): arrastar o slider de um Brightness/Contrast e de um Gaussian Blur em 4K. Errado = a imagem fica para trás do slider.
3. **Undo de camadas e ajustes** (B01, B05): mudar opacidade, modo, um parâmetro de ajuste, apagar um grupo; Ctrl+Z volta UM gesto de cada vez.
4. **Wet entre abas** (B02): pintar Wet, trocar de aba a meio da água, voltar. A água continua no documento certo; errado = tinta aparece na outra aba.
5. **Soltar fora da janela** (B03): pintar e soltar o botão fora da janela (ou Alt+Tab). O traço fecha; Ctrl+Z tira só ele.
6. **Borracha leva o relevo** (B06a): Impasto e Wet, apagar — o relevo some com a cor (prancha B06a).
7. **Aparência mudou** (ver as pranchas): Blur do Composite Brush (borda limpa, B24), Óleo Real com luz simples no gesto (B14b), formas no Mask e espelho do 1.º dab (B40b), spray/aerógrafo parados iguais a 60 e 144 Hz (B22.2), pó do Pastel (B49).
8. **Relatório de pincéis**: `target/relatorio-pinceis/index.html` — os 46 presets que o alarme diz que mudaram; só o dono aprova.
9. **App parado**: minimizado/desfocado o CPU cai (B23); no Wayland real o «minimizado» depende do `winit` vendorizado.

## 7. ABERTO depois da integração

BACKLOG: **117 feitos · 16 parciais · 9 recusas medidas · 174 abertos**; auditoria: 57 feitos · 7 parciais · 41 abertos.
1. **Medição «depois» no app** (B08c): `corre_depois_b08c.sh` com a vez da placa segura para a matriz toda ([DEPOIS.md](../missao_4k/medicoes/DEPOIS.md)).
2. **Por começar, decisões do dono que mudam a aparência:** B33 papel sem repetição a 256 px (B33.2 recusado) · B35b–d controlos do Wet · B36 Oil Wash · B28 meia definição/Color mixing grandes · B34 aparência pelo tamanho da tela · B32 mistura · B31 luz do Impasto · B30 relevo×tinta.
3. **Por começar, técnicos:** B09 oráculos Krita/MyPaint · B18 camada como `Arc` · B27 fila da caneta (B27.1/.2) · B37 · B38 · B39 · B42 orçamentos · B43 determinismo · B44 sondas/portões 4K · B45 pente fino · B46 docs.
4. **Parciais:** B13 (grade de fluxo `frow_tiles`, jan da `ladeira_do_oleo`) · B14.1/B14.2 · B16b (Rewet híbrido) · B17.5 (dirty de 1 rect: 270× sujo) · B12b ligar o Ctrl+Z Wet 4K no app · B26.1 · B35.3/.5/.6/.7 · B06.7 · B19.4/.6/.12 · B23.6 (KWin virtual panica na WSI) · B27.1.
5. **Gates em falta** das mutações sobreviventes (§5).

## 8. Riscos para o integrador

- ⚠️ **Árvore suja de OUTRO agente:** 25 ficheiros `M` em `ph2d-tool-painter` (reformatação de testes e alguns `.rs`) não
  commitados e NÃO desta linha; o binário do smoke compilou com eles. O integrador decide (`git diff` antes de qualquer rebase).
- `winit` vendorizado: um `cargo update -p winit` noutra linha não o apanha; o `.typos.toml` e o `rustfmt.toml` ignoram-no.
- Gates de custo sob carga (candidatos a flake, [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md)): os 4 vermelhos da óptica Wet a
  load 26–34 (B12c: 23/23 sozinho); `a_mistura_do_wet_wash_mix_nao_custa_mais_que_sem_ela`; o tecto do Wet Wash Mix 1,3→0,75 medido
  só no perfil dev (B07) — conferir no `ci-test`.
- `quer_quadro`: família nova com produtor sem evento que não o declare congela até à entrada seguinte (ADR-0203).
- Alarmes de pincel vermelhos por desenho (46 presets): não regravar.

## 9. Onde está o mecanismo

[BUGS_painter.md](../BUGS_painter.md) #86–#113 · [COMO_ACELERAR_UM_PINCEL.md](../Pinceis/COMO_ACELERAR_UM_PINCEL.md) §7–§8 ·
[ADR-0202](../../architecture/decisions/0202-o-compositor-cpu-e-os-kernels-espaciais-correm-no-pool-do-rayon.md) ·
[ADR-0203](../../architecture/decisions/0203-o-app-desenha-a-pedido-e-o-winit-vendorizado-diz-a-janela-oculta-no-wayland.md) ·
[BACKLOG](../missao_4k/BACKLOG.md) · [BACKLOG_AUDITORIA](../missao_4k/BACKLOG_AUDITORIA.md) · [medições](../missao_4k/medicoes/ANTES.md) ·
[diagnóstico 100 ms](../missao_4k/medicoes/DIAGNOSTICO_100MS.md) · `crates/ph2d-app-host/vendor/winit/VENDOR.md`.
