# HANDOFF de continuação — linha PainterWatercolor, noite de 05/11

Branch `line/PainterWatercolor`. Não é handoff de integração (o último é [HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-05.md](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-05.md)); complementa [HANDOFF_CONTINUACAO_2026-11-05_bordas_e_smooth_oleo.md](HANDOFF_CONTINUACAO_2026-11-05_bordas_e_smooth_oleo.md). Bugs: [BUGS_painter.md](../BUGS_painter.md) (#69, #70, #71). **Smoke entregue na conversa** (não copiado aqui).

## Commits da onda

| Commit | O quê |
|---|---|
| `9b8e5ffb3f` | Bug #69 curado: Edge Darkening por traço |
| `26b119b58d` | Bug #70 curado: a Borracha no Wet apaga a tinta antiga |
| `1a2cc2cf9c` | Smooth Edges 0,3 de fábrica em 13 pincéis a óleo |
| `9bd76cd02d` | Bug #71 medido; cura desligada atrás de `PH2D_PAPEL_SEM_DEGRAU` |

## A — Bordas (Bug #69, `9b8e5ffb3f`) — curado, a confirmar no smoke do dono

- `AparenciaCelula::borda` (28 B), média pela massa. `Grid::borda_fundo` (f64) = o do 1.º pincel que tocou a grade; nunca muda, nem no Clear (o histórico pode devolver células que o lêem). Célula com `BORDA_DO_FUNDO` (-1) lê o fundo: tela de um pincel só não aloca o campo.
- Placa: `aux.w`, nas duas leis (`seca.wgsl`, `seca_tinta.wgsl`); binding 7 acrescentado na `liga_tinta`.
- Vermelho antes: Wet Liner ED 0→56,25 (4332 px, Δ30); Wet Round ED→0 (9954 px, Δ22). Depois: (0,0) ambos. Alarmes de pincel verdes sem regravar.
- Sobra: `sonda_bordas_qual_knob` Wet Liner→Wet Dry Brush (92 px, Δ3) é do knob Capillary, global, só ele. Varredura dos 55 B com traço longe: ecrã até Δ 3 (antes Δ 46); o MESMO pincel como B dá (698 px, Δ2) = assentar normal/cadência, não a borda.
- Limitação: pigmento que a água leve além da orla virgem (2 células) numa grade com 2 bordas lê o fundo.
- NÃO verificado: cena `PH2D_WETPAINT_SMOKE=4` (se deixa de variar) e o dono ter dito «até seco».

## B — Borracha (Bug #70, `26b119b58d`) — curado (opção A), a confirmar no smoke

- `wetpaint/borracha_na_base.rs`: cobertura pela borracha normal num escudo opaco (lei de `aplica_borracha_do_traco`), com a sessão retirada para `wet_owns_the_dabs` dar falso; `splat_keep` para selecção e protecção; alfa preso não apaga; base copia só havendo o que apagar, 1× por passo (7,1 ms a 4096×2560); `merge_dirty_otica` recompõe.
- Undo: `PassoVivo.base_antes/base_depois`, Arc partilhado, contado no orçamento.
- Portões: `a_borracha_no_wet_apaga_a_tinta_antiga_de_cada_meio`, `desfazer_a_borracha_no_wet_devolve_a_tinta_antiga`, `a_selecao_protege_a_tinta_antiga_da_borracha`.
- ABERTO novo: o relevo do Impasto fica depois da Borracha, COM OU SEM Wet (105,97 → 105,97; controlo sem Wet igual, tinta 0,0): a Borracha do trilho tem `brush.impasto=false`, `impasto_batch_active` falso. Pré-existente; sonda `sonda_relevo_do_impasto_sob_a_borracha`.
- NÃO verificado: clique no app real; vidro de aguada em camadas; várias camadas (a sessão tem uma base só, a tela no nascimento). Respostas do dono sobre o menu vieram sem preencher.

## C — Relevo (Bug #71, `9bd76cd02d`) — medido, cura DESLIGADA

- Mecanismo: `relief_fields.rs height_at` = `h + dente×papel_a_vista`; o dente (0..0,0625, média 0,0313 em Cold a Relief 1) é tapado pela tinta, a altura média cai no traço: degrau para baixo. Luz `Light::KEY` azimute 230 (convenção não conferida; o sinal casa com a foto).
- Métrica S (`sonda_relevo_negativo`): luz da orla de baixo − de cima contra o papel nu, menos o mesmo com Relief 0; S>0 = afundada. 46 pincéis (10 sem orla: Blender/Smear/Water): média +0,048, máx +0,115 (Metallic Flat); Wet Liner +0,080 com ED 0, logo não é o Edge Darkening.
- Cura: `papel + meio_do_dente×(1−v)` com `PH2D_PAPEL_SEM_DEGRAU=1`: média +0,005, máx +0,024, mín −0,017. Papel nu ao byte por construção. Alarme de miniatura verde com a chave ligada. Paper Press 0×1 com Real Oil: Knife Stain (15634, Δ44) → (15449, Δ42); Knife Oily (25783, Δ136) → (25647, Δ136): o controlo fica.
- Custo visto: o miolo de Soft Pastel e Graphite 8B perde grão em relevo (até Δ 78 / Δ 50).
- Prancha: [relevo_antes_depois.png](material_2026-11-05/relevo_antes_depois.png).
- Não medido: só Cold; imagem de papel própria (`meio_do_dente` assume amp/2); alarme de largura com a chave.
- Se o dono aprovar: tirar a chave, portão com S, varrer papéis.

## D — Óleos (item 3, `1a2cc2cf9c`)

Smooth Edges 0,3 de fábrica nos 13 pincéis a óleo, regravados por ordem do dono; os alarmes acusaram só eles. Relatório guardado em `target/relatorio-oleos-smooth-0_3/index.html` (em `target/`, não versionado). No raio 2 a largura engorda: Flat Bristle 3,0→5,0; Metallic Flat 3,88→5,87; Express Oil (raio 26) 52,17→49,24.

## Próximos

1. Decisão do dono no #71 (ligar `PH2D_PAPEL_SEM_DEGRAU` por omissão ou não).
2. Lei antiga × nova (cena 4) e Bug #54: sem resposta do dono (vieram sem preencher no prompt).
3. Relevo do Impasto sob a Borracha (ABERTO em B).

## Regras do dono vigentes

Macro com Grain fica fora desta linha. Não pedir regra de furos. Os pincéis de hoje são a referência ([memória](../../../project-memory/project_brushes_as_they_are_are_the_reference_2026_11_05.md)).

## Prova

Provas de mutação NÃO foram feitas nesta onda (ficam para o fecho da linha). Suíte completa só na integração.

## Achados do fecho
- As 13 receitas (`tests/it/familias/oleo_acrilica.rs`, `cap01.rs`) levam `("smoothEdges", 0.3)`: sem isto o gerador apagava a ordem do dono. Os candidatos W2 não a herdam.
- A cena `PH2D_WETPAINT_SMOKE=4` mudou-se para `ph2d-tool-painter` (`wetpaint_casos.rs`): pôs `ph2d-app-painter` 193 linhas acima da catraca.
- ⚠️ `render.rs:393`: com o campo da aparência alocado, `paper: true` acende o emboss da massa mesmo com `relevo: false` (+15 em 46 de 6 052 células em `o_molhado_sobre_tinta_fina_nao_clareia_alem_das_tintas`). O produto corre sempre `relevo: true` (`otica_factos.rs:87`), logo alocar o campo não muda a tela; o portão passou a medir só a cor (`paper: false`). `medida_oleo_real_fenda` fixa Smooth Edges 0 para o hash do Clear glass valer.
- O Smooth Edges 0,3 custa memória no Dabber: pico 305→368 MiB a raio 300, 1 610→1 802 a raio 1000. Tecto de `a_memoria_do_dabber_cabe` remedido (386 · 1 892).

## Depois do smoke do dono (05/11, noite): lei nova em tudo
Veredictos: Borracha OK; óleos OK; relevo «melhorou quase 100%». Decisões: relevo ligado para todos (a chave `PH2D_PAPEL_SEM_DEGRAU` saiu); **lei nova para todos os pincéis, sem a grelha nem a régua de furos; o dono testa ele mesmo**. Backup: tag `lei-antiga` e ramo `backup/lei-antiga-2026-11-05` (`56f890198b`), locais; a lei velha continua no código e `PH2D_LEI_ANTIGA=1` pinta com ela. Mudou só o padrão da ferramenta (`lei_de_fabrica`, `wetpaint.rs`); o motor continua `Velha` por omissão nos testes dele.
**A integração NÃO pode correr ainda:** portões do produto escritos para a lei velha ficaram vermelhos, ex. `o_azul_abre_uma_mancha_redonda_no_vermelho_molhado` (`PH2D_LEI_ANTIGA=1` verde). Os alarmes de pincel ficam vermelhos de propósito, sem regravar, até o dono aprovar; relatório em `target/relatorio-lei-nova/index.html`. Contagem total dos vermelhos: não medida (o filtro `wetpaint` estourou 500 s a load 23). Próximo: dono testa → regrava miniaturas e larguras → triar os vermelhos por `PH2D_LEI_ANTIGA=1` → Bug #72.

## Revogado: de volta à lei antiga (dono 05/11, depois de testar a nova)
«problemas e mais problemas: melhor voltar para lei antiga e descartar a nova». A ferramenta volta a `LeiDaTinta::Velha`; `lei_de_fabrica` e `PH2D_LEI_ANTIGA` saíram. A secção acima fica só como história: os alarmes e os portões voltaram a valer. Ficam: relevo sem degrau (com portão `a_tinta_nao_fica_afundada_no_papel_de_alto_relevo`), bordas, Borracha, óleos, pen-down sem `marca_tudo`. O dono não disse quais problemas viu. ABERTO: apagar o CÓDIGO da lei `Pigmento` (`grid/tinta`, `ph2d-paint-tinta`, portões, ADR-0197) não foi feito: pedir confirmação do alcance. Bug #72 (1.º traço) continua aberto; os números da lei antiga são os que valem.
