# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, o código mau do compilador no bake (2026-10-11, prompt 21)

## As regras (R1–R9, [PROMPT_LINHAS_RAPIDAS §2](../../IntegracaoMultiAgente/PROMPT_LINHAS_RAPIDAS_2026-10-11.md))

- R1 janela curta: ~150 chamadas ou a onda fechada ⇒ commit, handoff ≤ 8 KB, PARE.
- R2 edite pelo `Edit` (novo = `Write`); script só para a MESMA troca em N ficheiros, com `assert` de contagem.
- R3 edição e teste nunca no mesmo comando: `Edit` → `cargo-check-narrow.sh` (⚠️ cego a `tests/it`: use
  `cargo check -p ph2d-tool-painter --test it`; e para os testes da lib `--lib --tests`) → teste.
- R4 no laço, teste por NOME; a suíte e o `nextest-impacted.sh` 1× no fecho, pelo `verificador`.
- R5 nada de `until … sleep`: > 2 min = `run_in_background`. R6 chamadas independentes na MESMA mensagem.
- R7 doc > 40 KB lê-se por `grep -n` + `Read` com offset. R8 escreva menos.
- R9 nenhum teste da suíte normal passa de 10 s em `dev`. Cura: (a) `opt-level = 2`; (b) fixture menor que
  ainda morda (prove com mutação); (c) só `diag_*`/`sonda_*`/`measure_*` vão para `#[ignore]`. ⛔ Lei nunca.
- Comando pesado por `bash scripts/ph2d-run.sh …`. Fish/zsh: listas de caminhos num `bash -c '…'`.

## §1 — Coordenadas

Base `edab70c9f` (o prompt 21; `main` = `8f20c60de`, não andou). Um commit desta janela. Sem push.

## §2 — O `paper_rows`: código mau do LLVM 21–22, curado no laço (FECHADO)

**Reprodutor mínimo** (sem `rayon`, 3 linhas de laço, `rustc` puro):
[`reprodutor_llvm_volta_mascarada.rs`](../NewFeaturesWetPaint/ferramentas/reprodutor_llvm_volta_mascarada.rs).
`#[inline(never)] fn kernel(y, row, tile)` com `tile[ty + ((x as i32 − 1) & 511) as usize]`:

| toolchain (LLVM) | o2 + overflow-checks | o3 + oc | o2 sem oc |
|---|---|---|---|
| 1.92 (21) · 1.94 (21.1.8) · 1.95 (22.1.2) · 1.98 (22.1.8) | ✗ 7 448 / 373 464 | ✗ | ✓ |
| nightly 1.100 (23.1.1) | ✓ | | |

Num projecto com perfis (cgu 16): `bad` (o2 + `lto="off"` + oc) ✗ · `lto=false` ✗ · `cgu=1` ✗ · `dev` o2 ✗ ·
`release` ✓. ⇒ a condição verdadeira é **o2+ com `overflow-checks` e o `y` opaco no laço**; `lto`/cgu/`rayon`
só decidiam, no motor, se o fecho era inlinado onde o `y` tem faixa conhecida (por isso o `dev` da crate passava).

**O código gerado** (`--emit=llvm-ir`, fecho paralelo, as duas configurações): nas duas o laço é vectorizado
(8 × f32, `load <4 x float>` ×2 contíguos a partir de `tile[ty | ((x+511) & 511)]`), e quem decide é a
verificação em tempo de execução (`vector.scevcheck`). Sem oc ela só entra com 1 iteração (efeito: nunca).
Com oc (o laço ganha o ramo de pânico de `x as i32 − 1` e a contagem fica limitada a 2³¹) a verificação
testa `(row ^ −512) < n` — o overflow de `row + 511 + x` em 32 bits, **sem a máscara**. Resultado: em `x = 0` o
índice é 511 e os 7 seguintes saem da linha seguinte do ladrilho; o mesmo em `x = 512`. É exacto o que o
portão via: `(dtx, dty) = (0, +1)` em 7 413 células, `x ∈ 1..7 ∪ 513..519` de cada linha.

**Issue:** nada com esta forma no rust-lang/rust nem no llvm-project (procura 11/10); parentes: llvm#212744
(flag no-wrap velha omite verificação), llvm#175729 (SCEV sem flags por uso). O LLVM 23 acerta. O reprodutor
está pronto para o dono decidir se reporta (⛔ nada saiu desta máquina).

**A cura (mecanismo):** [`bake_paper_rows`](../../../crates/ph2d-wet-paint/src/paper.rs) não tem índice
mascarado por célula: a célula 0 é o último texel da linha do tile, e da 1 em diante a linha repete-se em
troços de 512 (`copy_from_slice`). A mesma lei (o portão compara com o oráculo independente do laço de antes).
Prova: `paper_rows` 3/3 e os portões ao byte (`a_miniatura_de_fabrica…`, `com_as_opcoes_desligadas…`,
`smooth_edges_no_zero…`) verdes em **`ci-test`, `dev` e `release`**; `os_outros_meios_nao_mudam` na cerca (§6).
Antes reprovava 100/100 no `ci-test` (o vermelho visto). No reprodutor as três formas acertam em todos os
perfis: cópia de troços · `wrapping_sub` · máscara em `usize`.

## §3 — A mesma forma noutros sítios

- **Censo do código gerado** (`--emit=llvm-ir`, `and … 511|127|255|1023` DENTRO de um `vector.body`; controlo:
  acusa o fecho mau do reprodutor): `ph2d-wet-paint` `ci-test` e `dev` (49 laços vectorizados) → **zero**;
  `ph2d-tool-painter` `ci-test` (102) e `dev` (139) → **zero**. Os outros `wrap_mask` (`paper.rs` gerador/box-blur/estatística, `brush.rs` cerdas)
  são somas `f64` ou escritas condicionais: não vectorizam.
- **Portões série × paralelo:** todos os 26 usos de `walk_*` têm um, menos o **`sedimenta`** (Heavy Edge).
  Agora: porta `solver::sedimenta_rows(…, mode)` e o portão
  `parallel_rows::the_parallel_sedimentation_is_the_serial_one_to_the_byte` (linear e K–M, gravidade nos
  dois eixos, poça de DUAS cores; precondição: o passe muda `susp` e `susp_rgb`). O `diff` do ficheiro passa
  a comparar `susp_rgb` (vale para os 8 portões irmãos; todos verdes). Mutação: linha errada a meio da faixa
  na 1.ª e na 2.ª caminhada ✓ sangra · rota paralela esquece só a cor ✓ (nomeia `susp_rgb`) · passe sem cor
  ✓ (precondição); ⚠️ linha errada na linha 3 da faixa sobrevive (é orla seca: nada a mudar).

## §4 — ⛔ Recusas MEDIDAS

| proposta | medida | veredicto |
|---|---|---|
| `[profile.ci-test.package.ph2d-wet-paint] opt-level = 1` | passa, mas o código mau continua no `dev` o2 | silenciar |
| trocar o toolchain | só o nightly (LLVM 23) acerta; nenhum stable ainda | decisão do dono, não troquei |
| `lto = false` no `ci-test` | o reprodutor erra também com `lto = false` e cgu 1 | não é a causa |

## §5 — O que NÃO foi tocado

O R9 que sobra (camadas 44,7 s · largura 24,7 s · mídias 14,8 s; §4 do handoff VELOCIDADE_2) · o relevo que some
ao trocar de pincel · o óleo longe do Rebelle · a tinta/caneta (dono) · o Staining com K–M.

## §6 — Fecho

Bateria (agente `verificador`, em série, `load` 9–89): cerca ✓ (`os_outros_meios_nao_mudam` VERDE) ·
`ph2d-wet-paint` 149/149 em `dev` e em `ci-test` · `ph2d-tool-painter` lib + `--test it` 1572/1572 em `dev` (248 s)
e em `ci-test` (239 s) · censos 134 · 237 · 17 ✓ · `ph2d-app-painter` 103/103 · clippy `-D warnings` ✓ · fmt ✓
(depois de `cargo fmt`: três quebras de linha nas edições desta janela) · typos ✓ · tecto 700 ✓ (390 · 201 · 368 ·
657) · `nextest-impacted` 17 641/17 642 em 367 s: **o `paper_rows` passa**; a única falha é
`ph2d-app-light2d::it o_interior_do_opaco::o_rebote_nao_depende_da_oitava` (stderr: `Out of Memory` da placa sob a
suíte cheia), sozinha em `ci-test` 3/3 a `load` 20–24 — flake de carga de OUTRA linha, ainda fora da tabela do
`FLAKES_DE_CARGA.md` (registe quem é dono da Luz 2D). Perfil do laço: paralelismo 1,12 → 1,11 · test:check
2,4 → 2,6 × · Edit 50 → 46 % · contexto/passo 245 → 282 mil. ⇒ **pronta para o `main`** no que é desta linha.

## §7 — Depois do fecho (a mesma janela, ordens do dono)

Spacing do Tuning → `Hidden` (8 × 2 = imagem idêntica; 0,5 = 0,1 %) · os presets do óleo com os valores do dono
(`receitas_de_fabrica.rs`, `Receita.spacing`) e a `midias_de_fabrica` sem as três métricas que os reprovavam ·
Bugs #48 e #49 registados · pranchas `comparar_oleo/` por `ferramentas/prancha_comparar.sh <dir dos midia_*.ppm do
PH2D_SONDA_OUT da midias_de_fabrica> <saída> "<preset>|<foto>|<título>"`. Detalhe e tarefa:
[prompt 22](../NewFeaturesWetPaint/22_PROMPT_janela_relevo_e_verde.md).
