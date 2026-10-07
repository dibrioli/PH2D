# HANDOFF DE CONTINUAÇÃO — `line/MiroClone`, W2 fechada → W3 (2026-10-06)

> Para a PRÓXIMA JANELA desta linha (não é handoff de integração). Antecessor:
> [`HANDOFF_CONTINUACAO_W1_2026-10-06.md`](HANDOFF_CONTINUACAO_W1_2026-10-06.md).
> Plano: [`../02_plano.md`](../02_plano.md) — §2.3 (medidas da W2), §3 W2 (estado), §6 (recusas medidas).

## §0 — Identidade

- Branch `line/MiroClone`, worktree `Worktrees/line-MiroClone`, base `a46c4c200` (o `main` não andou).
- Commits da W2: `3e7c3564b..f4f4aafc2` (4) + o deste handoff. Diff `a779b01ed..HEAD`: 61 ficheiros.
- O 1.º smoke do dono (06/10) RECUSOU duas coisas, com capturas do Miro: *«curvas exageradas»* (a
  curva era o cotovelo suavizado do vectorial) e *«o algoritmo de evitar as formas próximas»* — ordem:
  **setas não se reajustam sozinhas; pontos de ajuste no caminho como no Miro**. Refeito em `b57451b43`.
- O 2.º smoke (06/10): *«o algoritmo da criação de pontos ficou correto»*; mas a curva *«não tão suave
  quanto no Miro»* (nova captura: uma curva por nove pontos) e *«o path liga-se à ponta da seta, não à
  base»*. Curado depois de `bd82abd15`: braço ½ nos pontos, 0,45 da distância nas pontas, a linha recua
  sobre o eixo da ponta (`pull_back`).
- O 3.º smoke (06/10): *«ainda não tem a curvatura suave e arredondada do Miro»* e *«na seta fechada a
  guia de pontos não corresponde ao desenho»*. Curado depois de `e67cf486e`: braço 0,4 MEDIDO (a olho
  tinha ficado ½, que mede pior) e a linha passou a ser a própria rota cortada (`chord_cut`).
- O 4.º smoke (06/10): *«curvatura OK»*; *«seta aberta ainda com problemas»* (o «V» virava de lado
  numa curva que dobrava à chegada). Curado depois de `13c428729`: haste recta debaixo de cada cabeça
  (`Routed::curved_with_stems`, `stems`; a chave da cache leva as hastes).
- O 5.º smoke (06/10): *«permita a seta ficar na direcção da normal da curva da forma onde se
  encaixa»* (num círculo/pill a seta chegava na horizontal). Curado depois de `1179e4d5b`: a curva
  encaixa na normal do contorno (`outline_normal`; canto vivo ⇒ a do lado).
- O 6.º smoke (06/10): *«smoke OK. As setas rectangulares ainda são bem nervosas e tremem ao serem
  ajustadas»* + *«permita deletar pontos de edição arrastando um sobre o outro»*. Sonda
  (`jitter_probe.rs`) mediu o tremer e três causas, curadas depois de `0aad8e5eb`: o A\* ordenava pelo
  custo cru (empates decididos pelo arredondamento — `f_key` no `ph2d-vec-connect`, o Vector ganha o
  mesmo), o recuo de 40 num vão curto (`facing_jetty`), e o trecho a contornar a forma da outra ponta.
  E largar um ponto sobre o vizinho (ou uma ponta) funde-o (`bend_up`).
- ✅ **Smoke da W2 APROVADO pelo dono (06/10, 7.ª rodada)** — passos no §5.

## §1 — As peças

| crate | o quê |
|---|---|
| `ph2d-vec-connect` | as leis ANTES do A\* mudaram-se para cá (`ends.rs`): `exit_point` (lado + ponto no contorno por fecho + spread preso à face), `port_side`, `bbox_exit`, `obstacles_in_play` (a lei ORIGINAL, só o Vector a usa), `ROI_PAD_K`. O `ph2d-app-vec::connector_live` usa-as sem mudar comportamento (27 testes de conector verdes) |
| `ph2d-board-model` | `ElementKind::Connector(Box<Connector>)` (no FIM do enum ⇒ `FORMAT_VERSION` continua 2 para o `main`; ⚠️ ficheiros gravados pelo binário desta linha entre `3e7c3564b` e `b57451b43` com setas não se leem — o campo `waypoints` entrou depois e a linha nunca integrou): `End::{Free, Bound}`, `Anchor::{Center, Fixed([u,v])}`, `Route::{Straight, Elbow, Curved (nascença)}`, `Head` (8), estilo, rótulo, `waypoints`. `Element::{connector, style, style_mut, translate}` (pontas soltas e pontos andam). `BoardDoc::rev()` (revisão da SESSÃO, contador global, fora do ficheiro e do `PartialEq`), `BoardDoc::live()` |
| `ph2d-board-geom` | `ray_exit` (maior t sobre a curva exacta), `nearest_on_outline`, `inside` |
| `ph2d-board-route` (nova) | `JETTY = 40` e `HEAD_SCALE = 2,94` medidos no oráculo; ⭐ `END_ARM = 0,45` da distância MEDIDO na captura do Miro do dono (cúbica ajustada aos píxeis: braço 154 para 342,6, erro 2,9 px) e `POINT_ARM = 0,4` de cada trecho MEDIDO na captura com pontos (régua `ferramentas/mede_curva_miro.py` sobre `ferramentas/capturas_miro/`); `chord_cut` (a linha é a rota cortada à distância da cabeça, e a cabeça aponta para o corte) e a HASTE recta debaixo de cada cabeça numa curva (`curved_with_stems`); `anchor_for` (miolo = centro, faixa = ponto fixo colado ao meio do lado se ele está NO contorno); `compute` (estações = pontas + pontos; curva / rectas / cotovelo por trechos com só as DUAS formas por obstáculo); `RouteCache` por diferença (revisão O(1) → passeio das versões → índice espacial para `bind_at`/`shape_at` → revê só a seta mudada ou com forma de ponta mudada); `Routed { stations, path, sides, bbox, mid, leg_mids }` + `from_points`, `polyline`, `waypoints`; `drawn` / `drawn_at` |
| `ph2d-board-edit` | `Tool::Connector`; `wire.rs`: criar arrastando (alvo realçado, `Ctrl` solta), `WireHandle::{End, Point, Mid}` — arrastar a ponta religa, o meio de um trecho cria um ponto (toque sem arrastar não cria), o ponto arrasta-se, duplo-clique apaga (`double_click` recebe agora o `History`); pontos azuis (`dots`, `dot_at`, `hover` pelo índice), `grow` (forma seguinte já ligada, UM passo), `release_ends`, `detach_outside` + `remap`. `Command::Grow`, `set_route`, `set_head`, rótulo pelo `text.rs` generalizado. `Metrics::{bind, dot}`, `NEXT_GAP = 80` |
| `ph2d-board-render` | seta (tracejado, pontas cheias pintadas / vazadas traçadas, pontas e dobras redondas), rótulo sobre recorte do fundo, nível de detalhe (< 4 px = traço); overlay: realce do alvo + ponto fixo, linha da seta seleccionada, pontas e pontos (círculos ocos do tamanho do alcance), meios (bolinhas cheias), pontos azuis |
| `ph2d-editor-core` | botão Seta (`A`/`5`); barra de estilo por TIPO de selecção (`Selected::{Shapes, Arrows, Both}`); `board_bar_look.rs` (filho: balões, selecção, ícones — a seta do ícone é `drawn_at` com a ponta de nascença do catálogo); `Ctrl+setas`; o passear alimenta o `hover` |
| `ph2d-app-board` | `PH2D_BOARD_SMOKE=3` (fotografada depois da refeitura) |

## §2 — Prova à saída

- Gate batched sobre o diff (base `a46c4c200`), re-corrido depois da W2 do Miro: `nextest-impacted` **17 074/17 074**; clippy workspace
  `--all-targets --features ph2d-spike/bevy_ecs -D warnings`; `check --workspace --all-targets`
  warnings=deny; fmt; machete; standalone-optional; workflow-packages; censos-da-árvore-combinada (12);
  `typos` — verdes. A 1.ª corrida deu 4 vermelhos + `typos` (LOC do `board_bar.rs` 813/700, glifo `→`
  sem fonte num teste, `large_enum_variant` no `BoardOp`, variável órfã; os ids aleatórios das saídas do
  oráculo), curados em `afeee0d26` e `f4f4aafc2`.
- Prova de mutação, 1.ª versão (com desvio): **15/15** depois de dois buracos tapados (M6 guarda
  redundante, saiu; M4 teste com faixa que não chegava ao meio vazio). Versão do Miro (`b57451b43`):
  **12/12** depois de quatro buracos tapados — N6 (a cache revia TODAS as setas sem nenhum teste o ver:
  faltava afirmar `revisited()`), N7 (mudar só os pontos de uma seta), N9 (dobrar o 2.º trecho de uma
  seta que já tem um ponto: `insert` × `push`), N11 (`translate` leva os pontos).
- Oráculo (`ph2d-board-route::oracle_tests`, 4 gates): `fixedPoint [1,½]/[0,½]` e o lado; o Z a meio do
  vão (x = 340); a volta de 6 pontos com recuo 40 nas DUAS escalas (`seta_cotovelo_volta{,_grande}`,
  novas); a caixa no meio IGNORADA como no oráculo (ordem do dono).
- Réguas (plano §2.3): a 10 mil formas + mil setas curvas, arrastar = 0,15 ms de cache + 1,2 ms de
  encode + 0,56 ms de placa; 1.ª sincronização de 10 mil setas 43 ms (era 207 com o desvio).
- Fotos da cena 3 apanharam 2 defeitos (spread a contar setas de ponto fixo — tirava a seta do vértice
  do losango, agora gate `a_fixed_point_arrow_does_not_spread_its_centered_neighbour`; pontos azuis
  pequenos demais — agora do tamanho do alcance do clique).

- `target/*/incremental` reclamado (26 GB). Binário `smoke` quente, 2.ª corrida depois do reclamo:
  `Finished \`smoke\` profile [optimized] target(s) in 0.29s`, zero `Compiling` (refeito depois da
  W2 do Miro, `5bfb0d455`). Foto final da cena 3 refeita conferida.
- `agent-loop-profile.sh` (20 sessões): paralelismo 1,11 ✗ · `test:check` 3,1× ✗ · edições pela
  ferramenta `Edit` **33 %** ✗ — esta janela editou muito por `python3` com `assert` de contagem (falha
  alto, mas é a lei do §2 que diz `Edit`); a próxima janela: `Edit` primeiro.

- Gate re-corrido depois das curas do tremer (HEAD `0b817658b`): clippy, check, fmt, typos, censos,
  machete, standalone-optional, workflow-packages verdes; `nextest-impacted` 17 080/17 081 — o vermelho
  era o controlo do gate do VECTOR `label_live::a_label_on_a_connector_sits_at_the_arclength_middle_of_the_route`:
  a montagem dele dava um Z cuja dobra encostava ao jetty pelo MESMO empate mal decidido que fazia o
  quadro tremer; com o desempate a funcionar o Z ficou simétrico e o controlo «arco ≠ contagem» deixou
  de morder. Montagem refeita num L verdadeiro (caixa alta + caixa larga) — 12/12 do `label_live`.
  ⚠️ O integrador: o `f_key` muda rotas do VECTOR também (para o centro do vão, que é a lei documentada).

## §3 — ⏳ O que fica aberto (por ordem)

1. ~~Smoke do dono~~ ✅ aprovado 06/10.
2. **W3 — notas adesivas** (plano §3).
3. Da W2, não pedidos: o cotovelo ainda não tem a pega «arrastar o SEGMENTO» do Miro (os pontos de ajuste
   valem nele, mas são pontos); o rótulo não se arrasta ao longo da linha; as 3 pontas de UML existem no
   documento mas não na barra; o braço da curva foi medido numa captura só (um afastamento) — se o
   dono achar a curva de duas caixas muito juntas ou muito longe diferente do Miro, medir outra captura.
4. Herdados: transbordo das abas; desfazer das operações de ABA; cursor sobre as pegas; copiar para fora
   do app; trackpad; menu do botão direito numa forma.

## §4 — O que custou e não se repete

- ⛔ Escolher a olho uma lei que se pode medir custou um smoke: a captura com pontos JÁ estava em
  ficheiro (`images/3`) quando escolhi ½ olhando para outra; medida, deu 0,4.
- ⛔ Um teste que calcula o esperado com a PRÓPRIA constante confirma-se a si mesmo: o do braço da
  curva sobreviveu à mutação 0,45 → 0,5 até passar a afirmar o número MEDIDO (154).
- ⛔ O `trim_path` do vectorial recua pela poligonal das âncoras — serve polilinhas, não uma cúbica
  única (entortava a curva inteira). E uma lei medida numa só captura pode ter gémeas: a de ½ «ao longo
  da saída» e a de 0,45 «da distância» cabem as duas na 1.ª captura; só a forma da volta as separou.
- ⛔⛔ **Reusar o roteador do vectorial «porque já é melhor que o oráculo» não perguntou ao DONO se ele
  queria desvio.** O plano dizia «terminar de o fazer brilhar»; o dono, ao ver, recusou o desvio e a
  curva-de-cotovelo com capturas do Miro. O oráculo (Excalidraw) media a forma; o PRODUTO de referência
  era o Miro — observar o produto do dono antes de herdar uma lei que ele não pediu.

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

## §5 — Smoke do dono (W2, refeita)

`cd ~/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=3 ./target/smoke/ph2d-host-desktop`

1. Abre o **Board 1** com o fluxograma ligado por **setas curvas**: Start → Collect ideas → Good idea? →
   (yes) → Build it → Done; um «no» que volta numa curva por baixo; um «skip» em arco por cima; e, mais
   abaixo, a nota verde ligada a **Later** por uma seta **já seleccionada**: tem um **círculo oco** (um
   ponto de ajuste) e **bolinhas cheias** a meio de cada trecho.
2. **Dobrar:** arraste a bolinha cheia de um trecho — nasce um ponto novo e a seta curva-se por ele.
   Arraste o círculo oco para outro sítio — a seta segue. Duplo-clique no círculo oco — o ponto some.
3. **Não se rearruma sozinha:** arraste a nota para cima, para o caminho do «skip» — a seta do skip
   **não** se mexe. Mova **Collect ideas** — só as setas presas a ela acompanham.
4. **Pontos azuis:** clique em **Done**, depois no ponto azul à direita — nasce uma caixa igual já
   ligada; `Ctrl+→` cria a seguinte.
5. **Desenhar:** carregue em **A** e arraste de uma caixa a outra — a caixa de chegada acende; com
   `Ctrl` a ponta fica solta.
6. **Estilo:** com uma seta seleccionada, a barra de cima troca **reta / cotovelo / curva**, as pontas,
   a cor e o tracejado; `Enter` escreve um rótulo.
7. **Errado se:** a curva fizer ondas ou voltas que não pediu, uma seta se mexer por causa de uma forma
   que não é dela, o ponto não aparecer ao arrastar a bolinha, ou `Ctrl+Z` não desfizer o ponto.
