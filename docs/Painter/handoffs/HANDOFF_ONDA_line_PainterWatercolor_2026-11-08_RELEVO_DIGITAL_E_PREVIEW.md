# Handoff de ONDA — relevo do papel no Digital + preview do Smear/Blur (08/11) — a linha CONTINUA

Linha `line/PainterWatercolor`. Nada integrado nem enviado. Commits (sobre `be4ff74ac4`): `79ccef6126` · `e9afb02024` ·
`d5e2ed32ba` · `e479b31c49`. Binário de smoke `target/smoke/ph2d-host-desktop` = `e479b31c49`. Fotos:
[`material_2026-11-08_relevo_digital/`](material_2026-11-08_relevo_digital/).

## Vereditos do dono (verbatim)

- Relevo no Digital: **«Melhorou! Vejo sombras do papel.»**
- Papel NA tinta (Wet / Same as Paper), as duas leis provadas no app: «Ambas me parecem boas» → seguiu a recomendação:
  **fica a RAMPA 0,6→1** de `01fb209238`.
- Smear Basic / Blur Basic: «No cartão de presets havíamos tirado os previews, mas como vc teve bom resultado com os
  outros, creio que conseguirá ter nesses» → feito; **«Smoke OK. Vamos seguir»**.

## O que entrou

- `79ccef6126` (PROJECT_SCHEMA 217 → 218): `ReliefFields::height_at` separa o papel à vista em (tinta do Wet, outra);
  a do Wet tapa o dente pela opacidade (§BUG-papel), a outra só pelo corpo (`CORPO_QUE_ENCHE`). A marca é
  `PainterTool::tintas_molhadas` (pen-down do Wet, DEPOIS do `before`), viaja em `ModelSnapshot` (comparada no
  `confined_to`), `StashedDoc`, instantâneo, duplicar/apagar camada, `doc_is_disposable`, e é o ÚLTIMO campo do
  `PaintedDocument`. Bytes antigos: âmbito `ph2d_tool_painter::LeituraSemTintas` (projeto `<= 217` em
  `project_load_read.rs`; documento da macro `VERSAO_DO_DOCUMENTO` 1 → 2, lê o 1). Sem tipo congelado (precedente
  `WireLayout`). Gates: `no_digital_o_relevo_do_papel_ve_se_na_tinta_opaca` (7,29 × 14,32; antes 0,00),
  `a_marca_da_tinta_do_wet_desfaz_se_com_o_traco`, `um_documento_da_versao_1_le_se_sem_as_tintas_molhadas`,
  `project_tintas_molhadas_tests` (2). Mutações: sem o âmbito o v217 sangra; sem repor a marca o undo sangra.
- `d5e2ed32ba`: `a_transmissao_e_o_relevo_do_papel_mostram_o_grao` estava VERMELHO desde `01fb209238` (a rampa dá 0,84
  a Relief 0,6 → 2,24 × Cycles 1,71). Âncora passa ao aprovado pelo dono (1,84..2,67); o Cycles fica no comentário.
  ⛔ Não «curar» de volta ao oráculo (memória `project_digital_paper_relief_must_show_on_paint_2026_11_08.md`).
- `e479b31c49`: `LinhaDaFerramenta::Preview` sai do NAO_LE do Blur e do Smear; a chave do preview vê o `modo`; o
  preview vivo de quem só mexe tinta usa `trajeto_das_tintas` (enquadramento da miniatura); as faixas pintam-se sem o
  Composite. Gate `o_preview_vivo_do_smear_e_do_blur_mostra_as_tres_tintas` (com/sem Composite, duas mutações
  vermelhas). Censo do preview: `mistura_camada`/`mistura_visivel` (vermelho pré-existente de `0604fd6f5c`) = TRACO.

Verde por nome: 263 de papel/relevo/undo, 115 de papel com a âncora nova, 43 de preview/esbatedores, os 2 alarmes de
pincel, Watercolor Basic ao byte, gates da shell (schema, v194, sfx, sons, tintas). Nada de suíte completa (é do
integrador).

## Conhecido / aberto

- Meio misto numa camada: um traço Digital numa camada já marcada pelo Wet fica com o enterro antigo (a marca é por
  camada). Documentar ao integrador; nenhum pedido do dono.
- O integrador reconta o degrau 218 (`python3 scripts/schema-recount.py`) e o `const { assert!(PROJECT_SCHEMA == 218) }`
  de `project_geometria_v194_tests.rs` (o varint `0xDA`).

## FILA (ordem)

1. **Pastel** — «Pastel está parecendo em Spray ruim. Não se parece pastel.» (smoke das curas, ANTES da revogação da
   #78, que devolveu Pastel/Chalk/Pointillism à miniatura aprovada). ⇒ PRIMEIRO mostrar ao dono o Pastel de HOJE e
   perguntar se ainda é spray; só então oráculo do produto do dono (correr, nunca ler), simular, o dono faz a triagem.
2. Abertos herdados: Wet Wash Mix (grão não aparece + ~4× lento) · Splash (o dono olha) · filme do Óleo Real (só por
   ordem) · `SPRAY_FIXO_MAX` não re-medido com Wet blend · painel antigo a 245 px corta Strength/Size · a caixa da
   dica do Tooth bate na borda direita.
3. Papel de IMAGEM sem dente com Grain próprio; `uma_passagem.rs` lê o grão cru.
4. Memória do 1.º traço (dono 07/11, «depois do CI»): `noite` no main — 08/11 **success**, 07/11 e 06/11 failure.

## Avisos

- Fotos no app: `bash scripts/ph2d-run.sh bash <script>` (o guarda recusa o binário por fora da porta), Xvfb próprio,
  `XDG_CONFIG_HOME`/`XDG_DATA_HOME` vazios, `import -window root`, NUNCA `spectacle`; recompile o smoke ANTES da foto.
  A janela abre 1024×768: Painter em (868,33); trilho: Smear (300,253), Blur (300,281).
- `foto_das_ferramentas` (app-painter) NÃO mostra o preview real — não serve de prova.
- Fecho da LINHA (não agora): clippy/fmt das crates tocadas 1×, `rm -rf target/*/incremental`, handoff de integração.
