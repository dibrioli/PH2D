# Continuação — `line/TimelinePrincipal`, onda 1 (a barra) → onda 2 (a fita)

- Worktree: `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · commit da onda `5eb2081ee4` sobre `1275bb190f`.
- Plano e decisões do dono: [`15_plano_redesenho_molde_rive.md`](../15_plano_redesenho_molde_rive.md) (versão **B**; «Keys | Curves» FORA;
  sem função nova). Onda anterior: [`HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-09_ONDA0.md`](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-09_ONDA0.md).
- Fotos: [`15_onda1_depois_animacao_smoke_1.png`](../fotos/15_onda1_depois_animacao_smoke_1.png) (app, `PH2D_ANIMACAO_SMOKE=1`) ·
  [`15_onda1_lado_a_lado_maquete_B1.png`](../fotos/15_onda1_lado_a_lado_maquete_B1.png) (em cima a maquete B1, em baixo o app) ·
  antes: [`15_antes_animacao_smoke_1.png`](../fotos/15_antes_animacao_smoke_1.png).

## O que a onda 1 fez (só a barra em cartões; `PH2D_CARDS=0` intacto)

`crates/ph2d-panel-timeline/src/transport_cartoes.rs` reescrito: tipo `Fluxo` (poe / a_direita / quebra / resta) e três fileiras.
Devolve `Barra { depois, clip, anim, opcoes, menu_do_clip }`.

1. **Tira do cabeçalho**: abas · migalha · à direita o dono (`TIMELINE_ANIM_DD`, chip via `chip_text`) + engrenagem `TIMELINE_OPTIONS`.
   O × é da moldura.
2. **Clipes** (`src/barra_clips.rs`): rótulo «Clips» (aba Keys) / «Clips & Containers» (outras); um chip-pílula por opção de
   `transport_clips::source_options` (mesmos ids, mesmo clique da lista); o escolhido com AccentSoft + contorno Accent; ▶
   (`IconId::Play`) no clipe da entrada (Autoplay); «⋯» `TIMELINE_CLIP_MENU` encostado ao chip escolhido (ausente se é container);
   «+» `ADD_CLIP`. **UMA fileira só**: o que não cabe vai para o chip «+N more» com id `TIMELINE_CLIP_DD`, que abre a lista de
   sempre (rolada). Os chips registam `Button` no store por `register_if_absent` — as linhas `timeline_source_opt` da Cena só
   entravam no store quando a lista se pintava (pintado-mas-morto, achado e curado nesta onda).
3. **Tocar**: 5 ícones · Time (s) · Frame · «{fps} fps · project» (só leitura) · Auto Key · Record while playing · «+ Marker».

`src/barra_menus.rs`: as duas janelinhas (`camada_das_listas::por_cima` + `alvos_desde`, fundo `RAIL_BACKDROP`, medida em passe seco);
fecham pelo clique de fora (`publish_own_list` / `take_own_list_dismissed`, a lei do «+ Track»).

- **Timeline options**: When the clip ends Stop·Loop·Ping-Pong (ids novos `TIMELINE_END_STOP/_LOOP/_PINGPONG`, que pedem o
  `PanelEvent::Toggle` de `TIMELINE_LOOP`/`_PINGPONG` que a caixa pedia; a célula já escolhida não pede nada — re-armar o Loop
  repunha a faixa), Length (s) (`TIMELINE_LENGTH_NUM`; ∞ sem duração autorada), Snap to frames, Run physics during Play,
  A position key is a point on the path, Show speed in the curves, Ghosts, Only at keys, Ghost settings…
- **Menu do clipe**: Rename, Duplicate, Reverse keys, Starts with this clip (Autoplay), Delete (`ButtonOff` com 1 clipe).
  Clique numa linha fecha o menu; Ghost settings fecha as opções.

Estado novo: `TimelinePanelState.opcoes_abertas`, `.menu_do_clip`. Ids novos em `src/ids/timeline_barra.rs`, registados em
`populate.rs`. Despacho: `barra_menus::apply_event` logo depois do `stack_event` em `event.rs`. `paint.rs`: tipo `Barra`,
`fecha_pelo_clique_de_fora` antes da barra, `barra_menus::paint` depois dos overlays.
i18n em `ph2d-i18n/src/timeline.rs` (NÃO no `lib.rs`, que está em 700/700): chaves `panel.timeline.card.*` novas e
`tip.options/clip_menu/starts_with`; a órfã `card.length` («Dur (s)») foi apagada.

## Superfície de colisão (o que um merge pode partir)

- `crates/ph2d-panel-timeline`: `transport_cartoes.rs` (reescrito), `barra_clips.rs` e `barra_menus.rs` (novos), `event.rs`,
  `ids.rs`, `ids/timeline_barra.rs`, `lib.rs`, `paint.rs`, `populate.rs`, `state.rs`.
- `crates/ph2d-i18n/src/timeline.rs`. Outra linha que mexa em chaves `panel.timeline.card.*` ou na barra de cartões conflita aqui.
- Testes: `os_cartoes.rs` + seams `add_clip_full`, `animation_chip`, `arrange_sources`, `autoplay`, `transport_motion_path`,
  `transport_onion`, `transport_physics`, `view_tabs`. Quem os edite noutra linha deve saber que o controlo agora mora
  numa janelinha (helper `onde_mora` em `os_cartoes.rs`).
- Contadores (DELTA): ids de porta novos 2 (`TIMELINE_OPTIONS`, `TIMELINE_CLIP_MENU`) + 3 células (`TIMELINE_END_*`) + `TIMELINE_LENGTH_NUM`;
  campos de estado +2; sem mudança de `PROJECT_SCHEMA`, de contrato §6 nem de formato gravado.
- Dois números de gate mexidos (ver abaixo): a altura da folha e o piso de vácuo de `todo_icone_mudo_tem_dica`.

## Gates emendados (confirmar com o dono)

- `a_folha_nao_perde_linhas_para_o_cromo` (`tests/it/os_cartoes.rs`): a folha paga no máximo **UMA** fileira de barra
  (medido a 1930 px: 116 px antes, 92 px em cartões), porque a versão B do dono troca duas fileiras por três. **O dono confirma
  no smoke** que a folha das trilhas continua alta o bastante.
- `todo_icone_mudo_tem_dica`: piso de vácuo 11 → 8 (ícones viraram linhas com texto).

## Prova de fecho (por nome; suíte completa só na integração)

- `os_cartoes`: 20 testes verdes. Novos: `a_barra_tem_tres_fileiras_no_molde_do_rive`, `stop_pede_o_que_desmarcar_a_caixa_pedia`,
  `as_janelinhas_abrem_e_fecham_pelo_clique_de_fora`, `com_muitos_clipes_os_chips_ficam_numa_fileira_e_o_resto_vai_para_a_lista`.
  `cada_controlo_pede_o_mesmo_que_o_painel_de_antes` agora abre a janelinha onde o controlo mora.
- Seams antigos levados ao sítio novo: pintando com a janelinha aberta (autoplay, add_clip_full, transport_onion, transport_physics,
  transport_motion_path, view_tabs) ou abrindo o ▾ só no modo antigo (animation_chip, arrange_sources);
  `com_muitos_clips_a_roda_rola_ate_ao_ultimo` corre em cartões pelo «+N».
- Unitários em `barra_menus` (`cada_celula_do_fim_pede_o_toggle_de_antes`, `a_dur_le_infinito…`) e `cartoes` (`o_fps_le_se_inteiro…`).
- Corrida da crate: 143 testes de integração + 17 lib verdes; `clippy -D warnings` limpo em `ph2d-panel-timeline` e `ph2d-i18n`; fmt.

## Diferenças conscientes face à maquete B1

- Dono + engrenagem na fileira do título (o plano manda; na maquete estão no fim da fileira de tocar).
- «Keys | Curves» fora (decisão do dono).
- Nomes das trilhas e barras de interpolação NÃO entram: são onda 2.
- O menu do clipe abre pelo «⋯», não pelo botão direito (o plano dá «ou»; botão direito num chip não-ativo exigiria selecionar antes).

## Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop`
2. Na Timeline em baixo: fileira 1 «Keys · Containers · Arrange» e à direita «Animation: Herói», a engrenagem e o ×; fileira 2
   «Clips» com «andar ⋯» (azul), «pular» e «+»; fileira 3 tocar, Time, Frame, «24 fps · project», Auto Key,
   Record while playing, «+ Marker».
3. Clicar «pular» → fica azul e as chaves mudam; clicar no «⋯» → menu Rename/Duplicate/Reverse keys/Starts with this clip/Delete;
   marcar «Starts with this clip» → ▶ aparece no chip; clicar na engrenagem → Timeline options; escolher Loop e tocar → repete;
   clicar fora → fecha.
4. Errado se: algum controlo de antes sumiu, um clique não faz nada, a janelinha não fecha ao clicar fora, ou a folha das trilhas
   ficou baixa demais para trabalhar.

## ABERTO / próximo

- **Onda 2 do plano (a fita)**: nomes em árvore com unidade; barra entre chaves por interpolação + tokens `TimelineInterp*`
  (hoje não existe token por interpolação nem de gravação; ver mapa da onda 0 — `ph2d-tokens/src/color.rs:437-459`);
  borda vermelha do Auto Key; fita listrada depois do fim; enquadrar ao abrir.
- Avaliar: botão direito no chip como atalho do menu do clipe; ícone antes do «fps»; a marca do Auto Key / Record como na maquete.
- Smoke do dono ainda NÃO feito; integrar não é aprovar. Não há push nem integração desta linha por conta própria.

## 1b — duas fileiras e acabamento (dono 09/11, depois do smoke da onda 1)

- **Duas fileiras** (`transport_cartoes::paint_bar`): fileira 1 (tira do cabeçalho) = abas · migalha · tocar · Time · Frame ·
  «24 fps · project» … dono · engrenagem · ×; fileira 2 = Auto Key · Record while playing · Marker · «Clips» · chips · «+».
  Em largura estreita o `Fluxo` quebra; `Fluxo::guarda_para` faz o grupo dos clipes descer INTEIRO quando não cabe no resto
  e cabe numa fileira (sem ele, a 900 px os chips iam para o «+N» e o `⋯` sumia).
- **Peças genéricas novas** em `ph2d-editor-core/src/panel/cards_banda.rs` (append-only): `Sinal` (ícone ou ponto, por
  token), `Emoldurada` + `emoldurada_w/emoldurada_em` (caixa de marcar com moldura, sinal antes e/ou no fim, a moldura é o
  alvo), `rotulo_com_sinal{,_w}`, `botao_com_icone{_w,_em}`. Medidas: as do cartão (campo 28 px; a maquete B1, medida em
  chrome headless, usa 22–26 px a 11 px de letra — a mesma proporção).
- **Acabamento**: Auto Key com ponto no fim (`Text3`, `Danger` quando liga); Record com anel `Danger` antes do nome; fps com
  `IconId::Fps`; Marker = `IconId::Plus` + «Marker» (a chave `panel.timeline.card.add_marker` perdeu o «+», só a barra em
  cartões a usa); chips com ícone por tipo (`Film` clipe, `Cube` container — só nos chips, a lista suspensa guarda os
  dela); o ▶ do clipe de entrada em `Warn` (âmbar, como a estrela da maquete); «plays a - b» vira selo âmbar
  (`breadcrumb::paint_selo`, `WarnSoft`/`Warn`) — o dado já existia (`Status::Plays`); `breadcrumb::paint` clássico intacto.
- **Gate de altura de volta a estrito** (`depois >= antes`): 1930 px → 116 antes, **126** agora (92 na onda 1); 1600 px →
  91 → 92; 1100 px → 0/0 (banda de 423 px, folha nula nos dois). Teste `a_barra_tem_tres_fileiras_…` →
  `a_barra_tem_duas_fileiras_no_molde_do_rive` (fileira e ORDEM: tocar na do título; Auto Key · Record · Marker antes do 1.º chip).
- Provas por nome: `os_cartoes` 20 ✓, `_seam` 78 ✓, `breadcrumb` 10 ✓, `barra_menus` 2 ✓, `o_fps` 1 ✓, `banda` (editor-core) 3 ✓;
  clippy `-D warnings` limpo em `ph2d-panel-timeline`, `ph2d-editor-core`, `ph2d-i18n`; fmt.
- Fotos: [`15_onda1b_depois_animacao_smoke_1.png`](../fotos/15_onda1b_depois_animacao_smoke_1.png) ·
  [`15_onda1b_lado_a_lado_maquete_B1.png`](../fotos/15_onda1b_lado_a_lado_maquete_B1.png) (a metade de cima é a mesma
  maquete da onda 1; a captura headless nova saiu em branco).
- Divergências: «Clips» fica como rótulo do grupo; o selo não tem «in Scene» (a frase de hoje é «plays a - b»; sem string
  nova); dono + engrenagem continuam na fileira do título; a moldura das caixas é preenchimento `CardField` sem traço (é o
  que a maquete faz: fundo a 4 %).
- Superfície de colisão nova: `cards_banda.rs` (editor-core), `breadcrumb.rs`.
