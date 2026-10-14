# 26 — O que está ABERTO no Painter (a lista única, 13/10)

> Pedido do dono (13/10): *«deixe tudo bem documentado para não esquecer»*. Um item por linha, com a FONTE
> (onde está medido) e o estado. Quem fecha um item risca-o aqui, com o commit, e move o mecanismo para o
> índice dos fechados ([`BUGS_painter_fechados_41_a_46.md`](../BUGS_painter_fechados_41_a_46.md)).
> ⚠️ **Ordem do dono (07/10): só o Wet Paint evolui** — Digital, Impasto e Watercolor ficam
> ([memória](../../../project-memory/project_wet_paint_is_the_only_painter_mode_to_evolve_2026_10_07.md)).

## A — Em trabalho

| # | O quê | Fonte | Estado |
|---|---|---|---|
| A1 | **A seção Shape vira a CASA dos pincéis em todas as mídias** — os pincéis do Wet Paint (hoje dentro do motor, invisíveis) passam a aparecer e a configurar-se no Shape; as texturas deles no dropdown; os knobs de ponta saem do Tuning. Engloba o **Bug #47** (Shape Noise no Wet Paint quase não pinta). | ordem do dono 13/10 · [plano 27](27_plano_shape_casa_dos_pinceis.md) · [BUGS #47](../BUGS_painter.md) | **W0, W1 e W2 feitas (13/10)**: o Wet Paint lê as cerdas do Shape, presets migrados ao byte, a família do #47 pinta — [27 §6](27_plano_shape_casa_dos_pinceis.md). Próxima: W3 a lei B para o Shape sobre o Grain + W4 o painel por mídia ([prompt 30](30_PROMPT_janela_shape_casa_W3_W4.md)) |
| A2 | ~~**O Spacing pontilha no Digital e não no Wet Paint**~~ — **feito (prompt 32, 13/10):** as cerdas do Digital/Impasto ARRASTAM pelo passo do dab (a média do tapete ao longo do passo, `Dab::step` → `ShapeFrame::Stroke`); CV ao longo a Spacing 0,1 / 0,25 / 0,5: 0,206 → **0,014** · 0,513 → **0,052** · 0,763 → **0,102**; a tinta é a de hoje (± 1 %); o Wet Paint ao byte. **Smoke do dono:** virou a OPÇÃO **Continuous** (caixa no Stroke, desligada por omissão), também no Wet Paint (pelo máximo) e com o 1.º dab arrastado ([31 §4.4](31_spacing_por_meio_o_pontilhado.md)). Dab-cápsula da redonda: **não entra** (medido). Restos: B5, C11. | relato do dono 13/10 · [31 §4](31_spacing_por_meio_o_pontilhado.md) | feito |
| A3 | **O Continuous não iguala o Spacing pequeno** — retângulos e menos nítido (smoke do dono 13/10, 3.ª rodada): a queda é pesada no centro do dab, não na sub-posição de cada pelo; no Wet Paint o máximo faz tracinhos. Desenho: o dab-cápsula com a textura dentro, para QUALQUER ponta e nos cinco meios (dono: *«continuous deve estar disponível para os outros modos de pintura também»*). | [prompt 34](34_PROMPT_janela_continuous_capsula.md) | próxima janela |


## B — Por decidir (do DONO)

| # | O quê | Fonte |
|---|---|---|
| B1 | **Fios do Sketchy/Wire/travessas da Ribbon com Randomize Color «a cada carimbo»:** hoje UMA cor por lote (a média da ponta, ou o Color). Uma cor por fio pede o `Thread` com cor (`[f32; 4]` hoje) e um rasterizador de cor. | [handoff #53 §6](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG53_SOLID_DE_CARIMBOS.md) |
| B2 | **A Máscara com «Use Texture Colors»:** a máscara pinta com `Dab { color: preto/branco }` e o branco dá a TEXTURA (antes do #51 pintava sempre a cor da textura). Deve ignorar a cor da textura (só luminância)? | [handoff #51 §5](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG51_TEXTURA_TINGE.md) |
| B3 | **O Push do Impasto (a tinta EMPURRADA):** *«adiar para o final de toda essa implementação. Fim da fila»* (dono, 12/07). | [BUGS #14](../BUGS_painter.md) |
| B4 | **O esfregão do Composite (resíduo do #24):** a perda segue o MÓDULO do deslocamento; a única cura medida é limitá-lo — o que o dono recusou. As duas saídas estão gateadas de cada lado. | [BUGS #24](../BUGS_painter.md) §Recusas |
| B5 | **O Spacing do Wet Paint com o significado do Digital:** hoje o passo sai do raio do MOTOR (1,0–2,6× o pedido, medido nos 12 presets); a lei proposta mede-o sobre o raio do PAINEL. Muda o traço com Spacing ≠ fábrica e 5 presets de fábrica (Charcoal, Soft/Oil Pastel, Wet Dry Brush, Oil Bristle) — ver a prancha. | [31 §4.3](31_spacing_por_meio_o_pontilhado.md) · [prancha](32_prancha_spacing_wet_hoje_proposta.png) |

## C — Defeitos e lacunas conhecidos (técnicos)

| # | O quê | Fonte |
|---|---|---|
| C1 | **«Use Texture Colors» é um controlo MORTO no Wet Paint e no Watercolor** (eles nunca lêem a cor da textura): tem de aparecer esmaecido nessas mídias — um `Dependente` em `inercia.rs` + a linha no censo `censo_dos_controlos`. ⚠️ Entra no plano 27 (o Shape por mídia). | [handoff #51 §5](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG51_TEXTURA_TINGE.md) |
| C2 | **Solid no Wet Paint / Watercolor:** lá a mancha é ÁGUA (enche ao soltar / cobertura da aguada) e não lê a ponta de imagem nem a cor da textura. ⚠️ Entra no plano 27 (Wet Paint). | [handoff #53 §6](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG53_SOLID_DE_CARIMBOS.md) |
| C3 | **A ponta de imagem no Watercolor é cortada pelo DISCO** (`dn ≥ 1` em `watercolor_accum_cor.rs`) — por lei da aguada, mídia congelada. | [handoff #52 §7](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG52_PONTA_CORTADA.md) |
| C4 | **O diário de desfazer tem uma segunda rede** que torna o `dabs_bounds` invisível ao portão (`desfazer_apaga_a_ponta_alargada_inteira` não sangra com `tip_reach = 1`, nem na fronteira de um bloco de 128). Descobrir qual. | [handoff #52 §4](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG52_PONTA_CORTADA.md) |
| C5 | **Per-Layer Color: linhas retangulares INTERMITENTES** — dormente, sem reprodução; a armadilha re-ativável está no doc. | [BUGS #11](../BUGS_painter.md) |
| C6 | **Watercolor OFF→ON no meio do traço** e **a paridade banda × serial depende da máquina** (num runner de 1 núcleo compara serial com serial). | [BUGS #13](../BUGS_painter.md) |
| C7 | **A cegueira do #25 sob Simetria/Spray/Rough** (o Composite). | [BUGS #25](../BUGS_painter.md) |
| C8 | **A Ribbon não se mede sem relógio entre eventos** (a mola não anda: 32 px redonda, 34 com folha) — os portões dos Types não a alcançam. | [handoff #53 §6](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG53_SOLID_DE_CARIMBOS.md) |
| C9 | **O Flow do Shape continua com a caixa do raio** (é a largura do traço; o comprimento ladrilha) — por desenho, a confirmar com o dono quando o Flow for revisto. | [handoff #52 §7](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG52_PONTA_CORTADA.md) |
| C10 | ~~**O 1.º dab de um traço com cerdas sai em pontos**~~ — curado com a opção ligada: o traço aquece como o Rake e o dab de abertura recebe um passo nominal ([31 §4.4](31_spacing_por_meio_o_pontilhado.md)). | feito |
| C11 | **O arrasto custa ~15 % numa CURVA a Spacing 0,25** (o passo em texels muda de dab para dab e cada um refaz os texels que toca; recto e os outros Spacings ≤ hoje). | [31 §4.1](31_spacing_por_meio_o_pontilhado.md) |
| C12 | **Wet Paint com Continuous a Spacing 0,25: uma costura vertical fina entre dois dabs** (vê-se na prancha; o rasto de cada dab acaba no meio do passo e a água não a apaga). | [31 §4.4](31_spacing_por_meio_o_pontilhado.md) · [prancha](32_prancha_continuous_desligada_ligada.png) |

## D — Processo

| # | O quê | Fonte |
|---|---|---|
| ~~D1~~ | ~~O `PROMPT_LINHAS_RAPIDAS_2026-10-11.md` (as regras R1–R9) por versionar no primário~~ — FECHADO: versionado em `docs/archive/integracao-jornadas/` e os links dos handoffs já apontam para lá (conferido 13/10). | [integração 12/10 §2](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-12.md) |
