# Handoff de continuação — Luz 2D, W5z: os níveis da holográfica em TEXTURA (2026-10-16)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `0aab2c1cb6`, por cima da W5y ([handoff](HANDOFF_CONTINUACAO_W5y_2026-10-16_O_RESTO_DO_HIGH_E_O_BERCO_FRAGIL.md)),
> da W5x ([handoff](HANDOFF_CONTINUACAO_W5x_2026-10-15_O_BERCO_DA_HOLOGRAFICA.md)) e das anteriores, todas não
> integradas. **A W5x ainda espera o smoke do dono; não integrar.** Commits da W5z: `c129875090` (régua, texturas,
> gate de bits) · `c54563b292` (nasce uma vez com texturas) · o deste handoff. Item 1 do §4 da W5y (blindar o
> berço), caminho **(A)**. Placa RTX 5060 Ti, `gpu-allocator` 0.28, `wgpu` 29.0.4.

## 1. A régua que vê a compressão de uma textura

`sonda_compressao_das_texturas` (`ph2d-app-light2d/tests/it`, `#[ignore]`, placa): lê na ordem dos LADRILHOS
(faixas de 64 B × 8 linhas = o GOB; ou × 128 linhas), por `texture_2d` e por `texture_storage_2d<…, read>`;
calibrada contra os buffers da W5x e contra si própria (2 processos, iguais ao ±1 %; GB/s):

| recurso (48 MiB salvo dito) | vácuo/zero por shader | ruído | ordem |
|---|---|---|---|
| buffer fresco, `vec4` · `vec2` | 606 · 553 | 606 · 551 | — |
| buffer HERDADO (berço), `vec4` · `vec2` | 2 009 · 835 | 610 · 540 | — |
| textura `Rg32Uint` armazém, amostra · armazém | **848 · 859** | 424–573 · 571 | ladrilhos |
| a mesma, em ordem de LINHA | 544 | 516 | linha |
| textura `Rgba32Uint` | 1 548 · 1 568 | 427–614 | ladrilhos |
| `Rg32Uint` 16 MiB (no L2: o teto de 8 B) | 732 · 742 | 727 | ladrilhos |

**Premissa de (A) confirmada:** uma textura de armazém NOVA com o vácuo comprime e lê-se ao ritmo do buffer
comprimido para o mesmo elemento de 8 B (o teto é a taxa de texels, não a DRAM). **Em ordem de linha a vantagem
some** (544 ≈ fresco): o índice 1D → 2D tem de ir por ladrilhos 8 × 8. `SondaDeMemoria::ms_textura` (o
`MEMORIA` do app) passou à ordem dos ladrilhos (a de linha era a régua cega da W5y §2).

## 2. A cura (`c129875090`, `c54563b292`)

- `Config::hrc_em_texturas` (padrão `true`; `PH2D_LIGHT2D_HRC_TEXTURAS=0` no app = os buffers e o berço).
  Efetivo: `Config::hrc_niveis_em_textura()` — há holográfica, cabe (`hrc::LARGURA_DOS_NIVEIS`² = 64 M
  entradas por nível) e o `modulo_rc` das réguas, se houver, traz o prelúdio de textura.
- Os T, o par R e os 3 vazios são **texturas da `PassList`** (`hrc::texturas`, `Rg32Uint`,
  `STORAGE_BINDING|TEXTURE_BINDING`, 8192 de largura × 8·k linhas, os mesmos rótulos `luz2d/buf/hrc_*`): nascem
  com as outras texturas; o G5 conta-as. O acumulado continua buffer (é denso: lê ~420 sempre).
- WGSL: as ligações 5–9 e cinco acessores (`ler_t_a`, `ler_t_b`, `ler_r_cima`, `escrever_t`, `escrever_r`) vêm
  de um prelúdio no `rc!` (`hrc_niveis_buffer.wgsl` · `hrc_niveis_textura.wgsl`, `texel_do_nivel`); a tabela
  `RC[aritmética][armazenamento][textura]`; `fonte_do_rc(a, b, niveis_em_textura)`.
- Rust: módulo irmão `solver_hrc_niveis.rs` (`Nivel::{Buffer, Textura}`, o layout 5–9). `solver_hrc.rs` 666/700.
- Com texturas a holográfica nasce UMA vez (`duas_vezes = hrc_nasce_duas_vezes && !em_textura`): medido igual (§3).
- **Bits:** gate novo `os_niveis_em_textura_dao_o_mesmo_bit` (3 degraus × 3 despachos — 3D/colunas,
  1D/colunas, 1D/linhas — × 2 cenas, ao bit, com controlo de que cada lado aloca o que diz): verde.

## 3. Números do app (cena 5, High, `RENASCE=2`, 4 000 quadros, c2 = mínimo da 2.ª janela por geração, ms)

Rodadas intercaladas e rodadas, 2 processos por caso, a placa só com o kwin antes de cada processo
(registado; a 1.ª esperou 603 s por um app ocioso da UIUX). Medianas por processo (máx entre parênteses):

| caso | buffers + berço (W5x) | **texturas** | texturas, nasce 1× |
|---|---|---|---|
| 1920 uso normal | 8,86 (9,44) · 8,98 (9,47) | **8,66 (8,84) · 8,67 (8,77)** | 8,67 (9,05) · 8,67 (8,79) |
| 1920 `SOZINHA=1` | 16,37 · 12,81 (berço falha) | **8,65 (8,68) · 8,66 (8,73)** | 8,65 · 8,68 |
| 2560 uso normal | 11,72 (12,94) · 11,85 (12,54) | **11,51 (12,71) · 11,57 (12,44)** | 11,59 (12,38) · 11,53 (11,79) |
| 2560 `SOZINHA=1` | 25,91 · 24,69 (berço falha) | **11,47 (11,91) · 11,41 (11,92)** | 11,51 · 11,46 |

Kill-criterion (≤ 11,5 / 8,8, os dois regimes): **passou**. Régua do app (`MEMORIA=lista`, 2560, 4 processos):
os 13 T e 2 R da vista leem 810–1 034 GB/s em todas as gerações (calibração `zeros_rg32` 1 029–1 063), com
`SOZINHA` 0 e 1 — comprimidos. Exceção: uma geração (g6) de um processo a 649–898 em TODOS os níveis por igual,
com testes de placa da PainterWatercolor a aparecer depois dele: contenção, não perda de compressão (essa
cairia nível a nível para ~570). As texturas do entorno (7–13 MiB) são pequenas demais para a régua (o
lançamento domina, como a de 4 MiB da sonda).

## 4. ABERTO, por ordem

1. **O resto das 1.ªs montagens do 2560** (W5y §2): picos de 12,3–12,7 numa geração, em buffers E em texturas —
   não é a compressão dos níveis. Agora há régua de texturas que vê a compressão: medir as texturas da lista
   (`MEMORIA=lista`) nas gerações lentas.
2. Simplificar: o berço e o nascer duas vezes só servem o caminho em buffer (comparação). Retirá-los é decisão
   de produto (perde-se o `HRC_TEXTURAS=0` como régua) — não feito.
3. Itens 2–3 do §5 da W5x.

## 5. ⛔ Recusas MEDIDAS (somam às da W5x §8 e da W5y §5)

| Recusa | Medida |
|---|---|
| Índice 1D → 2D em ordem de linha (`i & 8191, i >> 13`) | Lê 544 GB/s sobre vácuo comprimido (≈ buffer fresco): perde a compressão (§1). |
| `textureLoad` em ordem de linha como régua de textura | Cego: zeros e ruído iguais (§1; W5y §2). |
| (B) segurar os blocos do berço | Não necessário: (A) passa e não depende de ocupação. |
| Berço / nascer 2× com os níveis em textura | Neutro (§3). |

## 6. Para o integrador

Sem contrato congelado, sem schema. **API:** `fonte_do_rc` ganhou o 3.º argumento; campo público novo
`Config::hrc_em_texturas` (padrão `true`; literais exaustivos de `Config` precisam dele) e o método
`Config::hrc_niveis_em_textura`. Ficheiros novos: `solver_hrc_niveis.rs`, `wgsl/hrc_niveis_{buffer,textura}.wgsl`,
`tests/it/sonda_compressao_das_texturas.rs`. LOC: `solver_hrc.rs` 666 · `config.rs` 684 · `solver.rs` 690.
CLAUDE.md §5.1 Luz 2D: «Último:» aponta para este handoff.

## 7. Gate e smoke

**Gate de fecho** (W5x + W5y + W5z, régua = merge-base `0aab2c1cb6`, loadavg 19 → 110 por outras linhas):
`fmt` verde · `check --workspace --all-targets` deny verde · `clippy --workspace --all-targets -D warnings`
(`PH2D_SUITE=fecho`) verde · `nextest-impacted`: **11 754 corridos, 11 753 verdes, 1 vermelho real** — o censo
HR-15 `every_word_this_family_shows_comes_from_the_string_table`: três literais de instrumento da W5y
(`5191d72654`) em `quadro_leituras.rs` (rótulos de depuração do wgpu e a etiqueta «calibração» do terminal; a
isenção antiga nomeava `quadro.rs`, de onde o código saiu). Cura: três isenções nomeadas, com o porquê; o censo
e `every_named_exemption_still_shelters_what_it_names` verdes. `a_luz_recua_quando_a_placa_recusa_a_memoria`
pelo nextest isolado: **verde** · `os_niveis_em_textura_dao_o_mesmo_bit` · G5 · caps de LOC
(`workspace_src_files_under_loc_cap`, `shell_files_respect_hr18_loc_cap`) · `check-standalone-optional` ·
`cargo machete`: verdes. A placa esteve com a `line_motion_value` (cadeado recusou 300 s; repetido, sem forçar).

Smoke do dono: o da [W5x §7](HANDOFF_CONTINUACAO_W5x_2026-10-15_O_BERCO_DA_HOLOGRAFICA.md)
continua válido; a W5z muda só a memória (mesma imagem, mesmo número ou melhor). Comparar com
`PH2D_LIGHT2D_HRC_TEXTURAS=0` à frente do comando.
