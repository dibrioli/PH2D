# Integração — `line/TimelinePrincipal`: Retime por trecho (dono 15/11, plano 18)

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · merge-base `7b3dd983f7` (sem rebase).
- A ponta leva TAMBÉM os três commits do [handoff anterior](HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-15_AUTOKEY_NA_SESSAO.md)
  (Auto Key na sessão, marcas dos menus, escala da velocidade) — ainda não integrados. Commit deste: `1fde042af5`.
- Sem push, sem integração por esta linha.

## Pedido (verbatim, plano 18 §1)

> «A curva de velocidade na verdade reconstrói toda a animação. Mas se houvesse a opção de ajustar a velocidade sem
> destruir o easing […] se o usuário escolhe Override então funciona exatamente como agora. Mas se selecionar additive
> temos duas curvas sobrepostas de cores diferentes […] e uma segunda curva de velocidade que não destruiria o easing.»

Nome proposto e usado: **Retime** (somar velocidade faria falhar o valor da chave seguinte). ⚠️ Confirmar o nome no smoke.

## Medição primeiro (plano 18 §2)

O Time Remap do OBJECTO **não** exprime o pedido: o relógio é da entidade inteira, por isso o mesmo reajuste que retima o
Bounce de X arrasta também a recta de Y. Gate: `ph2d-timeline retime::tests::o_time_remap_do_objeto_retima_todas_as_propriedades_e_o_retime_so_o_trecho`
(com o relógio, Y muda; com o Retime da chave de X, Y fica ao bit e X muda).

## Lei e desenho

- `ph2d_anim::Retime { x1, y1, x2, y2: f32 }` (16 B, `crates/ph2d-anim/src/retime.rs`): `W` = Bézier CSS de (0,0) a (1,1),
  as quatro coordenadas presas em `[0,1]` ⇒ **monótona** (prova no doc-comment; gate `o_ritmo_e_monotono_em_todo_o_quadrado`).
- `Key.retime: Option<Retime>` no FIM da chave (não na `Interp`, que é `Copy` e está no tecto de 216 B). `Key::new(t, v, i)`
  é o literal sem ritmo. O trecho toca `lerp(v0, v1, E(W(u)))` no ÚNICO ponto `Track::segment_value`; `None` ⇒ `u` cru,
  **ao bit** como antes (gate `sem_ritmo_cada_familia_amostra_ao_bit_como_antes`, 10 famílias incl. BezierW e Spline).
- Compõem: `track_reverse` leva o ritmo com o easing (`W_rev(u) = 1 − W(1 − u)`); `extrap` Continue usa `E'·W'`;
  `duplicate_keys` e Paste levam o ritmo (`Track::upsert_whole`, `ClipboardKey.retime`; `set_from_absolute` recebe
  `(AnimTarget, Key)`); `graph::sample_keys`, `speed::sample_speed`/`segment_endpoint_speed` e a barra da lane
  (`linhas_maquete::pontos_da_curva`) aplicam `W`.
- Comando: `TimelineIntent::Retime(RetimeIntent::{Set{target,key,retime}, ResetSelected})` (`crates/ph2d-timeline/src/retime.rs`),
  um passo de desfazer; o arrasto vai dentro do `BeginEdit/EndEdit` de sempre.
- Vista de velocidade (painel, `graph_ritmo.rs`): estado de SESSÃO `TimelinePanelState.speed_retime`; pílula
  **Override · Retime** na coluna à esquerda da banda e, em Retime, **Reset timing** por baixo (repõe as seleccionadas).
  - **Override** (como hoje): curva no acento, alças do easing. Num trecho JÁ retimado a ponta da alça fica na
    velocidade que TOCA (`E'·W'`) e o arrasto escreve o easing dividido por `W'(ponta)` — o ritmo fica.
  - **Retime**: a curva do movimento na cor da curva (azul) e a do ritmo `ref · W'(u)` em **âmbar** (`ColorToken::Warn`)
    com as alças AE dela (comprimento = influência, altura = velocidade); `ref = (v1−v0)/span`, sem ritmo = recta
    horizontal na velocidade média. Trecho parado (`v0 == v1`) não mostra alças de ritmo. Puxar abaixo de zero pára,
    nunca anda para trás.
- O âmbar saiu da foto: acento e cor da curva são o mesmo azul neste tema — duas curvas azuis não se distinguiam.

## Formato

`PROJECT_SCHEMA` **242 → 243** (`Key.retime`), degrau escrito em `project_schema.rs` e na tripla
(`project_schema_tests.rs`); ⚠️ o integrador RECONTA pelo `main` (`python3 scripts/schema-recount.py`). Sem leitura dos
anteriores (§0.11). `DOC_VERSION` da timeline intocado (o mesmo que o degrau 238 da `Interp::Spline` fez).

## Provas (por nome)

- `ph2d-anim retime` (5): ao bit sem ritmo · Bounce retimado mantém as chaves e o número de quiques · monótona no
  quadrado · inverter espelha · Continue com `W'`. Regressão: `golden` 14 · `reverse` 12 · `extrapolation` 11.
- `ph2d-timeline retime` (6): a medição do Time Remap · velocidade desenhada = derivada numérica do que toca (Bounce
  retimado) · alça do ritmo ⇄ ponta inversas · nunca para trás · comando grava/desfaz/repõe · copiar/colar/duplicar levam.
  Regressão: `speed::` 11 · `graph::` 14 · `intent` 69 · `clipboard` 4.
- `ph2d-panel-timeline graph` (30, 3 novos): em Retime só ritmo (nenhum `SetInterp`) num passo · em Override o ritmo fica e
  a ponta toca a velocidade pedida · as células mudam o modo e o Reset repõe. `the_bar_between_two_keys…` verde.
- `ph2d-host-desktop project_schema` verde. `cargo check --workspace --all-targets` verde. Clippy `-D warnings` das 4 crates.
- **Mutação 5/5 sangram:** `W` no `Track::segment_value` · `W` no `sample_keys` · `W'` na velocidade · `W` no reverse ·
  ritmo no clipboard. ⚠️ Lição: restaurar o fonte com `shutil.move` do `.bak` devolve um mtime ANTIGO e o cargo reutiliza
  o binário MUTADO (o teste do reverse ficou vermelho depois de restaurado) — `touch` nos restaurados antes de voltar a testar.
- **Fotos (Xvfb, `PH2D_ANIMACAO_SMOKE=1`, cliques reais):** [Bounce em Retime](../fotos/retime_1_bounce_em_retime.png) ·
  [alças arrastadas](../fotos/retime_2_alcas_arrastadas.png) · [curva de valor: os 4 quiques ficam](../fotos/retime_3_curva_de_valor.png) ·
  [Reset timing](../fotos/retime_4_reset_timing.png) · [Override](../fotos/retime_5_override.png).

## Auditoria do dono (15/11) — «o arrastar em Retime treme e dá pulos com easing complexo»

- **Respostas:** «Retime» SIM · azul × âmbar SIM · esconder as alças do Bounce em Override NÃO (§4.5 fechado: fica como está).
- **Medido (sonda headless, arrasto suave de 200 quadros, Bounce e Elastic):** num arrasto dentro do tecto a ponta segue o
  rato (erro 0) e o pico desenhado a 2 px salta o mesmo que o verdadeiro (4 × 2,8 /s Bounce; 26 × 24 /s Elastic) — a
  amostragem NÃO é a causa. Puxando a alça para a direita E para cima, `y1 = s·x1` passava de 1, o `Retime::new` prendia-o
  e a ponta ficava **até 254 /s longe do rato** (de novo ao soltar o tecto: o pulo).
- **Cura** (`retime_with_speed_handle`): além do tecto honra-se a VELOCIDADE e o braço encurta para `1/s` (`y ≤ 1`, monótona).
  Gate `a_ponta_do_ritmo_segue_o_rato_mesmo_alem_do_tecto` (as duas alças, 200 quadros cada: ponta = rato, o braço só
  encurta, W monótona); mutação «sem encurtar» ⇒ vermelho. [Foto a meio do arrasto](../fotos/retime_6_alca_alem_do_tecto.png):
  a ponta na altura do rato, o braço curto.
- Fica (intrínseco, não defeito): a curva azul muda muito com o ritmo num easing complexo (`E'(W)·W'`) e, com o
  enquadramento congelado durante o arrasto, pode sair da banda até se soltar.

## Auditoria do dono, 2.ª volta (15/11) — «não parece bom e parece que piorou o Override»

- **Override:** sem ritmo era (e é) ao bit o de antes; o defeito era num trecho JÁ retimado — o arrasto reescrevia o
  easing e o ritmo velho ficava por baixo. Lei agora: **Override reconstrói** (o pedido: «funciona exatamente como
  agora»): o arrasto de velocidade em Override tira o ritmo (`RetimeIntent::Set{None}` no mesmo passo) e o easing novo
  guarda na OUTRA ponta a velocidade que tocava (`E'·W'`), sem salto. Gate
  `em_override_o_arrasto_tira_o_ritmo_e_a_outra_ponta_nao_salta`. A `end_rate` (dividir pela taxa do ritmo) saiu.
- **Retime, filmado quadro a quadro:** a cura da 1.ª volta (braço = `1/s`) fazia a ponta correr para a ESQUERDA com o
  rato a ir para a direita e para cima; e a escala congelada cortava o pico da curva azul. Curas:
  - a ponta vai ao ponto ALCANÇÁVEL (`s·a ≤ 1`) **mais perto do rato em pixels** (`retime::alcancavel`: varredura em
    log + secção áurea; `retime_with_speed_handle` recebe `px` da vista). Gate
    `a_ponta_do_ritmo_e_o_ponto_alcancavel_mais_perto_do_rato` (contra busca exaustiva de 20 000 pontos, ponta contínua
    ≤ 2× o passo do rato, W monótona); mutação «y preso» e «braço = 1/s» ⇒ vermelho as duas.
  - em Retime a escala congelada do arrasto só CRESCE (`graph::band_for`): a curva cabe, nada reescala para trás.
- [Tira do arrasto, quadro a quadro](../fotos/retime_7_arrasto_quadro_a_quadro.png) (círculo vermelho = rato).

## Retime = caixa do tempo (dono 15/11, «vamos tentar A» — pesquisa 19 §5)

- **Lei do gesto:** em Retime a alça é o ponto `(x1, y1)`/`(x2, y2)` de `W` dentro de uma caixa do tamanho do trecho
  (`graph_ritmo::caixa`: de chave a chave, a altura da banda menos `Spacing::Sm` em cima e em baixo, presa ao RECTÂNGULO
  da banda, nunca à escala da velocidade). Além da borda, o ponto da caixa mais perto do rato (por eixo) —
  `ph2d_timeline::retime_with_time_handle`, função só do ponteiro. Formato e lei `E(W(u))` intactos; Override intacto.
- **Desenho:** a velocidade do movimento na cor da curva (só lê); por cima, a caixa tracejada (`Border`), `W` em âmbar
  (`Warn`) de canto a canto e as duas alças (braço do canto à ponta, ponta âmbar, `CurveHandle` de sempre). A curva
  âmbar de velocidade `ref·W'` saiu (duas âmbar confundiam); a banda enquadra só a velocidade do movimento.
- **Saiu:** `retime_with_speed_handle`, `alcancavel`, `rhythm_handle_tip`, `segment_rhythm`, `graph_ritmo::{extent,
  paint_rhythm, curva_e_alcas}` e os três testes deles (o modelo foi recusado na pesquisa 19).
- **Provas por nome:** `ph2d-timeline retime::tests` 7 ✓ — novos `a_alca_do_tempo_e_a_ponta_dela_sao_inversas`,
  `alem_da_borda_a_alca_fica_na_borda_e_o_tempo_nunca_volta`, `o_mesmo_ponto_do_rato_da_sempre_o_mesmo_ritmo` (3
  caminhos, por dentro/por fora/zigue-zague, dão o mesmo ritmo ao bit; a ponta nunca anda mais do que o rato) ·
  `ph2d-panel-timeline graph_ritmo` 4 ✓ — novos `em_retime_a_alca_e_o_ponto_da_caixa_num_passo`,
  `alem_da_caixa_a_alca_fica_na_borda_debaixo_do_rato` (duas escalas de velocidade, a mesma alça).
- **Mutações:** `W` virado no arrasto (`(y − c.y)/c.h`) ⇒ as 2 do painel vermelhas · projecção RADIAL em vez da caixa ⇒
  as 2 da lei vermelhas · folga da caixa = 0 sobrevive (cosmética, equivalente: o teste lê a mesma `caixa`). Restaurado.
- **App (Xvfb, cliques reais, `PH2D_ANIMACAO_SMOKE=1`, roteiro [`fotos_retime_caixa.sh`](../ferramentas/fotos_retime_caixa.sh)):**
  [caixa do tempo](../fotos/retime_10_caixa_do_tempo.png) · [alça puxada acima da caixa, a meio](../fotos/retime_11_alca_presa_na_borda_a_meio.png)
  (fica na borda de cima, na coluna do rato) · [a outra para baixo e à direita, a meio](../fotos/retime_12_outra_alca_no_canto_a_meio.png)
  (fica no canto) · [depois de soltar](../fotos/retime_13_depois_de_soltar.png). A borda era `Border` e não se via na
  1.ª foto — passou a `Text3`. fmt ✓ · clippy `-D warnings --all-targets` (timeline, panel-timeline) ✓.

## Auditoria da caixa (15/11, 2 lentes independentes) — «vamos corrigir todos até os antigos»

| # | Achado (mecanismo) | Gate que faltava · por que o verde não via | Cura |
|---|---|---|---|
| 1 | Alça com a ponta fora da banda (zoom/rolagem) registava clique: `paint_caixas` não tinha o guarda horizontal que `paint_speed_handles` tem | nenhum teste pintava a caixa | `area_de_clique`: ponta fora da banda ⇒ sem clique; área cortada à banda. `a_alca_fora_da_banda_nao_deixa_clique` (ponta 3 px fora) |
| 2 | Os dois testes do painel calculavam o rato com a própria `caixa` — ida e volta pela função sob teste; `paint_caixas` nunca corria | oráculo = função sob teste | `o_clique_no_centro_da_alca_desenhada_apanha_essa_alca`: pinta no `MockPanelHost`, posição esperada por conta à mão |
| 3 | Duas alças no mesmo sítio: vencia a última registada (`hit.rs` itera ao contrário), mesma cor | — | as áreas partem-se na mediana pelo eixo de maior afastamento: vence a MAIS PERTO do rato. `com_duas_alcas_juntas_o_clique_vai_a_mais_perto` |
| 4 | Trecho com < 14 px: nem caixa nem alças, nenhuma forma de editar | — | o corte saiu (só caixa sem área). `um_trecho_estreito_ainda_tem_as_duas_alcas` |
| 5 | (antigo, onda 18) Banda < ~71 px escondia a pílula Override · Retime e o Reset; o mínimo arrastável era 64 | — | `graph::graph_h_min()` = máx(64, `graph_ritmo::altura_do_seletor()` + 2·inset) — uma função para a pílula e para o mínimo. `na_banda_mais_baixa_a_pilula_e_o_reset_continuam_clicaveis` |
| 6 | (antigo) roteiro de fotos com píxeis fixos, sem conferência: layout mudado ⇒ fotos erradas sem erro | — | o roteiro confere a janela 1600×900, âmbar ausente ANTES do Retime (controlo) e presente depois, e que cada arrasto muda a foto; senão `RECUSA` |

- **Mutações** (uma a uma, restauradas): sem o guarda da banda · sem a mediana · corte de 14 px de volta · mínimo 64 de
  volta · pintura com `W` espelhado ⇒ **as 5 vermelhas**. O guarda da banda sobreviveu à 1.ª versão do teste (ponta 17 px
  fora: o corte à banda já a matava) — o teste passou a pôr a ponta 3 px fora.
- **Descartados ao conferir:** «a vista de VALOR não mostra o ritmo» (falso: `graph::sample_keys` aplica `W`,
  `ph2d-timeline/src/graph.rs:41`) · «falta pt-BR» (o app é só inglês) · «o clique invade a linha vizinha» (`Spacing::Sm`
  6 + inset 2 ≥ raio 7) · «NaN vai ao canto» (só com evento de rato inválido) · arrasto aberto ao rolar (anterior, não medido).
- Provas: `graph_ritmo` 9 ✓ · `graph::tests` 23 ✓ · `resize` 7 ✓.

## Vista de VALOR sobre um trecho retimado (15/11, 2.ª janela — o 1.º aberto)

- **Defeito:** `segment_handle_points` dava os pontos do easing `E` sem o ritmo; a curva desenhada é `E(W(u))` ⇒ os braços
  não tocavam a curva. A vista de velocidade já mostrava o que TOCA (`speed_handle_tip` usa `E'·W'`); faltava a de valor.
- **Lei (uma para as duas vistas):** num trecho retimado o braço é a TANGENTE do que toca (`segment_endpoint_speed`), com a
  influência do easing ENCURTADA até a ponta caber nos valores que o trecho toca (`arm_inside` sobre
  `segment_value_range`, 64 amostras) — sem isso um ritmo íngreme punha a ponta fora da banda (medido na 1.ª foto: nem se
  via nem se agarrava). Chegada/saída VERTICAL (alça do tempo num canto da caixa, `W'` infinito): o braço SOME
  (`segment_handle_drawn`), como na velocidade — antes caía no ponto do easing, que mentia.
- **Gesto:** puxar o braço na vista de valor = Override, como na velocidade: `resolve_drag` tira o ritmo
  (`RetimeIntent::Set { retime: None }`) no mesmo passo de undo e grava o `BezierW` a partir dos pontos DESENHADOS (o outro
  lado fica onde estava). A `Spline` com pontos retimada também passa a `BezierW` (os pontos dela não sabiam do ritmo).
  `weighted_with_speed_handle` perdeu a sua correção própria — `segment_handle_points` já a dá.
- **Provas por nome:** `ph2d-timeline graph::` 18 ✓ — novos `num_trecho_retimado_a_alca_de_valor_e_tangente_a_curva_desenhada`
  (oráculo = diferença finita de 2.ª ordem sobre `sample_keys`, 3 famílias, os dois lados),
  `com_ritmo_ingreme_a_ponta_do_braco_fica_dentro_do_que_o_trecho_toca`, `com_a_chegada_na_vertical_aquele_braco_some_e_o_outro_fica`,
  `puxar_a_alca_de_um_trecho_retimado_deixa_a_outra_ponta_onde_estava` · `ph2d-panel-timeline graph` 37 ✓ — novo
  `na_vista_de_valor_puxar_a_alca_de_um_trecho_retimado_tira_o_ritmo` · `speed` 20 + 4 ✓ · `retime` 9 + 1 ✓.
- **Mutações (uma a uma, restauradas e conferidas com `cmp`):** bloco do ritmo desligado · sinal do lado de chegada
  trocado · `Spline` retimada mantida · tirar o ritmo só na velocidade ⇒ **as 4 vermelhas**.
- **App (Xvfb, cliques reais, [`fotos_retime_valor.sh`](../ferramentas/fotos_retime_valor.sh), com conferências):**
  [ritmo dentro da caixa](../fotos/retime_14_ritmo_dentro_da_caixa.png) · [braços tangentes](../fotos/retime_15_valor_bracos_tangentes.png) ·
  [braço puxado](../fotos/retime_16_valor_braco_puxado.png) · [antes × depois](../fotos/retime_17_valor_antes_e_depois.png)
  (esquerda: binário com o bloco desligado — braços na reta antiga; direita: a cura). ⚠️ A cena do Bounce com a alça no
  canto (`fotos_retime_caixa.sh`) dava fotos IGUAIS antes e depois: o Bounce sai parado (`E'(0)=0`) e o canto dá `W'(1)=∞` —
  a cena do valor usa Straight e alças longe dos cantos por isso.
- **Não feito, por decisão:** «Reset timing» no cartão da chave — seria um botão inerte em quase toda chave (só serve a
  trecho retimado); fica ao lado da pílula. Se o dono o pedir, é uma linha na tabela do menu + o resolvedor.

## Pontos da curva Custom no gráfico (dono 15/11, depois do smoke OK de tudo)

- **Pedido (verbatim):** «Os pontos da curva custo não aparecem no gráfica da timeline. Permita criar pontos no gráfico da
  timeline e editar sem criar keyframes, exatamente igual é feito no easing custom».
- **Lei** (`ph2d-timeline/src/graph_pontos.rs`): `segment_curve(k0,k1)` = a `EaseSpline` do trecho (Spline · Bezier ·
  BezierW normalizada · **Linear = Bézier (⅓,⅓)(⅔,⅔) exata**); `None` em trecho RETIMADO (a curva desenhada é `E(W(u))`,
  os pontos ficariam fora dela — recusado mostrá-los), parado (`Δv = 0`), sem duração e famílias sem Bézier exata.
  `unit_to_plane`/`plane_to_unit` (`t = t0 + x·Δt`, `v = v0 + y·Δv`), `interp_of_curve` (sem pontos ⇒ `Bezier`).
- **Painel** (`ph2d-panel-timeline/src/graph_pontos.rs`): na vista de VALOR, trechos de chave seleccionada. Hits novos
  `TimelineHitKind::{CurvePoint{target,key,node,part}, CurveLine{target,key}}` (editor-core). A linha regista pedaços de
  6 px com ±8 px (o `CURVA_HIT` do cartão) ANTES das âncoras/alças (o último registado ganha); alças antes dos pontos.
  Gestos = os do cartão: carregar na linha cria e agarra (`insert`, de Casteljau); arrastar ponto/alça (`move_point`/
  `move_handle`); Alt ao agarrar alça = Corner; Corner · Smooth · Symmetric (chaves i18n do Vector) na coluna da
  esquerda com um ponto escolhido; botão direito no ponto apaga. Um passo de undo por gesto, só `SetInterp` — nenhuma
  chave. A faixa da banda congela no gesto (`band_for`).
- **Duas costuras que a 1.ª volta deixou mortas (fotos do app, antes do commit):** o painel DESCARTAVA todo gesto
  Secondary (`interact.rs`, «reserved») ⇒ `dispatch_secondary`; o botão direito num `CurvePoint{part:0}` sai do
  `handle_down_menus` como Click secundário ANTES de qualquer menu; os três tipos não estavam no `populate` ⇒ pedra.
  Os testes unitários chamavam as funções direto e ficaram verdes — por isso `tests/it/curve_points_seam.rs` (painel
  pintado, despacho real, a curva que o shell recebe).
- **Provas:** timeline `graph_pontos` 3 · painel `graph_pontos` 7 + `curve_points_seam` 3 · editor-core `curve_point` 2
  (+ `types` com as variantes novas) · vizinhos por nome verdes (graph 44, graph_ritmo 9, interact 12, timeline 45,
  context_menu 31, pointer 24, key_card 7…) · `cargo check -p ph2d-host-desktop` ✓ · LOC: `apply_event` do painel voltou
  a ≤ 200 (porta única `graph::apply_event`), `intent_apply.rs` 702 → 700 (vinha do commit do Retime).
- **Mutações (8, uma a uma, `cmp` ao restaurar):** sem o braço Secondary · sem o Corner no `populate` · sem os hits da
  linha · `part: 9` no despacho · `move_point(x, x)` · toque na linha não cria · retimado com pontos · `BezierW` sem o
  `1 +` ⇒ **as 8 vermelhas** (o teste de clique do ponto não vê a 3.ª — esperado, pinta os hits direto).
- **App** ([`fotos_pontos_no_grafico.sh`](../ferramentas/fotos_pontos_no_grafico.sh), conferências por passo):
  [sequência](../fotos/pontos_6_sequencia.png) — toque cria · arrastar dobra · Corner (ponto quadrado) · só a alça
  puxada anda · direito apaga; o Summary fica com as duas chaves. Ctrl+Z não conferido no Xvfb (o teclado não chega,
  como no handoff anterior) — coberto pelos testes do passo de undo; conferir no smoke.

## O cartão da chave desenha a curva com a cara da timeline (dono 15/11)

- **Pedido:** «a mesma curva aparece invertida em y no editor de easing custom. permita que no editor de easing fique
  sempre como na timeline». Mecanismo: o editor desenha a `EaseSpline` normalizada (sempre de (0,0) a (1,1), a subir);
  a timeline desenha `v0 + y·Δv` — com `Δv < 0` a mesma curva DESCE.
- **Cura:** `BezArea.desce` (editor-core): `tela`/`unidade` viram o eixo vertical — tudo o que o editor pinta e clica
  passa por essas duas (pintura, `alvo_em`, `na_curva`, arrasto). A curva gravada não muda. A timeline publica-o com a
  curva (`cartao_bezier::publica_e_drena`: a 1.ª chave do alcance, valor da seguinte menor) por
  `WidgetStore::set_key_card_desce`. As miniaturas dos presets ficam como estão (são a forma, não esta chave).
- **Provas:** editor-core `key_card` 7 (ida e volta nos dois sentidos; com `desce` o início (0,0) fica em `(100, 90)` e o
  fim (1,1) em `(300, 170)`, conta à mão) · painel `cartao_bezier` 4 (novo `a_chave_que_desce_publica_o_cartao_virado`,
  desce e sobe). App: [antes × depois](../fotos/cartao_curva_desce_antes_e_depois.png) — chave do quadro 24 de
  Position X (o trecho desce), Custom; em cima o binário com `desce` desligado, embaixo a cura.

## Superfície de colisão

- `ph2d_anim::Key` ganhou um campo: todo literal `Key { t, value, interp }` fora desta árvore deixa de compilar — a cura é
  `Key::new(..)` ou `retime: None`. Idem `KeyView` (`ph2d-timeline`) — o censo de literais desta árvore está todo feito.
- `ph2d_timeline::TimelineClipboard::set_from_absolute` mudou de assinatura (`&[(AnimTarget, Key)]`).
- `TimelineIntent` ganhou a variante `Retime` (um `match` exaustivo fora da timeline reprova).
- `ph2d_editor_core::interaction::TimelineHitKind` ganhou `CurvePoint` e `CurveLine` (os `match` exaustivos desta árvore —
  `timeline_cursor.rs`, `interact.rs` — estão feitos; um de outra linha reprova).
- `crates/ph2d-panel-timeline/src/{graph,graph_paint,event,populate,state,lib,ids,linhas_maquete,legenda}.rs` (aditivo/pequeno) ·
  `crates/ph2d-i18n/src/timeline.rs` (+7 chaves) · `shells/desktop/src/project_schema{,_tests}.rs` (degrau).
- Tectos: `track.rs` 696/700, `intent.rs` 700/700, `intent_apply.rs` ≤ 700 — os verbos novos foram para ficheiros filhos.

## Aberto

- ⭐ **Dono 15/11 (smoke OK de tudo): «vamos resolver tudo antes de fechar»** — as três escolhas desta janela viram
  feitas, num bloco só (CLAUDE.md §0.10), ANTES de `/pd-linha-fechar`:
  1. **Pontos da Custom num trecho RETIMADO** (hoje `segment_curve` devolve `None`): desenhar cada nó e alça em
     `u = W⁻¹(x)` (o inverso da Bézier de `W` = a mesma com as coordenadas trocadas: `Retime::new(y1,x1,y2,x2).eval(x)`),
     valor `v0 + y·Δv` ⇒ o ponto fica EM CIMA de `E(W(u))`; o arrasto lê `x = W(u_rato)` e MANTÉM o ritmo; o toque na
     linha insere em `W(u)`. Prova: o ponto desenhado = `sample_keys` no tempo desenhado. ⚠️ Decidir e escrever: o braço
     da PONTA num trecho retimado com pontos hoje é Override (tira o ritmo e vira `BezierW` ⇒ APAGA os pontos) — na
     Spline retimada o braço da ponta deve editar a alça de ponta da Spline pelo mesmo mapa e manter ritmo e pontos.
  2. **«Reset timing» no cartão da chave**, só quando o trecho da chave tem ritmo (a timeline publica `retimado` com a
     curva, como o `desce`; a linha não existe sem ritmo — nada de botão morto); clique ⇒ `RetimeIntent::Set{retime:
     None}` nas chaves do alcance, um passo de undo. Gate de costura com o despacho real (como `curve_points_seam`).
  3. **Miniaturas do cartão viradas quando o valor desce** (`key_card.rs`, botões Curve/Straight/…/formas: o mesmo
     `store.key_card_desce()` que o editor já usa). Foto antes × depois.
  Depois: fotos no app de cada um (roteiros em `docs/Timeline/ferramentas/`), smoke do dono, e então fechar a linha.

- ⭐ **Dono 15/11: «Override OK. Retime precisa ser pesquisado em quem faz isso corretamente.»** A próxima janela
  começa por [19 — pesquisa do Retime](../19_pesquisa_retime.md). O Retime atual fica no código (funciona, provas
  verdes), mas o dono NÃO o aprovou como gesto.
  - **Pesquisa FEITA (15/11):** [19 §5](../19_pesquisa_retime.md#5-resultado-1511--medido-fixtures-em-oraculosretime)
    (Blender, Godot e Natron corridos por script; AE/Premiere/Resolve pela documentação), [folha lado a lado](../fotos/retime_8_pesquisa_lado_a_lado.png).
    Recomendação: em Retime editar a curva do TEMPO numa caixa (alça presa à borda sob o rato), velocidade só para ler;
    alternativas B/C/D no §5.
  - **Dono 15/11: «vamos tentar A» — FEITO** (ver «Retime = caixa do tempo» abaixo) e a auditoria das 2 lentes curada.
    **Smoke do dono 15/11: OK** («Muito bom! Parece ótimo!»; depois da auditoria: «smoke OK»).

- ~~Na vista de VALOR as alças continuam as do easing~~ — FEITO (secção «Vista de VALOR…»); smoke do dono pendente.
- O «Reset timing» fica só ao lado da pílula (decidido na 2.ª janela de 15/11, ver a mesma secção).
- Do handoff anterior: smoke do Auto Key na sessão e dos acréscimos 13–14; o campo do Inspector digitado (não medido).
