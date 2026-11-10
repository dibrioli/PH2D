# Continuação — `line/TimelinePrincipal`, plano 17 (funções novas), onda 1 (itens 1–4)

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · commits da onda sobre `4060362f15`: `5c5ae7c309`, `07f10ab416`, `14dc0b695d`,
  `0afb8341f1`, `4f1727a9db`, `46dd98fd7d` (HEAD).
- Plano: [`17_plano_funcoes_novas.md`](../17_plano_funcoes_novas.md) (10 funções, 4 ondas; dono 10/11 «escolha e siga»). Onda 1 = itens 1–4.
  Onda anterior: [`..._ONDA3.md`](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-10_ONDA3.md). Cena: [`smoke_nest.md`](../smoke_nest.md).
- Fotos (`../fotos/`, todas com o patch local FOTO-HACK revertido): `24_voltas_e_contorno.png`, `24_voltas_zoom.png`, `24_trecho.png`, `24_trecho_zoom.png`,
  `24_lado_a_lado_trecho.png`, `24_lado_a_lado_arrange.png`, `24_losango.png`, `24_losango_zoom.png`, `24_losango_classico_zoom.png` (antigo, antes do
  tamanho novo), `24_lado_a_lado_losango.png`.

## O que a onda fez

### Item 1 — contorno da peça escolhida
- `TimelineState.strip_escolhida` (estado do painel; não grava, não desfaz). Clique sem arrasto no corpo (`strip_drag.rs`, BODY) escolhe; vazio da faixa
  (`TimelineHitKind::Lane`) larga; `esquece_a_sumida` por quadro.
- Esc larga no FIM da cadeia do Esc: `ESC_LIVRE`/`take_esc_livre` em `ph2d-editor-core` `interaction/dispatch/key.rs`, lido em `interact.rs`. Quem já tinha o
  Esc fica com ele (menu aberto fica com o Esc).
- Anel da maquete B4 (2 px claros dentro + foco 60 % fora) via `paint_strip(.., escolhida)`. Clássico: mesmo estado, sem anel.
- Teste `tests/it/a_peca_escolhida.rs`.

### Item 4 — voltas do loop
- `ClipStrip::volta()` = fatia ÷ velocidade (0 para Once). `StripView.volta` derivado no `rebuild` (`DOC_VERSION`/`PROJECT_SCHEMA` iguais).
- `pecas_cartoes::{voltas_x, pinta_voltas}`: pontilhado da `.c13lm` (pontos 1,5 px, vão 1,5, 70 %). Piso `TIMELINE_VOLTA_MIN_PX` = 6 (raciocínio, não medição).
  O shake da cena 4 mostra 3 riscos.

### Item 2 — tocar só o trecho entre dois marcadores
- Motor `crates/ph2d-timeline/src/trecho.rs`: estado de sessão `trecho` + `trecho_armado` em `TimelineState`; `trecho::arma` a cada quadro em `arm_view_loop`;
  agulha ao início, toca, Wrap. Deixa de valer com ✕, outro clipe/aba/container/lista ou marcador mudado. Não grava, não desfaz, não reescreve `loop_range`.
  Intent `PlayStretch(Option<(usize,usize)>)`.
- Painel: faixa «Markers» da B6 (`crates/ph2d-panel-timeline/src/faixa_marcadores.rs`, 22 px, última linha da lista, rola). Chip 15×15 r4: ⚡ fundo
  `TimelineMarker` quando tem sinal; etiqueta a 12 %/30 % quando só nome; nome a 9 px, vira à esquerda só quando não cabe. Vão entre chips = alvo
  `TimelineHitKind::MarkerGap{a,b}`; caixa do trecho 20 %/fio 65 %.
- Marcadores são do DOCUMENTO (tempo da cena); a faixa aparece em Arrange e Keys sem pilha, nunca dentro de container, só com ≥ 1 marcador.
- Régua dos cartões: sem bandeirolas/nomes; o laço fino do topo mostra o TRECHO enquanto vale; as pontas do laço do clipe não se pintam nem se agarram
  enquanto o trecho toca (controlo que mente); todos os números voltam.
- Gestos dos marcadores (arrastar, Alt+clique, clique busca, botão direito) passaram para o chip, mesmos ids; alvo 20×8 → 20×22.
- Pílula «▷ takeoff → top ✕» na barra (`transport_cartoes.rs`); alvo = a pílula inteira.
- Pisos OKLab em `Theme::ALL`: fundo × régua 0,104 (piso 0,09), contorno 0,341 (0,30), número × caixa 0,195 (0,17).
- Cena `PH2D_NEST_SMOKE=2`: takeoff 0 / top 1,1 / land 2 s e `set_scene_length(10.6)` (antes 2 pulos apareciam «além do fim»). Shell líquido 0.
- Testes: `o_trecho_entre_marcadores`, `trecho`, `faixa_marcadores_tests`, `ruler_trecho_tests` (`o_trecho_armado_e_o_laco_da_regua`,
  `a_regua_mostra_todos_os_numeros`, `a_caixa_do_trecho_le_se…`).

### Item 3 — losango com os estados do Rive
- Porta única `pinta_losango`/`cor_do_losango` (`key_mark.rs`) para o clássico e `card_field.rs`.
- Glifo da B1: caixa 9 px a 45°, raio 1,5, traço 1,5 (tokens `KEY_MARK_GLYPH`, `_RADIUS`, `KEY_MARK_STROKE` em `ph2d-card-stack` `metrics.rs`; ponta a ponta
  11,49 px dentro da caixa 12; `KEY_MARK_BOX` igual).
- Livre = losango vazio `Text3` (clássico)/`CardLabel` (cartões) a `TIMELINE_LOSANGO_LIVRE_ALPHA` 0,75. Animado = contorno `TimelineTinta::Chave`
  (#DCE2F5 ref.). Chave = cheio no mesmo. Mudado = `TimelineTinta::Mudado` #FF9F5A + `MudadoContorno` (162,107,71).
- Nos cartões o campo Mudado ganha contorno e valor laranja (`paint_field`; não com foco/arrasto/travado). Clássico: só o losango.
- O clássico (`PH2D_CARDS=0`) também muda: Livre deixa de ser ponto (desejado). Campo sem canal = ponto inerte.
- Mapa: o losango do Pivot mostra só o pivô (antes acendia com Position; o clique continua a gravar pivô+posição+collider por `ARRASTA_DO_PIVO`);
  `marcas()` e `canal_do_campo` leem o PRIMEIRO grupo.
- Pisos: Livre × Animado ≥ 0,25 (pior PlumberGreen clássico 0,294); Mudado matiz 41,7–54,6°; Mudado × chave pior Oled 0,153 (piso 0,15 — margem 0,003, apertada).
- Portões novos: `o_losango_de_cada_campo_e_o_canal_que_o_seu_escritor_grava` (Transform + Sprite; Física/Junta fora), `o_mapa_e_o_clique_leem_o_mesmo_grupo`,
  `o_mudado_e_laranja_em_todo_tema`, `o_glifo_tem_o_tamanho_da_maquete_e_nao_sai_da_caixa`, `o_livre_e_o_animado_leem_se_diferentes_em_todo_tema`.

### Curas de portão
- `toda_crate_que_pinta_campos_escolheu_o_seu_lado` estava vermelho DESDE a onda 3 (a isenção do olho não viajou para `faixas_coluna.rs`): curado em `4f1727a9db`.
- `the_shell_only_shrinks …and_every_app_crate_too`: `ph2d-app-components` 80 972 → 80 849 / teto 80 869 (teste do escritor enxugado; história verbatim em
  `docs/archive/integracao-jornadas/COMENTARIOS_CORTADOS_line_TimelinePrincipal_2026-11-10.md`).
- `ruler.rs` 606 → 592; `ids/timeline.rs` → `ids/timeline_entry.rs`; `intent.rs` ≤ 700.

## Superfície de colisão (para o integrador)

- `ph2d-panel-timeline`: `faixa_marcadores*`, `ruler*`, `ruler_trecho*`, `transport_cartoes`, `strip_*`, `pecas_cartoes*`, `stack_lane_paint*`, `interact`,
  `state`, `event`, `populate`, `geom`, `tracks`, `lib`, `ids/timeline_entry`, `tests/it/*`.
- `ph2d-timeline`: `trecho*`, `stack`, `snapshot*`, `state`, `intent*`, `view_clock`, `lib`.
- `ph2d-editor-core`: `key_mark*`, `card_field`, `interaction/{types,timeline_cursor,dispatch/key}`, `tests/it/architecture_every_editable_field_is_a_channel.rs`.
- `ph2d-app-components`: `marcas_inspector*`, `lib`, `camera_2d`, `component_registry_for_tests`, `instantiate`.
- `ph2d-card-stack` `metrics`; `ph2d-tokens` (`chrome`, `timeline_tinta`, `lib`); `ph2d-i18n` `timeline`; `shells/desktop` `nest_smoke`; docs.
- Contadores como DELTA: sem `PROJECT_SCHEMA`, sem contrato §6, sem formato gravado; shell líquido 0; `ph2d-app-components` −123 B (teto 80 869).

## O que um merge pode partir

- `ph2d-panel-timeline` (`ruler.rs`, `state.rs`, `lib.rs`, `geom.rs`, `strip_*`) é tocado por outras ondas: conferir `collision-surface.sh` pelo caminho absoluto.
- `ph2d-app-components`: teto de LOC soma entre linhas (80 849/80 869) — outra linha que cresça ali reprova `the_shell_only_shrinks`.
- Cena `NEST_SMOKE=2` mudou (fim 10,6 s); outra linha que numere cenas do nest recontar.
- Esc: `ESC_LIVRE` fica no fim da cadeia; outra linha que adicione um consumidor do Esc deve ficar ANTES dele.
- O clássico mudou o Livre (ponto → losango vazio): fotos/testes de outra linha que leiam o ponto reprovam.

## Fecho

- `fmt --all` sem mudança; clippy `-D warnings --all-targets` nas 8 crates tocadas verde; testes por nome todos verdes; smoke compilado `--profile smoke`.
  **Sem suíte completa nem `nextest-impacted`** (do integrador).
- Mutações feitas, todas vermelhas e restauradas: escolha desligada, voltas vazias, anel ignorado, ✕ não devolve, vão fora da faixa, laço pinta `loop_range`,
  Livre volta a ponto (clássico e cartão), Pivot com posição, Mudado em Warn, raio 0 do glifo.

## Diferenças restantes contra a maquete

Amarelo do chip ⚡ (Warn do tema) mais pálido que #FFCF4D · faixa Markers sem fio separador de baixo · B6 está na aba Keys e a cena 2 tem pilha → só Arrange ·
pílula fica depois de «Marker» · B4 não tem anel de peça (a maquete marca o chip) · anel na faixa muda sai desbotado · Mudado num campo numérico não foi
fotografado (cai no Tint por causa do achado abaixo).

## Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && PH2D_NEST_SMOKE=4 ./target/smoke/ph2d-host-desktop` — Timeline em Arrange.
   Clique no meio da peça «swing» (azul): ganha um contorno. Clique num espaço vazio da faixa (ou Esc): o contorno some. Abra o olho da faixa FX: a peça
   «shake» mostra 3 pontilhados verticais, um em cada recomeço do loop.
2. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && PH2D_NEST_SMOKE=2 ./target/smoke/ph2d-host-desktop` — tecla L abre a Timeline,
   aba «Arrange». Embaixo da faixa Jumps há a faixa «Markers» com takeoff, top e land. Clique entre takeoff e top: a agulha volta a 0 e toca só a subida, em
   loop; na barra aparece «▷ takeoff → top ✕» e o fio verde no topo da régua cobre só esse pedaço. Clique no ✕: volta a tocar a cena toda.
3. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop` — pause a Timeline; no
   Inspector abra «Transform». Rotation: losango vazio e apagado (não anima). Position: losango com contorno claro (anima, sem chave aqui). Digite 0 em
   Frame: Position fica cheio (chave aqui). Desligue Auto Key e digite outro número em Position X: losango laranja, e o campo ganha borda e número laranja.
4. Deu errado se: o contorno não aparece/não some; não há pontilhados no shake; o clique entre marcadores toca o pulo inteiro ou o ✕ não volta; os números
   da régua 0/1/2 sumiram; dois losangos de estados diferentes parecem iguais; `PH2D_CARDS=0` ficou com outra cara além do losango.

## ABERTO / próximo

1. **ACHADO «Opacity» mente numa Sprite.** `PropKind::Opacity` = alfa do Tint (`apply_prop.rs:162`); o Inspector «Opacity (%)» = `Sprite::opacity` (canal
   OBJETO 11, rotulado «Sprite Opacity» na Timeline). Cura (a) planejada em 9 passos em [`17_plano_funcoes_novas.md`](../17_plano_funcoes_novas.md)
   §«Achado da onda 1»; abre a onda 2 e sobe `PROJECT_SCHEMA` (recontar por `python3 scripts/schema-recount.py`).
2. Onda 2: Keys|Curves, arrastar chip → faixa, arrastar chave no canvas.
3. Ondas 3 (quem ouve; Animating·Done) e 4 (container no lugar).
4. Smoke do dono não feito. Depois: `/pd-linha-fechar`. Sem push nem integração.
