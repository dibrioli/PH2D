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

## Superfície de colisão

- `ph2d_anim::Key` ganhou um campo: todo literal `Key { t, value, interp }` fora desta árvore deixa de compilar — a cura é
  `Key::new(..)` ou `retime: None`. Idem `KeyView` (`ph2d-timeline`) — o censo de literais desta árvore está todo feito.
- `ph2d_timeline::TimelineClipboard::set_from_absolute` mudou de assinatura (`&[(AnimTarget, Key)]`).
- `TimelineIntent` ganhou a variante `Retime` (um `match` exaustivo fora da timeline reprova).
- `crates/ph2d-panel-timeline/src/{graph,graph_paint,event,populate,state,lib,ids,linhas_maquete,legenda}.rs` (aditivo/pequeno) ·
  `crates/ph2d-i18n/src/timeline.rs` (+7 chaves) · `shells/desktop/src/project_schema{,_tests}.rs` (degrau).
- Tectos: `track.rs` 696/700, `intent.rs` 700/700, `intent_apply.rs` ≤ 700 — os verbos novos foram para ficheiros filhos.

## Aberto

- Smoke do dono (o roteiro vai na resposta da janela): o nome «Retime» e as cores.
- Plano 18 §4.5 — Override sobre Bounce/Elastic/Back/Custom com pontos: o 1.º toque na alça ainda troca a família (como
  hoje, pedido do dono: «Override = exatamente como agora»). Recomendação ainda de pé: esconder as alças com a dica
  «Choose Curve or Custom to edit with handles» — decisão do dono.
- Na vista de VALOR as alças continuam as do easing (sobre um trecho retimado não ficam em cima da curva desenhada).
- O «Reset timing» mora ao lado da pílula, não no menu da chave (o plano dizia «menu da chave/linha»); se o dono o quiser
  também lá, é uma linha no cartão da chave.
- Do handoff anterior: smoke do Auto Key na sessão e dos acréscimos 13–14; o campo do Inspector digitado (não medido).
