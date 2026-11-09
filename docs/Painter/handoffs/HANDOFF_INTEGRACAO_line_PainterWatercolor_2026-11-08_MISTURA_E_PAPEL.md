# Handoff de integração — line/PainterWatercolor, mistura Wet blend + papel na tinta (08/11)

Leitor: o agente integrador. Detalhe de cada onda nos handoffs de onda linkados; aqui só o que evita colisão e
regressão. Nada integrado nem enviado; **o dono pediu só a preparação** (08/11).

## 1. Identidade

- Branch `line/PainterWatercolor`, HEAD `o commit deste handoff (código em `10435a4196`)`, merge-base com o main `652b506b58`, 41 (40 + o deste handoff) commits.
- Ondas desde o registo anterior ([tinta viva, 07/11](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-07_TINTA_VIVA.md)):
  [undo/refazer + lote 07/11](HANDOFF_CONTINUACAO_2026-11-07_undo_redo_e_lote.md) ·
  [curas da mistura](HANDOFF_ONDA_line_PainterWatercolor_2026-11-08_CURAS_DA_MISTURA.md) ·
  [fila pós-smoke](HANDOFF_ONDA_line_PainterWatercolor_2026-11-08_FILA_POS_SMOKE.md) ·
  [Relief × Tooth + busca](HANDOFF_ONDA_line_PainterWatercolor_2026-11-08_RELIEF_TOOTH_BUSCA.md) ·
  [relevo no Digital + preview Smear/Blur](HANDOFF_ONDA_line_PainterWatercolor_2026-11-08_RELEVO_DIGITAL_E_PREVIEW.md).

## 2. Superfície de colisão (`bash scripts/collision-surface.sh`, 08/11, depois do fecho)

```

SUPERFÍCIE DE COLISÃO — line/PainterWatercolor contra main
  merge-base 652b506b58   ·   40 commit(s)   ·   435 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        218   (base: 217)
  ⚠   └ tripla do gate               (218, 13, 29)   (base: (217, 13, 29))
    VEC_SCENE_SCHEMA                       29   (base: 29)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 25   (base: 25)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              120   (base: 120)
  ✗ ph2d-render (espelho = ecs + K)       —   (base: —)
  ✗ ph2d-script (espelho = ecs + K)       —   (base: —)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0198   próximo livre: 0199
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
    nenhum arquivo da linha passa do teto
───────────────────────────────────────────────────────────────────────────────
  ✗ SONDA CEGA em: ph2d-render (espelho = ecs + K), ph2d-script (espelho = ecs + K) — a const mudou de ficheiro e nenhum candidato a tem.
    Corrija o caminho NESTE script: um «—» dos dois lados lia-se «ninguém mexeu» (13/09).
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
```

⚠️ A tabela mede contra o main de 08/11. O integrador lê o main **no ficheiro** e conta o degrau como DELTA.

- **`PROJECT_SCHEMA` 217 → 218** (`79ccef6126`): `PaintedDocument` ganha `tintas_molhadas` como ÚLTIMO campo; bytes
  `<= 217` lêem-se sob o âmbito `ph2d_tool_painter::LeituraSemTintas` (`shells/desktop/src/project_load_read.rs`).
  Tripla do gate `(218, 13, 29)`. Recontar com `python3 scripts/schema-recount.py`; o
  `const { assert!(PROJECT_SCHEMA == 218) }` de `project_geometria_v194_tests.rs` (varint `0xDA`) segue o degrau.
- Documento da macro: `VERSAO_DO_DOCUMENTO` 1 → 2 (lê o 1).
- Contratos congelados §6: intocados (`node.rs`, `tool.rs`). Nenhum ADR. Nenhum pacote externo novo no `Cargo.lock`.
- O «✗ SONDA CEGA» do registo de componentes (`ph2d-render`/`ph2d-script`) é do script, não desta linha (a linha não
  toca componentes; `ph2d-ecs` 120 = base).

## 3. Foundational / partilhado tocado — e porque é aditivo

| Onde | O quê | Aditivo? |
|---|---|---|
| `ph2d-editor-core/src/widget/text_input/` | corte de LOC: `TextInputState`/`TextInput` para `model.rs`, re-exportados | sim, move puro |
| `ph2d-editor-core/src/screens/hero/topbar/mod.rs` | `tooltip_rect` afasta a dica `gap` px das bordas da janela | muda a aritmética de um widget partilhado: toda dica encostada a uma borda anda `gap` (Spacing::Sm) para dentro; os 4 testes da colocação verdes |
| `ph2d-i18n` | chaves novas (Wet blend, Sample, Fixed dab size, Dab size (px), Tooth/Relief tips, dicas de inércia); sai `inerte.relief_and_no_grain` (sem outro leitor) | sim |
| `ph2d-wet-paint` | `brush/unidade.rs` (Size em px), `tuning/defs.rs`, `grid/snapshot.rs` | sim |
| `ph2d-panel-wet-tuning/src/rows.rs` | fileiras do Size px / Bristle Combing no Shape | sim |
| `ph2d-app-painter/src/composite_smoke.rs` | `PH2D_COMPOSITE_PRESET=<nome>` arma um pincel de fábrica na cena do smoke | sim |
| `shells/desktop/src/project_*` | o degrau 218 e o âmbito de leitura v217 | degrau de schema (acima) |
| `shells/desktop/src/input_dispatch/keyboard_cadeia.rs`, `keyboard_escapes.rs` | campo de texto focado ⇒ as teclas do Painter (`[`,`]`,`E`, Delete, clipboard da seleção, Enter/Esc da figura) não são consumidas | sim; a shell da linha fecha 125 linhas abaixo de onde estava (comentários do Painter encurtados), ver §5 |

## 4. O que entrou (factos; o porquê está nas ondas)

- **Undo/refazer** (07/11): pânico do undo vivo com `flow_ratio > 1` (`c887156cbb`); o refazer separava o relevo da
  tinta (`3fc47dc81d`).
- **Color mixing «Wet blend»** à moda do Clip Studio no Digital (`f77c01494e`) e **22 pincéis CSP** revistos contra o
  CSP corrido em 3 fundos (`e50fe51395`, `b5b902db6f`): **100 pincéis de fábrica** (o §5 dizia 78 — atualizado).
- Fixed dab size + Dab size (px) no Jitter; Size em px em todo meio; Grain «Same as Paper» sem Paper = sem grão;
  Pointillism Hardness 0,5; curas #73–#80 da mistura; Sample «This layer · All visible layers».
- Fila pós-smoke: Composite em px, o Paper morde por cima do Grain, **#78 revogado** (Pastel/Chalk/Pointillism voltam à
  miniatura aprovada), busca no dropdown de presets, prévia dos esbatedores.
- Paper: **Relief × Tooth** (o Tooth é o multiplicador do efeito do papel); busca numa faixa fixa.
- **Relevo do papel visível na tinta Digital** (schema 218) e a âncora `a_transmissao_e_o_relevo_do_papel_mostram_o_grao`
  no 2,24 aprovado pelo dono (⛔ não «curar» de volta ao oráculo Cycles).
- Preview do Smear/Blur de volta (as três tintas que eles mexem).
- **Fecho (`37bf6b2afa`)**: 9 ficheiros + 1 fn acima dos tectos de LOC, todos crescidos nesta linha, curados por MOVER
  (tabela na mensagem do commit); e as 4 curas da auditoria (abaixo).

### Auditoria de fecho (2 lentes, sobre `fb9e141f68..e479b31c49`)

| Achado | Cura | Portão (mutação) |
|---|---|---|
| a 1.ª marca do Wet numa camada mudava a lei da camada inteira, mas a luz só se refazia no retângulo do traço | `invalidate_composite` quando a marca é nova | `a_primeira_marca_do_wet_refaz_a_imagem_inteira` (2 mutações RED) |
| o Tooth do Digital esmaecia com um Grain escolhido, e ele morde por cima do Grain | regra `papel_pode_morder`; dica `paper_and_relief` | `o_tooth_do_digital_age_com_um_grain_escolhido` (RED) |
| no cartão, a explicação do Relief/Tooth tapava a dica de inércia | `poe_dicas` só põe a explicação sem inércia | `o_tooth_esmaecido_mostra_o_que_o_liga_nos_dois_paineis` (RED) |
| escrever na busca do Preset disparava atalhos do Painter («e» ligava a Borracha) | guarda do campo focado na cadeia e no Enter/Esc da figura | `a_busca_do_preset_fica_com_as_teclas_do_painter` (2 mutações RED) |
| a dica do Tooth encostava na borda direita | `tooltip_rect` com margem `gap` | `a_target_at_the_right_edge_keeps_the_tooltip_inside` (RED) |

Mutação: **7 de 7 sangram**. Limpo pela auditoria: undo/stash/instantâneo/duplicar/apagar camada com a marca, leitura
v217 e macro v1, a env `PH2D_PAPEL_NA_TINTA` toda removida, a chave do preview, px ↔ multiplicador do Composite, HR-15,
hit rect da faixa da busca.

## 5. Prova de fecho (08/11)

Árvore `10435a4196` (código; o commit seguinte só toca docs). Nada de suíte completa nem `nextest-impacted`: são do
integrador (dono 23/10).

| Portão | Resultado |
|---|---|
| `cargo fmt --check` das 9 crates tocadas | ✓ |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings` | ✓ (antes: 2 `clone_on_copy` + 2 `cloned_ref_to_slice_refs`, curados) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ |
| `cargo machete` | ✓ nenhuma dependência não usada |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | ✓ · ✓ (antes: `ph2d-app-painter` sozinha não compilava — curado) |
| `ph2d-editor-core`: `loc_cap` 7/7 · `the_shell_only_shrinks` 2/2 · `safe_clamp` 2/2 · `claude_md` · `memory` | ✓ |
| shell: `file_loc_caps` · `painter` 20/20 · `schema` · `v194` · `tintas_molhadas` 2/2 | ✓ |
| alarmes de pincel: `a_largura_de_fabrica_e_a_aprovada` · `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` · `o_watercolor_basic_nao_mudou_com_o_pigment_mix` | ✓ nenhum pincel mudou |
| por nome em `ph2d-tool-painter`: relevo 43 · papel 75 · tintas 8 · preview 43 · undo 152 · tooth 13 · marca 42 | ✓ |
| por nome em `ph2d-panel-painter-layers`: busca 8 · size_px_e_dab_fixo 3 · h6_os_cartoes 19 · censo_dos_controlos 5 · cards 13 · seam_same_as_paper 6 | ✓ |
| `ph2d-painter-brush`: accumulate 7 · every_word 3 | ✓ |
| `censos-da-arvore-combinada.sh` | ✓ 133/133, 12 de 12 censos |
| mutação das curas da auditoria | 7/7 RED |

- **Shell:** 188 997 linhas contra o tecto 189 041 (a linha subia-a a 189 166; curado encurtando comentários
  históricos do Painter na shell, `painter_canvas_input.rs` −120 e `keyboard_painter.rs` −49, só comentário). Folga
  de **44**: outra linha que cresça a shell soma com esta — o integrador reconta.
- Binário de smoke: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`
  — 2.ª saída `Finished \`smoke\` profile [optimized] target(s) in 0.34s`, zero `Compiling`. `target/*/incremental`
  apagado (59 G + 12 G) DEPOIS do build.
- `agent-loop-profile` (20 sessões): paralelismo 1,17/passo ✗ (alvo 1,5) · test:check 2,5× ✗ (alvo 1) · contexto
  228 mil/passo ✓ · edições sem conferir 2,0 % ✓.

## 6. ABERTO, por ordem (para a retomada pós-integração)

1. **Pastel parece spray** — dono 08/11, vendo a [foto de hoje](material_2026-11-08_pastel/pastel_hoje.png):
   «parece ruim. Mas vamos deixar para depois da integração.» Perguntar o produto de referência dele para pastel
   (instalados: Krita, MyPaint; antes comparou com o Clip Studio); oráculo CORRIDO em 3 fundos; simular antes de
   construir; o dono faz a triagem. Memória `project_pastel_after_integration_2026_11_08.md`.
2. **Marca do Wet é por CAMADA** (auditoria F1): Digital numa camada já marcada fica com o enterro antigo; apagar toda
   a tinta Wet não tira a marca (só o undo até antes do 1.º traço Wet, ou apagar a camada). Nenhum pedido do dono;
   cura possível = fonte por píxel/por traço. A Borracha do Wet provavelmente também marca (não confirmado).
3. Abertos herdados: Wet Wash Mix (grão não aparece + ~4× lento) · Splash (o dono olha) · filme do Óleo Real (só por
   ordem) · `SPRAY_FIXO_MAX` não re-medido com Wet blend · painel antigo a 245 px corta Strength/Size · dicas longas
   numa linha só: transbordam o painel para a esquerda (sem quebra; a margem da borda foi curada).
4. Papel de IMAGEM sem dente com Grain próprio; `uma_passagem.rs` lê o grão cru.
5. **Memória do 1.º traço** (dono 07/11, «depois do CI»): `TECTO_DA_MEMORIA_MIB` ainda `[(300,515),(1000,2175)]`
   (`tests/it/presets_de_fabrica/memoria.rs`); `noite` do main 08/11 success. Entra na rodada depois desta integração.
6. Teclado: a mesma exposição a atalhos existia antes para outros campos de texto (renomear camada) — a guarda nova
   cobre-os também na cadeia do Painter; outras cadeias não foram varridas.

## 7. Smoke do dono (só depois da integração, quando o Enio mandar)

Comando (antes da integração, o binário da worktree; depois, o do main):
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop`

**A. A busca de pincéis não dispara atalhos**
1. Abra o **Painter** (canto superior direito). No cartão **Brush**, clique na lista **Preset**.
2. No campo de busca do topo, escreva `wet`.
3. Certo: o campo mostra «wet» e a lista filtra; a borracha NÃO liga. Errado: aparece «wt» ou o pincel vira borracha.

**B. O Tooth acende com Grain no Digital**
1. **Paint Mode: Digital**. No cartão **Paper**, escolha um papel e ponha **Relief** acima de 0.
2. Escolha um **Grain** qualquer. Certo: o **Tooth** continua aceso e mexê-lo muda o traço. Errado: Tooth cinzento.
3. Ponha Relief em 0 e passe o rato no Tooth. Certo: a dica diz «Acts when a Paper is chosen and Relief is above 0».

**C. Relevo do papel no Digital** (aprovado 08/11 — só conferir que continua): traço opaco sobre papel com Relief:
as sombras do papel vêem-se na tinta.
