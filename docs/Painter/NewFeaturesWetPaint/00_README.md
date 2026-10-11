# NewFeaturesWetPaint — o Rebelle 8 como oráculo do Painter (2026-10-07)

> **Pedido do dono (07/10):** *«avaliação paralela de ambos [Painter × Rebelle]; o que o Rebelle tem que
> ainda não temos; porque o traço dele é superior em algumas características; como ele configura cada
> pincel de cada mídia; o que falta para simular todas as mídias; como configurar os presets de cada mídia»*
> — e dois docs: o plano técnico do que implementar e como, e os **presets para o Wet Painter**.
>
> **Oráculo:** Rebelle 8 Pro 8.3.4 (teste de 30 dias) sob Wine 11, prefixo `~/Apps/rebelle/wineprefix`.
> Proprietário ⇒ **oráculo que se corre, nunca fonte que se lê** (CLAUDE.md §0.9). Nada do executável
> foi lido. O que se leu: o **manual** instalado (documentação pública), os **DADOS** dos presets de
> fábrica (o que o painel Brush Creator mostra a qualquer utilizador — ficheiros `*.rebelle-brush.png`
> com os parâmetros em texto), e as **imagens** que ele pinta a partir de gestos NOSSOS. ⛔ As texturas
> de forma/grão/papel embutidas nos presets são **assets** do alvo: nunca se copiam (§3 passo 3 do
> [método](../../_ComoInvestigarApps/00_o_metodo.md)). ⛔ A mistura por pigmentos do Rebelle é o
> **Mixbox (CC BY-NC 4.0)** — não é porta para o PH2D; a nossa é a Kubelka–Munk própria (`ph2d-pigment`).

> ⚠️ **Decisão do dono (07/10, depois dos docs 01–03):** *Digital, Impasto e Watercolor NÃO mudam; só o
> Wet Paint evolui* — e passa a simular todas as mídias («Wet Painter»). O plano do 01 §4 re-aponta-se ao Wet
> Paint no **[04_PROMPT_linha_wet_painter.md](04_PROMPT_linha_wet_painter.md)**, que é o que o agente da linha
> recebe.

| Doc | O que tem |
|---|---|
| [19_PROMPT_janela_linha_rapida.md](19_PROMPT_janela_linha_rapida.md) | **O prompt da PRÓXIMA janela** (dono 11/10): a onda da velocidade — `opt-level` para `ph2d-wet-paint`/`ph2d-pigment`, os testes > 10 s (incluindo os dois portões da fábrica de 11/10), o gerador em paralelo, o 05 partido com o índice no mesmo nome |
| [18_mapa_das_midias.md](18_mapa_das_midias.md) | **O mapa das mídias** (11/10): cada mídia do dono contra os 281 presets e 25 fotos do Rebelle, medida à largura igual com o controlo dentro (refeita a pedido do dono: pigmento alto, o relevo de volta, o grão do pincel) — aquarela + **Wet Liner**, óleo, acrílica, Express Oil, grafite, carvão, pastel seco e de óleo (fábrica de 12); a métrica do CRUZAMENTO (a tinta e o marcador escurecem, os nossos tapam); o que fica de fora e a decisão do dono |
| [17_PROMPT_janela_presets_todas_as_midias.md](17_PROMPT_janela_presets_todas_as_midias.md) | O prompt da janela de 11/10 (cumprido: 05 §4b «Presets de todas as mídias») — os presets de TODAS as mídias no Wet Paint: o mapa das mídias medido contra os 281 presets e as fotos do Rebelle, a W2 (a largura prometida, a base de todo preset) e o 1.º lote de presets novos, cada um com um portão que reprova o Wet Round |
| [16_PROMPT_janela_auditoria_mistura_simplificacao.md](16_PROMPT_janela_auditoria_mistura_simplificacao.md) | O prompt da auditoria multi-agente da mistura (o azul-claro do Wet Drip com as caixas K–M) — o estado no 05 §4b «Mistura II» |
| [15_PROMPT_janela_auditoria_mistura.md](15_PROMPT_janela_auditoria_mistura.md) | O prompt da auditoria e cura da mistura de pigmentos (amarelo sobre azul seco dá preto e nunca cobre; smoke do dono 10/10), as hipóteses a medir e a barra pelo Rebelle |
| [14_smooth_edges_a_borda_suave.md](14_smooth_edges_a_borda_suave.md) | **O Smooth Edges do Wet Paint** (W1g, aprovado pelo dono 10/10): a lei da borda suave por raio, as três tentativas recusadas e a lição (a dureza que se vê é o contorno escuro por DENTRO da mancha), os portões e o custo |
| [13_PROMPT_janela_W1g_opcoes.md](13_PROMPT_janela_W1g_opcoes.md) | O prompt da W1g (Live Water, Heavy Edge, fios finos) — o estado no 05 §4b «W1g» |
| [11_PROMPT_janela_W1e_agua_papel_tilt.md](11_PROMPT_janela_W1e_agua_papel_tilt.md) | **O prompt da PRÓXIMA janela** (W1e — escorrimento realista só com água, pigmento e Tilt; um só papel, o do painel; o pincel seco nos picos; o Tilt no preset), com as medições de 09/10 |
| [10_PROMPT_janela_W1d_cura.md](10_PROMPT_janela_W1d_cura.md) | ⛔ OBSOLETO — as gotas foram DESFEITAS a 09/10 (05 §4b «⏪ DESFEITO»; ficou só o Smooth Edges, desligado). Registo da auditoria de 09/10 ([09](09_PROMPT_janela_auditoria_W1c.md)) |
| [07_PROMPT_janela_W1b_gotas.md](07_PROMPT_janela_W1b_gotas.md) · [08](08_PROMPT_janela_W1c_borda.md) | Os prompts das janelas W1b (as gotas pela lei do Rebelle) e W1c (a borda e as gotas pela auditoria de 08/10) |
| [05_plano_da_linha_e_o_antes.md](05_plano_da_linha_e_o_antes.md) | **O tracker da linha**: o «antes» do Wet Paint medido e reproduzido, onde cada onda entra no código, a barra de cada gate tirada do vale, as decisões do dono e o estado de cada onda |
| [06_gotas_rebelle_x_ph2d.md](06_gotas_rebelle_x_ph2d.md) | **As gotas**: a lei do Rebelle (tela + papel, nunca o pincel) lida dos dados e medida na bancada, e a nossa ao lado; a proposta que o dono aprovou |
| [04_PROMPT_linha_wet_painter.md](04_PROMPT_linha_wet_painter.md) | **O prompt da linha `line/WetPainter`**: objectivo, leituras, a cerca (os outros meios byte-idênticos), as ondas W0–W8 re-apontadas ao Wet Paint com o gate de cada uma, as regras |
| [01_o_que_o_rebelle_tem_e_nos_nao.md](01_o_que_o_rebelle_tem_e_nos_nao.md) | A avaliação paralela: inventário dos dois lados, a tabela de lacunas por família, **porque o traço dele é superior** (as sete leis medidas e as refutadas), e o **plano de implementação** por onda, com gates |
| [02_presets_para_wet_painter.md](02_presets_para_wet_painter.md) | Como o Rebelle configura cada meio (esquema de 200 parâmetros, lido de 281 presets de fábrica), a tradução para o nosso `BrushSpec`, o **formato de preset** proposto e os **presets de arranque de cada mídia** |
| [03_tracos_medidos_rebelle_x_ph2d.md](03_tracos_medidos_rebelle_x_ph2d.md) | A bancada: os mesmos gestos nos dois programas, a régua (`ferramentas/mede_traco.py`), as tabelas e o que elas dizem |
| [fixtures/](fixtures/) | `rebelle/` (as 25 fotos do oráculo sobre os nossos gestos, com cabeçalho) · `rebelle_presets_281.json` (os parâmetros dos 281 presets, em texto) · `sonda_28/` (a nossa sonda em `main` 2a925859d: o «antes» e a imagem da cerca) |
| [ferramentas/](ferramentas/) | `cerca.sh` (a cerca por ficheiros: os outros meios só ganham linhas) · `fixtures_rebelle.py` · `mede_todos.sh` · `extrai_pinceis.py` · `schema.py` · `por_meio.py` · `curvas.py` · `difere.py` (os dados dos presets) · `mede_traco.py` (a régua dos dois lados) · `midias_regua.py` + `prancha_midias.sh` (as mídias, 18) · `tracos_rebelle.sh` · `reinicia.sh` · `xtree.py` (a porta do oráculo) |

A sonda do nosso lado vive em `crates/ph2d-tool-painter/tests/it/sonda_tracos_por_meio.rs` (`#[ignore]`):

```
PH2D_SONDA_OUT=/tmp/x cargo test -p ph2d-tool-painter --profile smoke --test it sonda_tracos_por_meio -- --ignored --nocapture
```
