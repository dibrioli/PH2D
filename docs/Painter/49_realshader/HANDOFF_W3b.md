# HANDOFF W3b — Óleo Real: bordas, transparência viva, painel só no futuro (27–28/10)

Linha `line/PainterWatercolor` (Modo L). Commits: `151cf87c10` · `65d6a6a16d` · `ad16f60b75` · `70e770b372` (JANELA_6)
· `6cab0a4026`. Sem push, sem integração. Smoke do dono: 1.º e 2.º aprovados com correções; 3.º («Restante smoke está
OK», 28/10) — o teste final do 3.º binário fica para depois, com a JANELA 6 já a trabalhar.
Auditoria de 2 lentes: [`AUDITORIA_W3b.md`](AUDITORIA_W3b.md) (C1–C6, U1–U6 curados).

## A lei nova: o que é do TRAÇO e o que é da TELA (decisões do dono 27–28/10)

| Controlo | Onde vive | Guardado em |
|---|---|---|
| Real Oil ligado/desligado · Depth · Gloss | pincel (caixa no cartão Wet Paint; Depth/Gloss = 2 primeiras linhas de Tuning → Material) | `OleoCelula` (`grid/oleo_celula.rs`, 12 B) |
| Transparency · Transmission · Transmission mode (Blender \| Clear glass) · Paper Relief | pincel (Tuning → Material) | `MaterialCelula` (append-only; último campo `vidro_claro`) |
| Paint Texture · Glaze layering · visibilidade/grão do papel no traço | pincel (cartão Wet Paint) | `AparenciaCelula` (`grid/aparencia.rs`, 16 B) |
| Paper (tipo, profundidade, dente) · **Smooth Edges** (última linha do cartão Paper; age na cor E no relevo) | TELA | documento |
| Luz, sombra, ambiente do Real Oil | TELA (cartão Real Oil) | documento |

As três células: preguiçosas (0 B no valor de fábrica), misturadas por massa, vestem a orla virgem (`veste_virgens`),
viajam por snapshot/patch/undo/clear/`bytes_retidos`. Glaze (`glazeLayering`), Paint Texture (`paintTexture`) e a caixa
Real Oil viajam no preset (gravados só quando ligados ⇒ fábrica ao byte) e por ferramenta (`*_por_slot` em `tool_link`).
`paper_depth` saiu de `CAMPOS_DO_PINCEL` (21 presets Digital + Watercolor Basic regenerados).

**Gates da família (todo knob novo entra nestes censos):** `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada`
(parametrizado sobre `KNOB_DEFS` + FACTOS + cartão) · `um_controlo_do_pincel_nao_recompoe_a_tela` · `glaze_no_preset_tests`
· `oleo_por_traco_tests` · `material_do_pincel_tests` · `preview_censo_tests` (isenção `WET_FORA_DO_PREVIEW`).
Mudança de valor do cartão recompõe na varredura do tick, 1× por quadro (`oleo_no_quadro.rs`).

## Leis do motor (`ph2d-wet-paint/src/oleo_real.rs`)

- Bordas profissionais (substituto do «nano pixel»): `borda_suave` só em borda real (fração de fora ≥ 0,20) ·
  `sem_entalhes` (célula ≥ 0,10 abaixo da mediana 3×3 enche até à mediana) · `sombra_sem_furo` (`saturating_sub` —
  C1 era overflow `u8` só visível em dev; os gates de lei `u8` correm também em dev). Dabber: 65 → 0 pontinhos no papel
  branco (Rebelle 1). Folha `out/BUG_bordas_W3b_folha.jpg`.
- Smooth Edges: `amacia_a_silhueta`, raio `Smooth × 6 px ÷ ratio`; UMA silhueta para cor, as 3 alturas, normal, brilho
  e sombra; o raio entra na `ChaveCampo`. Folha `out/SmoothEdges_relevo_folha.jpg`.
- Transmission: **Blender** (omissão) = filtro `a`, `K_BLENDER` 0,5, sombra opaca — papel creme L/papel 0,183 vs Cycles
  0,187 · **Clear glass** = Beer–Lambert de película fina `a^(2k)`, k 0,2, sombra fraca. Tinge a cor REAL do papel do
  documento (`OleoReal::papel`); sombra coerente sob a tinta e no papel nu (`transparencia_a_luz` /
  `sombra_transparente_a_luz`). Oráculo Cycles em `oraculo/` (README §Transmission rondas 1–4). Folhas
  `out/Transmission_folha.jpg`, `out/Transmission_modos_folha.jpg`.
- Paper Relief: grão do papel somado ao campo do óleo, `RELIEF_NA_TINTA_MIN` 0,6; `ChaveDoGrao` recompõe ao mudar
  papel/lâmpada.

## Aberto (vai para a fila da JANELA 6)

1. **⭐ Dono 28/10: os pincéis de fábrica da pasta Oil vêm com o Real Oil LIGADO.** Hoje nenhum preset de fábrica o
   liga, e escolher um pincel de fábrica desliga-o (a caixa é do pincel). Gravar `realOil` nos presets Oil/* — muda a
   miniatura e a receita: regenerar e passar `cada_preset_de_fabrica_e_a_sua_receita` +
   `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`; os aprovados contra o Rebelle passam de novo pelo EXAME.
2. Tinta sem óleo não recebe a sombra da tinta com óleo vizinha.
3. Papel a ≤ 24 células de um traço mais alto escurece (sombra de fissura).
4. Blender mode: croma a 1 acima do Cycles.
5. ~10 px de pontinhos do Dabber sobre Cold 0,6 (família §BUG-papel, `oraculo/README.md`).
6. Presets Wet de fábrica ainda levam `paper_depth: 1.0` ignorado (limpar ao regenerar o item 1).
7. Pré-existente: o Painter não tem Auto Key (decisão 19/10 «grava tudo»); mexer num knob não entra no desfazer.

Fecho da linha (clippy `PH2D_SUITE=fecho`, nextest-impacted) **não corrido** nesta janela — fica no fecho da JANELA 6
(`JANELA_6.md` item 6).
