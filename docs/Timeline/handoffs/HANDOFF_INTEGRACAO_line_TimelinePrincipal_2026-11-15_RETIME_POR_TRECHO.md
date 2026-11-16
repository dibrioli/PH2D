# Integração — `line/TimelinePrincipal`: Retime por trecho + pontos da Custom no gráfico (dono 15/11)

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · merge-base `7b3dd983f7` · **sem rebase**
  (ordem do dono no fecho: «NÃO faça rebase») ⇒ o rebase sobre o `main` e os censos da árvore combinada são do integrador.
- **23 commits** `23db12c893..ae78635856`: os 3 do [handoff anterior](HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-15_AUTOKEY_NA_SESSAO.md)
  (Auto Key na sessão, marcas dos menus, escala da velocidade) + os deste. Sem push, sem integração por esta linha.
- Narrativa completa (medições, auditorias, recusas, fotos de cada volta): [jornada, verbatim](JORNADA_line_TimelinePrincipal_2026-11-15_RETIME_POR_TRECHO.md).
- **Smoke do dono 15/11: OK de tudo** — Retime (caixa do tempo), Override, vista de valor retimada, pontos da Custom, cartão
  virado, e a última volta (abaixo).

## O que entra

1. **Retime por trecho** (plano 18 + pesquisa [19 §5](../19_pesquisa_retime.md), opção A): `ph2d_anim::Retime{x1,y1,x2,y2}`
   (Bézier CSS monótona, coordenadas em `[0,1]`), `Key.retime: Option<Retime>`; o trecho toca `lerp(v0,v1,E(W(u)))` no único
   `Track::segment_value`; `None` ⇒ ao bit como antes (10 famílias). Comando `TimelineIntent::Retime(RetimeIntent::{Set,
   ResetSelected})`, um passo de undo. Vista de velocidade: pílula **Override · Retime** + **Reset timing**.
   - **Override** reconstrói (tira o ritmo no mesmo passo; a outra ponta guarda a velocidade que tocava).
   - **Retime** = caixa do tempo: a alça é o ponto de `W` numa caixa do tamanho do trecho; além da borda, o ponto da caixa
     mais perto do rato (`retime_with_time_handle`, função só do ponteiro). Curva do movimento azul, `W` em âmbar (`Warn`).
   - Reverse, Continue, duplicar, copiar/colar levam o ritmo. Vista de VALOR num trecho retimado: braços = tangentes do que toca.
2. **Pontos da curva Custom no gráfico** (`graph_pontos.rs` na timeline e no painel): criar (toque na linha), mover, Corner ·
   Smooth · Symmetric, Alt = Corner, botão direito apaga — um `SetInterp` por gesto, nenhuma chave. Hits novos
   `TimelineHitKind::{CurvePoint, CurveLine}`.
3. **Cartão da chave com a cara da timeline**: curva e miniaturas viradas quando o valor desce (`BezArea.desce`,
   `key_card_desce`); a curva gravada não muda.
4. **Última volta (dono 15/11, «resolver tudo antes de fechar»)** — `ed872267b8..ae78635856`:
   - **Pontos num trecho RETIMADO** (`ed87`): `unit_to_plane/plane_to_unit` levam o ritmo (`x = W(u)`, `u = W⁻¹(x)` pela Bézier
     de coordenadas trocadas) ⇒ pontos, alças, toque e arrasto ficam em cima de `E(W(u))` e o ritmo fica. O braço da PONTA de
     uma Spline retimada edita a alça de ponta pelo mesmo mapa (sem Override, que apagaria os pontos); sem pontos = Override.
   - **«Reset timing» no cartão** (`ed87`): `CTX_MENU_TL_KEY_RESET_TIMING` entre Rove e Done, pintado e registado só com
     `key_card_retimado` (sem ritmo a linha não existe); clique ⇒ `BeginEdit · RetimeIntent::Set{None}` por chave · `EndEdit`.
     ⚠️ Revoga a decisão da 2.ª janela («fica só ao lado da pílula»): o dono pediu-o.
   - **Miniaturas viradas** (`ed87`): `botao()` recebe `desce`; `area_do_botao` partilhada entre pintura e clique.
   - **A banda contém cada nó e alça da Custom** (`9dfc`): `drawn_extent` saltava Spline e trechos retimados ⇒ ponta acima da
     curva fora da banda, sem alvo. `custom_drawn_values` dá cada nó/`h_in`/`h_out` pelo ritmo.
   - **Braço de velocidade não apaga pontos** (`68c2`): `spline_with_speed_handle` = gesto do AE sobre a alça de ponta da
     `EaseSpline` (`E' = v·Δt/(W'·Δv)`); `end_handle_keeps_points` decide nas duas vistas.
   - **Braços de velocidade nos PONTOS, só em Override** (`8cc5`): `point_speed_arms` (`E'·W'·Δv/Δt` de cada lado),
     `spline_with_point_speed` (x pelo ritmo, preso entre vizinhos; Smooth acompanha, Corner livre).
   - **Braço de curva pronta encurta até caber** (`eb6c`): Interp derivada (Elastic, Back…) sem ritmo ⇒ `arm_inside` sobre
     `segment_value_range`, mesma inclinação; só quando encurta (o resto ao bit).
   - **Toque numa curva PRONTA vira Custom com a mesma cara** (`ae78`, `graph_pontos_pronta.rs`): Hermite cúbica por nós no
     valor exato do preset, inclinação lida de CADA lado (Corner onde diferem), arco de círculo para tangente quase vertical;
     até `SPLINE_MAX_POINTS−1` nós (um fica para o toque). **Medido:** Quad, Cubic, Back, Bounce In/Out exatos; Sine, Expo,
     Circ, Elastic In/Out ≤ 1,1 %; **Elastic e Bounce In-out ~4,5 %** (o tecto de 8 pontos). ~40 ms por curva ⇒ só no toque.

## Formato, contratos, superfície de colisão

- `PROJECT_SCHEMA` **242 → 243** (`Key.retime`), degrau em `project_schema.rs` + tripla em `project_schema_tests.rs`;
  ⚠️ **o integrador RECONTA** pelo `main` (`python3 scripts/schema-recount.py`). Sem leitor do antigo (§0.11). `DOC_VERSION` intocado.
- Contratos §6: nenhum tocado (Nodes/Tools/Vector — `ph2d-anim`, `ph2d-timeline`, `ph2d-panel-timeline`, `ph2d-editor-core` não são superfícies congeladas).
- Quebra fora desta árvore (os censos DESTA árvore estão feitos): literal `Key { t, value, interp }` / `KeyView` sem `retime`
  (cura `Key::new` ou `retime: None`) · `TimelineClipboard::set_from_absolute(&[(AnimTarget, Key)])` · `match` exaustivo em
  `TimelineIntent` (variante `Retime`) e em `TimelineHitKind` (`CurvePoint`, `CurveLine`).
- Ficheiros partilhados: `crates/ph2d-panel-timeline/src/{graph,graph_paint,event,populate,state,lib,ids,linhas_maquete,legenda}.rs`
  · `crates/ph2d-editor-core/src/{ids/menus_timeline.rs, interaction/state/key_card_ops.rs, screens/hero/{key_card,menu_row_look,chrome/timeline_segment}.rs}`
  · `crates/ph2d-i18n/src/timeline.rs` (chaves novas) · `shells/desktop/src/project_schema{,_tests}.rs`.
- Tectos de LOC perto do limite (SOMAM entre linhas): `track.rs` 696/700, `intent.rs` 700/700, `intent_apply.rs` ≤ 700.
- ⚠️ O `CLAUDE.md` desta árvore já tem a entrada do §5.1 Timeline apontando para este handoff (commit `bdfda9e49c`): o
  integrador fica com ela ou reescreve-a no primário. **Link novo:** este ficheiro. **Frase nova** (≤ 300 c, acrescento à de
  hoje): «…; Override · Retime na velocidade; pontos da Custom editáveis no gráfico».

## Provas (por nome; cada commit tem testes, seam com o despacho real e mutações)

- **Última volta:** timeline `num_trecho_retimado` 4 · `a_faixa_contem_cada_alca_da_custom_com_pontos` ·
  `na_vista_de_velocidade_o_braco_da_custom_com_pontos_mantem_pontos_e_ritmo` · `os_pontos_tem_bracos_de_velocidade_que_tocam_e_se_puxam`
  · `o_braco_de_uma_curva_pronta_fica_dentro_do_que_o_trecho_toca` · `a_curva_pronta_convertida_tem_a_mesma_cara_e_deixa_lugar_ao_ponto`
  (24 presets + 3 exatas); painel/seam (`tests/it/curve_points_seam.rs`, painel pintado, despacho real) `num_trecho_retimado` 2 ·
  `o_reset_timing_do_cartao` · `a_ponta_do_braco_acima_da_curva_continua_a_vista_e_agarravel` · `…_nao_apaga_nada` ·
  `no_override_o_ponto_tem_braco_de_velocidade_que_se_puxa` · `um_toque_numa_curva_pronta_cria_o_ponto_debaixo_do_rato`;
  editor-core `reset_timing_so_existe` (clique pelo `HeroScreen`) · `as_miniaturas_viram`.
- **Mutações da última volta: 15 + 2 + 3 + 5 + 1 + 5 = 31, as 31 vermelhas.** Das voltas anteriores (jornada): Retime 5/5,
  caixa 2+2, auditoria da caixa 5/5, vista de valor 4/4, pontos no gráfico 8/8.
- **Gate do fecho (1×, sobre o diff acumulado):** fmt ✓ · clippy `--workspace --all-targets --keep-going -D warnings` ✓
  (2 avisos curados: `clone_on_copy` e `collapsible_if` em `graph_pontos.rs`) · `check-standalone-optional` ✓ (11) ·
  `check-workflow-packages` ✓ (33 × 526) · `file_loc_caps` 4/4 · `architecture_workspace_file_loc_cap` 2/2 (depois de partir
  `graph.rs`) · `arch_safe_clamp_only` 2/2 · por nome: timeline `graph_pontos` 13 · `graph_pontos_pronta` 1 · `graph::` 19 ·
  `speed` 20 · `retime` 9 · painel `graph_pontos` 7 · `curve_points_seam` 11 · `cartao_bezier` 5 · `graph_ritmo` 9 · `graph` 44 ·
  editor-core `key_card` 9 · `curve_point` 5 · host `project_schema` 1 — **todos verdes**.
- Impactados e suíte completa ficam para a integração.

## Auditoria do fecho (2 lentes independentes sobre `ed872267b8^..ae78635856`)

Lente 1 = lei matemática (`ph2d-timeline`); lente 2 = costura de UI e controlo morto (painel, editor-core). Curas num
bloco, cada uma com teste e mutação (restaurado com `touch`, conferido por igualdade de texto):

| # | Achado (mecanismo) | Cura | Prova · mutação |
|---|---|---|---|
| 1 | Braço da PONTA de uma Custom com pontos, puxado na velocidade além do ponto vizinho: o `y = declive·x` era calculado com o `x` do rato e o `arruma` prendia só o `x` ⇒ toca 1,6× a velocidade pedida | `spline_with_speed_handle` prende `x` entre a ponta e o nó vizinho ANTES da altura | `o_braco_da_ponta_alem_do_ponto_vizinho_mantem_a_velocidade_do_rato` · vermelha |
| 2 | A ponta desse braço, num trecho RETIMADO, era desenhada pelo braço encurtado (`segment_handle_points`) e o gesto gravava `x = u`: as vistas de valor e de velocidade punham a mesma alça em tempos diferentes e agarrar sem mexer na horizontal mudava a forma (as duas lentes) | `speed_handle_tip` lê `value_handle_points` (a alça guardada pelo ritmo) e o gesto grava `x = W(u)` (`plane_to_unit`) | o teste que só conferia a ponta SEM ritmo passou a conferir com ritmo · `na_vista_de_velocidade_puxar_na_vertical_nao_muda_a_influencia_da_custom_retimada` (seam) · as 2 mutações vermelhas |
| 3 | Alça de ponto quase vertical (`dx` ~1e-7) ganhava braço de velocidade a ~10⁶/s, fora da banda | piso `0,5·MIN_INFLUENCE` em `point_speed_arms` (metade: o gesto deixa o braço no piso e ele tem de continuar agarrável) | `a_alca_quase_vertical_de_um_ponto_nao_tem_braco_de_velocidade` (Corner) · vermelha |
| 4 | `paint_points`/`paint_speed_arms` registavam alvos fora da banda (zoom/rolagem): alvo invisível sobre a coluna de rótulos, a ganhar de tudo (registado por último) | `regista` (o único sítio) recusa o centro fora de `band.rect`, como `paint_handles` | `o_ponto_fora_da_vista_nao_deixa_alvo_de_clique` (as duas vistas, controlo na vista inteira) · vermelha |
| 5 | `x.clamp(antes, p − m)` em `spline_with_point_speed` com dois nós a `X_EPS` pode ter `min > max` em f32 ⇒ pânico | `max`/`min` | `dois_pontos_encostados_nao_fazem_o_braco_de_velocidade_entrar_em_panico` — ⚠️ a mutação **SOBREVIVE**: o caso não se alcança por `move_point` (medido: 0 em 1000 com o controlo); fica como defesa, não cura provada |

- **Descartados ao conferir:** a banda «esmagada» pela alça quase vertical (falso: a própria curva toca a ~10⁶/s ali e a
  banda amostra-a — só o braço era indevido) · «Reset timing» sem dica (nenhum botão do cartão tem) · buraco entre Rove e
  Done (o Rove é caixa + rótulo à esquerda; a área é só de clique) · teste de ponta a ponta do Reset (as duas metades
  partilham o `WidgetStore`: `key_card_tests` clica pelo `HeroScreen` até `take_…`, `cartao_bezier_tests` drena até ao
  intent) · inverso de `W` (exacto: a mesma curva paramétrica com as coordenadas trocadas, monótona).
- **Fica aberto (decisão de desenho, não defeito):** na vista de velocidade o PONTO é só marca (os braços dele é que se
  agarram); escolher/apagar/Corner faz-se na vista de valor. · `preset_as_curve`: o ajuste fino parte do último ensaio e
  não do melhor (nunca piora o melhor; só pode perder optimização).
- `graph.rs` passou do tecto (732/700) com esta volta: os testes foram para `graph_tests.rs` (`#[path]`, como o
  `graph_pontos_pronta`) ⇒ 376.

## Smoke (o dono já fez; para reconferir)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal
PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop
```
Fotos (Xvfb, cliques reais, roteiro com conferências por passo [`fotos_pontos_retimado.sh`](../ferramentas/fotos_pontos_retimado.sh)):
[pontos r0..r8](../fotos/pontos_r_sequencia.png) — [r4 Reset timing no cartão](../fotos/pontos_r4_cartao_com_reset_timing.png) ·
[r6 miniaturas viradas](../fotos/pontos_r6_miniaturas_viradas_quando_desce.png) · [r7 Elastic com braços à vista](../fotos/pontos_r7_elastic_bracos_a_vista.png) ·
[r8 toque no Elastic cria o ponto](../fotos/pontos_r8_toque_no_elastic_cria_o_ponto.png); [velocidade rv1..rv3](../fotos/pontos_rv_sequencia.png).

_BUILD_

## Aberto

- Ctrl+Z dos gestos de pontos não conferido no Xvfb (o teclado não chega) — coberto pelos testes do passo de undo.
- Do handoff anterior: smoke do Auto Key na sessão e dos acréscimos 13–14; o campo do Inspector digitado (não medido).
- Elastic/Bounce In-out convertidos em Custom desviam ~4,5 % (tecto `SPLINE_MAX_POINTS` = 8): subir o tecto é decisão de formato.

## Perfil do agente (`bash scripts/agent-loop-profile.sh`)

```
PERFIL DO LOOP DO AGENTE — 20 sessao(oes) mais recentes
  ✗ paralelismo de ferramenta              1.16/passo   alvo: >= 1,5  (8% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                142   alvo: <= 800
  ✗ cargo test : cargo check                 303 : 88   alvo: <= 1,0  razao 3.4x (baseline: 4,3x)
  ✓ edicoes SEM conferir que casaram             1.9%   alvo: <= 2%  (14 de 723)
  ✓ contexto relido por passo (media)         243 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               68 mil   alvo: <= 80 mil
```
