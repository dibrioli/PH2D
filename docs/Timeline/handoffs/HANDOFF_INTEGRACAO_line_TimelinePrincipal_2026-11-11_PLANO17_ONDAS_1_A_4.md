# Integração — `line/TimelinePrincipal`: plano 15 onda 3 (Containers/Arrange) + plano 17 ondas 1–4 (funções novas)

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · HEAD `f5a933e292` (+ o commit deste handoff) ·
  merge-base `bc4a6ce930` (main de 10/11) · 48 commits · 313 arquivos, +18 697 / −1 900.
- Antecessor já no main: [ondas 1–2 do plano 15](HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-09_ONDAS_1_2.md). Esta linha continua dali.
- Planos: [15](../15_plano_redesenho_molde_rive.md) (onda 3, spec [16](../16_onda3_containers_arrange.md)) · [17](../17_plano_funcoes_novas.md)
  (todas as «respostas do dono»). Narrativa e mecanismo por onda (não repetidos aqui):
  [plano 15 onda 3](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-10_ONDA3.md) ·
  [17 onda 1](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-10_FUNCOES_ONDA1.md) ·
  [17 ondas 2/2b](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-10_FUNCOES_ONDA2.md) ·
  [17 onda 3](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-11_FUNCOES_ONDA3.md) ·
  [17 onda 4](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-11_FUNCOES_ONDA4.md).
- **Smokes do dono OK:** plano 15 onda 3; plano 17 ondas 1, 2, 2b, 3 (com as curas do Done) e 4 (com as curas de «a instância acompanha» e
  «marca da posição original»). ⚠️ **Depois do último smoke, a auditoria de fecho MUDOU o caminho do Done** (ver «Auditoria») →
  **re-smoke do dono do Done PENDENTE** (aberto, abaixo).
- Sem push, sem rebase no main, sem integração por esta linha.

## O que a linha entrega (só cartões; `PH2D_CARDS=0` com a saída de antes, salvo o losango Livre)

1. **Plano 15 onda 3:** Containers e Arrange no molde do Rive (faixas de 30 px com peso em pílula e olho, peças coloridas por clipe,
   lista de containers com «used 2×»/«not placed», chips em dois grupos); a Timeline flutua/redimensiona/encaixa nos cartões; estados das alças,
   cursor tingido, linha do fim, laço fino na régua.
2. **Plano 17 onda 1 (itens 1–4):** peça escolhida acende o anel (Esc larga); voltas do loop pontilhadas; tocar só o trecho entre dois
   marcadores (`trecho.rs`, estado de sessão); faixa «Markers» (marcador = linha no tempo exato + alça do fade); losango com os estados do Rive.
3. **Onda 2/2b (itens 5–7):** cura do Opacity da Sprite (canal Opacity conduz `Sprite::opacity`); chip → faixa por arrasto
   (`TimelineHitKind::SourceChip`); chave no canvas e alças do arco (`ph2d-app-motion`, checkbox «Edit curves on canvas»). A vista Curves foi
   RECUSADA pelo dono e saiu.
4. **Onda 3 (itens 8–9):** cartão «Heard by» do sinal (quem ouve); sessão «Animating · Done» (prévia conduzida e devolvida).
5. **Onda 4 (item 10):** o container aberto no lugar (`no_lugar.rs`): clicar a instância abre as lanes de dentro por baixo, moldura laranja,
   irmãs esmaecidas, edição convertida para o tempo do container.

## Foundational/compartilhado tocado (todos ADITIVOS salvo onde dito)

- `ph2d-editor-core`: `TimelineHitKind::{MarkerGap{a,b}, SourceChip{option}}` (variantes novas); `ESC_LIVRE`/`take_esc_livre` (`interaction/dispatch/key.rs`);
  `key_mark*`, `card_field`, `cards_banda_tons.rs` (`degrade_vertical_em`), `text_elide::largura_do_titulo`; gate
  `architecture_every_editable_field_is_a_channel` (entrada nova em `SEM_COLUNA_FORA`: `no_lugar_paint.rs::coluna`, NaoAnimavel).
- `ph2d-tokens`: `TimelineTinta` + `PecaA..PecaE, PecaContainer, PesoPilula, OlhoMudo, LacoDaRegua, Mudado, MudadoContorno`;
  **REMOVIDA** a variante `Listra` (morta); `CurvaPrincipal/CurvaSecundaria` entraram e saíram na 2b (quem as usar parte). Métricas em `chrome.rs`.
  `color.rs` intocado (700/700).
- `ph2d-preview-drive`: `PreviewDrive::driven_held` (`held.rs`); `ph2d-timeline-preview`: `declare_timeline_writes_held`, `done.rs`
  (`devolve_agora`). A ponte segue com UMA declaração.
- `ph2d-signal-ends`: `Quem`, `Listener`, `SignalListeners`; `ph2d-signal-census`: porta `listeners(world, fora, name)`; os leitores
  passam a entregar `(Entity, &str)` (census e fingerprint inalterados).
- `ph2d-ecs`/`ph2d-render`/`ph2d-app-components`/`ph2d-app-constraints`/`ph2d-constraints`: Opacity (canal OBJETO 11 «Sprite Opacity» REMOVIDO,
  **nunca reusar o número**; `Sprite::cascaded_opacity`; `Prop::Opacidade` por `write_opacity`/`read_opacity`).
- `ph2d-timeline`: `container_uses`, `trecho.rs`, `no_lugar.rs`, `snapshot_lane.rs`, `nest_map::voltas_of`; `TimelineIntent::{PlayStretch, NoLugar}`;
  `TimelineState.{strip_escolhida, trecho, animating, no_lugar}`. `ph2d-cursor-art` (cursores ↔ e mão), `ph2d-card-stack` (`KEY_MARK_*`),
  `ph2d-i18n` (timeline/canais/constraints), `ph2d-ui-testkit` (`caixas_pintadas`), `ph2d-app-motion` (`motion_key_points*`, `motion_key_handles`).
- Cargo: `Cargo.toml` de panel-timeline (+`ph2d-signal-ends`, `ph2d-tokens` movido), timeline-preview (+`ph2d-anim` e 2), cursor-art (+`image` png,
  já no lock), app-components/constraints/motion. `Cargo.lock`: +4 arestas, **nenhum pacote externo novo**. `cargo machete` NÃO foi corrido nesta
  linha — o integrador/ship corre.
- Shell (`shells/desktop`): `render_loop/{fase_snapshots_publish,fase_timeline_drain,fase_timeline_view,timeline_bridge*,sim_extract*,
  fase_canvas_overlays,animacao_smoke}`, `undo_app*.rs`, `input_dispatch/despacho_metodos_*`, `nest_smoke*`, `project_schema*`.

## Símbolos que podem colidir

`PROJECT_SCHEMA` 231→232 e tripla (232, 13, 29) (onda 2); variantes novas de `TimelineHitKind` e `TimelineTinta` (ordem do enum/`ALL`); canal
OBJETO 11 removido; chaves i18n `panel.timeline.*` (card/no_lugar/marker_readout…); ids em `ph2d-panel-timeline/src/ids/{timeline_barra,
timeline_cartao,timeline_no_lugar,timeline_entry}`; cena `PH2D_ANIMACAO_SMOKE` (marcador «land» a 1,0 s, Signal Actions no Inimigo; «pular» com
Position X); `PH2D_NEST_SMOKE=2` (fim 10,6 s) e `=4`; env `PH2D_ANIMACAO_SMOKE=0` desliga a cena.

## Superfície de colisão (saída de `bash scripts/collision-surface.sh`, colada)

```
SUPERFÍCIE DE COLISÃO — line/TimelinePrincipal contra main
  merge-base bc4a6ce930   ·   48 commit(s)   ·   313 arquivo(s)
  ⚠ PROJECT_SCHEMA                        232   (base: 231)
  ⚠   └ tripla do gate               (232, 13, 29)   (base: (231, 13, 29))
    VEC_SCENE_SCHEMA 29 (base 29) · FLIP_SCHEMA 13 (base 13) · DOC_VERSION (timeline) 25 (base 25)
    ph2d-ecs 121 (base 121)
  ✗ ph2d-render / ph2d-script (espelho = ecs + K)  —  (base —)   [SONDA CEGA: defeito do script, já conhecido]
  CONTRATO §6: nodegraph/node.rs intocado · editor-core/tool.rs intocado
  ADR: nenhum criado (próximo livre 0201) · Cargo.lock: nenhum '+name' externo · marcadores de conflito: nenhum
  TETOS de LOC nos arquivos tocados: nenhum passa do teto
```

⚠️ Referência, não evidência: o integrador lê o `main` NO ficheiro e reconta o degrau como DELTA.

## Contadores (DELTA)

- `PROJECT_SCHEMA`: **+1** (231→232, onda 2). A escada e a tripla moram em arquivos irmãos (`project_schema*.rs`): reconte por
  `python3 scripts/schema-recount.py <valor do main>` (sem argumento dá IndexError). Mais nenhum degrau, registo ou canal novo.
- Shell LOC **189 041 = TETO 189 041** (exatamente no teto). ⚠️ Qualquer outra linha que cresça a shell colide aqui; a cura é MOVER.
- `ph2d-app-components` 80 849 / teto 80 869 (soma entre linhas). `intent.rs` e `intent_apply.rs` de `ph2d-timeline` em 699/700.
- `color.rs` 700/700; i18n `lib.rs` 700/700 (não tocado).
- §6: nenhum contrato congelado tocado (nenhum arquivo em nodegraph/tool-api/vector/imageio no diff).

## O que um merge pode partir

- Quem constrói `TimelineState` (campos `strip_escolhida`, `trecho*`, `animating`, `no_lugar`) ou casa `TimelineIntent`/`TimelineHitKind`/`TimelineTinta`
  exaustivamente; quem chama `state::caminho()` (agora devolve o par `(edit_path, no_lugar)`).
- Quem lê os leitores do census (agora `(Entity, &str)`) ou constrói `Listener`/`SignalListeners`.
- Quem escreve o canal de Opacity / `PropKind::Opacity` (agora `Sprite::opacity`; alfa do Tint vai com o canal do Tint, `TINTA_FIM = TINTA + 4`).
- Testes de altura/fileiras da barra (`os_cartoes`: pílula do Animating na 2.ª fileira) e `stack_bands` (desloca as lanes de baixo).
- Outras linhas que acrescentem `ColorToken`/`TimelineTinta`: reconte os censos de apelidos; `Theme::ALL` varrido por vários gates de piso OKLab.
- O clássico (`PH2D_CARDS=0`) mudou só no losango Livre (ponto → losango vazio).
- Cenas de smoke de outras linhas que contem marcadores de `PH2D_ANIMACAO_SMOKE`.
- Esc: `ESC_LIVRE` fica no FIM da cadeia; consumidor novo do Esc entra ANTES.

## Auditoria de fecho (2 lentes: correção + costura de UI/paridade) — tudo curado com RED→GREEN e mutação que sangra

- Arrasto/chip pela banda PINTADA com o container aberto (`geom::linha_sob`): `arrastar_de_fora_le_a_lane_pelas_bandas_pintadas`,
  `o_chip_largado_cai_na_linha_pintada_sob_o_ponteiro`, `o_container_nao_se_larga_no_proprio_interior`. Chip numa linha de dentro = `NoLugar{AddStrip}`.
- `vista` das lanes de dentro só no Arrange (`as_linhas_de_dentro_so_existem_no_arrange`); renome com a altura da linha pintada
  (`o_renome_da_lane_tem_a_altura_da_linha_pintada`); cartão do sinal estável por (índice, tempo, contagem)
  (`tirar_um_marcador_antes_nao_troca_o_cartao_de_marcador`).
- **Done:** campo NÃO conduzido sobrevive (`ph2d-preview-drive/src/held.rs`, `while_animating_an_edit_on_an_undriven_field_survives_done`); o Done espera
  o botão solto e o jogo parado e grava o diff pendente ANTES de restaurar, com a chamada movida para antes dos settles
  (`ph2d_timeline_preview::devolve_agora` em `done.rs`; `done_held_button_or_game_keeps_the_end_armed`). Sem teste no nível da shell (no teto).
- Apagar container não destrava a instância recortada de outro: comprimentos casados por NOME do container, nomes duplicados saltam
  (`apagar_um_container_nao_mexe_na_instancia_recortada_de_outro`).
- Stretch e trim no mesmo canto deslocam a marca um do outro e guardam as duas origens (`stretch_then_trim_on_one_edge_keep_both_originals`);
  o teste antigo `a_trim_and_a_stretch_at_the_same_edge_keep_separate_marks` foi mudado para a regra do dono de 11/11. Volta invertida (ping-pong)
  espelha as marcas `[-m1,-m0,-m3,-m2]`.
- Ouvintes por entidade (`dois_objetos_com_o_mesmo_nome_sao_dois_ouvintes`); encaixe de quadro dentro da peça aberta na grelha do container
  (`NoLugarView.mapa: Volta`, `snapped_em`; `dentro_da_peca_aberta_o_encaixe_e_na_grelha_do_container`,
  `o_chip_largado_dentro_cai_na_grelha_do_container_a_velocidade_um`).

## Prova de fecho (por nome; suíte completa e `nextest-impacted` ficam para o integrador)

- `cargo fmt --all --check` ✓ · clippy `-D warnings` nas 24 crates tocadas ✓.
- Gates por nome, verdes depois das curas: `the_shell_only_shrinks`; `fn_loc_caps` (`fase_snapshots_publish` 201→corte de comentário);
  `architecture_panel_loc_cap` (`apply_event` 202→helper `clique_dos_cartoes`; `strip_drag_tests.rs` 603→`strip_drag_marca_tests.rs`;
  `stack_lane_paint_tests.rs` 601); `file_loc_caps`; `architecture_workspace_file_loc_cap`; `arch_safe_clamp_only`;
  `architecture_every_editable_field_is_a_channel`; `the_corner_of_a_control_comes_from_the_theme` (raio da moldura por `paint::frame_radius`);
  `every_word_this_panel_shows…`; `o_claude_md_cabe_no_orcamento`.
- ⚠️ `the_timeline_declares_what_it_writes` NÃO casou nenhum teste por nome (só um `mod` em `shells/desktop/tests/it/main.rs:240`): o integrador confirma
  que roda na suíte da shell.
- `bash scripts/agent-loop-profile.sh` (fecho): paralelismo 1,15/passo ✗ (alvo ≥ 1,5) · test:check 2,9× ✗ (alvo ≤ 1) · contexto relido 251 mil/passo ✗
  (alvo ≤ 250 mil) · respostas/sessão 179 ✓.

## Smokes (comando inteiro; reconferir no main depois de integrar, trocando o `cd` pelo primário)

Todos: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && <ENV> ./target/smoke/ph2d-host-desktop` (binário `--profile smoke`).

1. `PH2D_ANIMACAO_SMOKE=1` — Timeline (aba Keys), clique «pular»: pílula «● Animating pular  Done» na 2.ª fileira e borda azul. Arraste a agulha, solte
   (o Herói muda de pose); Done devolve o Herói. Role até «Markers», clique a alça de «land»: cartão «Signal at frame 24 · land» com
   «✓ Heard by Inimigo · Signal Actions». Marque «Edit curves on canvas»: arco tracejado, losangos e pontinhos; arrastar um pontinho entorta o trecho, 1 Ctrl+Z.
   Errado se: o Done não devolve a pose, o Ctrl+Z anda pela agulha, o cartão cobre a barra ou diz «Nobody listens».
2. `PH2D_NEST_SMOKE=4` (aba Arrange, pausar) — clique o 1.º «Run & jump»: abrem-se por baixo Base (walk, jump) e Upper body (swing) dentro da moldura
   laranja; a 2.ª instância fica esmaecida e acompanha. Errado se: a irmã deixa-se arrastar, a edição cai deslocada, Ctrl+Z não desfaz, as lanes de baixo ficam cobertas.
   Também: clique o corpo de «swing» (contorno), olho da FX (3 pontilhados nas voltas do shake).
3. `PH2D_NEST_SMOKE=2` — tecla L, aba Arrange: faixa «Markers» (takeoff, top, land); clique entre takeoff e top: toca só a subida, pílula «▷ takeoff → top ✕».
4. Arrastar um chip da barra para uma faixa (e para baixo da última): fantasma tracejado, largada cria a peça (1 Ctrl+Z).

## ABERTO

- ⚠️ **Re-smoke do dono do Done** (passo 1 acima), pois a auditoria mudou o caminho (espera o botão/jogo, grava o pendente antes de restaurar,
  campo não conduzido sobrevive). Sem teste no nível da shell.
- `reset_trail` sem chamador na shell; foto `43_a_moldura_acompanha.png` não tirada.
- Pílula «plays a – b» na fileira 1 e sem o crumb «Scene › Container» (deliberado, vs maquete C2); `SetContainerLoop` embrulhado sem undo (como o original);
  linhas de ouvinte de script nomeiam o objeto do script, não o alvo de `connect(obj…)`.
- Custo do `apply_project_with` no Done e custo por quadro das amostras do caminho no canvas: NÃO medidos. Depois do Done o Auto Key pode gravar
  se o artista arrastar (já desligado pelo Done; ver memória `project_timeline_done_turns_off_autokey_2026_11_11`).
- Fila do plano 17 acaba na onda 4. **Onda 5 (itens 11–13: editar curvas a tocar, mouse só no visível, easing que danifica a curva) NÃO começada** — só por ordem do dono.
- Memória tocada nesta linha (no diff): `project_timeline_done_turns_off_autokey_2026_11_11.md` (novo), `reference_topic_owner_product_decisions.md` e
  `MEMORY.md`. A memória `project_timeline_curves_live_on_canvas_2026_11_10.md` foi escrita no primário (fora deste diff).
