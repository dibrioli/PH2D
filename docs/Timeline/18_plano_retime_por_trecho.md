# 18 — Retime por trecho: a curva de velocidade que NÃO destrói o easing (dono 15/11)

> Plano para a próxima janela da `line/TimelinePrincipal`. Estado da linha e o que já está feito hoje:
> [handoff de 15/11](handoffs/HANDOFF_INTEGRACAO_line_TimelinePrincipal_2026-11-15_AUTOKEY_NA_SESSAO.md).
> Nada deste plano está implementado.

## 1. Pedido e decisão do dono (verbatim)

Fotos do dono: um Bounce em Position X; a vista de velocidade dele (dente de serra, correcto: a velocidade do Bounce
salta no quique); ao tocar a alça de velocidade «o easing anterior se perde»; a curva nova de X é uma cúbica lisa.

> «A curva de velocidade na verdade reconstrói toda a animação. Mas se houvesse a opção de ajustar a velocidade sem
> destruir o easing, ou seja: algo como nas lanes que tem o override e o additive. Ou seja: se o usuário escolhe
> Override então funciona exatamente como agora. Mas se selecionar additive temos duas curvas sobrepostas de cores
> diferentes (a curva de easing ou a edição manual do usuário) e uma segunda curva de velocidade que não destruiria
> o easing.» — e depois: «deixe o plano escrito para a próxima conversa começar por ele».

**Ajuste proposto ao dono e não recusado:** o modo chama-se **Retime**, não «Additive». Somar VELOCIDADE ao easing faz o
objecto falhar o valor da chave seguinte (∫v muda); o que preserva as duas chaves e o desenho do easing é mexer no
TEMPO do trecho. ⚠️ Confirmar o nome com o dono no 1.º smoke (as opções «Override · Retime» são o rótulo proposto).

## 2. Mecanismo de hoje (medido e lido em 15/11)

- `graph::resolve_drag` (`crates/ph2d-panel-timeline/src/graph.rs:~227`): TODO arrasto de alça — vista de valor
  (`weighted_with_handle`) ou de velocidade (`weighted_with_speed_handle`) — grava `SetInterp` com um par de tangentes
  `BezierW`. Bounce/Elastic/Back (e `Spline` com pontos) não cabem em duas alças ⇒ o 1.º toque troca a família.
- `graph_paint::paint_speed_handles` pinta alças em qualquer trecho seleccionado, seja qual for a `Interp`.
- O AE não tem este defeito porque lá o quique nunca é interpolação de chave (Linear/Bezier/Continuous/Auto/Hold; o
  Bounce é expressão ou plugin por cima). Maya tem o **Time Warp** (curva que reajusta o tempo sem tocar nas chaves);
  AE o **Time Remap** (por camada); Blender, o tempo da faixa NLA. Nenhum o edita na própria vista de velocidade.
- O PH2D já tem **Time Remap por OBJECTO** (`ph2d_timeline::remapped_time`, track `TimelineTrackTimeRemap`): reajusta o
  relógio da entidade inteira. Falta o reajuste POR TRECHO entre duas chaves, editado no gráfico de velocidade.
  ⇒ Antes de construir, MEDIR se compor com o Time Remap já exprime o pedido (CLAUDE.md §5.0): a hipótese é que NÃO
  (é por entidade e ao longo do clip, não por trecho, e não se edita como velocidade).

## 3. Lei

Para o trecho `k0 → k1`, com `u ∈ [0,1]` o tempo normalizado e `E` o easing da chave (qualquer `Interp`):

- **Override** (hoje): `valor(u) = lerp(v0, v1, E(u))`, e a alça reescreve `E`.
- **Retime**: `valor(u) = lerp(v0, v1, E(W(u)))`, com `W: [0,1] → [0,1]`, `W(0)=0`, `W(1)=1`, **monótona** (a mesma regra
  CSS do `cubic-bezier` que a `EaseSpline` já impõe: `x` das alças dentro de `[0,1]`). `W` identidade = byte-idêntico a
  hoje. As duas chaves, os instantes e o desenho de `E` (todos os quiques) ficam; só o ritmo muda.
- Velocidade desenhada em Retime: `d valor/dt = (v1−v0) · E'(W(u)) · W'(u) / (t1−t0)`.
- `W` = uma Bézier de duas alças no quadrado unitário (4 × `f32`, 16 B) — reutilizar o solver da `curve_bezier.rs` /
  `EaseSpline::from_bezier` (já monótono, já `eval`/`slope` deterministas).

## 4. Desenho

1. **Formato** — `W` mora na CHAVE, não na `Interp` (a `Interp` é `Copy`, está no tecto de **216 B** com
   `clippy::large_enum_variant`; ver `curve_spline.rs`). `ph2d_anim::Key` ganha `retime: Option<Retime>` (ou um
   `[f32;4]` com identidade); medir `size_of::<Key>()` antes/depois e escrever a tabela no doc-comment. Sobe o
   `PROJECT_SCHEMA` (contar pela `scripts/schema-recount.py`; o main estava em 242 em 15/11). Sem compatibilidade (§0.11).
2. **Avaliação** — o ponto único onde `Track` amostra um trecho aplica `W` antes de `E` (procurar o sampler que
   `Interp::value`/`remap` alimenta; `slope`/`tangent_handles`/`reversed` têm de compor também: `reversed` espelha `W`).
   Determinismo: polinomial, sem `HashMap`.
3. **Modo** — `Override · Retime` por **vista de velocidade** (estado do painel, não do documento) ou por chave?
   Proposta: estado do painel (`TimelinePanelState.speed_mode`), escolha na fileira da vista de velocidade, como o
   Override/Additive da lane (menu ou segmentado, marcado — ver `menu_marcas`). Com Retime as alças de velocidade
   escrevem `SetKeyRetime { target, key, w }` (intent nova, um passo de undo por arrasto: `BeginEdit/EndEdit`).
4. **Pintura** — Retime: duas curvas de cores diferentes (tokens; zero hex): a velocidade do easing ORIGINAL (fina,
   apagada) e a velocidade FINAL (acento) com as alças; na vista de valor a curva mostra o resultado com `W`. A barra
   da linha (`linhas_maquete::pinta_curva`) desenha o resultado também.
5. **Override com família que não cabe em duas alças** (Bounce/Elastic/Back/Spline): decidir no smoke — esconder as
   alças com a dica «Choose Curve or Custom to edit with handles», ou avisar. O dono ainda não escolheu; a recomendação
   foi esconder.
6. **Reset** — menu da chave/linha: «Reset timing» (W = identidade).

## 5. Provas pedidas

- `W` identidade ⇒ amostras byte-idênticas às de hoje (todas as famílias).
- Retime sobre Bounce: `valor(0)=v0`, `valor(1)=v1`, o número de quiques (cruzamentos/extremos) igual ao de `E`; a
  velocidade amostrada = derivada numérica.
- Override inalterado (testes de `graph` actuais).
- Mutação: ignorar `W` no sampler ⇒ vermelho.
- Fotos (Xvfb, `PH2D_ANIMACAO_SMOKE=1`, roteiro em `$SCRATCH/foto_menu.sh` — recriar se sumiu: ESQUERDO/TIPO/DIREITO/
  DEPOIS/ARRASTA; o ▶ de Position X fica em `365 955`, engrenagem `1541 826`, «Show speed» `1336 824`): Bounce →
  Retime → arrastar a alça ⇒ os quiques ficam, o ritmo muda; e a curva de valor ao lado.
