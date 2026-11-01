# E10h — pequenos (restos da janela 5, HANDOFF_01_11_janela5 §4 item 5)

Todas as tarefas deixam o produto byte-idêntico, salvo a 5 (uma linha nova no painel da borracha da água).

## 1. Chave i18n órfã `panel.wet_tuning.knob.levelingOwnStroke`
- Saiu de `crates/ph2d-i18n/src/wet_tuning_avancado.rs` (era a l. 63). O knob saiu do registo na E10c2; o painel
  monta `knob.{key}` a partir de `KNOB_DEFS` (`ph2d-panel-wet-tuning/src/rows.rs:106`), que já não tem a chave. A
  `CHAVE_PAREDE` de `tp/tool/paint/wetpaint_preset/controlos_e10.rs:18` é a migração que lê o FICHEIRO, não o rótulo.
- Portão: `every_key_of_this_panel_exists_on_both_sides` (ph2d-panel-wet-tuning) — **vermelho ANTERIOR, não desta
  tarefa**: acusa `panel.wet_tuning.inerte.real_oil_on` como órfã, mas ela é lida por `tp/tool/paint/inercia.rs:280`
  (fora da pasta que o censo varre). Falso vermelho de alcance do censo desde `a886591a47` (28/10). Aberto.

## 2. `BrushSettingsSnapshot::wet_opacidade`
- Zero leitores (só dois escritores, ambos `OPACIDADE_DE_FABRICA`). Saiu de `tp/tool/paint/brush_settings.rs`
  (o campo, ~l. 463), `tp/tool/paint/snapshot.rs` (~l. 467) e `crates/ph2d-panel-painter-layers/src/brush_fallback.rs`
  (~l. 296). `OPACIDADE_DE_FABRICA` fica (a migração E4/E10 e a preview leem-na).
- Prova: o `--list` da tarefa 3 compila a ph2d-tool-painter; a corrida dos censos do painel compila a
  ph2d-panel-painter-layers.

## 3. `wetpaint_preset/tests.rs` (830 linhas) partido por tema — mudança pura
- `tests.rs` (571): ajudantes + campos do pincel, ida-e-volta do preset/macro, Tilt/W1g, ondas futuras.
- `material_tests.rs` (92, novo): `o_material_do_pincel_vai_e_volta_pelo_preset`,
  `o_preset_de_antes_da_w3_abre_com_o_material_convertido`.
- `volumes_tests.rs` (179, novo): `os_volumes_sao_o_que_o_painel_mostra`,
  `a_opacity_do_wet_paint_e_a_quantidade_de_pigmento`, `com_a_opacity_baixa_a_cor_acumula_ate_a_escolhida`,
  `a_opacidade_nao_tem_leitor_no_wet_paint` (+ o ajudante `miolo_do_traco`).
- `W` e `risca` passam a `pub(super)` (`tests.rs:22`, `:400`); `mod` em `wetpaint_preset.rs` (694 linhas).
- Prova: `cargo test -p ph2d-tool-painter --profile smoke --lib wetpaint_preset -- --list` → **103 antes = 103
  depois**, os mesmos nomes (diff vazio pelo último segmento do caminho); zero avisos.

## 4. Restos da parede do traço no shader GPU (`ph2d-wet-gpu`)
- `advect.wgsl`: saíram `@binding(18) traco`, `separa()`, `parede()`; `K.isola` → padding `_k1` (layout 16 × 4 B
  igual). As três escritas que usavam a parede ficam incondicionais: `suaviza` (l. 232), `campo` Leveling (l. 293) e
  a escrita do fluxo (l. 328). Como `isola` era sempre 0 (`Zeroable::zeroed()` em `Knobs::from_params`), `parede()`
  devolvia sempre `(false,false)` e `select(x, 0, false) = x`: byte-idêntico.
- `lib.rs`: `Knobs::isola` → `_k1` (l. 231); saíram o campo/buffer `traco`, a entrada `(18, &traco)` (`all` 19 → 18,
  l. 364) e o 18 do `usa` de `suaviza` e `campo`. `passo.rs:17`: doc sem a parede.
- ⚠️ Layout automático: `suaviza` só usava o `k` (binding 15) pela `parede()` — o `usa` dele perdeu 15 e 18.
- Prova (sem placa): validador naga 30.0.1 (a versão do lock) num crate de rascunho — HEAD: OK, 21 entradas,
  bindings 0..18; editado: OK, 21 entradas, bindings 0..17; os bindings por entrada do shader editado casam
  EXACTAMENTE com o `usa` novo (o validador apanhou um 3.º uso de `wl` que a leitura tinha perdido).
  `cargo check -p ph2d-wet-gpu --all-targets` verde. ⛔ Nenhum teste da ph2d-wet-gpu corrido (proibido: NVIDIA).

## 5. Hardness e a borracha d'água
- Painel escreve: `PAINTER_WETPAINT_HARDNESS` → `WetKnobs::set_hardness` → `Knob::PaperGate`
  (`tp/tool/paint/wetpaint_settings/controlos_e10.rs:105,136`).
- Quem lê: a borracha do Wet Paint → `tp/tool/paint/wetpaint/pousa_o_dab.rs:594` `dispatch_pressure_dab_erase` →
  `wp/tools.rs:106` `apply_erase_impl`: `gg = stamp − (1 − tooth)·gate`; `gg ≤ 0` não apaga a célula. **O leitor
  DECIDE.** As ferramentas Water/Dry/Blow/Smear não leem o `PaperGate` (o Blend lê: `wp/trail.rs:603`).
- Achado: o INVERSO de controlo morto — knob vivo SEM linha. A regra em `paint_wetpaint.rs:84-87` escondia a
  Hardness na borracha (`!brush.eraser && Blend`). Cura (regra de visibilidade pura): `le_o_papel = eraser || Blend`.
  A Hardness não pertence aos ids de `DepositoMolhado` no censo, por isso o censo da borracha a mede.
- Portões: `a_borracha_em_cada_meio_so_ve_o_que_o_meio_lhe_muda` verde (cada linha que o Wet Paint acrescenta à
  borracha muda o apagado); `cada_ferramenta_da_agua_so_ve_o_que_le` — ver o retorno do agente.
- Aberto: sem prova de mutação de que o censo MEDIU a Hardness na borracha (a leitura do código diz que a linha é
  pintada: `linhas_e10_depois` corre depois do slot do Pickup mesmo quando o Pickup está escondido).
