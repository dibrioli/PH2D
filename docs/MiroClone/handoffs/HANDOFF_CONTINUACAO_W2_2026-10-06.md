# HANDOFF DE CONTINUAÇÃO — `line/MiroClone`, W2 fechada → W3 (2026-10-06)

> Para a PRÓXIMA JANELA desta linha (não é handoff de integração). Antecessor:
> [`HANDOFF_CONTINUACAO_W1_2026-10-06.md`](HANDOFF_CONTINUACAO_W1_2026-10-06.md).
> Plano: [`../02_plano.md`](../02_plano.md) — §2.3 (medidas da W2), §3 W2 (estado), §6 (recusas medidas).

## §0 — Identidade

- Branch `line/MiroClone`, worktree `Worktrees/line-MiroClone`, base `a46c4c200` (o `main` não andou).
- Commits da W2: `3e7c3564b..f4f4aafc2` (4) + o deste handoff. Diff `a779b01ed..HEAD`: 61 ficheiros.
- ⏳ **Smoke da W2 por fazer pelo dono** — passos no §5.

## §1 — As peças

| crate | o quê |
|---|---|
| `ph2d-vec-connect` | ⭐ as leis ANTES do A\* mudaram-se para cá (`ends.rs`): `exit_point` (lado + ponto no contorno por fecho + spread preso à face), `port_side`, `bbox_exit`, `obstacles_in_play` e `obstacles_in_play_near<K>(near, a, b, pad, limit)` (chave genérica + TECTO opcional), `ROI_PAD_K`. O `ph2d-app-vec::connector_live` usa-as (sem mudar comportamento: 27 testes de conector verdes) |
| `ph2d-board-model` | `ElementKind::Connector(Box<Connector>)` (no FIM do enum ⇒ `FORMAT_VERSION` continua 2; em caixa pelo `large_enum_variant`): `End::{Free, Bound{target, anchor}}`, `Anchor::{Center, Fixed([u,v])}` (o `fixedPoint` do Excalidraw), `Route::{Straight, Elbow (nascença), Curved}`, `Head` (8), `Connector::DEFAULT_HEADS` = nada/«V» (oráculo), estilo + rótulo. `Element::{connector, style, style_mut, translate}` (seta: só as pontas soltas andam). `BoardDoc::rev()` — revisão da SESSÃO (contador global, `#[serde(skip)]`, `PartialEq` sempre igual), muda só quando uma op muda algo. `BoardDoc::live()` (por id, sem ordenar) |
| `ph2d-board-geom` | `ray_exit` (o cruzamento de MAIOR t sobre a curva exacta, a lei do `boundary_hit`), `nearest_on_outline`, `inside` |
| `ph2d-board-route` (nova) | `JETTY = 40` e `HEAD_SCALE = 2,94` MEDIDOS no oráculo; `anchor_for` (a regra da ligação: miolo = centro, faixa `band` junto ao contorno = ponto fixo, colado ao meio do lado se ele está NO contorno); `RouteCache` por diferença (revisão O(1) → passeio das versões → índice espacial por id → só revê setas cuja região toca uma forma mudada → A\* só se a CHAVE geométrica mudou), `DETOUR_K` (tecto da região), `bind_at`, `shape_at`; `Routed` (polilinha, `VecPath`, lados, caixa, meio) + `from_points`; `drawn` (linha recuada + pontas do `Marker` do vectorial); `LABEL_WRAP`, `label_origin` |
| `ph2d-board-edit` | `Tool::Connector`; `wire.rs`: criar arrastando (alvo realçado, `Ctrl` solta, não se prende de volta à origem), arrastar a ponta de uma seta seleccionada, pontos azuis (`dots`, `dot_at`, `hover`), `grow` (forma seguinte já ligada, salta sítio ocupado, UM passo), `release_ends` (apagar solta as pontas onde estão, no mesmo passo), `detach_outside` + `remap` (copiar/duplicar/Alt-arrastar religam as cópias; seta sozinha solta-se). `Command::Grow`, `set_route`, `set_head`, rótulo pelo `text.rs` generalizado. `Metrics::{bind, dot}`, `NEXT_GAP = 80` |
| `ph2d-board-render` | seta (tracejado, pontas cheias pintadas / vazadas traçadas, pontas e dobras redondas), rótulo sobre recorte do fundo, nível de detalhe (< 4 px = traço); overlay: realce do alvo + ponto fixo, linha e pegas da seta seleccionada, pontos azuis. `paint(…, routes)` |
| `ph2d-editor-core` | botão Seta (`A`/`5`) na barra curta; barra de estilo por TIPO de selecção (`Selected::{Shapes, Arrows, Both}`: na seta, cor, espessura, traço, rota, 5 pontas de início e de fim, opacidade, letra); `board_bar_look.rs` (módulo filho: balões, selecção, ícones — a seta do ícone é `drawn`); `Ctrl+setas`; o `pointer_move` sem botão alimenta o `hover` (e devolve `false`) |
| `ph2d-app-board` | `PH2D_BOARD_SMOKE=3` (fotografada) |

## §2 — Prova à saída

- Gate batched sobre o diff (base `a46c4c200`): `nextest-impacted` **17 068/17 068**; clippy workspace
  `--all-targets --features ph2d-spike/bevy_ecs -D warnings`; `check --workspace --all-targets`
  warnings=deny; fmt; machete; standalone-optional; workflow-packages; censos-da-árvore-combinada (12);
  `typos` — verdes. A 1.ª corrida deu 4 vermelhos + `typos` (LOC do `board_bar.rs` 813/700, glifo `→`
  sem fonte num teste, `large_enum_variant` no `BoardOp`, variável órfã; os ids aleatórios das saídas do
  oráculo), curados em `afeee0d26` e `f4f4aafc2`.
- Prova de mutação **15/15** (leis das setas). A 1.ª corrida deu 13/15: **M6** (`targets ∈ changed`) era
  guarda REDUNDANTE com a região (a caixa da forma de uma ponta está sempre dentro dela) e saiu; **M4**
  (o meio de lado tem de estar NO contorno) sobrevivia porque o teste usava uma faixa que nunca chegava
  ao meio vazio do triângulo — o teste passou a faixa 40 e sangra.
- Oráculo (`ph2d-board-route::oracle_tests`, 4 gates): `fixedPoint [1,½]/[0,½]` e o lado; o Z a meio do
  vão (x = 340); a volta de 6 pontos com recuo 40 nas DUAS escalas (`seta_cotovelo_volta{,_grande}`,
  novas); nenhum segmento dentro da caixa que o oráculo atravessa.
- Réguas (plano §2.3): a 10 mil formas + mil setas, arrastar = 0,19 ms de cache + 1,2 ms de encode + 0,55
  ms de placa.
- Fotos da cena 3 apanharam 2 defeitos (spread a contar setas de ponto fixo — tirava a seta do vértice
  do losango, agora gate `a_fixed_point_arrow_does_not_spread_its_centered_neighbour`; pontos azuis
  pequenos demais — agora do tamanho do alcance do clique).

## §3 — ⏳ O que fica aberto (por ordem)

1. **Smoke do dono** da W2 (§5).
2. **W3 — notas adesivas** (plano §3).
3. Da W2, não pedidos: as 3 pontas de UML (losango, losango vazado, círculo vazado) existem no documento
   mas não na barra (estreita); a seta não tem pontos intermédios (waypoints — o vectorial tem
   `stations`); o rótulo não se arrasta ao longo da linha (vive no meio); não há seta solta de dois
   cliques (nasce de um arrasto); a 1.ª sincronização de um quadro com 10 mil setas custa 207 ms (uma vez).
4. Herdados: transbordo das abas; desfazer das operações de ABA; cursor sobre as pegas; copiar para fora
   do app; trackpad; menu do botão direito numa forma.

## §4 — O que custou e não se repete

- ⛔⛔ **A lei do vectorial para os obstáculos (ponto fixo SEM tecto) engole um quadro denso**: o vão de
  nascença (80) é menor que a folga dupla (2 × `ROI_PAD_K × JETTY` = 240), e cada seta via TODAS as
  formas (1 000/1 000, 10 000/10 000; a régua a 100 mil ficou 12 min sem acabar). O tecto (`DETOUR_K`)
  é parâmetro da MESMA função — o vectorial continua sem ele.
- ⛔ Uma régua que pára sem dar número é um achado: medir o que ela conta (obstáculos por rota) antes
  de cronometrar explicou o porquê numa linha.
- ⛔ O `0.5001` do oráculo é desempate, não lei (memória «empate por arredondamento»).
- ⛔ O afastamento de paralelas só para setas presas ao CENTRO nas duas pontas (a foto o mostrou).
- ⛔ A regra de que o ícone é a MESMA geometria do quadro vale para a seta: `Routed::from_points` + `drawn`.
- `screens/hero.rs` continua a **699/700**; `board_bar.rs` 607 (o filho `board_bar_look.rs` 226).
- As saídas do oráculo estão isentas do `typos` (`.typos.toml`): são bytes de outro programa.

## §5 — Smoke do dono (W2)

`cd ~/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=3 ./target/smoke/ph2d-host-desktop`

1. Abre o **Board 1** com o fluxograma **ligado por setas**: Start → Collect ideas → Good idea? → (yes)
   → Build it → Done; um «no» que volta de baixo do losango para a 2.ª caixa; um «skip» que passa POR
   CIMA das duas caixas do meio; e, por baixo, a nota verde «Notes — move me into an arrow's way».
   A caixa **Done** já está seleccionada, com **quatro pontos azuis** à volta.
2. **Mover:** arraste **Collect ideas** para baixo — as setas presas a ela vão atrás e mudam de lado.
3. **Desviar:** arraste a **nota** para cima, para o meio de uma seta — a seta contorna-a.
4. **Pontos azuis:** clique no ponto azul à **direita** de **Done** — nasce uma caixa igual, já ligada.
   Com ela seleccionada, `Ctrl+→` (ou `Ctrl+↓`) cria a seguinte.
5. **Desenhar uma seta:** carregue em **A** (ou no 3.º botão da barra à esquerda) e arraste de uma
   caixa até outra — a caixa de chegada **acende** antes de largar. Com `Ctrl` a ponta fica solta.
6. **Estilo da seta:** clique numa seta — na barra de cima escolha **reta / cotovelo / curva**, a
   ponta do início e a do fim, a cor e o tracejado. `Enter` escreve um rótulo nela.
7. **Religar:** com a seta seleccionada, arraste a bolinha da ponta para outra caixa.
8. **Apagar:** apague uma caixa (`Delete`) — a seta fica onde estava, solta; `Ctrl+Z` devolve as duas.
9. **Errado se:** uma seta atravessar uma caixa no caminho, uma seta não seguir a caixa que se move, a
   caixa de chegada não acender ao arrastar, o ponto azul não criar a caixa ligada, ou `Ctrl+Z` não
   desfizer de uma vez a caixa e a seta que o ponto azul criou.
