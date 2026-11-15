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

## Superfície de colisão

- `ph2d_anim::Key` ganhou um campo: todo literal `Key { t, value, interp }` fora desta árvore deixa de compilar — a cura é
  `Key::new(..)` ou `retime: None`. Idem `KeyView` (`ph2d-timeline`) — o censo de literais desta árvore está todo feito.
- `ph2d_timeline::TimelineClipboard::set_from_absolute` mudou de assinatura (`&[(AnimTarget, Key)]`).
- `TimelineIntent` ganhou a variante `Retime` (um `match` exaustivo fora da timeline reprova).
- `crates/ph2d-panel-timeline/src/{graph,graph_paint,event,populate,state,lib,ids,linhas_maquete,legenda}.rs` (aditivo/pequeno) ·
  `crates/ph2d-i18n/src/timeline.rs` (+7 chaves) · `shells/desktop/src/project_schema{,_tests}.rs` (degrau).
- Tectos: `track.rs` 696/700, `intent.rs` 700/700, `intent_apply.rs` ≤ 700 — os verbos novos foram para ficheiros filhos.

## Aberto

- ⭐ **Dono 15/11: «Override OK. Retime precisa ser pesquisado em quem faz isso corretamente.»** A próxima janela
  começa por [19 — pesquisa do Retime](../19_pesquisa_retime.md). O Retime atual fica no código (funciona, provas
  verdes), mas o dono NÃO o aprovou como gesto.
  - **Pesquisa FEITA (15/11):** [19 §5](../19_pesquisa_retime.md#5-resultado-1511--medido-fixtures-em-oraculosretime)
    (Blender, Godot e Natron corridos por script; AE/Premiere/Resolve pela documentação), [folha lado a lado](../fotos/retime_8_pesquisa_lado_a_lado.png).
    Recomendação: em Retime editar a curva do TEMPO numa caixa (alça presa à borda sob o rato), velocidade só para ler;
    alternativas B/C/D no §5.
  - **Dono 15/11: «vamos tentar A» — FEITO** (ver «Retime = caixa do tempo» abaixo). Aguarda o smoke do dono.

- Smoke do dono depois da cura da auditoria (o arrasto além do tecto).
- Na vista de VALOR as alças continuam as do easing (sobre um trecho retimado não ficam em cima da curva desenhada).
- O «Reset timing» mora ao lado da pílula, não no menu da chave (o plano dizia «menu da chave/linha»); se o dono o quiser
  também lá, é uma linha no cartão da chave.
- Do handoff anterior: smoke do Auto Key na sessão e dos acréscimos 13–14; o campo do Inspector digitado (não medido).
