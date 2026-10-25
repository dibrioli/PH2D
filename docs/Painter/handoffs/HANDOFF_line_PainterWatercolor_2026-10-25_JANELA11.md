# Handoff — line/PainterWatercolor, janela 11 (25/10) — os abertos da janela 10 fechados; 11 vermelhos herdados curados; a Borracha do Wet e as macros «mundo» corrigidas

Commits (sobre `d124c3a184`): `ec248aedbb`, `35b263150f`, `9b9d30fcca`, `ea39947224`, `049b675ab4`, `38d59946c1`, `f005bda4f4`, `e240f125f9`, `7786fdc00c`, `8070c531b6`, `86ca0e7df0`,
`d0fef8ec62`, `1a1c635bf8`, `ed7f86c33d`, `9cf49ff2eb`, `a30e987773`, `3259ca0fe2`, `9b32db7a3d`. Texto exato: `git show <hash>`. Smoke do dono: «Smoke OK» (25/10).
Anterior: [`HANDOFF_..._JANELA10.md`](HANDOFF_line_PainterWatercolor_2026-10-25_JANELA10.md). Seguinte: [`PROMPT_NOVA_JANELA_12.md`](../smoke_2026-10-24/PROMPT_NOVA_JANELA_12.md).

## 1. Controlos da frente (família dos abertos 4, 5 e 7 da janela 10)

- **Rota órfã removida** (`f005bda4f4`): `PAINTER_BRUSH_PRESET` / `apply_brush_preset` saíram; as fixturas passam a `preset_aplicar`; o teste `preset_dropdown_reconfigures_the_brush` foi apagado e a
  linha «Preset» saiu de `every_paper_knob_reaches_every_brush_slot`.
- **«Reset to brush preset»** (`9b9d30fcca`): a disponibilidade viaja em `BrushSettings::mao_do_preset` (o thread-local saiu). `toda_porta_do_painter_tem_decisao` estava VERMELHO em `d124c3a184`
  (`toggle_brush_wash`, `set_mao_do_preset` sem decisão declarada). `set_mao_do_preset` passou depois de INTERNAS para Ambiente em `portas.rs` (`3259ca0fe2`): a ponte `lembrar_a_mao_do_ativo`
  (`painter_bridge_presets.rs`) chama-a de fora; gate `nenhuma_porta_interna_tem_chamador_de_fora`.
- **Defeito 5 do Pen** (`ec248aedbb`): `Tilt::effect` devolve `None` quando neutro (pegada deitada neutra não vai à rota por dab).
- **`ExtDryBrush` é lei VIVA** (`35b263150f`): `dry_gate` abre quando `b<7` (ex.: pressão→opacidade desligada + Speed opacity) e entra no `Tuning` salvo. Fixado por
  `o_extdrybrush_so_mexe_com_dry_gate_positivo`; comentário de `KNOBS_NO_CARTAO` corrigido.
- **Wet Color/Solid/Type/On empty** (`e240f125f9`): medidos por `os_controlos_de_cor_e_modo_do_wet_mudam_a_tinta` — sem defeito. Light scatter = `pressure_jitter` e a S-curve estão vivos (itens fechados).

## 2. Cauda do Digital (`38d59946c1`)

Accumulate ON + Flow 0,3 sem Continuous: cauda 146 → 72 (corpo 72). `adensa` usava n=⌈x⌉ dabs; agora composição por dab do `ralo` 1−(1−a)^r. Ressalto de entrada (F,F) 74 → 69 (corpo 67): a 1.ª cápsula
da cadeia abre de cobertura zero (ligar ao dab do corpo foi medido e RECUSADO: o ressalto mexeu e o degrau passou de 64 a 74).
Gate `a_cauda_com_a_taxa_afila_sem_discos` agora com 4 combinações, janela 410..600, limite corpo+2/255; sonda `sonda_cauda_quatro`.
**Aberto:** ondulação da cauda (F,T) ±5 dentro da ondulação do corpo; mergulho −3 depois da entrada em (F,F); pincéis de fábrica com Accumulate+Flow<1+taper ficam com cauda mais clara (nenhum gate de imagem de referência conferido).

## 3. Borrachas e macros (família do aberto 6)

- **Macro «mundo»** (`ea39947224`, `049b675ab4`): grava embutindo a tinta por assar via `PainterTool::imagem_por_assar` (UMA porta); só falhava com o Painter fora de ferramenta ativa (0/1 → 1/1). Teste `a_macro_mundo_leva_a_tinta_por_assar`.
- **Borracha do Wet sobre tinta viva obedece ao Flow** (`7786fdc00c`): era 25,9 em toda combinação; agora Flow 1/0,3/0,1 → 25,9/53,6/62,2. Teste `a_borracha_do_wet_obedece_ao_flow`. Strength segue escondida e inerte no Wet.
- **Preview da Borracha no Wet** (`8070c531b6`): `e_a_borracha` testava `PaintMode::Paint`; agora `ferramenta_dos_presets()==Some(Borracha)`. Teste `o_preview_da_borracha_no_wet_pinta_a_borracha`.
- **Texture Eraser à Strength 1** (`86ca0e7df0`): `stroke_cover_wanted` liga o tecto quando o grão está preso à tela (Tiled/Stencil): 255±0 → 161,4±45,9. 21 receitas de fábrica não-Wet idênticas por hash. Teste `texture_eraser_a_strength_1_morde_o_grao`.
- **Aberto:** Borracha do Wet sobre tinta seca ~1,9× mais larga que a do Digital (item c do handoff FERRAMENTAS_2).

## 4. Onze vermelhos herdados (vermelhos em `d124c3a184`)

- **10 vindos de `4ce1155a83`** (Composite só no Digital): fixturas de `composite_relevo_tests`, `diag_o_relevo_da_pilha` e `impasto_body::sculpting` passam ao Digital + Shape›Relief 1; limiares 30/15 (metade do medido 60,89/32,14);
  dois testes reescritos/renomeados: `so_relevo_com_o_composite_ligado_no_impasto_nao_pinta_cor`, `a_cor_da_pilha_e_a_do_avulso_com_e_sem_filme` (`ed7f86c33d`, `9cf49ff2eb`).
- **1 vindo de `4b6186562d`**: `o_pincel_guardado_pinta_o_mesmo_traco` — a regra de abertura do Continuous (só Digital) armava-se no Wet porque `leis_do_digital()` não conhecia o meio.
  Cura final `a30e987773`: `BrushSpec::fora_do_digital`, posto em `authored_spec` quando o meio ≠ Digital; `leis_do_digital()` = !watercolor && !impasto && !fora_do_digital. Consumidores conferidos:
  só `abertura_na_capsula` muda; os outros já eram off fora do Digital. O reset a Flow 1 de `1a1c635bf8` saiu; `CAMPOS_FORA` 77 → 78.
  Teste `a_borracha_do_wet_nao_entra_na_abertura_do_digital` (não vacuoso desde `9b32db7a3d`: soma os dabs de begin/extend/finish; controle com leis do Digital = 4 dabs mudados, Wet = 0).

## 5. Prova de fecho

- **Mutação:** 12 + 2 corridas; todas as relevantes mortas. Sobreviventes inofensivos: `a_tilt_nobody_reads_paints_the_stroke_of_before` não guarda `Tilt::effect` (guarda-o `tilt_smooth_zero`);
  o teste de painel `o_reset_para_o_preset_despacha_a_sua_opcao` constrói o seu `BrushSettings` (mata-o `o_reset_da_mao_tem_duas_portas`, da ferramenta).
- **Gates:** clippy `-D warnings --all-targets` limpo em `ph2d-tool-painter`, `ph2d-app-macros`, `ph2d-app-painter`, `ph2d-painter-brush`, `ph2d-panel-painter-layers`, `ph2d-wet-paint`; `fmt --all` limpo;
  `the_shell_only_shrinks` verde (shell intocada). Suíte completa NÃO corrida (lei do dono; é do integrador).
- **Contadores (delta):** `CAMPOS_FORA` +1 (77 → 78); linha «Preset» fora de `every_paper_knob_reaches_every_brush_slot`; nenhum registo/FIELDS/schema mexido.

**Um merge pode partir:** `BrushSpec::fora_do_digital` / `leis_do_digital()`; a categoria de `set_mao_do_preset` em `portas.rs`; qualquer linha que reintroduza `apply_brush_preset`; `stroke_cover_wanted`; `adensa`/`ralo` na cauda; `CAMPOS_FORA`.

**RISCO:** as macros guardam eventos de painel por número; uma macro gravada antes de 25/10 que escolheu no menu «Basic brush» (removido na janela 10) agora reproduz isso como um no-op silencioso.

## 6. Abertos

1. Decisões do dono 1–3 do PROMPT_11 (forma fechada × Newton; Borracha da Watercolor (a)/(b); arrasto do transfer com instantâneo / Grid Size 2) e decisão 2 da janela 7 (escondidos do Digital sem linha).
2. Borracha do Wet seca ~1,9× mais larga (janela 4); resíduos da cauda (§2).
3. Pedidos NOVOS do smoke de 25/10 (ferramentas do Wet na barra da esquerda, água do Smear, Inspector por ferramenta, presets de Smear/Blend): ver [`PROMPT_NOVA_JANELA_12.md`](../smoke_2026-10-24/PROMPT_NOVA_JANELA_12.md).
