# 26 — O que está ABERTO no Painter (a lista única, 13/10)

> Pedido do dono (13/10): *«deixe tudo bem documentado para não esquecer»*. Um item por linha, com a FONTE
> (onde está medido) e o estado. Quem fecha um item risca-o aqui, com o commit, e move o mecanismo para o
> índice dos fechados ([`BUGS_painter_fechados_41_a_46.md`](../BUGS_painter_fechados_41_a_46.md)).
> ⚠️ **Ordem do dono (07/10): só o Wet Paint evolui** — Digital, Impasto e Watercolor ficam
> ([memória](../../../project-memory/project_wet_paint_is_the_only_painter_mode_to_evolve_2026_10_07.md)).

## A — Em trabalho

| # | O quê | Fonte | Estado |
|---|---|---|---|
| A1 | **A seção Shape vira a CASA dos pincéis em todas as mídias** — os pincéis do Wet Paint (hoje dentro do motor, invisíveis) passam a aparecer e a configurar-se no Shape; as texturas deles no dropdown; os knobs de ponta saem do Tuning. Engloba o **Bug #47** (Shape Noise no Wet Paint quase não pinta). | ordem do dono 13/10 · [plano 27](27_plano_shape_casa_dos_pinceis.md) · [BUGS #47](../BUGS_painter.md) | **W0–W4 feitas (13–14/10)**: o Wet Paint lê as cerdas do Shape, presets migrados ao byte; a **lei B** (o padrão do Shape multiplica depois do portão) fecha o #47 — 27 §6.8; o painel por mídia (C1) e o Solid de carimbos na água (C2) — 27 §6.9. Próxima: W5 (Flat/Fan do motor saem, doc do Shape) e o smoke do dono (B6) |
| A2 | ~~**O Spacing pontilha no Digital e não no Wet Paint**~~ — **feito (prompt 32, 13/10):** as cerdas do Digital/Impasto ARRASTAM pelo passo do dab (a média do tapete ao longo do passo, `Dab::step` → `ShapeFrame::Stroke`); CV ao longo a Spacing 0,1 / 0,25 / 0,5: 0,206 → **0,014** · 0,513 → **0,052** · 0,763 → **0,102**; a tinta é a de hoje (± 1 %); o Wet Paint ao byte. **Smoke do dono:** virou a OPÇÃO **Continuous** (caixa no Stroke, desligada por omissão), também no Wet Paint (pelo máximo) e com o 1.º dab arrastado ([31 §4.4](31_spacing_por_meio_o_pontilhado.md)). Dab-cápsula da redonda: **não entra** (medido). Restos: B5, C11. | relato do dono 13/10 · [31 §4](31_spacing_por_meio_o_pontilhado.md) | feito |
| A3 | ~~**O Continuous não iguala o Spacing pequeno**~~ — **feito (prompt 34, 13/10):** o dab-CÁPSULA (`ph2d-painter-brush/src/capsula.rs`): os sub-dabs de volta ao centro anterior a 0,025 do diâmetro, a ponta INTEIRA deslocada — qualquer ponta, os cinco meios (carimbo: a União `1 − Π(1 − c·x_k)`; Watercolor: o máximo por sub-dab; Wet Paint: os sub-dabs entregues ao motor). A 0,25 com a caixa = o traço a 0,025 (régua ≤ 0,017, tinta ± 3 %) a 0,26–0,57× o custo dele; a varredura do prompt 32 saiu. **Smoke do dono ✓ (13/10: «funciona muito bem»).** Restos: C13–C15. | [31 §5](31_spacing_por_meio_o_pontilhado.md) · [prancha](34_prancha_continuous_capsula.png) | feito |


## B — Por decidir (do DONO)

| # | O quê | Fonte |
|---|---|---|
| B1 | **Fios do Sketchy/Wire/travessas da Ribbon com Randomize Color «a cada carimbo»:** hoje UMA cor por lote (a média da ponta, ou o Color). Uma cor por fio pede o `Thread` com cor (`[f32; 4]` hoje) e um rasterizador de cor. | [handoff #53 §6](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG53_SOLID_DE_CARIMBOS.md) |
| B2 | **A Máscara com «Use Texture Colors»:** a máscara pinta com `Dab { color: preto/branco }` e o branco dá a TEXTURA (antes do #51 pintava sempre a cor da textura). Deve ignorar a cor da textura (só luminância)? | [handoff #51 §5](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG51_TEXTURA_TINGE.md) |
| B3 | **O Push do Impasto (a tinta EMPURRADA):** *«adiar para o final de toda essa implementação. Fim da fila»* (dono, 12/07). | [BUGS #14](../BUGS_painter.md) |
| B4 | **O esfregão do Composite (resíduo do #24):** a perda segue o MÓDULO do deslocamento; a única cura medida é limitá-lo — o que o dono recusou. As duas saídas estão gateadas de cada lado. | [BUGS #24](../BUGS_painter.md) §Recusas |
| B6 | **A ponta LISA do Wet Paint (Shape None) deposita 4,7–6,2× a tinta das cerdas de fábrica** — é a lisa que o dono espera? Se não, a calibração é um número só (`texv = 1`), medido contra o Round do Rebelle (`fixtures/rebelle/ponta_textura_round_*`). E sobre o GRÃO dos pastéis o Shape procedural rende 4–5× menos que sobre a lisa (Noise 0,11 × 0,45–0,60; previsto pela W0) — forte o bastante? | [27 §6.7–6.9](27_plano_shape_casa_dos_pinceis.md) |
| B5 | **O Spacing do Wet Paint com o significado do Digital:** hoje o passo sai do raio do MOTOR (1,0–2,6× o pedido, medido nos 12 presets); a lei proposta mede-o sobre o raio do PAINEL. Muda o traço com Spacing ≠ fábrica e 5 presets de fábrica (Charcoal, Soft/Oil Pastel, Wet Dry Brush, Oil Bristle) — ver a prancha. | [31 §4.3](31_spacing_por_meio_o_pontilhado.md) · [prancha](32_prancha_spacing_wet_hoje_proposta.png) |

## C — Defeitos e lacunas conhecidos (técnicos)

| # | O quê | Fonte |
|---|---|---|
| ~~C1~~ | ~~**«Use Texture Colors» MORTO no Wet Paint e no Watercolor**~~ — **feito (W4, 14/10):** `Dependente::CorDaTexturaSemLeitor` (esmaece com a água ou a aquarela) + o gesto `PontaColorida` e a pré-condição no censo ([27 §6.9](27_plano_shape_casa_dos_pinceis.md)). Resto: C16. | [handoff #51 §5](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG51_TEXTURA_TINGE.md) |
| ~~C2~~ | ~~**Solid no Wet Paint não lê a ponta de imagem**~~ — **feito (W4, 14/10):** com ponta de IMAGEM a mancha da água é a União das silhuetas dos carimbos, pela máscara do motor (dab a dab pela rota do traço ficava rala, 39/400 — recusado) ([27 §6.9](27_plano_shape_casa_dos_pinceis.md)). A Watercolor fica (mídia congelada). | [handoff #53 §6](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG53_SOLID_DE_CARIMBOS.md) |
| C3 | **A ponta de imagem no Watercolor é cortada pelo DISCO** (`dn ≥ 1` em `watercolor_accum_cor.rs`) — por lei da aguada, mídia congelada. | [handoff #52 §7](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG52_PONTA_CORTADA.md) |
| C4 | **O diário de desfazer tem uma segunda rede** que torna o `dabs_bounds` invisível ao portão (`desfazer_apaga_a_ponta_alargada_inteira` não sangra com `tip_reach = 1`, nem na fronteira de um bloco de 128). Descobrir qual. | [handoff #52 §4](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG52_PONTA_CORTADA.md) |
| C5 | **Per-Layer Color: linhas retangulares INTERMITENTES** — dormente, sem reprodução; a armadilha re-ativável está no doc. | [BUGS #11](../BUGS_painter.md) |
| C6 | **Watercolor OFF→ON no meio do traço** e **a paridade banda × serial depende da máquina** (num runner de 1 núcleo compara serial com serial). | [BUGS #13](../BUGS_painter.md) |
| C7 | **A cegueira do #25 sob Simetria/Spray/Rough** (o Composite). | [BUGS #25](../BUGS_painter.md) |
| C8 | **A Ribbon não se mede sem relógio entre eventos** (a mola não anda: 32 px redonda, 34 com folha) — os portões dos Types não a alcançam. | [handoff #53 §6](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG53_SOLID_DE_CARIMBOS.md) |
| C9 | **O Flow do Shape continua com a caixa do raio** (é a largura do traço; o comprimento ladrilha) — por desenho, a confirmar com o dono quando o Flow for revisto. | [handoff #52 §7](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG52_PONTA_CORTADA.md) |
| C10 | ~~**O 1.º dab de um traço com cerdas sai em pontos**~~ — a cápsula vai PARA TRÁS, então o 1.º dab é o 1.º do traço denso (régua da abertura ≤ 0,016); o aquecimento e o passo nominal saíram ([31 §5](31_spacing_por_meio_o_pontilhado.md)). | feito |
| C11 | ~~**O arrasto custa ~15 % numa CURVA a Spacing 0,25**~~ — a varredura saiu; a cápsula custa 0,26–0,57× o traço denso que iguala ([31 §5.3](31_spacing_por_meio_o_pontilhado.md)). | feito |
| C12 | ~~**Wet Paint com Continuous a Spacing 0,25: uma costura vertical fina entre dois dabs**~~ — os sub-dabs chegam ao motor: costura 0,000–0,002 (a do alvo) ([31 §5](31_spacing_por_meio_o_pontilhado.md)). | feito |
| C13 | **A cápsula não chega à rota das camadas de COR do Shape** (Per-Layer Color com imagem: `stamp_dabs_per_layer_dynamic` / `cached_color*`) nem ao AA do filme do Impasto (Smooth Edges: cada sub-dab leva o corte do filme, sem o anel fraccional). | [31 §5.4](31_spacing_por_meio_o_pontilhado.md) |
| C14 | **Reconfirmar o custo da cápsula no Wet Paint com a máquina calma**: medido a `load` 13–120, o traço com pressão deu 1,43× o denso (o resto 0,53–0,90); `measure_custo_da_capsula`. | [31 §5.3](31_spacing_por_meio_o_pontilhado.md) |
| C16 | **As linhas «Layer N Color» (Shape de VÁRIAS camadas, Per-Layer ligado) não esmaecem no Wet Paint/Watercolor** — o pai («Per-Layer Color») esmaece; os filhos têm ids de fábrica (`painter_shape_layer_color_check_id`), fora do `Dependente::ids` estático e do censo (que só arma uma camada). | [27 §6.9](27_plano_shape_casa_dos_pinceis.md) |
| C15 | **Dois ficheiros no tecto de LOC**: `tool/paint/stamp_cache.rs` 700/700 e `painter-brush/src/stroke.rs` 698/700 — o próximo campo ou rota ali exige CORTE primeiro (nunca isenção). | handoff P34 §4 |

## D — Processo

| # | O quê | Fonte |
|---|---|---|
| ~~D1~~ | ~~O `PROMPT_LINHAS_RAPIDAS_2026-10-11.md` (as regras R1–R9) por versionar no primário~~ — FECHADO: versionado em `docs/archive/integracao-jornadas/` e os links dos handoffs já apontam para lá (conferido 13/10). | [integração 12/10 §2](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-12.md) |
