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
- ⚠️ Há outra linha a usar telas virtuais: feche a do Rebelle por `ferramentas/gotas_fecha.sh` (só a sua), NUNCA o
  `fecha.sh` do oráculo. O teste do Rebelle tem 21 dias (16/10).

## §3 — A lei

- **Motor** (`ph2d-wet-paint`): `Knob::Glaze` (chave `glaze`, Paint, 0..1, omissão 0 = a lei da obra AO BYTE, último do
  registo, `KNOB_COUNT` 55). Plano `Grid::vela` (snapshot, patch, clear): a fracção que FILTRA, pousada com a tinta (média
  pela massa suspensa) e só onde há camada por baixo; fixada no pouso por `colorops::vela_da_camada` = Glaze ×
  min(1, 0,5 + 0,7·d). No pouso (`trail/transfer.rs`) a tinta molhada de OUTRO pigmento (`outra_tinta`, corte
  `OUTRA_TINTA_MINIMA` 0,1) assenta pela lei do ecrã e leva a sua água. `Ecra::Cobre { vela }` + `com_vela` em render,
  secagem (as duas rotas), Dry canvas, re-molhar e ferramentas: o ponto mistura o PRODUTO das duas camadas sobre o branco.
  `assenta`: palpite com a inversa fechada do produto e `CEGO_COM_GLAZE` 0,01 (só com vela).
- **Recusados (medido, 42 §2–§6)**: as caixas da obra no preset (não escurecem a caneta nem o marcador) · K–M + velatura
  (preto) · a fracção pela diferença de pigmento só (0,34 no marcador) · o Glaze em toda a tinta (o recto escurecia ao secar)
  · a fracção lida da cor de agora (clareava 19 → 33 ao secar) · mais iterações / Newton 3×3 · assentar pela cobertura do
  carimbo (custa aos marcadores) · prender o arrasto (estrias em riscas).
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

**13/13 sangram** (agente de mutação; controlo verde antes, âncoras 1×): o Glaze desligado · sem «camada por baixo» ·
sem o assentar no pouso · a camada sem a sua água · `com_vela` mudo · o produto desligado · `CEGO_COM_GLAZE` = 0,1 · sem a
inversa fechada · a mesma tinta 0 · `outra_tinta` sempre 1 · o undo sem `vela` (motor e recorte) · a captura sem `glaze`.
Os 4 últimos sobreviviam na 1.ª rodada e ganharam portão próprio.

## §6 — Aberto

- **O azul sobe ao secar** no cruzamento das tintas com água (+5 a +24 no canal B): é o ESCOAMENTO do traço vizinho
  ainda molhado (com nivelamento, capilaridade e gravidade desligados +1 a +5; a lei de hoje sem Glaze +5). Marcadores ≤ 5.
- **Ink Brush**: a faixa de cima do cruzamento (~8 px) fica vermelha — o arrasto (lei antiga, também com Glaze 0) empurra o
  azul para baixo; uma linha azul fina nas bordas da coluna (o azul vizinho entra na célula da orla). Halo +35 (era +72).
- **Chão do marcador**: 0,025–0,043 contra 0,016 (Glaze 1 é o tecto da lei).
- **A foto `marker` do Rebelle**: o vermelho vertical sai tão largo como o do Bullet (o horizontal tem 18 px) — conferir a
  sessão de 11/10 antes de usar a coluna vertical dela.
- **O plano `vela` não ESCOA** com a água (como o `relevo`): a tinta de Glaze que corre para uma célula vizinha mostra-se
  pela lei dela. Medir antes de mexer (o custo é nos passes quentes).
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
