# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor`, prompt 53: os 11 pincéis novos de fábrica (2026-10-19)

> Leitor: o integrador e a próxima janela da linha. Números e tabelas: [53_medicao](../NewFeaturesWetPaint/53_medicao_os_11_pinceis.md)
> (não repetidos aqui) · [prompt 53](../NewFeaturesWetPaint/53_PROMPT_os_11_pinceis_novos.md) · anterior:
> [integração P51](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-19_P51.md).
> Smoke do dono: **PENDENTE** (integrar não é aprovar).
>
> ⛔ **Depois do smoke (19/10):** o Filbert, o Flat Oily e o Knife Smooth SAÍRAM da fábrica ([53 §8](../NewFeaturesWetPaint/53_medicao_os_11_pinceis.md)).
> Os números de RECONTAGEM abaixo passam a: `PRESETS_DE_FABRICA` **28**, `MIDIAS` **24**, `IRMAOS` **2**, `NASCE_DE` **8**,
> chaves i18n de pasta **+4** (sem `flat`/`knife`), 8 PNG novos; `shape_color_tests.rs` deixou o literal (não há o que recontar).

## §1 — Coordenadas

- Branch `line/PainterWatercolor`; merge-base `100d34c8e0` (= o `main` na abertura; 0 atrás). **Nada integrado, nada enviado.**
- P53 por cima dos commits P44–P52: `cc3dafa4b2` (prompt 53) · `f36796705b` (margem da janela do rasto pelo CAMINHO)
  · `192c10198c` (os 11 presets, 20 → 31) · `4fb49aa8b8` (lei dos IRMÃOS + 2H com carga 50) · `4d8a22d94b` (fecho: 6 rótulos
  i18n, contagem 31, `dab_route.rs` no tecto, doc 53, abertos C38–C40).

## §2 — O que entrega

- 11 presets Wet Paint de fábrica: Graphite 2H/HB/2B, Colored 4B, Scratch, Square Hard, Wax Scrape, Filbert, Dabber,
  Flat Oily, Knife Smooth, em `assets/brushes/WetPainter/{Pencil/Graphite, Pencil/Colored, Pencil/Scratch, Pastel/Dry,
  Pastel/Scrape, Oil/Flat, Oil/Dabber, Oil/Knife}`. Cada um nasce do ficheiro do vizinho (`NASCE_DE`).
- Medição contra o Rebelle: [53](../NewFeaturesWetPaint/53_medicao_os_11_pinceis.md) · prancha `53_prancha_os_11.png` ·
  ferramentas `ferramentas/{tamanho_dos_presets,identidade_53,avalia_53,prancha_53}.py`.
- Memória: RSS 2,04 → 0,51 GiB (Oil Bristle+Jitter 0,5, r300) e 4,50 → 1,05 (r1000), píxeis iguais ao bit (fnv: 8B,
  Charcoal, Oil Pastel r15/r300; Oil Bristle ±Jitter r300/r1000). O gerador do Dabber a 64 workers chega a 21,9 GiB da
  fatia de 24 G (apertado; `PH2D_LEI_FIOS` baixa).

## §3 — Superfície de colisão e o que um merge pode partir

- `ph2d-wet-paint`: `trail.rs` (+`Trail::on_segment_no_caminho`) · `painter/doors.rs` (+`Engine::direct_segment_no_caminho`).
- `ph2d-tool-painter`: `tool/paint/wetpaint/dab_route.rs` (passa o arco; **700/700 LOC — tecto exacto**; quem somar
  linhas ali corta, nunca isenta).
- `ph2d-painter-brush`: `spec/queries.rs` (+`BrushSpec::alcance_do_jitter`; usado por `dab_build.rs` e `solid_carimbos.rs`).
- Testes `crates/ph2d-tool-painter/tests/it/`: `receitas_de_fabrica.rs` (`PRESETS_DE_FABRICA` 31; `NASCE_DE`; as 5 bases como
  consts `GRAPHITE_8B/CHARCOAL/SOFT_PASTEL/ACRYLIC_ROUND/OIL_BRISTLE`; `lapis()`/`lamina()`) · `presets_de_fabrica.rs`
  (nasce do vizinho; campo `pincel` aceito se em `CAMPOS_DO_PINCEL`; lei da largura em fila do maior primeiro,
  `PH2D_LEI_FIOS`) · `midias_de_fabrica.rs` (`MIDIAS` 16 → 27, `IRMAOS` + a verificação dos irmãos).
- i18n `ph2d-i18n/src/app_painter.rs`: +6 chaves `app.painter.presets.folder.{colored,scratch,scrape,flat,dabber,knife}`;
  `ph2d-app-painter`: `painter_bridge_presets.rs` (`rotulo_da_pasta` +6) e `shape_color_tests.rs` (contagem 31).
- **RECONTAR se outra linha também somar presets de fábrica** (contado, nunca escolhido): `PRESETS_DE_FABRICA` (gate
  `the_factory_presets_are_counted`), o literal `31` em `shape_color_tests.rs`, `MIDIAS` (27) e as chaves i18n de pasta.
- 11 PNG binários novos de preset: em conflito, REGENERAR (comando do [P51 §3](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-19_P51.md)), nunca fundir à mão.
- Sem `PROJECT_SCHEMA`, sem contrato §6, sem ADR, sem `Cargo.lock`, sem knob novo.

## §4 — Prova de fecho

- Suíte de fecho (`PH2D_SUITE=fecho` `nextest-impacted` sobre o merge-base): 19 341 testes; 3 falhas reais, curadas em
  `4d8a22d94b` e reverificadas POR NOME, verdes; 1 TIMEOUT de carga
  `o_pigmento_por_comprimento::a_tinta_nao_muda_com_o_spacing` (180 s na suíte; 3/3 verde sozinho a `load` 11,6–14,0;
  membro conhecido de [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)).
- `clippy -D warnings --all-targets` ✓ nas 3 crates tocadas · `fmt` ✓ · i18n 23/23.
- Gates de fábrica verdes: `the_factory_presets_are_counted` · `a_regra_do_spacing_da_fabrica` ·
  `cada_preset_de_fabrica_volta_ao_byte` · `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` · `a_largura_e_a_prometida` ·
  `a_lei_de_omissao_e_a_do_wet_round` · `cada_preset_parece_a_sua_midia` (27 mídias + `IRMAOS`) ·
  `a_cor_sobrevive_a_agua_nos_pinceis_de_fabrica` (contagem 31).
- Mutação (vermelho primeiro, controlo verde): 6/6 sangram. Identidade por dados: 11/11.

## §5 — Smoke do Enio (binário `smoke` JÁ compilado; 2.ª compilação: `Finished \`smoke\` profile [optimized] target(s) in 0.31s`)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && PH2D_WETPAINT_SMOKE=2 ./target/smoke/ph2d-host-desktop`
2. No Painter (Wet Paint), abra a lista de pincéis: há pastas novas (Colored, Scratch, Scrape, Flat, Dabber, Knife) e
   pincéis novos em Graphite e Dry — 11 ao todo.
3. Risque um traço com cada um: cada pincel deve pintar diferente dos vizinhos, com a cara do material (lápis, giz, tinta a óleo, faca).
4. Deu errado se: dois pincéis pintam igual, um pincel antigo mudou, ou algum pinta em branco.

## §6 — ABERTO

- **C38** lâmina que gira com o arrasto (Flat Oily/Knife): a nossa fica fixa a 0°, um traço vertical vira bloco largo; no
  Rebelle o traço fino e recto serve em qualquer direcção — motor.
- **C39** as pistas que o Jitter alto gera (Dabber): resta a memória ×N e talvez o aspecto rasgado; juntá-las muda a imagem de
  Jitter ≳ 0,25 — decisão de lei, do dono.
- **C40** Cv das lâminas finas (resolução da grelha do motor).
- Mais os itens do [52 §6](../NewFeaturesWetPaint/52_medicao_os_pinceis_por_fazer.md) (4 cobertos, Marker/Effect, Granulation/Spatter) e os 4 do P51 ADIADOS pelo dono; C38–C40 em [26](../NewFeaturesWetPaint/26_ABERTOS.md).
