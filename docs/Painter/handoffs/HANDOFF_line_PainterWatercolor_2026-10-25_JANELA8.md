# Handoff — line/PainterWatercolor, janela 8 (25/10) — água do Wet pela rota MEDIDA; smoke do dono achou artefatos retangulares

Commits: `16facb026b` (rota medida) e `c69b3d47d9` (gate Wet bloom + Tip pickup), sobre `d2af24b013`. Texto exato: `git log -3`.
Anterior: [`HANDOFF_..._JANELA7.md`](HANDOFF_line_PainterWatercolor_2026-10-25_JANELA7.md). Seguinte: [`PROMPT_NOVA_JANELA_9.md`](../smoke_2026-10-24/PROMPT_NOVA_JANELA_9.md).

## 1. Acelerar a água do Wet (`16facb026b`) — escolha do dono na janela 5

Medido com sondas novas (cena de largura de fábrica: Wet Round, 320 passos): a SECAGEM é 69 % do tempo da água e custa
~160 ns/célula·passo em série. A lei do Rebelle (`Ecra::Cobre`) inverte o ecrã por Newton em `assenta`/`levanta`
(~3,1 avaliações de 97 ns por célula; `diferenca_de_pigmento` 21 ns ×2 por avaliação; `powf` só 5 ns).
Os pisos de células `MIN_CELLS_*` (`crates/ph2d-wet-paint/src/par.rs`) tinham sido medidos na poça de diagonais cruzadas com a
célula barata da porta; na janela densa do produto a secagem em paralelo já ganha 1,64× a 1 380 células e ~14× a 120 k;
advect/flow ganham de ~4 k, project/smooth de ~10 k.

Cura: `Rows::Medida(piso_ns)` em `par.rs` — a linha do meio corre em série e cronometrada; o resto vai ao pool se
`decorrido × linhas ≥ piso` (desenho do `PISO_NS_DEPOSITO`). Braço em todo caminhante (`medida`, `medida_reduce`);
`Rows::pick_medido` nas seis passagens da água (`drying.rs`, `solver.rs` smooth, `flow_jacobi`, `advect_jacobi`, `project`,
`sedimenta`). Acima dos pisos de células: paralelo como antes. `PISO_NS_AGUA` = 20 µs (tabela no doc-comment).
Mais `tapa`/`sobre_branco_tapando` em `assenta.rs`: `powf` uma vez por célula no Newton (−9 % na secagem).

Ciclo de 12 passos do produto (`sonda_o_piso_de_tempo_da_agua`), antes → depois: raio 55 236,5 → 27,8 ms (8,8×);
26 102 → 15; 13 58,7 → 10,5; janela mínima 1,02 → 0,97; acima dos pisos igual.

NÃO alcançado: o gate de fábrica `a_largura_e_a_prometida` NÃO encurtou (104,9/108,2 s → 104,5/98,8 s,
`sonda_o_portao_da_largura_com_o_piso_de_tempo`): 232 cenas em 16 núcleos saturam a CPU; só uma célula mais barata mexe nisso.
Forma fechada no lugar do Newton numérico MUDA bits ⇒ decisão do dono, perguntada, PENDENTE.
Nota velha: «57 ms/passo 14 Hz» em `docs/Painter/29_offthread_sim.md` está obsoleta (hoje a mesma cena 4096² de 3 pistas
~7,8 ms/passo em média). `samply` precisa de `perf_event_paranoid ≤ 1` (sudo) — indisponível; mediu-se por sondas.

Gates (por nome, verdes): `a_rota_medida_e_a_serie_ao_byte` (`crates/ph2d-wet-paint/tests/it/parallel_rows.rs`, 6 passagens,
`Medida(0)`/`Medida(MAX)`, poça Rebelle), `a_reducao_medida_conta_cada_linha_uma_vez` (`src/par_medida_tests.rs`),
`o_tapar_de_uma_vez_e_o_de_cada_avaliacao` (+ valor absoluto; `src/assenta/tests.rs`),
`a_rota_medida_nao_muda_o_wet_com_bloom_e_pickup` (`crates/ph2d-tool-painter/src/tool/paint/wetpaint/tests_doc22.rs`, ∞/fábrica/0 quadro a quadro, `c69b3d47d9`).
Mutação: 5 sangram (meio largado, item saltado, meio fora do reduce, `esconde+1e-9`, `h` errado); `pick_medido` sempre Serial
SOBREVIVE (lei de velocidade, ponto cego conhecido); M2 «f(meio) duas vezes» não se exprime (o tipo proíbe).
Sondas (ignoradas): `crates/ph2d-wet-paint/tests/it/measure_stage_cost.rs` (custo por etapa a 4096² e à escala de fábrica;
joelho da secagem Rebelle); em `crates/ph2d-tool-painter/tests/it/presets_de_fabrica.rs`: `sonda_de_que_a_cena_da_largura_e_feita`,
`sonda_o_piso_da_secagem_na_cena_da_fabrica`, `sonda_o_piso_de_tempo_da_agua`, `sonda_o_portao_da_largura_com_o_piso_de_tempo`;
bench `sonda_o_custo_da_lei_do_rebelle` (`assenta/tests.rs`).

## 2. Gate vermelho desde a janela 7, curado

`tests_doc22::os_grupos_avancados_do_tuning_chegam_aos_knobs_e_repoem_so_o_seu` usava o chip `extBackrun` do Tuning, que
passou ao cartão em `4b6186562d`. Agora ExtFingering no Tuning e Backrun pela porta do cartão.

## 3. Smoke do dono (25/10) — DEFEITO NOVO, ainda sem causa

Fotos: `docs/Painter/smoke_2026-10-24/fotos_janela8/` (`retangulos_inicio_1.png`, `retangulos_inicio_2.png`, `resolvido_suave_3.png`).
Palavras do dono: «Artefatos e bugs visuais retangulares ao aumentar wet bloom e tip pickup. Manchas brancas que começam
retangulares no início da simulação mas que se resolvem com resultado suave (última foto).» e «observa-se também
significativa queda de FPS ao mesclar as cores».

Estabelecido: NÃO é da janela 8 (bytes idênticos entre rotas — gate acima). Provável exposição pela janela 7, que pôs
Wet bloom (`Knob::WetOpen`) e Tip pickup (`Knob::TipReservoir`) no cartão. Os retângulos brancos de bordas em escada, alinhados
aos eixos, parecem uma janela de dab/caixa (caixa do depósito, janela do bico `MIN_CELLS_TIP`, ou rect de janela) a levantar/abrir
tinta — mecanismo NÃO encontrado. Queda de FPS ao mesclar: NÃO medida (candidatos: custo do tip pickup por dab, Newton do Rebelle
no depósito, K–M).

## 4. Prova de fecho

`clippy -D warnings --all-targets` e `fmt` limpos em `ph2d-wet-paint` e `ph2d-tool-painter`. Suítes por nome verdes:
wet-paint `parallel_rows`, `fingerprint`, `resumable_step`, `sim_resumable`, `assenta`, `solver_symmetry`, `drying`,
`acceptance`, `deposit_rows`, `paper_rows`, `product_doors`, `flow_symmetry`; tool-painter `offthread`, `a_largura_e_a_prometida`,
`cada_preset_parece_a_sua_midia`, `borracha_por_meio`. Suíte completa NÃO corrida (é do integrador).
Contadores: nenhum registo/FIELDS/Tuning mexeu (delta 0). Um merge pode partir: o braço `Rows::Medida` em qualquer walker novo
de `par.rs` (o `match` é exaustivo) e os pisos `MIN_CELLS_*`/`PISO_NS_AGUA` se outra linha os mover.

## 5. ABERTO

1. NOVO: retângulos brancos com Wet bloom/Tip pickup altos + queda de FPS ao mesclar cores (§3).
2. Decisão do dono PENDENTE — célula do Rebelle em forma fechada (muda bits) para encurtar `a_largura_e_a_prometida`.
3. Decisão do dono PENDENTE — Borracha da Watercolor que mantém a sessão: (a) buraco limpo (a aguada some, aparece o papel, o resto
   segue molhado) ou (b) levantar com mata-borrão (a aguada sai e a borda do buraco ganha o filete escuro da aquarela). Modelo da
   sessão mapeado: buffers em `state.rs` (`stroke_coverage`, `stroke_color`, `wet_session_base`, `wet_session_canvas` Arc,
   `canvas_wet`…); lei de render `watercolor_render.rs`; hoje, com borracha, `watercolor_render_active()` é false e
   `wet_session_continues` false no pen-down, união esvaziada em `stroke_lifecycle.rs:126-161`; apagar no Digital = `BrushBlend::EraseAlpha`
   (`blend.rs:177`); o gate `o_apagado_na_aguada_molhada_fica_apagado` tem de continuar.
4. Da janela 7: Accumulate ON + passagem translúcida (Flow 0,3) sem Continuous escurece a cauda; ressalto ~+10 % na entrada da
   janela da cauda sem Continuous; Wet presets não gravam Press; S-curve, Light scatter, Speed lag; canal de disponibilidade do
   «Reset to brush preset»; defeito 5 do Pen (Tilt smooth/Rotation); Dry brush (`ExtDryBrush`) knob morto; decisão 2 do dono
   (escondidos do Digital sem linha).
5. Da janela 4: família «mundo» das macros lê a sprite sem a tinta por assar; Borracha do Wet × Strength; preview da Borracha no Wet;
   Texture Eraser à Strength 1; a shell a 23 linhas do tecto.

Integração no main: só por ordem do Enio (`/pd-integracao`); a suíte completa é do integrador.
