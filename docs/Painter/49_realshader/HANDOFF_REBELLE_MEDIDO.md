# HANDOFF — «O que o Rebelle faz melhor, medido» (linha PainterWatercolor, 30/10)

Leitor: a próxima LLM. Estado vivo: [ESTADO_REBELLE_MEDIDO.md](ESTADO_REBELLE_MEDIDO.md). Prompt: [PROMPT_JANELA_REBELLE_MEDIDO.md](PROMPT_JANELA_REBELLE_MEDIDO.md).
Detalhe: [A1_oleo_na_vista.md](A1_oleo_na_vista.md) · [B_nucleos.md](B_nucleos.md) · [SONDAS_C_f32_v3.md](SONDAS_C_f32_v3.md).

## 1. Missão e base
Base `341341b233`. Backup: ramo `backup/line-PainterWatercolor-antes-do-rebelle-medido-2026-10-30` + tag
`painter-antes-do-rebelle-medido-2026-10-30`. ⚠️ O doc do dono (artifact `QUaFiJcbRj1YCed8ttaU1L`) NÃO esteve
acessível a esta janela: os factos do Rebelle são os que o prompt transcreve.

## 2. Os quatro números

### A — nitidez e custo (óleo iluminado à resolução da vista, CPU)
`crates/ph2d-wet-paint/src/oleo_vista.rs`: altura por **B-spline cúbica interpolante C2** (pré-filtro de Unser,
z=√3−2), normal e hessiana ANALÍTICAS; cor/gloss/céu por Catmull-Rom preso. `s=1` = caminho de hoje **ao byte**;
`s<1` fora.
| régua (recorte 64×64, 400 %) | bilinear | Catmull-Rom r1 | B-spline |
|---|---|---|---|
| var. laplaciano | 54,64 | 135,24 (2,47×) | **178,58 (3,27×)** |
| pico período-célula (≤ 1,5) | 1,851 | 1,344 | 1,275 |
| outliers 6σ | 0,0289 | 0,0162 | 0,0203 |

O Catmull-Rom dava listras ao passo da célula (hessiana só C1): ver `capturas/a1_prancha.png`. As listras que
restam já estão no documento a 100 % (brilhos por fio).
Custo, 1 fio, ns/px (mín · med): hoje 1066·1201 · s=1 1061·1076 · s=2 1164·1215 · s=4 778·795 ⇒ 4K inteira ≈
6,4–8,8 s num fio. **A lei não é tempo real na CPU.** A GPU (W1 GPU na vista) é > 1 janela: o composite corre à
res. do documento e não conhece a câmara (zoom só no sprite, `ph2d-app-painter/src/painter_bridge.rs:83`; filtro
do preview não confirmado, `ph2d-render/src/image_filter.rs:14-19`).

### B — núcleos (wash 4096×2560, água 100 %, pigmento 85,4 %)
7,5 passos/s (128–133 ms; o produto pede 40/s) · 29,2 / 30,8 núcleos (mediana/pico), **28,9 úteis** (espera
rayon/futex 0,99 %) · 32/32 threads · FP escalar/128/256 = 37,5/62,5/0 % (Rebelle 25/47/28) · 428 B/px
(Rebelle 215–234). Passo: secagem→assenta ~46–55 % (≈350–420 ms em cada passo que seca, 1 em 6;
`sobre_branco_tapando` `assenta.rs:334` 11,1 %), advecção 33,7 % (~43 ms; `transport_rows` `advect_jacobi.rs:381`
10,9 %, `commit_rows` `:512` 8 %, `outflow_rows` `:314` 5,9 %), `pow` libm 2,9 %. Binário corrido FORA do
`ph2d-run` (o cgroup de 50 % falsificaria os núcleos).

### C.1 — f32 no solver (poça 4096²)
Advect paralelo 5,21→5,17 ms (−1 %), série −3,9 %. Desvio após 12 passos: sett máx 8,1 (de 294) em 1,36 % das
células; **cor INFINITA em 1–1,6 %** (1/want estoura em f32).

### C.2 — despacho v3
**Zero bits mudam** (Rust não funde a*b+c; `mul_add` exacto nos dois). Só a série ganha (−8,5 % advect / −1,4 %
drying contra a cópia); o rayon não herda o `target_feature` ⇒ no caminho paralelo do produto é ruído. Exige
`unsafe` (o workspace proíbe). A sonda compila com `--cfg sonda_v3 --cap-lints warn`.

## 3. Provas
- Testes por nome: `bash scripts/cargo-test-narrow.sh ph2d-wet-paint oleo_vista` (7 testes).
- Mutação: 10 mutantes; 6 sangraram de início; **M2/M6/M6a/M7 sobreviveram** ⇒ 3 testes novos (`28b374e391`:
  escala da curvatura, alinhamento de meio pixel, borda), vermelhos sob cada um.
- Prancha/custo: `bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo run --release -p ph2d-wet-paint --example prancha_oleo_vista [-- --mede]`.

## Verificação do fecho
- `cargo fmt -p ph2d-wet-paint --check` ✓ · `PH2D_SUITE=fecho cargo clippy -p ph2d-wet-paint --all-targets --profile smoke -D warnings` ✓
  (a sonda herda `explicit_counter_loop`/`manual_clamp` de `src/lib.rs:23-24` por ser cópia verbatim) · `check --examples` ✓.
- Por nome, verdes: `oleo_vista` (7) · `a_fenda_neutra_e_bit_identica_a_de_antes` · `oleo_real_campo` · `oleo_borda` · `oleo_real` ·
  `pintar_tudo_com_o_real_oil_e_a_imagem_de_antes_ao_byte` · `a_tela_depois_da_varredura_e_a_recomposicao_inteira_ao_byte` ·
  `o_cache_do_campo_e_a_recomposicao_de_raiz_ao_byte` · `diag_w3b` (6).
- Vermelhos ANTERIORES à missão (não tocados): `a_aguada_re_molha_o_oleo_soluvel`, fenda forte (54<70), Blender×Cycles — já no
  ESTADO J11 «vermelhos de antes»; `o_traco_seguinte_usa_o_depth_o_gloss_e_a_caixa_novos` — vermelho com `lib.rs`/`oleo_real.rs`/
  `render.rs` repostos em `341341b233` (conferido 30/10); `o_campo_nasce_com_o_oleo…` (`oleo_celula_tests.rs:140`, `size_of` 16≠12:
  o `soluvel` do J10 `84df135c02`; `grid/` sem diff desde a base). Vão ao fecho da LINHA, não a esta missão.

## 4. Commits
`b393430f66` (C) · `bdbb414984` (A.1 r1) · `6c690ae097` (A.1 B-spline) · `95cd9830c8` (B) · `182dfc661e` (B r2) ·
`28b374e391` (testes de mutação A.1).

## 5. Aberto
- **D**: Rebelle 4K × 20 pinceladas (`custo_facas.sh`), no ecrã real — espera aval do dono.
- **W1 GPU na vista**: passar a câmara ao composite + passe wgpu da lei `oleo_vista`, paridade por placa, K/N por
  placa; tabela de custo RTX/Radeon antes de qualquer tecto.
- Sombra no papel na vista; `s<1` (mip); `ratio ≠ 1`.
- A.1 **não está ligada ao app** (sem smoke no ecrã; a prancha é o artefacto).
- Secagem/assenta como alvo n.º 1 de velocidade da água (medido, não decidido).
- Smoke J11 e valores Re-wet/Drag/Solubility pendentes do dono (item à parte).

## 6. Duas perguntas ao dono
1. **f32 no solver?** Os números dizem: ganho ≤ 4 %, com defeito de cor infinita a curar primeiro.
2. **Despacho v3?** Zero bits mudam; ganho só na via série; exige `unsafe` + ADR.

## 7. ⛔ Recusas MEDIDAS desta janela
- Catmull-Rom para a ALTURA na vista: hessiana C1 ⇒ listras ao passo da célula (prancha r1).
- Régua só de variância do laplaciano: premeia artefacto — exige as anti-artefacto (pico da célula, outliers 6σ).
- f32/v3 «por velocidade» sem os números acima.

## D — Rebelle 4K
Tela 3840×2160 criada FORA do relógio das pinceladas, R=3 intercalado, RealShader ligado, ecrã real. Detalhe: [D_rebelle_4k.md](D_rebelle_4k.md).

| variante (×3) | span | CPU acima do ocioso | núcleos pico | placa mediana / máx |
|---|---|---|---|---|
| criação 4K | 1–2 s | 3,2–3,3 s | 2,95–3,44 | 4 % / 33–94 % |
| 20 pinceladas | 8 s | 9,1–11,4 s | 1,72–2,46 | 4 % / 22–25 % |

Resposta: **as pinceladas pesam ~3× a criação da tela 4K**. ⚠️ A régua de calma do script bate no tecto de 90 s quando o ocioso do Rebelle (~0,1 núcleo em 4K) encosta ao limiar — o tempo real saiu do traço de CPU por segundo.
