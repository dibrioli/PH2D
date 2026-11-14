# Handoff de integração — line/PainterWatercolor, desempenho total do Painter (12/11)

Leitor: o agente integrador. Continua o [handoff pós-missão 4K](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-11_POS_MISSAO_4K.md)
(§2 vendor `winit`, §8 riscos e §9 continuam a valer); aqui só o que esta janela acrescentou. Ordem do dono (11–12/11):
«resolver todos os problemas de performance do módulo a todo custo … handoff de integração ao final … não integre».
Plano por famílias e estado: [PLANO_DESEMPENHO_TOTAL_2026-11-12.md](../missao_4k/PLANO_DESEMPENHO_TOTAL_2026-11-12.md); medições:
[REGRESSOES_TOQUE_E_UNDO_2026-11-11.md](../missao_4k/medicoes/REGRESSOES_TOQUE_E_UNDO_2026-11-11.md) (r12, r13, p1, rf).
**Nada integrado nem enviado.**

## 1. Identidade

- Branch `line/PainterWatercolor`, merge-base com o main `bc4a6ce930` (o main não andou durante a janela).
- Esta janela: `4381ee6d4a..HEAD` (52 commits; 87+ ficheiros, +4 651/−523 antes do corte dos tectos). Tocado:
  `ph2d-tool-painter` (44 ficheiros) · `ph2d-wet-paint` (19) · `ph2d-painter-brush` (4) · `ph2d-app-painter` (4) ·
  `ph2d-render` (3) · `ph2d-pigment` (3) · `ph2d-panel-painter-layers` (3) · `Cargo.toml` (perfil smoke) · `Cargo.lock` ·
  `docs/` (ADR-0204, medições, plano). **`shells/desktop` intocado.**
- O trabalho foi feito em worktrees de agentes (`.ph2d-ag-<x>`, ramos `tmp/painter-<x>`) e trazido por `cherry-pick -x`
  (cada commit cita o original); os ramos tmp e as worktrees são limpos no fim desta janela.

## 2. Superfície de colisão (`bash scripts/collision-surface.sh`, 12/11)

- Contrato congelado §6: `node.rs`/`tool.rs` intocados. Nenhum tipo de componente/registo novo.
- **ADR: esta linha cria 0202, 0203 e agora 0204** (`0204-o-restauro-do-historico-do-wet-paint-poe-e-tira-os-tiles-no-pool-do-rayon.md`,
  com 2 adendas). O main tem até 0200 — **reconte contra o main do dia** (outra linha pode ter criado 0202+).
- `Cargo.lock`: só `rayon` acrescentado às deps da `ph2d-app-painter` (pacote já existente na árvore). Nenhum pacote novo.
- `Cargo.toml` raiz: `[profile.smoke.package.ph2d-pigment] codegen-units = 1, incremental = false` (como painter-brush e
  wet-paint). Colide só se outra linha mexer na mesma secção de perfis — união trivial.
- Tectos de LOC: os 4 ficheiros que esta janela passou dos 700 foram cortados (§5); os de antes (vendor `winit`, examples da
  wet-paint, `main.rs`/`app_state.rs` da shell = main) não mudaram.

## 3. Números que se CONTAM (DELTA desta janela)

- **`PROJECT_SCHEMA`: +0** (a linha continua +2 sobre o main, do handoff anterior).
- **`VERSAO_DO_DOCUMENTO` do documento da macro (`ph2d-tool-painter/src/macro_track`): 2 → 3** (`37090f02a3`): o
  `PaintedDocument.fator_aparencia` (B34.1, `602f277f35`) tinha entrado no fim sem subir a versão; a impressão
  `IMPRESSAO_DO_DOCUMENTO_V3 = 0x227b_81b4_002d_bafd`; o leitor da v1 (`LeituraSemTintas`, `DOCUMENTO_SEM_TINTAS`) saiu
  (CLAUDE §0.11). Se a linha Macros mexer no mesmo número, **conte**: a versão sobe uma vez por mudança de formato.
- ADR +1 (0204). Registo de componentes 0. Pacotes externos 0.

## 4. O que entra (antes → depois; 4K na Radeon integrada, `--profile smoke`, mínimo de blocos intercalados)

Matriz final rf (base `4381ee6d4a` × HEAD `a5fcd15668`, mesmo target, 72 corridas, carga mediana 16,7):

| cena | toque | Ctrl+Z | outro |
|---|---|---|---|
| Wet 4K | 62,9 → **50,7** | 49,9 → **34,0** | traço p95 17,1 → 13,8 |
| Filbert 4K | 71,8 → 67,3 | 61,9 → **48,1** | |
| Wet + Óleo Real 4K | 68,6 → **55,6** | 64,7 → **46,8** | soltar 33,2 → 27,9 |
| W5 Flat Bristle `]`×10 | 44,3 → **31,0** | 88,4 → **47,9** | traço p95 (mediana) 564 → 434 |
| W6 Wet `]`×10 | 64,1 → **51,7** | 48,1 → **32,9** | CPU parado 468 → 160 % |
| W9 Bristle 16·4000 | 55,9 → **30,7** | 62,7 → **49,2** | |
| Filbert 1080p | 28,7 → 25,2 | 37,0 → **25,1** | soltar 29,8 → 20,0 |
| Impasto 4K · D1 Round Mixing `]`×10 | iguais | iguais | já não eram problema |

Por família (o commit diz o mecanismo, a medição e os testes):
- **Toque e Ctrl+Z (envio):** `d9bd722622` os planos da luz dobram-se direto em buffers `MAP_WRITE` e sobem por
  `copy_buffer_to_texture` (sem `Vec` nem o `memcpy` do `write_texture`); portão `the_shell_folds_the_window_the_tool_reports_dirty`
  reancorado (3 mutações).
- **Wet:** `b5f4a90128` Ctrl+Z sem o memset do fluxo · `62bb7c2efa` o preparo do toque arma-se no evento (o laço a pedido
  não dá `on_tick` antes do 1.º gesto; o preparo vale 15 ms, mesmo binário, r13) · `3848f195aa` provas.
  `22010692a6` `f7bc76d564` `7e9091c171` `7e2b088c64` (ADR-0204): o histórico do motor TROCA os tiles com a grade em vez de
  copiar 2× (densos e esparsos), no pool; a foto repõe só o que o «depois» não partilha. Oráculos verbatim em `cfg(test)`.
- **Cerdas largas (B14.3):** `c572c293e2` (memo do espectro + `ph2d-pigment` num pedaço no smoke + pow fora do laço) ·
  `ac4730824c` `61c3ceb0bc` (a mistura espectral em 12 pistas SSE, ao bit; prova exaustiva 2³² do log2/exp2, `#[ignore]`) ·
  `f67aadf902`…`a5fcd15668` (14 cortes ao bit no caminho das sub-amostras: corte por linha, memo, inline, tabelas por dab).
  `mede_o_dab 1 preset "WetPainter/Oil/Flat/Flat Bristle" 240`: **662 → ~368 G instruções (−44 %)**, hash
  `fe1a4c3dc2e7193a` sempre igual (e os de Flat 70, Knife Oily, Express Oil).
- **Miniatura:** `065e4300a9` a miniatura viva do pincel num pool próprio (¼ dos núcleos; `PH2D_MINIATURA_FIOS`), fora do
  pool da tela — o CPU parado depois de `]` e a disputa com o 1.º traço.
- **Água com Óleo Real (B11.3):** `c7c3e2a9c2` `3602c2b884` os laços de célula do Óleo Real saem do fio do quadro
  (325 → 141 G instruções nele); hashes iguais.
- **Camadas:** `e5ebe3f068` (B39.A3: `current_layers()` devolve o Arc) · `d648f295f9` (cópias de plano em `par_clone`) ·
  `d872d3811c` `3230d3ba5f` (B18.3: trocar de camada não recompõe a pilha quando a figura é provada igual — o quadro
  seguinte 47 → 0 ms; a troca 12 → 9; teste `troca_figura` compara com a recomposição inteira ao bit em 10 estados).
- **Vermelhos herdados da missão curados** (todos verdes no main, vermelhos na linha; bissectados): `ea81d00445`
  (`o_meio_viaja…`: degrau Water·Oil de `03df88ea8c`) · `a5fe97a911` (regrava hashes de leis aprovadas: B33.1 `93013c9134`,
  B32.1 `00f3ca1751`) · `ca87074e10` (3 cenas pagam a dívida da luz Cheia antes de comparar: B14.2 `afb02a91db` e C3
  `f983140019`, este era instável pelo relógio) · `37090f02a3` (formato da macro, §3).

## 5. Prova de fecho (12/11)

Bateria em série sobre `ea02e0f4da` (`$SP/fecho.sh`, cada passo com o exit; carga 39 → baixa):
- `cargo fmt --check` nas 7 crates tocadas ✓ · clippy `-D warnings --all-targets` (`PH2D_SUITE=fecho`) nas 7 ✓ (as curas
  do fecho: `9ff7a25615` chunks_exact, `3cd3538d09` repoe só em teste + nome do fio HR-15, `ea02e0f4da` items_after_test_module).
- Tectos: `workspace_src_files_under_loc_cap` ✓ · `overage_allowlist_has_no_stale_entries` ✓ · `the_shell_only_shrinks` ✓ ·
  `architecture_claude_md_cabe_no_orcamento` ✓. Os 4 ficheiros que esta janela passou dos 700 foram CORTADOS
  (`1dc13974c0` impasto_light 869 → 662 · `435d54066a` impasto_gpu 771 → 593 · `6609de8859` pousa_o_dab 709 → 682 ·
  `074e50164e` historia_tests 723 → 541; só mover; o portão de texto `the_fold_walks_in_parallel…` lê agora `impasto_gpu/dobra.rs`).
- Testes por nome (≈ 45 grupos, todos ✓): `the_impasto_fold_walks_the_dirty_rect`, HR-15 `every_word…`, `espectral` (9),
  `impasto_light` (6), wet-paint `historia_` (21) `a_troca_da_o_bit` `o_restauro_por_plano` `as_recontagens_por_pedaco`
  `o_meio_viaja…` `bristles` (22) `lamina` `pente` `campo_esparso` `tip`; tool-painter `undo_vivo` (11) `undo_tinta_viva` (10)
  `o_preparo_fica_pronto_sem_tique` `o_repoe_` `a_miniatura_nao_depende_do_numero_de_fios` `troca_figura`
  `camadas_integridade` (9) `duplicat` `impasto_gpu` (7) `pousa_o_dab` `ponta_das_cerdas` `sil_memo` `corte` `cerdas` (12)
  `footprint` + os 6 herdados curados; painter-brush `footprint` (13) `bristles`; panel-painter-layers `ler_a_pilha_publicada`.
- Vermelhos POR DECISÃO (§7): `o_oleo_pouco_diluido…`, `impasto_shine_glints…`, `wax_bleeds…`.
- **Alarmes de pincel (não regravados):** largura = Oil_Wash, Smear_Wet · miniatura = os mesmos 74 do fecho de 11/11
  (comparados nome a nome). Relatório `target/relatorio-pinceis/index.html`.
- O censo da árvore combinada (`censos-da-arvore-combinada.sh`) não fechou nesta janela: a load ~220 (a `suite-completa` de
  outra sessão) o `sccache` falhou sem diagnóstico ao compilar `ph2d-app-motion`/`ph2d-app-vec` (lib test); as duas compilam
  limpas sozinhas (`cargo test -p ph2d-app-motion -p ph2d-app-vec --lib --no-run`, 37 s). **Corra-o na integração.**
- Binário smoke: compilado 2× no fim (2.ª saída abaixo, §6).
- `agent-loop-profile` (20 sessões): paralelismo 1,14/passo ✗ · test:check 3,1× ✗ · respostas/sessão 184 ✓ · edições sem
  conferir 1,5 % ✓ · contexto relido 246 mil ✓ · início 68 mil ✓.
- Limpo: as 12 worktrees de agentes `.ph2d-ag-*` e os targets delas, `.ph2d-bis-4k` + `.ph2d-bis-4k-target`,
  `.ph2d-perf-target`, os ramos `tmp/painter-*`; `target/*/incremental` da linha (54 GB).

## 6. Smoke do dono

Binário: `Worktrees/line-PainterWatercolor/target/smoke/ph2d-host-desktop`, compilado 2× depois de `rm -rf target/*/incremental`
(54 GB); a 2.ª: `Finished \`smoke\` profile [optimized] target(s) in 0.27s` (zero `Compiling`). A cena é a do
`PH2D_COMPOSITE_SMOKE` numa tela 4K de papel; `PH2D_COMPOSITE_PRESET` escolhe o pincel.

1. **Wet a 4K — tocar e desfazer.** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_PILHA=0 PH2D_COMPOSITE_PAINEL=wet PH2D_COMPOSITE_TELA=3840x2160 PH2D_COMPOSITE_FUNDO=papel PH2D_COMPOSITE_PAPEL=1 ./target/smoke/ph2d-host-desktop`
   → pinte um traço e aperte Ctrl+Z. Certo: o traço começa sem engasgo e o Ctrl+Z tira-o quase na hora. Errado: um
   atraso visível ao tocar ou ao desfazer.
2. **Pincel largo.** O mesmo comando com `PH2D_COMPOSITE_PRESET="Flat Bristle"` → passe o rato na tela, aperte `]` dez
   vezes e pinte. Certo: o app responde logo depois de mudar o tamanho (a miniatura do pincel refaz-se sem travar o resto)
   e o traço anda, ainda pesado (é o caso aberto n.º 1). Errado: o app parado vários segundos depois do `]`.
3. **Trocar de camada.** No comando 1: aba **Layers** → **+** → clique na camada de baixo e na de cima. Certo: a troca é
   instantânea e a pintura não pisca. Errado: um engasgo a cada clique.
4. **Decisão do dono (Filbert Dry).** O comando 1 com `PH2D_COMPOSITE_PRESET="Filbert Dry"`: pinte azul, depois amarelo por
   cima. Certo: um verde que passa suave. Errado (é o que o teste acusa): faixas/degraus de cor visíveis na mistura.

## 7. Decisões que são DO DONO (não decididas aqui)

- `o_oleo_pouco_diluido_cruza_o_amarelo_no_azul_em_verde` **fica vermelho**: com a mistura espectral (B32.1) o **Filbert Dry**,
  amarelo sobre azul, mostra 25 degraus de matiz (> 3° entre linhas) contra os 10 do portão — o verde está lá, sem ferrugem.
  Portão de defeito: não afrouxado. Smoke: o dono pinta e decide.
- `impasto_shine_glints_on_the_wall_without_bleaching_the_rim` (ganho 34 < 40) e `wax_bleeds_the_paints_own_colour_into_the_shadow`
  (1,6 : 1 < 2×) continuam vermelhos (B29.3, `d7f3af061a`, lei aprovada com prancha — BACKLOG B18.Y).
- Alarmes de pincel: largura 2 (Oil Wash, Smear Wet), miniatura 74 — **a mesma lista do fecho de 11/11**; nada regravado.
- O W5 (Flat Bristle a Size 240): ver §8 n.º 1 — tirar o depósito do quadro muda QUANDO a tinta aparece (atraso visível
  em vez de congelar); decisão de produto se o dono quiser.
- Do handoff anterior: B14.2 (luz cheia no gesto), B13.3 (sono da água), B43.6 (cgu1).

## 8. ABERTO, por ordem de impacto

1. **W5 / pincéis muito largos:** o depósito de um dab do Flat Bristle 240 a 4K ainda custa ~0,4 s no pool (o fio da UI
   fica a 0 %, à espera). A alavanca seguinte é ARQUITECTURA: o depósito fora do quadro (fila de traços como a do Krita: a UI
   desenha, a tinta chega depois), sobre o `offthread` do Wet (o motor já viaja por canal) — implica o undo, a recolha de cor
   e a óptica esperarem a fila. Cortes ao bit que restam (perfil de `a5fcd15668`): `rgb4_do_log` 8 % + `mistura_no_meio` 4 %,
   silhueta base (`falloff_t`, `footprint.rs:556`) 5 %, `CerdasDoDab::em` 4,5 %, a flag «linha limpa» do `tip_rows` (7
   escritores), `penteia_o_relevo_do_dab` 3,7 %. Fora do bit (dono): bilinear da ponta, memo f32, corte analítico.
2. **Ctrl+Z do Wet/Óleo:** Wet 34 ms (alvo 33), Filbert 48, Óleo 47. Resta: `FotoDoMotor::de` 3–6 ms (precisa de
   rastrear os tiles que a água mudou), o chunk esparso como tile 64×64 para trocar ponteiros (ADR-0204 adenda 2).
3. **Água com Óleo Real:** o campo refaz-se na janela inteira a cada imagem (a revisão muda a cada passo) — guardar por
   linhas; B13.5/B13.6 (o passo cresce com os traços) não atacados; B13.7 (Jacobi) muda bits = dono.
   ⛔ **Recusa MEDIDA (12/11, ramo `tmp/painter-gama2`): o cache do campo por LINHAS não tem o que poupar.** A caixa
   já é a das células que mudaram ao bit (B14.1); sonda `sonda_agua_oleo_4k` (OLEO=1, 2 traços, 12 quadros),
   linhas da janela do campo cujo `sett`/`susp` (o que a massa e a cobertura leem) mudou desde a última composição:
   durante o traço **492/564**, nos quadros sem pincel **~800/880** — as que ficam iguais são só o anel do alcance
   (35 células de cada lado), e o campo desse anel lê as mudadas: refaz-se de qualquer modo. A água move o `susp` em
   toda a envolvente molhada a cada passo. O ganho tem de vir de cortar o custo POR CÉLULA do campo
   (`visibilidade_e_ceu`, `rebote_no_papel`, `desfoca`) ou de B13.5/B13.6, não de reuso.
4. **Camadas:** B18.1 cópia sob demanda (`LayerImage.rgba8` → `Arc<Vec<u8>>`, ~35 construções, serde `rc`), B18.2 salvar
   por empréstimo (precisa da shell, `project_painter.rs`), B19.10 resto (faixas, resumo por ladrilho), a pista GPU recompõe
   sempre na troca.
5. **Achado transversal (linha da UI/Editor, não do Painter):** parado, cada quadro custa ~38 M ciclos no fio da UI —
   `WidgetStore`/a11y em `BTreeMap` (~12 %), uma consulta ao modelo do Quadro (`BoardId`, 4,4 %) numa cena de pintura,
   `format` por quadro. É o piso de todo quadro do traço.
6. Não atacados nesta janela: B21.1 (Mailbox), B27.1 (fila da caneta), B42.3/B17.9 (memória do undo), B20.6/B20.11.
7. §0.11 fora do Painter: `PaintedDocumentSemPapel` nas migrações congeladas da shell (`project_schema.rs`) e o teste
   `um_estado_inicial_de_antes_da_w6c_reproduz_o_mesmo_splash` — candidatos a sair, do dono da shell.

## 9. Riscos para o integrador

- **Medições com carga:** a máquina esteve com load 15–190 (outra sessão a integrar + os agentes). Os ganhos foram aceites
  por instruções (`perf stat`), por A×B no MESMO binário ou por matriz intercalada com o mínimo; o relógio absoluto vale pouco.
- **Mudança de arquitectura de paralelismo:** a `ph2d-wet-paint` passa a usar o pool no restauro do histórico (ADR-0204);
  a `ph2d-app-painter` cria um pool rayon próprio para a miniatura. Testes de contagem de fios do SO
  (`as_faixas_fora_do_compositor_nao_abrem_fios_do_so`) foram vistos instáveis só em lote (passam sozinhos).
- **Testes lentos novos:** `o_restauro_por_plano_da_o_bit_do_restauro_em_serie` (27–55 s sob carga),
  `a_troca_da_o_bit_do_inverso_e_do_restauro`; a prova exaustiva do log2/exp2 é `#[ignore]` (4,4 s).
- `a_divida_cheia_acaba_com_a_agua_a_correr` pode falhar sob carga muito alta (passou 2× a load 175 no HEAD).

## 10. Onde está o mecanismo

Cada commit tem o porquê e a medição. Sondas: `crates/ph2d-tool-painter/examples/mede_o_dab.rs` (dab, instruções + hash),
`sonda_as_fases_do_ctrl_z_wet_a_4k`, `sonda_o_ctrl_z_wet_a_4k` (ph2d-wet-paint), `sonda_agua_oleo_4k` (`PH2D_SONDA_OLEO=1`),
`sonda_trocar_duplicar_e_salvar_camada_a_4k`, `sonda_troca_recompoe_contra_mantem_a_4k`; matriz do app:
`docs/Painter/ferramentas/mede_pen_up_4k.sh` (o modo `perf` aceita agora `KNOBS`/`TECLAS`/`ROT`). Binários com símbolos:
`CARGO_PROFILE_SMOKE_STRIP=none CARGO_PROFILE_SMOKE_DEBUG=line-tables-only` num target à parte; o `perf` aqui não grava
pilha (paranoid 2): `perf annotate -l` dá a linha.
