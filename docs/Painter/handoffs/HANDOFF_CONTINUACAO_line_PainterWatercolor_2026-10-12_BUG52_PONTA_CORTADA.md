# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, o Bug #52: a ponta da Shape de imagem cortada (2026-10-12, prompt 25)

## As regras (R1–R9, [PROMPT_LINHAS_RAPIDAS §2](../../archive/integracao-jornadas/PROMPT_LINHAS_RAPIDAS_2026-10-11.md))

- R1 janela curta: ~150 chamadas ou a onda fechada ⇒ commit, handoff ≤ 8 KB, PARE.
- R2 edite pelo `Edit` (novo = `Write`); script só para a MESMA troca em N ficheiros, com `assert` de contagem.
- R3 edição e teste nunca no mesmo comando: `Edit` → `cargo-check-narrow.sh` (⚠️ cego a `tests/it`) → teste.
- R4 no laço, teste por NOME; a suíte e o `nextest-impacted.sh` 1× no fecho, pelo `verificador`.
- R5 nada de `until … sleep`: > 2 min = `run_in_background`. R6 chamadas independentes na MESMA mensagem.
- R7 doc > 40 KB lê-se por `grep -n` + `Read` com offset. R8 escreva menos.
- R9 nenhum teste da suíte normal passa de 10 s em `dev`; só `diag_*`/`sonda_*`/`measure_*` vão para `#[ignore]`.
- Comando pesado por `bash scripts/ph2d-run.sh …`. Fish/zsh: listas de caminhos num `bash -c '…'`.

## §1 — Coordenadas

Sobre `6ea40b6f96` (o `main` não andou: merge-base `8f20c60de2`). Um commit desta janela. Sem push.

## §2 — O que mudou: UMA porta para «até onde chega a ponta»

A ponta de uma Shape de IMAGEM é `|tex| ≤ 1` no referencial dela (`shape_value`); rodada chega a `√2·r`, com
Size < 1 passa de `r`. Todo núcleo percorria `centro ± raio` e o assado só `[-1, 1]²`. Agora:

- **A lei:** `texture::image_tip_extent(s, basis)` — os 4 cantos desfeitos Offset/Size, a rotação da Shape
  (`M⁻¹`) e a pegada do dab (`FootprintDeform::unapply_linear`; curva ⇒ `extent_at(ρ)`), em raios por eixo.
  `None` para Shape procedural (o falloff manda) e Flow (ladrilha). `image_tip_radius` = o disco invariante.
- **Núcleos por-pixel:** `tip::dab_extent(base, shape, basis)` — `dab.rs` (`max` com a pegada), `blit_canvas_cached`,
  `accumulate_shape_layer_rgba`, o lote dinâmico, o relevo e a escultura (`height::tip_reach` →
  `BrushSpec::dab_tip_reach`).
- **Assados:** `tip::stamp_domain` (`≥ 1`) — `StampMask`/`ColorStampMask` ganham `reach`; o assado percorre
  `[-reach, reach]²` e quem amostra usa `radius · reach` (os 3 blits + os 4 acumuladores); o `axis_taps` fica igual
  (o quadrado unitário é o do CARIMBO). A resolução segue o domínio: `mask_size_for(max_r · stamp_domain)`.
- **Regiões do tool:** `BrushSpec::tip_reach()` (invariante à rotação) em `caixa_do_lote`, `dab_batch_region`,
  `dabs_bounds`/`caixa_das_camadas` (novo argumento `reach`).
- **Wet Paint** (o mesmo corte, achado na varredura): o `trail::Dab` do motor ganha `reach` (o caminho com forma
  percorre `r · reach`) e `window_reach` (a janela do rasto cabe o máximo do traço); o hospedeiro põe
  `Engine::silhouette_reach` por dab (`wetpaint/dab_route.rs`). O motor próprio deixa `[1, 1]`.
- **0° / Size 1 / sem Offset ⇒ `[1, 1]` ao bit** em todas as portas (`× 1` e `/ 1` exactos).

## §3 — Portões

- `ponta_cortada` (tool, 7 + 2 sondas): os dois `diag_` VERMELHOS viraram leis (`a_ponta_rodada_nao_e_cortada`
  — Jitter E Angle estático —, `size_menor_que_um_nao_corta_a_ponta_nas_duas_rotas`) · CONTROLO
  `a_zero_graus_size_um_a_ponta_e_a_caixa` (10/10/400 nas duas rotas) · re-carimbo da Linha e do Drag Dot sem
  rasto · desfazer na fronteira de um bloco de 128 · `o_wet_paint_nao_corta_a_ponta`.
- `tip_kernel_tests` (motor): o oráculo MENTIA (`|p − c| ≤ r` nos eixos da tela = a caixa que cortava; a
  fixtura a 0° era o único caso em que coincide). Agora `in_tip` é a definição por trigonometria (pegada,
  Flatten, Angle, Size, Offset), e `no_kernel_cuts_or_overflows_a_rotated_or_scaled_tip` corre 10 kernels ×
  5 casos (rodado · Size 0,6 · Offset · Jitter 45° · Flatten 0,4 + Offset + Jitter 30°): por-pixel exactos,
  assados ±1 px.
- **O Angle estático** (pedido do prompt) cortava igual: `cache=true 45°` dava 356 como o Jitter.

Medido pela sonda (raio 10, quadrado cheio): a 45° a ponta chega a 14 (antes 10); Size 0,6 dá 1 156 px nas
duas rotas (antes 441 · 400); Wet Paint Size 0,6: alcance 30–39 contra 18 (antes = a ponta de Size 1).

## §4 — Mutação (5 + 2 re-corridas)

| mutação | sangra |
|---|---|
| `dab_extent` devolve `base` | kernel gate · rodada · Size 0,6 |
| `stamp_domain` = 1 | kernel gate (assados) · rodada/Size `cache=true` |
| `image_tip_extent` sem a rotação | kernel gate (Angle 30°, Offset) · rodada |
| `tip_reach` = 1 | ⚠️ sobreviveu à 1.ª bateria ⇒ portão novo: o Drag Dot sangra |
| `unapply_linear` com o seno trocado | ⚠️ sobreviveu (ponta simétrica: `+φ` e `−φ` dão a mesma caixa) ⇒ caso assimétrico com Flatten: sangra |
| Wet: `silhouette_reach = [1, 1]` | a sonda dá a mancha da Size 1 em todos os casos ⇒ `o_wet_paint_nao_corta_a_ponta` |

O controlo a 0° ficou verde em todas. ⚠️ **`desfazer_apaga_a_ponta_alargada_inteira` NÃO sangra com `tip_reach` = 1**,
nem na fronteira de um bloco: o diário tem outra rede (por ver qual); fica como lei do produto, não como prova
do `dabs_bounds`.

## §5 — Custo (medido, `measure_o_custo_do_alcance`, `--release`, A/B no mesmo processo, mínimo de 7 blocos, `load` ~2)

Rota dinâmica, raio 40: **0°/Size 1 114,3 → 113,7 µs (1,00×, a mesma caixa)** · Jitter 45° 107,5 → 109,8 µs
(1,02×: a caixa dobra, os píxeis fora da ponta saem no `|tex| > 1`) · Size 0,6 112,9 → 157,7 µs (1,40×: a
ponta É 2,8× maior — tinta real). O custo só cresce onde a ponta cresce.

## §6 — Fecho

Bateria (`verificador` + re-corrida, `load` 10–52): fmt ✓ · clippy `-D warnings --all-targets` (brush, wet-paint,
tool) ✓ · `ph2d-wet-paint` lib 52 + `it` 99 (o fingerprint do motor incluído — o caminho próprio deixa `[1, 1]`)
✓ · portões de doc do `ph2d-editor-core` 33/33 ✓ · `nextest-impacted` **17 275/17 276**: o vermelho era meu
(`painter.rs` do motor a 701 > 700 com o campo novo; cura por CORTE do comentário, re-corrido verde). Na 1.ª
bateria o `it` do motor não compilava (5 `Dab` à mão nos testes sem `reach`/`window_reach`) — curado. R9: o
mais lento dos novos 0,115 s.

## §7 — Aberto

- **Watercolor** (a aguada antiga) recorta a água num DISCO por lei (`dn ≥ 1` em `watercolor_accum_cor.rs`);
  a ponta de imagem lá é cortada pelo círculo. Não tocado: a mídia está congelada (dono 07/10, «só o Wet
  Paint evolui»).
- O diário de desfazer: descobrir a segunda rede que torna o `dabs_bounds` invisível ao portão (§4).
- Flow continua com a caixa do raio (é a largura do traço; o comprimento ladrilha) — por desenho.

## §8 — Smoke ao dono

`cargo run -p ph2d-host-desktop --profile smoke` · Digital · Shape ▸ imagem QUADRADA cheia (Alpha From Image)
· Stroke ▸ Jitter ▸ Rotation no máximo · Shape ▸ Size 0,6 · um traço: cada carimbo é o quadrado INTEIRO, rodado,
maior que o círculo do cursor, sem lados retos de corte · Ctrl+Z apaga tudo · o mesmo no Wet Paint.
