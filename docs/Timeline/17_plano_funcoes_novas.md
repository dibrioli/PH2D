# 17 — Funções novas da Timeline (linha `line/TimelinePrincipal`)

> Fonte: a crítica `~/Documentos/GameEngineSamples/critica_ph2d.html#timeline` §4 «Acrescentar», maquetes B4–B7 e C2–C4,
> e o que ficou de fora do [plano 15](15_plano_redesenho_molde_rive.md) e da [onda 3](16_onda3_containers_arrange.md).
> Dono (10/11): «escolha e siga» — a ordem é a proposta abaixo (barato primeiro; o grande sozinho no fim).

## Regra

Função nova nos **cartões** (caminho por omissão); `PH2D_CARDS=0` pinta e responde como hoje, salvo onde a função é do
motor (ex.: o trecho em loop toca igual nos dois). HR-15 (tokens, i18n em inglês). Cada gesto novo ganha teste de costura
que clica o id PINTADO e confere o efeito. Fotos lado a lado com a maquete antes do smoke do dono.

## Inventário medido (10/11, leitura do código)

| # | Função | Já existe | Falta | Custo |
|---|---|---|---|---|
| 1 | Contorno da peça escolhida | anel na pintura (`strip_paint`), seleção não-desfazível é estado de painel (`intent.rs:323`) | `selected_strip` no estado, clique sem arrasto seleciona, vazio/Esc limpa | P |
| 2 | Tocar só um trecho (B6) | marcadores com nome (`doc.rs`), loop vivo do `Playhead` (`ph2d-core/src/playhead.rs:61,99`), `view_clock.rs:83` | o trecho TRANSITÓRIO (não gravado, sem undo) que sobrepõe o loop do clipe; o gesto; a etiqueta «takeoff → top» + ✕ | P |
| 3 | Losango de 3 estados | `KeyMark::{Livre,Animado,Chave,Mudado}` (`editor-core/.../key_mark.rs:23`) | `Livre` é um ponto: passa a losango vazio apagado; `Animado` = contorno vivo (maquete B1) | P |
| 4 | Marcas das voltas do loop | `StripView.loop_mode` (`snapshot.rs:138`) | o comprimento de UMA volta (fonte × velocidade, derivado) e a pintura dos riscos | P–M |
| 5 | Keys \| Curves | `graph_paint.rs` (grafo por trilha) | o botão e a troca da área inteira | M |
| 6 | Arrastar chip → faixa (B4) | chips (`barra_clips.rs`), «+» da faixa | arrasto com fantasma do chip, alvo de largada na faixa | M |
| 7 | Arrastar a chave no canvas (C4) | caminho + hit-test (`ph2d-app-motion/src/motion_path_overlay.rs:444,479`) | pontos de chave clicáveis no canvas e o gesto que reescreve a chave | M |
| 8 | Quem ouve um sinal (B5) | `Marker.signal` gravado (v13), `signals_crossed` | registo de ouvintes (componentes, Luau, câmera) — investigar `ph2d-runtime` primeiro | M–G |
| 9 | Animating · Done (B7) | transporte não-desfazível (`intent.rs:14`) | confirmar ONDE a agulha vira passo de undo; capturar a pose à entrada e repô-la ao sair sem passo | G |
| 10 | Container aberto no lugar (C2) | entrar = `nest.rs`, `container_open` | sanfona das lanes de dentro no tempo da cena, instâncias irmãs esmaecidas | G |
| 11 | Curvas do canvas editáveis com a animação a TOCAR (dono 10/11) | alças/losangos (`motion_key_handles.rs`, `motion_key_points.rs`) | hoje o arrasto conta com a agulha parada; investigar o gesto sobre um quadro que anda (tempo da chave fixo, não o da agulha) | M |
| 12 | O mouse só toca as curvas quando estão VISÍVEIS (dono 10/11) | hit e desenho pela mesma porta (`key_points`/`alcas`, `0c16f59992`) | provar a costura em TODA condição que esconde o desenho (painel oculto, outra aba, play, seleção, Path) — um gate por condição | P |
| 13 | Mudar o easing DANIFICA a curva do canvas (dono 10/11) | a forma do arco e o ritmo vivem no MESMO `Interp` por canal (`BezierW` 1/3–2/3 = forma; mexer no easing na setinha muda a forma) | separar forma espacial de ritmo (o idioma do After Effects: tangentes espaciais + velocidade por tempo); pede investigação e talvez schema | G |

Nenhum item toca contrato §6. Nenhum muda formato gravado por si (o 9 só se a pose for gravada — não será).

## Ondas

- **Onda 1 (P):** 1, 2, 3, 4. Smoke: `PH2D_NEST_SMOKE=4` (contorno, voltas) e uma cena com marcadores (trecho, losango).
- **Onda 2 (M):** abre pela cura do «Opacity» da Sprite (abaixo), depois 5, 6, 7.
- **Onda 3 (M–G):** 8 e 9 — cada um abre por uma investigação curta (onde mora o ouvinte; onde nasce o passo da agulha) que
  pode mudar o desenho; série só aí.
- **Onda 4 (G):** 10, sozinho.
- **Onda 5 (FIM DA FILA, dono 10/11):** 11, 12 e 13 — as curvas no canvas. O 13 abre por investigação (o modelo da chave muda?).

Cada onda fecha com `/pd-onda-fechar` (smoke do dono, janela nova).

## Onda 1 — desenho

1. **Contorno:** clique no corpo de uma strip sem passar a folga de arrasto = escolher; clique no vazio da faixa ou `Esc` =
   nenhuma. A escolha vive no estado do painel (não grava, não desfaz). Strip escolhida = o anel já ligado (`TimelineTinta`
   da maquete B4). A strip que deixou de existir limpa a escolha.
2. **Trecho:** na faixa dos marcadores (maquete B6 «Markers»), clicar ENTRE dois marcadores vizinhos toca esse trecho em
   loop. A barra mostra «takeoff → top» com ✕; ✕, trocar de clipe ou trocar de aba devolve o loop do clipe. O trecho não
   se grava nem entra no desfazer, e não reescreve o `loop_range` do clipe.
3. **Losango:** `Livre` = losango vazio na tinta apagada (`Text3`), `Animado` = contorno na tinta da chave, `Chave` = cheio,
   `Mudado` = cheio laranja; campo sem canal continua o ponto inerte. Os dois Inspectors (cartões e clássico) leem o mesmo.
4. **Voltas:** numa strip em Loop/Ping-Pong, um risco fino na altura da peça a cada volta (no Ping-Pong, cada inversão).
   Derivado no `rebuild` (volta = comprimento da fonte ÷ velocidade). Riscos mais juntos que `TIMELINE_*` px somem (zoom).

## Achado da onda 1 — «Opacity» mente numa Sprite (abre a onda 2)

Medido (10/11) ao fotografar o losango: `PropKind::Opacity` cai no alfa do Tint da Sprite (`ph2d-timeline/src/apply_prop.rs:162`,
`tint[3]`), mas a Timeline chama-o «Opacity», que no Inspector é o slider «Opacity (%)» = `Sprite::opacity` (canal OBJETO 11, que a
Timeline rotula «Sprite Opacity»). No vetor `PropKind::Opacity` já é a opacidade final do objeto (`VecDrivenStyle.alpha`) — certo.
Cura escolhida (a): numa Sprite «Opacity» conduz `Sprite::opacity`; o alfa do Tint vai com o canal do Tint.

1. `apply_prop.rs` escreve/lê `sprite.opacity` em vez de `tint[3]`.
2. `ph2d-ecs/canal_objeto.rs`: sai `OPACIDADE = 11`; o canal do Tint leva o alfa (`TINTA_FIM = TINTA + 4`, como `AUTO_TINTA`); docs em `canal_componente_donos.rs`.
3. `ph2d-app-components`: `canais_objeto.rs`, `marcas_inspector.rs` (`INSP_SPRITE_OPACITY`/`_CHIP` → `[PropKind::Opacity]`, amostra do Tint → `[ob(TINTA)]`), `tween_bridge.rs`.
4. `ph2d-preview-drive` (`facto.rs`, `lib.rs`): o condutor `SpriteAlpha` passa a `opacity`.
5. `ph2d-panel-inspector/event_sprite_value.rs`: o slider grava por `PropKind::Opacity`.
6. `ph2d-app-constraints/smoke_interruptor.rs`: destino `Prop::Opacity`.
7. Testes que prendem o sentido velho: `canais_objeto_donos_tests.rs:92`, `marcas_inspector_tests.rs:203–263`, a tabela `escritores()`.
8. `PROJECT_SCHEMA` sobe (`python3 scripts/schema-recount.py`); sem migração (§0.11).
9. Portão novo: por tipo de objeto, o nome de cada canal na Timeline = o rótulo do campo do Inspector cujo escritor o grava
   (junta `the_channel_says_the_inspectors_word` com `escritores()`).

## Onda 2 — desenho medido

> Maquetes a 1300 px, escala 1, `.ph` isolado em chrome headless (método do [doc 16](16_onda3_containers_arrange.md)); rects por
> `getBoundingClientRect`, estilos por `getComputedStyle`. Fotos: [B1](fotos/maquete_B1.png) · [B2](fotos/maquete_B2.png) ·
> [B4](fotos/maquete_B4.png) · [C3](fotos/maquete_C3.png) · [C4](fotos/maquete_C4.png). Leitura de código, 10/11.

### 5 — Keys | Curves (B1, B2)

> ⛔ RECUSADA pelo dono 10/11 (smoke): substituída pela checkbox «Edit curves on canvas» + alças no canvas (onda 2b, `ab823f94ae`). Não reconstruir. O que segue é histórico.

- **Maquete.** Segmentado `.seg.sm` na 1.ª fileira, à direita, antes do dono «Hero»: 100,4×25,6, raio 6, fundo
  `oklch(.235 .028 267)`, folga 2; células 42,6 / 53,8 × 21,6, raio 4, texto 10,5 px; ligada = fundo branco 14 % + texto
  `oklch(.916 .01 248)`, desligada = `oklch(.70 .02 255)`. Só na aba Keys. **Curves troca a ÁREA das faixas inteira**
  (735×150; nomes ficam): curva da linha escolhida (Position Y) `#B79CFF` 2 px, eixo esquerdo nas unidades dela (96/0, mono
  9,5 px `oklch(.56 .02 255)`); a outra visível (Position X) `#5FBFF2` 1,5 px tracejado 4/4 a 60 %, **normalizada 0–1 no eixo
  direito** (1/0 em `#5FBFF2` a 80 %); Scale Y (sem olho) não se desenha. Chaves 9 px a 45° (`#DCE2F5`; escolhida no acento, anel
  1,5 escuro), alças: fio branco 45 % 1 px, ponta 7 px anel 1,5. Grelha 1 px branco 5 %. Legenda por baixo. Inset 14 px × 12 px.
- **Existe.** Grafo POR TRILHA numa banda sob a linha aberta pelo twirl (`interact.rs:232` → `state.rs:373 toggle_expanded`;
  `state.expanded`, `graph_h` 132 px, `graph.rs:28`); encaixe vertical por trilha (`graph_paint.rs:85`, `graph::band_for`);
  velocidade `speed_view` (`state.rs:172`), ligada no menu de opções (`barra_menus.rs:191`, `event.rs:198`). Nenhum eixo direito,
  nenhuma normalização entre trilhas, nenhum botão de vista na barra (`transport_cartoes.rs:313 dono_e_opcoes`).
- **Desenho.** `TimelinePanelState.vista: {Keys, Curves}` — sessão, como `speed_view` (não grava, não desfaz). Ids
  `TIMELINE_VIEW_KEYS`/`_CURVES`, i18n `panel.timeline.view.{keys,curves}` (chave própria, não a da aba). Pinta-se em
  `dono_e_opcoes` (`f.a_direita`) com o mesmo `Seg` das abas, só com `tab == Keys`. Em Curves, `linhas` pinta os nomes e a área
  vira UMA banda: `paint_curve/anchors/handles` reaproveitados sobre a trilha escolhida; as outras trilhas visíveis só curva,
  normalizadas pelo próprio intervalo (eixo direito); `speed_view` vale também aqui. Editar só na escolhida (os mesmos
  `CurveAnchor/CurveHandle`, `BeginEdit/EndEdit`). Mudar de aba, de clipe ou abrir a lista larga gestos e volta a Keys.
  `PH2D_CARDS=0`: sem botão; o twirl continua.
- **Provas.** Clicar o id pintado `_CURVES` → a área tem `CurveAnchor` e zero `Key`; arrastar a âncora pintada = 1 Ctrl+Z; o eixo
  direito lê 0/1 e a curva secundária toca as duas bordas; fora de Keys o id não existe; clássico sem o id. **Custo M.**

### 6 — Arrastar um chip para uma faixa (B4)

- **Maquete.** O chip na mão fica `.dr`: fundo `#FF9F5A` 25 % + anel interno 1,5 `#FF9F5A` (92,9×20,3, pílula). Na faixa sob o
  ponteiro, o FANTASMA da peça no lugar exato da largada: 244,3×22 (= 2 s × 122 px/s), topo da faixa + 4, fundo `#FF9F5A` 22 %,
  borda 1 px TRACEJADA `#FF9F5A`, raio 5, ícone + nome 10,5/600. Começa em 4 s; a agulha não mexe. Dica «Drop: a new instance of
  Run & jump, 2 s long». Largar = a 2.ª instância com a duração da fonte.
- **Existe.** Chips = `Button` no hit index (`barra_clips.rs:146`). O «+» da faixa já manda `AddStrip{lane, source, t_start, t_end}`
  (`event.rs:432–458`, intent `intent.rs:545`, aplica em `intent_apply_stack.rs:73`): fonte = container escolhido (duração
  `containers[c].length`) ou `transport_sources::placed_clip`; vazio → `MIN_NEW_STRIP_S` 1 s (`event.rs:28`); tempo = AGULHA.
  Arrasto de strip entre faixas e snap: `strip_drag.rs:217 lane_at`, `:262 snapped` (quadro com `frame_snap`). Arrasto entre painéis
  `DragPayload`/`InFlightDrag` (`editor-core/interaction/drag_payload.rs:30,107`) — exige cola do shell; **não serve aqui**.
- **Desenho.** Painel-local: o chip passa a registar `TimelineSurface{kind: TimelineHitKind::SourceChip{opcao}}` (variante nova,
  append-only, `types.rs:302`) no lugar do `Button`; `Click` = o clique de hoje (escolhe), `Begin/Move` passado o limiar da casa
  arma `state.chip_drag {fonte, len, cursor}`. Por quadro: faixa = a da altura do ponteiro (só sobre a área das faixas), `t =
  snapped(ponteiro)`. `stack_lane_paint` pinta o fantasma; o chip pinta-se `.dr`. Up sobre faixa = o MESMO `AddStrip` do «+»
  (função partilhada que devolve `(source, len)`); fora = nada; Esc cancela. Arrange e Containers (no Containers só fontes que não
  criam ciclo — a recusa que `add_strip_to` já faz). `PH2D_CARDS=0`: sem chips, só o «+».
- **Provas.** Arrastar do id pintado do chip até x da faixa 0 → `AddStrip{lane 0, Container(i), t, t+2}` e 1 Ctrl+Z; clique parado
  continua a escolher; largar no vazio sob as faixas não cria nada; fantasma = rect que a peça nova pinta. **Custo M.**

### 7 — Arrastar a chave no canvas (C3, C4)

- **Maquete.** Pontos das chaves de posição no canvas: losango 9 px a 45° (`#DCE2F5`; o arrastado no acento) com anel 1,5 escuro;
  no arrastado, anel 26 px 1,5 px acento + brilho 14 px a 60 %. Caminho NOVO branco 70 % 1,5 px traço 3/4; o de ANTES branco 25 %
  1 px traço 2/4. Fantasmas (onion) 72,9 px a 32 %, saturação 0,4, sobem junto. Na fita a linha Position Y diz **104** (mono 10 px)
  e a chave do quadro 12 fica escolhida (anel 3 px acento 55 %). Agulha parada.
- **Existe.** Só o modo **Path** (ADR-0141) desenha trajetória: `motion_path_overlay.rs:163 active_path` exige `clip_path` (só Keys,
  `keys_tab`); âncoras e alças agarráveis (`:444 motion_path_hit`, `:479 _curve_hit`; `MarkRole::Dot/Anchor` em `_marks.rs:74`). O
  arrasto: press `despacho_metodos_modos_e_alcas.rs:201` (`history.begin`), move `:320` (`move_path_anchor`), Up
  `despacho_clique_select.rs:64` (`commit_if_changed`) = 1 passo. **Eixos separados (`TranslationX/Y`, `prop.rs:22`) — o caso da
  C4 — não desenham nada no canvas.** Onion: `ph2d_timeline_onion::collect_onion_ghosts` (`fase_canvas_overlays.rs:55`) relê o doc
  a cada quadro (seguem a edição de graça). Escrever valor: `SetKeyValue` (`intent.rs:216`).
- **Desenho.** Em `ph2d-app-motion` (shell não cresce): `key_points(keys_tab, doc, sel)` = um ponto por tempo com chave em
  `TranslationX` OU `Y` do clipe ativo, posição = a MESMA amostragem do apply nesse tempo (não a pose ao vivo), só sem caminho
  Path. Caminho tracejado amostrado (mesma porta). `MotionPathGrab::Key{t, x: Option<KeyId>, y: Option<KeyId>}` — reusa o
  `path_drag` e o par `begin/commit_if_changed` do shell (1 passo); no Down guarda-se a polilinha de antes para o traço fraco.
  Move: mundo→valor pela inversa do apply (pai e px/m incluídos) e escreve só os eixos que TÊM chave nesse quadro. A agulha e a
  seleção do objeto ficam; a chave vira a escolhida na fita. Prioridade: ponto (7 px) > alça do gizmo. Modo Path inalterado.
  `PH2D_CARDS=0`: igual (é do canvas).
- **Provas.** Ponto pintado = posição do objeto com a agulha nesse quadro (para cada chave); arrastar o ponto do quadro 12 → só
  a chave Y muda, `time` igual, 1 Ctrl+Z devolve; Arrange/Containers sem pontos (irmão de
  `the_motion_path_is_offered_only_on_the_keys_tab`); objeto com pai cai onde o ponteiro largou. **Custo M–G.**

### Perguntas para o dono (produto)

1. **Curves:** a curva aberta por linha (twirl) continua em Keys, ou some agora que Curves existe?
2. **Curves:** que curvas aparecem além da escolhida — todas as linhas com chaves, ou só as que você marcar com o olho (a maquete
   mostra um olho «Shown» no topo)?
3. **Arrastar a chave:** se nesse quadro só Y tem chave e você puxa o ponto para o lado, cria-se a chave de X (como o Auto Key)
   ou o ponto só anda na vertical?
4. **Arrastar o chip:** largar ABAIXO da última faixa cria uma faixa nova, ou só vale em faixa que já existe (como na maquete)?

### Onda 2 — respostas de produto (10/11; o dono delegou: «escolha e siga»; ele pode vetar)

1. A curva aberta por linha na aba Keys FICA; «Curves» acrescenta a vista da área inteira (nada some — regra do plano 15).
2. No Curves aparecem todas as linhas com chaves; o olho de cada linha («Shown» da B2) esconde a que não se quer ver.
3. Arrastar o ponto no canvas num quadro onde só Y tem chave CRIA a chave de X (como o Auto Key; Rive/Roblox: mexer grava).
4. Largar um chip abaixo da última faixa cria uma faixa nova (Unity Timeline / Premiere).

## Onda 3 — desenho medido (10/11, leitura do código; maquetes [B5](fotos/maquete_B5.png) · [B7](fotos/maquete_B7.png))

### 8 — Quem ouve um sinal (B5)

**Medido.** O sinal é só o NOME exato (`ph2d-runtime` `Signal { name }`; marcador `Marker.signal`, `doc.rs:172`, publicado por
`signals_crossed` → `timeline_bridge.rs:64`). Já existe o CENSO por nome: `ph2d-signal-census` (tabela `CAMPOS` do mundo, `FORA`
dos documentos — Motion `value.input`, estados de UI —, e a varredura estática dos scripts Luau `luau.rs`), mas ele só CONTA
(`SignalCensus.names: nome → {emit, listen}`) e só o Inspector o lê (`signal_hint.rs`). Ninguém responde «QUEM».
⚠️ «Heard by Camera» da maquete não casa com o modelo: quem ouve é o `ShakeEmitter` (no objeto que treme a câmera), a câmera é alvo.

**Desenho.**
- `ph2d-signal-census`: o leitor de cada linha passa a entregar a ENTIDADE com o nome (`fn(&World, &mut dyn FnMut(Entity, &str))`);
  `census` e `fingerprint` continuam a contar igual. Porta nova `listeners(world, fora, name) -> Vec<Listener>`:
  `Listener { object: Option<String> (o `Name`), what: Quem }`, `Quem` = a linha da tabela (componente) ou `Fora` (Motion, UI state)
  ou `Script`. Uma linha por (objeto, componente), sem repetidos, na ordem da tabela. Nome dinâmico de script ⇒ `script_may`.
- O painel recebe a lista por uma porta de dados (thread-local, o idioma do `set_signal_census`): a shell só a calcula quando há
  um sinal ABERTO no painel (`ph2d_panel_timeline::signal_asked()`), ~58 µs/quadro medidos para o censo inteiro.
- **Gesto (cartões):** clique parado na alça de um marcador COM sinal = vai até ele (como hoje) **e abre o cartão** «Signal at frame
  24 · land», «Name» (o nome, só leitura — renomear segue no botão direito) e a lista «Heard by»: uma linha por ouvinte
  «Hero · Signal Actions», tinta normal; ninguém ⇒ «Nobody listens» na tinta de aviso; script com nome não literal ⇒ «A script
  may listen». Fecha com Esc, clique fora, marcador apagado, ou outra aba. Marcador sem sinal: só vai até ele, como hoje.
  `PH2D_CARDS=0`: igual a hoje (sem cartão).

### 9 — Animating · Done (B7)

**Medido.** Não há caminho «da agulha» no undo: o passo nasce do diff por quadro (`undo_app.rs:416`). O scrub escreve a pose da
curva no mundo e declara-a pré-visualização (`declare_timeline_writes`, `ph2d-timeline-preview`); quando a agulha PARA, o `settle`
esquece a entidade, a pose vira documento e o clique seguinte grava um passo. As chaves vivem no `TimelineDoc` (passos próprios,
`history.rs` + `liga_gesto_da_cena`). O laranja (`Mudado`) não tem armazenamento: é o mundo ≠ curva (`channel_mark.rs:47`).

**Desenho (o idioma do Construct, e o do «condutor persistente» que o ledger já tem — `ScriptPose`, `still_driving`).**
- `ph2d-timeline-preview`: `Animating` — enquanto vive, guarda para cada (entidade, facto) que a Timeline conduziu o valor de
  ANTES da primeira escrita (a pose da cena à entrada) e, a cada quadro, SEGURA essa condução no ledger com esse valor como
  autorado (porta nova `PreviewDrive::hold`). Logo: a captura vê sempre a cena de entrada ⇒ ir e vir com a agulha, tocar, ou mexer
  sem chave (laranja) **não grava passo**; as chaves (doc) gravam como hoje. **Done** = escreve a pose de entrada, larga a condução
  (sem passo: o mundo volta ao que a base já diz). Os ossos (campos) pelo mesmo caminho do `FieldDrive`.
- Começa sozinho na PRIMEIRA vez que a Timeline mexe a cena na aba Keys com um clipe (cartões). Barra: pílula «● Animating jump»
  + botão «Done» à esquerda do transporte, e contorno azul na área das faixas (B7). Acaba com Done, trocar de clipe ou de aba,
  esconder a Timeline, ou dar Play no jogo — sempre devolvendo a cena. `PH2D_CARDS=0`: como hoje (sem sessão).
- O painel «Undo History» da B7 não existe no app (só o do Pixel): fica de fora, anotado.

**Respostas de produto (escolhidas pela maquete e pelo Construct; o dono pode vetar):** começa sozinho (sem botão «Animate»);
mudança laranja some no Done; trocar de clipe/aba = Done.

### Onda 3 — respostas do dono ao smoke (11/11)

- **O Done desliga o Auto Key** (a pergunta aberta): acabou de animar, arrastar o objecto volta a ser mexer na cena.
  `ph2d_panel_timeline::take_done()` (aviso de uma vez) → a shell faz `flags.auto_key &= !…` no drain.
- **O Done não larga a seleção** (defeito do smoke): o restauro usava `apply_project_with(&base, None)`, o caminho do
  load, que só transporta a seleção de campo; passa agora `Some(&field_selection_mark())`, como um passo de undo.
- **O Done funciona com a agulha a tocar** (defeito do smoke): tocar é gesto e desfazia o Done no mesmo quadro.
  `take_done(para)` corre ANTES da `sessao` e pausa a agulha; teste `done_while_playing_stays_done` (mutação sangra).
