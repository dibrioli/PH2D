# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, a família C contra o Rebelle (2026-10-12, prompt 23)

## As regras (R1–R9, [PROMPT_LINHAS_RAPIDAS §2](../../IntegracaoMultiAgente/PROMPT_LINHAS_RAPIDAS_2026-10-11.md))

- R1 janela curta: ~150 chamadas ou a onda fechada ⇒ commit, handoff ≤ 8 KB, PARE.
- R2 edite pelo `Edit` (novo = `Write`); script só para a MESMA troca em N ficheiros, com `assert` de contagem.
- R3 edição e teste nunca no mesmo comando: `Edit` → `cargo-check-narrow.sh` (⚠️ cego a `tests/it`: use
  `cargo check -p ph2d-tool-painter --test it`) → teste.
- R4 no laço, teste por NOME; a suíte e o `nextest-impacted.sh` 1× no fecho, pelo `verificador`.
- R5 nada de `until … sleep`: > 2 min = `run_in_background`. R6 chamadas independentes na MESMA mensagem.
- R7 doc > 40 KB lê-se por `grep -n` + `Read` com offset. R8 escreva menos.
- R9 nenhum teste da suíte normal passa de 10 s em `dev`. Cura: (a) `opt-level = 2`; (b) fixture menor que
  ainda morda (prove com mutação); (c) só `diag_*`/`sonda_*`/`measure_*` vão para `#[ignore]`. ⛔ Lei nunca.
- Comando pesado por `bash scripts/ph2d-run.sh …`. Fish/zsh: listas de caminhos num `bash -c '…'`.

## §1 — Coordenadas

Rebase sobre o `main` `9ea782138` (21 commits; um conflito de ADJACÊNCIA no `CLAUDE.md` §5: a entrada do Painter
é da linha, a do Vector do `main` — resolvido assim). Um commit desta janela. Sem push. Perfil do laço ANTES
(10 sessões): paralelismo 1,22 · test:check 3,6 × · Edit 48 % · contexto/passo 347 mil · início 63 mil.

## §2 — A régua (escrita nas pranchas)

[`ferramentas/mede_familia_c.py`](../NewFeaturesWetPaint/ferramentas/mede_familia_c.py) sobre a cena da bancada
(1000×640): `lado` (largura do vermelho DE PÉ em D4/D6) · `alto` (do azul deitado A2–F2) · `tinta` (mediana do
miolo, ABSOLUTA) · `falhas` · `estrias` · `grao` (diferença entre vizinhos AO LONGO do traço: sal × liso). As
pranchas levam a régua numa faixa por baixo: [`comparar_oleo/janela23_antes/`](../NewFeaturesWetPaint/comparar_oleo/janela23_antes/)
(hoje, depois de #48–#50) e [`janela23_depois/`](../NewFeaturesWetPaint/comparar_oleo/janela23_depois/) (o aceite).

A varredura: `sonda_midias::as_midias_com_os_knobs_de_hoje` ganhou `"base"`/`"raio"`/`"opacity"` no JSON de
`PH2D_SONDA_RECEITAS` — parte do preset de FÁBRICA e pinta a cena ao raio do portão; os três controlos deram a
cena de fábrica AO BYTE. ~180 variantes em 10 rodadas, um build, 1–60 s cada (`PH2D_SONDA_SECA=1320` = seco).

## §3 — O que mudou (aceite do dono 12/10: «Oil Bristle, Express Oil»)

| | lado | alto | tinta | falhas | estrias | grão |
|---|---|---|---|---|---|---|
| Rebelle `oil` | 13 | 41 | 0,91 | 0,024 | 0,05 | 0,003 |
| Oil Bristle antes → agora | 38 → **12** | 40 → 40 | 0,892 → 0,909 | 0 → 0 | 0,107 → 0,06 | 0,021 → 0,005 |
| Rebelle `express` | 34,5 | 40 | 0,772 | 0,048 | 0,137 | 0,006 |
| Express Oil antes → agora | 41,5 → 39,5 | 42 → 40 | 0,906 → **0,776** | 0,023 → 0,024 | 0,095 → 0,186 | 0,02 → 0,027 |

- **Oil Bristle = pincel CHATO**: `dab_flatten` 0,68 (= 1 − 13/41) e `dab_angle_deg` 90, FIXO na tela (sem rake:
  o Bristle Flat do Rebelle é largo deitado e fino de pé, o S tem a mesma altura toda); `felt` 0 → 0,05 (sem
  ele a lâmina fina parte o deitado em listas e encolhe-o — 29 px a 0,65). Campo novo `Receita::achatado`.
- **Express Oil**: `pigment` 750 → 60, `bristle_size` 2 → 4, `water` 0,09 → 0. Ver §4 (porquê 60 e porquê sem água).
- **Acrylic Round — RECUSADA pelo dono** (fica como estava). A proposta medida: `bristleSize` 0,5 → 2,0 dá corpo
  cheio (alto 22 → 42 contra 39; tinta 0,49 → 0,91 contra 0,90; falhas 0,29 → 0,045 contra 0,049).
- Presets regerados: só os dois mudaram (os outros dez ao byte); as tabelas de largura deles também.
- Portão: `cada_preset_parece_a_sua_midia` ganhou `Chato` (lado/alto, vale) no Oil e `Tinta` (FAIXA ±0,048) no
  Express. Mutação «preset de antes no disco» (cada um sozinho): Oil 0,950 e Express 0,906 sangram; reposto e
  conferido por sha256. ⚠️ A Tinta é faixa e não vale: o controlo (Wet Round, aquarela) dá 0,18 — do lado CLARO —
  e o defeito era o opaco; o vale aprovava o Express antigo (medido).

## §4 — ⛔ Recusas MEDIDAS (não reconstruir)

| Tentativa | Medido | Porquê não |
|---|---|---|
| Express: pigmento 250–400 (cerdas 2) | tinta 0,80, grão 0,07 | claro mas SAL — não é o liso dele |
| Express: água 0,2–1,0 | tinta 0,81–0,90 | só alarga o traço (até 52 px); não clareia |
| Express: knobs de papel/grão a 0 (`visualGrain`, `paperVisibility`, `ext*Gran*`, `paperContrast`…) | bytes iguais | não chegam a esta cena (só o `emboss` mexe) |
| Express: 200–450 cerdas (tufos separados) | falhas 0,09–0,24 | o carácter dos tufos, mas papel a mais em qualquer raio 21–28 |
| Express: `intensity` 0,6 | tinta −0,05 | quase nada |
| Oil: achatar sem feltro | alto 29 a 0,65 · listas | o deitado encolhe e parte-se |
| Oil: `intensity` 2 + `hardness` 1 | 14/41, estrias 0,04 | igual ao aceite, com dois knobs a mais |

Mecanismo da claridade: com o Spacing 0,01 do dono cada célula leva ~100 carimbos, por isso 250 por carimbo
satura; a claridade lisa pede ~60. A água fazia o pouco pigmento assentar nos poros (grão 0,065 → 0,027 sem ela).

## §5 — ABERTO (a próxima janela)

- **Express Oil**: os tufos dele têm riscas brancas NÍTIDAS; os nossos são suaves (estrias 0,186 contra 0,137,
  grão 0,027 contra 0,006 — o mesmo grão dos nossos opacos aprovados). Os discos das cerdas têm ombro smoothstep;
  tufos nítidos seriam motor (as pontas agrupadas em tufos), não knob.
- **Cerdas de duas espessuras** (a pista das fotos): o tile JÁ tem duas populações (`brush.rs:23-35`: FINE 74 %,
  sub-gate, sem depósito; CORE 26 %), ambas escaladas pelo mesmo `bristleSize` — nenhuma composição dá dois fios
  visíveis. Não construído.
- **Oil Bristle**: o feltro limpa as pontas esfiapadas do Rebelle (as dele são franjadas e quadradas; as nossas
  são a meia-elipse da pegada).
- Do prompt 23, não tocados: o realce do Emboss por diferença central · «Keep volumes» · o R9 que sobra (camadas
  44,7 s · largura 24,7 s · mídias 14,8 s; a `midias_de_fabrica` mediu 25–39 s a `load` 29–46) · famílias A e B ·
  tinta/caneta e Staining K–M · os flakes de carga da máscara.

## §6 — Fecho

Bateria (`verificador`, em série, `load` 11–49): cerca ✓ · `ph2d-wet-paint` 151/151 `dev` e `ci-test` ·
`ph2d-tool-painter` lib 1566/1566 e `it` 9/9 em `dev`; em `ci-test` a lib 1565/1566 — o vermelho é o flake de
carga `the_mask_stroke_cost_does_not_follow_the_canvas` (1,11 × 2,59 ms a `load` ~30; sozinho a 12,6 verde), `it`
9/9 · censos 134 · 237 · 17 ✓ · `ph2d-app-painter` ✓ · fmt (aplicado no fecho, só espaços) · clippy `-D warnings`
✓ · typos ✓ · tecto 700 ✓ (maior: 437) · `nextest-impacted` 17 651/17 651 · binário `smoke` ✓.
Perfil do laço (10 sessões, antes → depois): paralelismo 1,22 → 1,20 · test:check 3,6 → 4,0 × · Edit 48 → 47 % ·
contexto/passo 347 → 357 mil. ⚠️ O `test:check` subiu: as 10 rodadas da sonda são `cargo test` (a varredura vive
num teste ignorado) — uma varredura por JSON não precisa de `check`, e a régua conta-a como teste.

## §7 — Depois do fecho (12/10): o Bug #51 ABERTO e medido

Achado do dono: imagem no Shape com «Use Texture Colors» ⇒ o Color não tinge e o Randomize Color morre.
Mecanismo, os três portões vermelhos (`tests/textura_tinge.rs`, `--ignored`, controlo verde) e a cura desenhada
em [BUGS #51](../BUGS_painter.md). A cura é a próxima janela:
[prompt 24](../NewFeaturesWetPaint/24_PROMPT_janela_textura_tinge.md).
