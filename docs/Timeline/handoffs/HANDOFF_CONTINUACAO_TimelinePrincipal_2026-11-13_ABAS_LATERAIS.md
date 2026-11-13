# HANDOFF CONTINUAÇÃO — line/TimelinePrincipal, abas laterais (dono 13/11)

Leitor: a próxima LLM. Sem rebase, sem push, sem integração. Tudo o que veio antes está commitado (HEAD `bea69a5a8b` + este doc)
e anotado no [handoff de integração](HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-12_PLANO17_ONDA5.md) (acréscimos 1 e 2).

## Feito em 12–13/11 (não refazer)
- Ícones do tipo (filme/cubo) na lista de clipes aberta e FECHADA; o Done não solta as faixas nem troca o clip (`ph2d_timeline_persist::recola`).
- Auto Key/Marker/Edit Path com contraste e corpo de campo.
- **Aba dos controlos** (`crates/ph2d-panel-timeline/src/transport_flutuante.rs`): ● ⏮ ‹ ▶ ■ › ⏭ colada ao topo-esquerdo da lista, tinta `KanbanList`,
  cantos de cima redondos, desce `LIST_RADIUS` para dentro da moldura; a caixa dos botões com as quinas de baixo pontudas e o fundo dela
  na linha do topo da lista. O HUD de diagnóstico cede-lhe (`bottom_hud::publica_fila_flutuante`). ● = `TIMELINE_AUTOKEY`, ■ = `TIMELINE_STOP`.
- A Length na 1.ª fileira entre as abas e o Time, com separadores. Fotos em `docs/Timeline/fotos/17_ajustes_1{2,3}_11_*.png`.

## PEDIDO ABERTO (dono 13/11, verbatim)
> «Keys, Containers e Arrange em três abas que se sobrepõem parcialmente na vertical esquerda. As abas inativas aparecem
> parcialmente, inativas e apagadas, com início do nome visível.»

Imagens do dono: [hoje](../fotos/18_pedido_abas_laterais_hoje.png) (as três abas em pílula na fileira do título) ·
[esboço](../fotos/18_pedido_abas_laterais_esboco.png) (três retângulos vermelhos empilhados FORA da borda esquerda do painel, do topo da
lista para baixo: o 1.º alto, ~do cabeçalho até ao Summary; os outros dois menores, por baixo).

Leitura proposta (confirmar com a FOTO antes do smoke, não perguntar o que a imagem já diz):
- As abas saem da fileira do título e passam a ABAS DE PASTA na borda esquerda da lista, por fora, como a aba dos controlos é no topo:
  mesma tinta, coladas à moldura, texto na vertical (rodado 90°, a ler de baixo para cima, como abas laterais de pasta).
- A ATIVA inteira, à frente, na tinta do painel e com o nome inteiro; as INATIVAS por trás dela, parcialmente cobertas (sobreposição
  vertical), mais apagadas, mostrando só o início do nome. Clicar numa inativa troca a aba (o id e o evento de sempre:
  `TIMELINE_TAB_KEYS|CONTAINERS|ARRANGE`, `abas_e_migalha` em `transport_cartoes.rs`; `transport_tabs::items`).
- A fileira do título perde as abas (o «TIMELINE» passa a encostar à Length?) — medir e fotografar.
- Peças: o texto rodado precisa de transformação na cena (ver se `ph2d_editor_core::paint` tem texto rodado; senão Affine no Vello);
  o alvo de clique de cada aba é a parte VISÍVEL dela (a ativa por cima ganha o clique onde cobre).
- Gates a ajustar: `os_cartoes` (duas fileiras, alvos das abas ≥ 36,8/69,9/55,2 × 24 — o alvo não pode encolher), `o_cartao_estreito`
  (o que se pinta fora do cartão: hoje só a aba de cima), `controles_flutuantes`, `bottom_hud` (o HUD cede às abas se cruzarem a faixa dele).
- `PH2D_CARDS=0` não muda.

## Como fotografar sem cliques
`bash docs/Components/ferramentas/fotografa_cena.sh PH2D_ANIMACAO_SMOKE=1 1930 1040 <saida.png> 6` (corre por `bash scripts/ph2d-run.sh`).
Para um estado que pede clique, um gancho TEMPORÁRIO por env (nunca commitado) — ver o histórico desta janela: `PH2D_FOTO_*_TMP`.
