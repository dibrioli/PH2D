# F3 — Corpo físico e gatilho (plano, 09/11)

Origem: crítica externa `GameEngineSamples/critica_ph2d.html` + `estudo_paineis_game_engines.html`.
Decisões do dono (09/11, «faça conforme sugere»): [memória](../../../project-memory/project_physics_critique_answers_2026_11_09.md).

Já existia (conferido no código, não reconstruir): tipo de corpo na 1.ª linha do cartão e campos de
Dynamic escondidos para Static/Kinematic (`physics_cartao.rs:148-186`); colisor e gatilho no MESMO
cartão da forma, `Touch: Solid | Sensor` (`physics_cartao_forma.rs:210-227`); aviso de corpo sem forma.

## Onda 0 — espaço
✅ feita — commits `9029969369`, `6cfd80dddf`.
`ph2d-app-physics` no tecto exato (55 515). Cenas de smoke puras saem para `ph2d-physics-smoke-scenes`
(re-exportadas; nenhum chamador muda); `SceneCtx`/`SceneSetup` vão com elas. Cena nova desta linha
nasce lá.

## Onda 1 — quem ouve o sinal
✅ feita — commits `007650926b`, `d00414503a`, `0f66f436f8`, `6d37f1b32a`, `a6afdcd621` ([handoff](../handoffs/HANDOFF_CONTINUACAO_GEF02_F3_ONDAS_0_1_2026-11-09.md)).
- Crate pura `ph2d-signal-census`: `census(&World, scripts) -> SignalCensus` =
  `BTreeMap<nome, {emitem, ouvem}>` + «há script com nome dinâmico» por direcção. Tabela única
  `CAMPOS: &[(componente, campo, Papel)]` das 31 caixas (25 emitem, 6 ouvem — censo de 09/11) + os
  sem caixa: HUD `UiButton.signal`, partículas `start_on/stop_on/restart_on/finished_signal`,
  `AnimWhen::Signal`, Luau `ph2d.emit` / `ph2d.connect` / `on_signal` (literais).
- Entrega: `thread_local` em `ph2d-panel-inspector/src/state.rs` (o padrão de `CURRENT_INSPECTOR_ANIM`),
  publicado UMA vez por quadro em `snapshots_inspector.rs::publish_global` (custo de shell ≤ 2 linhas).
  Medir o custo por quadro antes de pôr cache.
- Desenho: helper único `signal_hint` — sufixo cinzento (`Text3`) dentro do campo, à direita, como uma
  unidade; «nobody …» em `Warn`; vazio = sem sufixo (vazio é mudo de propósito). Script dinâmico do
  outro lado ⇒ «a script may …» em `Text3`, nunca «nobody».
- Textos (i18n, inglês): `1 listens` · `{n} listen` · `nobody listens` · `1 sends` · `{n} send` ·
  `nobody sends` · `a script may listen` · `a script may send`.
- Gates (nascem vermelhos, com mutação): cada linha de `CAMPOS` é contada (World com todos os
  componentes); cada id `INSP_*` de caixa de sinal chama `signal_hint` (censo textual com controlo
  positivo); o sufixo aparece no ecrã (hit/scene).
- Smoke: cena de física com moeda (`On Hit = coin_taken`), Signal Action a ouvir `coin_taken`, e uma
  segunda moeda com `coin_takn` → «1 listens» × «nobody listens».

## Onda 2 — medidas e camadas
Half Size → Width/Height na forma (o documento guarda o que o artista escreve; o motor recebe metade).
Layer mostra o nome da camada no campo.

## Onda 3 — corpo mais limpo
Parte «More», fechada por omissão, com os campos raros (nenhum some). Remover o corpo deixa recibo
«Removed — Undo». Aviso quando o corpo está escalado.

## Onda 4 — tecla no Trigger
Escolher a tecla no próprio cartão do Trigger; o Input Map continua a ser a mesma fonte.
