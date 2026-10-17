# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 42: a lei da tinta por pincel (2026-10-16)

## §1 — Coordenadas

Sobre o `main` `0aab2c1cb6` (merge-base, sem rebase). Sem push. [Prompt 42](../NewFeaturesWetPaint/42_PROMPT_a_lei_da_tinta_por_pincel.md)
· medição e lei: [42_medicao](../NewFeaturesWetPaint/42_medicao_a_lei_da_tinta.md) (§1–§6) · [prancha 42](../NewFeaturesWetPaint/42_prancha_lei_da_tinta.png)
· doc [18](../NewFeaturesWetPaint/18_mapa_das_midias.md) §5 bloco A ✅ · lista [26](../NewFeaturesWetPaint/26_ABERTOS.md) B13/B14.
Anterior: [P41](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-16_P41_SHAPE_COLOR_WET.md). Commits `f46fcffb2a` (medida) ·
`f0cfea859d` (identidade, ANTES da lei) · `df71919d58` (lei) · `be63c0226b` · `5479608d0a` · `5ad2fe4f2e` · `9c8d3e3ca4` ·
`6b5e131277` (5 pincéis) · `614a78f106` · `1c5de0a080` (portões da mutação) · o deste fecho.

## §2 — Medido primeiro

- **O mecanismo do «tapa»** (`sonda_midias::sonda_a_celula_do_cruzamento`): na bancada o azul está MOLHADO quando o
  vermelho chega; o carimbo denso pousa com o peso da cobertura ≈ 1 e substitui a cor da água. O 18 §3 dizia «o traço
  seco tapa» — errado, corrigido. K–M + velatura ia ao preto (13,13,13; o H1 do 05c).
- **O oráculo corrido** (fixtures `*seco.png`, `mbazulmolhado`, `inkazulazul`, `mblaranja`; `gera_eventos.py`
  ganhou `--seca_cruz`, `--cor_base`, `--cor_cruz`): a tinta e o marcador escurecem IGUAL molhado e seco (Ink 0,117/0,101
  · Dip Pen 0,223/0,304 · Ink Brush 0,239/0,187 · marcadores 0,016/0,016) ⇒ lei de CAMADAS. Fracção multiplicada:
  a mesma tinta 0,5; amarelo/laranja 0,61–0,89; vermelho/azul 0,98.
- ⚠️ Feche o Rebelle só com `ferramentas/gotas_fecha.sh` (há outras telas virtuais).

## §3 — A lei

- **Motor** (`ph2d-wet-paint`): `Knob::Glaze` (chave `glaze`, Paint, 0..1, omissão 0 = a lei da obra AO BYTE, último do
  registo, `KNOB_COUNT` 55). Plano `Grid::vela` (snapshot, patch, clear): a fracção que FILTRA, pousada com a tinta (média
  pela massa suspensa) e só onde há camada por baixo; fixada no pouso por `colorops::vela_da_camada` = Glaze ×
  min(1, 0,5 + 0,7·d). No pouso (`trail/transfer.rs`) a tinta molhada de OUTRO pigmento (`outra_tinta`, corte
  `OUTRA_TINTA_MINIMA` 0,1) assenta pela lei do ecrã e leva a sua água. `Ecra::Cobre { vela }` + `com_vela` em render,
  secagem (as duas rotas), Dry canvas, re-molhar e ferramentas: o ponto mistura o PRODUTO das duas camadas sobre o branco.
  `assenta`: palpite com a inversa fechada do produto e `CEGO_COM_GLAZE` 0,01 (só com vela).
- **Recusados (medido)**: a lista com os números está no 42 §2–§7 — não reconstrua sem a ler.
- **A mistura por pincel NÃO entrou** (decisão técnica, 42 §4): nenhuma mistura reproduz o cruzamento e o Rebelle não depende
  do molhado — seria um controlo sem leitor que decide. A «Pigment mixing» fica da obra. Pergunta ao dono: B14.
- **Formato**: o Glaze vai no mapa `engine` do preset, por nome; um preset sem a chave lê 0. **`SCHEMA` fica 1** — o leitor
  ignora chaves que não conhece (um build antigo pinta-o sem Glaze). A macro leva-o (o estado inicial é o preset capturado).
- **Painel**: Tuning › Paint ganhou «Brush glaze» (i18n); a nota do cartão Experimental diz que as caixas valem para a
  obra inteira e o Glaze do pincel age por cima delas, na tinta dele.
- **Fábrica 12 → 17** (`assets/brushes/WetPainter/Ink/{Pen,Brush}`, `Marker/Marker`): Ink Bullet (Glaze 0,9, cruzamento
  0,113 · Rebelle 0,117) · Dip Pen (0,8; 0,217 · 0,223) · Ink Brush (0,8; 0,257 · 0,239) · Marker Bullet (1,0; 0,025 ·
  0,016) · Marker Liner (1,0, dureza 0,55; 0,043 · 0,016). O gerador ganhou `PH2D_REGERA_SO`.

## §4 — Portões (a identidade nasceu verde ANTES da lei)

`ph2d-tool-painter` `it/a_lei_da_tinta.rs`: `sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes` (14 cenas, fnv
pinados em `f46fcffb2a`) · `o_glaze_do_pincel_escurece_o_cruzamento` (graduado) · `o_traco_sozinho_nao_muda_com_o_glaze`
(ao byte) · `trocar_de_pincel_nao_muda_a_tinta_da_tela` (ao byte) · `um_pincel_gravado_leva_o_seu_glaze`.
`it/midias_de_fabrica.rs`: métrica Cruzamento (faixa 3 × 0,06 do Rebelle; o vale aprovaria 0,49) e as 5 mídias (13).
`ph2d-wet-paint` `it/a_lei_do_glaze.rs` (7): a fracção medida · Glaze 0 = a lei da sessão ao bit · Glaze 1 = o produto ·
assentar ≤ a lei da obra (300 estados: obra 41 · 42,6; Glaze 0,5 11 · 6,4; Glaze 1 15 · 12,7) · o filme que esconde um
canal · a água que desce · desfazer devolve a vela. Censo do painel, linhas e caixas do Tuning ✓.

## §5 — Mutação

**13/13 sangram** na lei (os 4 sobreviventes da 1.ª rodada ganharam portão: a água que desce, a inversa fechada, o
undo da `vela`, o pincel gravado leva o Glaze); depois 5/5 (3.ª camada) e 4/4 (bordas). A lista: 42 §5–§8.

## §6 — Aberto

- **O azul sobe ao secar** no cruzamento das tintas com água (+5 a +24 no canal B): é o ESCOAMENTO do traço vizinho
  ainda molhado (com nivelamento, capilaridade e gravidade desligados +1 a +5; a lei de hoje sem Glaze +5). Marcadores ≤ 5.
- **Ink Brush**: a faixa de cima do cruzamento (~8 px) fica vermelha — o arrasto (lei antiga, também com Glaze 0) empurra o
  azul para baixo; uma linha azul fina nas bordas da coluna (o azul vizinho entra na célula da orla). Halo +35 (era +72).
- **Chão do marcador**: 0,025–0,043 contra 0,016 (Glaze 1 é o tecto da lei).
- **A foto `marker` do Rebelle**: o vermelho vertical sai tão largo como o do Bullet — conferir a sessão de 11/10.
- **O plano `vela` não ESCOA** com a água (como o `relevo`) — medir antes de mexer.
- **Custo**: o traço que cruza outra tinta com Glaze 1 custa 42 ms contra 15 sem Glaze (Wet Liner, `sonda_o_custo_do_glaze`);
  a água +9 %.
- **B13** (a Shape Color no pincel salvo) — pergunta do smoke, a mesma família do que o preset leva.

## §7 — Fecho

`cargo fmt --check` ✓ · clippy `-D warnings --all-targets` (`PH2D_SUITE=fecho`; wet-paint, tool-painter, panel-wet-tuning,
panel-painter-layers, i18n, app-painter com `panel-painter-layers`) ✓ · `nextest-impacted` (base `0aab2c1cb6`)
**18 499/18 506** em 683 s (`load` 85 → 221): as 5 desta linha curadas e refeitas por nome — a contagem 12 → 17 em
`shape_color_tests`, a porta `wet_sonda_motor` declarada interna da macro, os rótulos das pastas Ink/Pen/Brush/Marker,
`assenta.rs` 721 → 426 LOC (os testes em `assenta/tests.rs`), um `as f64` do clippy; fora do diff: `motion_path_perf`
(passou sozinho, carga) e os dois K3 da Luz 2D (a placa estava com a `line_luz2d`; os mesmos do P41 §6).
Smoke `--profile smoke` 2×: «Finished … in 0.50s».

## §8 — Smoke do dono (16/10): a 3.ª camada da mesma tinta MOLHADA clareava ([42 §7](../NewFeaturesWetPaint/42_medicao_a_lei_da_tinta.md))

Dois mecanismos no pouso: (1) a mesma tinta molhada nunca virava camada (só `outra_tinta` assentava) e o escoamento
clareava as 3 camadas; (2) o resto de assentado que o arrasto deixa contava como camada (ao secar, assentado branco,
até +15,5). Cura: o 1.º toque de um traço numa célula molhada ANTES dele assenta-a (`Trail::por_assentar`); a camada só
filtra na cobertura que tem (`colorops::vela_sobre`, `COBERTURA_QUE_FILTRA` 0,5). Portões `a_terceira_camada_da_mesma_tinta_nao_clareia`
(vermelho no `2e43def369`) e `o_primeiro_toque_nao_assenta_o_proprio_traco_com_agua_viva`; mutação 5/5. Cruzamentos de fábrica:
0,104 · 0,208 · 0,257 · 0,015 · 0,037. Aberto: +5 em 30 células do Marker Bullet (o arrasto leva a orla de uma camada SECA).
- **17/10, as linhas claras nas bordas** ([42 §8](../NewFeaturesWetPaint/42_medicao_a_lei_da_tinta.md)): a água traz a mesma tinta
  à orla e o «tapar extra» (esconde 5) tratava o produto dela como OUTRA tinta — catraca até (165,53,53). Cura: só com
  Glaze, a mesma tinta medida na densidade óptica (`colorops::a_mesma_tinta_filtrada`) não tapa. Marker Bullet 975 → 60
  células; portões `as_bordas_nao_esbranquicam_depois_de_varios_cruzamentos`, `a_mesma_tinta_que_entra_nao_clareia_o_glaze`; 4/4.
