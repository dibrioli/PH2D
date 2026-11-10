# Integração — `line/TimelinePrincipal`, ondas 1 e 2 (a barra e a fita no molde do Rive, versão B)

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · merge-base `f254e42bf1` · 18 commits.
- Plano e decisões do dono: [`15_plano_redesenho_molde_rive.md`](../15_plano_redesenho_molde_rive.md) (versão B, «Keys | Curves»
  FORA, só aparência e organização, os mesmos ids e eventos). Narrativa por onda:
  [onda 0](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-09_ONDA0.md) · [onda 1, 1b, 1c](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-09_ONDA1.md) ·
  [onda 2, 2b–2i](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-09_ONDA2.md).
- **O dono aprovou o smoke visual** das ondas 1 e 2. Sem push, sem rebase no `main`, sem integração por esta linha.

## O que a linha entrega (só cartões; `PH2D_CARDS=0` com a mesma saída)

1. **Barra** (`transport_cartoes.rs` reescrito, `barra_clips.rs`, `barra_menus.rs`): duas fileiras. Fileira 1: abas · migalha ·
   tocar · Time · Frame · «24 fps · project» … dono · engrenagem · ×. Fileira 2: Auto Key · Record while playing · Marker | «Clips» ·
   chips-pílula (um por opção de `source_options`, «+N more» abre a lista de sempre) · «⋯» do clipe · «+». Janelinhas
   **Timeline options** (Stop·Loop·Ping-Pong, Length, Snap, física, ghosts…) e **menu do clipe** (Rename, Duplicate, Reverse keys,
   Starts with this clip, Delete), fechadas pelo clique de fora. Abas com a pílula ativa em destaque; Auto Key/Record/Marker discretos.
2. **Fita**: árvore objeto → propriedades (cabeçalho do objeto em negrito com fundo, ícone do tipo e pontos por chave; a faixa diz só
   a propriedade com unidade `(px)`/`(°)`); barra entre chaves pela interpolação (curva ciano · reta violeta · segura tracejada);
   chaves com contorno; Summary forma C; legenda Curve · Linear · Hold; zona depois do fim listrada; moldura da fita, vermelha com o
   Auto Key; abrir e trocar de clipe enquadram o clipe; não persegue o cursor depois do fim do clipe; texto centrado nas maiúsculas.
3. **Cores** da maquete B1: `ph2d-tokens/src/timeline_tinta.rs` (`TimelineTinta` = base da paleta + desvio OKLab amostrado; ao bit
   no tema de referência Forge e nos escuros, degrau invertido nos claros).

## Commits

```
b25a132a34 fix(timeline): fecho da linha — tetos de LOC, porta do tema, censo de apelidos, 12 temas, janelinhas no painel escondido
a72d76c11e onda 2i · d8c89d414b onda 2h · 3b108e1de6 onda 2g · ea4d69c5ec onda 2f · 2a7d0dada3 onda 2e · e687c09c12 onda 2d
774dbb6d49 onda 2c · fd011b5613 onda 2b · 1bbf593f53 docs onda 2 · 8694c376c4 onda 2 · a5d54be996 onda 1c · 10512766bc onda 1b
73691b882a docs onda 1 · 5eb2081ee4 onda 1 · 1275bb190f docs onda 0 · 27dd2235a9 docs plano
```
(+ o commit deste handoff.)

## Foundational: o que mudou e por que é aditivo

- **`ph2d-editor-core/src/panel/cards_banda.rs`** — só inserções (0 linhas removidas no diff das ondas): `Sinal`, `Emoldurada`,
  `emoldurada_w/_em`, `rotulo_com_sinal{,_w}`, `botao_com_icone{_w,_em}`, `separador_em` + `SEPARADOR_W`. No fecho, as peças da
  onda 2/2i (`LISTRA_PASSO`, `listras_em`, `TonsDaBarra`, `abas_em`, `emoldurada_discreta_em`, `botao_com_icone_discreto_em`)
  mudaram de casa para o irmão **`cards_banda_tons.rs`** (novo; `pub use tons::*` em `banda`, o caminho público é o mesmo) — o
  ficheiro tinha 749/700. Nenhuma função existente mudou de assinatura ou comportamento.
- **`ph2d-tokens`** — 6 `ColorToken` novos, todos APELIDOS (sem valor no `tokens.json`): `TimelineInterpCurve→Info`,
  `TimelineInterpLinear→CodeKeyword`, `TimelineInterpHold→Text3`, `TimelineRecord→Danger`, `TimelineObjectRow→Bg3`,
  `TimelineSummaryRow→Accent` (`color.rs`, `color_alias.rs`, `skin_body.rs`). A única linha mudada em `color_alias.rs` acrescenta
  `TimelineRecord` ao braço de `TimelineMissing`. Nenhum token partilhado mudou de valor ou de pai. Métricas novas em `chrome.rs`
  (`TIMELINE_KEY_HALF_PX` 6,5 · `_RING_PX` 1,5 · `_HIT_HALF_PX` 8 · `TIMELINE_OBJECT_DOT_HALF_PX` 4 · `TIMELINE_FRAME_PX` 1,5 ·
  `TIMELINE_SUMMARY_*_ALPHA`). Módulo novo `timeline_tinta.rs` (`TimelineTinta`), re-exportado por `lib.rs`. **`color.rs` está em
  700/700** — token novo noutra linha que caia em `color.rs` estoura o tecto na árvore combinada.
- **`ph2d-i18n/src/timeline.rs`** — chaves `panel.timeline.card.*`/`tip.*` novas; `card.length` apagada (sem referências). O
  `lib.rs` (700/700) não foi tocado.

## Contratos e schemas — NÃO tocados (prova)

`bash scripts/collision-surface.sh`: `PROJECT_SCHEMA` 225 (base 225), tripla (225, 13, 29) igual, `VEC_SCENE_SCHEMA` 29,
`FLIP_SCHEMA` 13, **`DOC_VERSION` (TimelineDoc) 25 (base 25)**; §6 `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs`
**intocados** (`PanelEvent=4`: o diff só EMITE `PanelEvent::Toggle` existente, em `barra_menus.rs:120`); sem ADR; Cargo.lock sem
pacote externo novo; `git diff f254e42bf1..HEAD -- crates/ph2d-timeline shells` vazio. Estado de painel +3 campos
(`opcoes_abertas`, `menu_do_clip`, `enquadrado`), não gravados. Ids novos: `TIMELINE_OPTIONS`, `TIMELINE_CLIP_MENU`,
`TIMELINE_END_STOP/_LOOP/_PINGPONG`, `TIMELINE_LENGTH_NUM`, registados em `populate.rs`.

## Números medidos

- Altura da folha a 1930 px: 116 px antes → **126** (gate `a_folha_nao_perde_linhas_para_o_cromo`, estrito `depois ≥ antes`);
  1600 px 91 → 92. Linha 22 px (`ROW_H_PX`, a maquete mede 22).
- Alvos da barra (gate `os_alvos_da_barra_nao_encolheram_com_a_cara_nova`, medidos antes): abas ≥ 36,8/69,9/55,2 × 24 px,
  Auto Key 108×28, Record 176×28, Marker 75×28. Chave: desenho 6,5 + 1,5 de contorno, alvo 8 (meia largura).
- Cores (OKLab): três barras ≥ 0,10, pior **0,107**; Summary fundo ≥ 0,044 (**0,045**), texto/losango ≥ 0,24 (**0,249**);
  cabeçalho × propriedade ≥ 0,014 (a maquete 0,015); moldura com Auto Key muda 0,378. Distância maquete × app 0,339 → 0 (valor),
  0,219 → 0 (fita do cabeçalho). **No fecho estes gates passaram a varrer `Theme::ALL` (12 temas)**, eram 8 à mão — verdes nos 12.

## Gate do fecho (1× sobre o diff; suíte completa e impactados ficam para a integração)

- `cargo fmt --all --check` ✓ · `PH2D_SUITE=fecho` clippy `-D warnings --all-targets --all-features` em `ph2d-panel-timeline`,
  `ph2d-editor-core`, `ph2d-tokens`, `ph2d-i18n` ✓.
- Por nome, `ph2d-editor-core`: `workspace_file_loc_cap` 2 ✓ · `architecture_panel_loc_cap` 4 ✓ · `arch_safe_clamp_only` 2 ✓ ·
  `no_magic_numeric` 3 ✓ · `no_literal_color` 3 ✓ · `hr15_no_hardcoded` 1 ✓ · `architecture_no_orphan_source_file` 4 ✓ ·
  `the_corner_of_a_control_comes_from_the_theme` 3 ✓ · `every_frame_goes_through_the_theme_door` 3 ✓ ·
  `every_field_painter_asks_the_theme` 3 ✓ · `banda` 3 ✓. Shell: `file_loc_caps` 4 ✓.
- Crates inteiras com `motivo:`: `ph2d-tokens` 193 ✓ (módulo novo + 6 tokens no `ColorToken::ALL`); `ph2d-i18n` 30 ✓ (censo de
  chaves órfãs/em falta).
- `ph2d-panel-timeline` por nome: `os_cartoes` 23 · `_seam` 81 · `linhas` 21 · `summary` 13 · `a_fita_abre` 2 · `tracks` 10 ·
  `geom` 12 · `box_select` 7 · `scrollbar` 6 · `view::` 20 · `duration_chip_gesture` 15 · `every_word` 3 · `one_door` 2 ·
  `barra_menus` 2 · `breadcrumb` 10 · `o_fps` 1 · `interact` 12 · `key_drag` 11 — todos ✓.
- **Vermelhos achados no fecho e curados** (`b25a132a34`): `cards_banda.rs` 749/700 · `linhas.rs` 672/600 (pintura da maquete →
  `linhas_maquete.rs`) · `view.rs` 610/600 (testes → `view_tests.rs`) · `paint()` 217/200 (três passos → `paint_passos.rs`) · quina
  à mão em `legenda.rs` (→ `paint::frame_radius`) · dois contornos crus (chip escolhido, moldura da fita) declarados `FRAME-RAW-OK`:
  o contorno É a mensagem (seleção; Auto Key a gravar) e a porta do tema apagá-lo-ia nas peles planas · o censo
  `an_alias_has_no_value_it_has_a_parent` (+6 apelidos com o pai medido; família `timeline-` 16 → 22; varre `Theme::ALL`).

## Auditoria (duas lentes independentes + controlo morto)

- **(a) Costura de UI.** REAL: esconder o painel não fechava as janelinhas (`release_everything_a_hidden_panel_must_not_hold`) —
  curado, asserção em `as_janelinhas_abrem_e_fecham_pelo_clique_de_fora`, mutação sangra. Conferidos OK: alvo da chave = desenho +
  contorno; chip regista até antes do «⋯», que tem alvo próprio; polegar fino com o trilho inteiro como alvo; clicar na engrenagem
  ou no «⋯» abertos fecha sem reabrir (`dismiss_own_list` exclui a âncora). `PH2D_CARDS=0`: toda mudança de pintura está atrás de
  `cards_on()`; fora dela só código inerte no clássico (registo dos ids novos, `fecha_pelo_clique_de_fora`, `hier::publica_icone`).
  Gates emendados com motivo (onda 1): piso de `todo_icone_mudo_tem_dica` 11 → 8 (ícones viraram linhas com texto); o
  `aberto(CLIP_DD)` saiu porque os chips substituem a lista quando cabem.
- **(b) HR-15 e temas.** Sem hex, sem string de UI fora do i18n; literais marcados `LITERAL-PX-OK`/conta; os `[u8;3]` de
  `timeline_tinta.rs` são a amostra medida da maquete e vivem no dono da paleta. REAL: os gates de cor da fita cobriam 8 de 12
  temas (PlumberRed, PlumberGreen, Sunset, Candy fora) — curado, verdes nos 12.
- **(c) Controlo morto.** `TIMELINE_END_STOP/_LOOP/_PINGPONG` → `barra_menus::apply_event` empurra `PanelEvent::Toggle` de
  `TIMELINE_LOOP`/`_PINGPONG` (o mesmo que a caixa clássica pedia) → a shell traduz (`timeline_bridge::intent_for_transport`) → o
  relógio decide (gate `stop_pede_o_que_desmarcar_a_caixa_pedia`). Menu do clipe: cada linha usa o id clássico (`TIMELINE_DUP_CLIP`
  … `TIMELINE_CLIP_AUTOPLAY`) e o despacho de sempre (gate pede `DuplicateClip`). Pintado sem alvo, conhecido: o ▾ do cabeçalho do
  objeto e os pontos do cabeçalho.

## Superfície de colisão com o `main`

`ph2d-panel-timeline` quase inteira (barra, fita, testes de costura que agora abrem a janelinha via `onde_mora`); `ph2d-tokens`
`color.rs` (**700/700**), `color_alias.rs`, `skin_body.rs`, `chrome.rs`, `lib.rs`; `ph2d-i18n/src/timeline.rs`;
`ph2d-editor-core/src/panel/cards_banda.rs` + `cards_banda_tons.rs`. Outra linha que acrescente `ColorToken` conflita em `color.rs`/
`skin_body.rs` (ordem do `ALL`) e no censo `an_alias_has_no_value_it_has_a_parent` (contagem por família) — **reconte**. ⚠️ O
`collision-surface.sh` dá «SONDA CEGA» em `ph2d-render`/`ph2d-script` (espelho do registo de componentes): defeito do script, não
desta linha (os dois lados «—»).

## Smoke (aprovado pelo dono; para reconferir depois da integração)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop`
   (no primário depois da integração: `cd /home/enio/Documentos/Projetos/PH2D && PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop`).
2. Timeline em baixo: em cima «Keys · Containers · Arrange», os botões de tocar, Time, Frame e «24 fps · project»; à direita
   «Animation: Herói» e a engrenagem. Embaixo Auto Key, Record while playing, Marker e «Clips» com «andar ⋯», «pular» e «+».
3. Coluna da esquerda: «Herói» em negrito, por baixo «Translate X (px)» e «Translate Y (px)», depois «Espada» e «Rotation (°)».
   Entre as chaves, barras violetas; no canto a legenda Curve · Linear · Hold; depois do fim do clipe a fita listrada.
4. Clicar «pular» → o clipe fica azul e a vista enquadra-o; «⋯» → menu do clipe; engrenagem → Timeline options; clicar fora → fecha.
   Ligar Auto Key → a moldura da fita fica vermelha.
5. Errado se: um controlo de antes sumiu ou não responde, uma janelinha não fecha ao clicar fora, um nome sai cortado ou duplicado,
   clicar numa chave não a seleciona. `PH2D_CARDS=0` deve mostrar a Timeline de antes.

## Fica aberto

- **Onda 3 do plano**: Containers e Arrange (chips «used 2×»/«not placed», lanes com peso e olho, o mesmo desenho de faixas).
- **«Translate X» × «Position X»**: a palavra da Timeline ou a do Inspector nos canais embutidos — decisão do dono, pendente.
- O **▾ do cabeçalho do objeto** é só pintura (não há id/evento de recolher objecto).
- A **legenda**: o smoke só tem chaves lineares; curva e segura só aparecem na legenda (falta cena com as três).
- **Ícones e peso do texto** face à maquete (ícone por propriedade aproximado, peso do negrito) por afinar com o dono.
- Os nomes `…_nos_oito_temas` dos gates de cor ficaram (agora varrem 12); renomear é cosmético.
- Impactados e suíte completa ficam para a integração.

## Para o CLAUDE.md §5 (feito nesta linha, por ordem do coordenador)

Entrada **Timeline**: «barra e fita no molde do Rive, em cartões» + «Último:» → este handoff.
- ⚠️ §5 do CLAUDE.md fica em 13 990/14 000 bytes: a entrada da Timeline caiu de 365 para 356 (saíram a frase do blob do `TimelineDoc` e os links BUGS/handoffs/história, alcançáveis pelos docs). Reconte na árvore combinada.

## Perfil do agente

`bash scripts/agent-loop-profile.sh` (fecho, 09/11):

```
PERFIL DO LOOP DO AGENTE — 20 sessao(oes) mais recentes
──────────────────────────────────────────────────────────────────────────────
  ✗ paralelismo de ferramenta              1.09/passo   alvo: >= 1,5  (7% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                134   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                304 : 126   alvo: <= 1,0  razao 2.4x (baseline: 4,3x)
  ✓ edicoes SEM conferir que casaram             0.8%   alvo: <= 2%  (4 de 478; o guarda R6 recusa-as desde 14/10)
  ✓ contexto relido por passo (media)         224 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               67 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
──────────────────────────────────────────────────────────────────────────────
  As leis moram no CLAUDE.md §2 (sempre carregado); a DIRETIVA_IMPLEMENTACAO aponta pra la'.
  ⚠️ Rode com poucas sessoes para ver o HABITO recente; 'all' e' o baseline historico.
```
