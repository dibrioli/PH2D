# 45 — O esmaecer da pressão e a Shape Color no pincel salvo — o que se MEDIU (17/10)

> Prompt [45](45_PROMPT_shape_color_no_pincel_e_o_esmaecer.md). Barra: a do [18 §2](18_mapa_das_midias.md) (o
> vale entre o Rebelle e o controlo Wet Round à mesma largura). Régua da rampa: `regua_midia::rampa` (a cobertura
> mediana > 0,08 na pressão ~0,46 ÷ na 1, linha 2 da bancada). Sonda `as_formas::sonda_o_esmaecer_da_pressao`
> (`PH2D_SONDA_CURVAS=1,1.5,…`, `--profile smoke`, cena seca de 1 800 quadros). Fecha B13, C27 e a pressão da C28.

## §1 — O mecanismo (perguntado ao código)

`d.coverage = strength · coverage_scale(p) · overlap` → a porta do Wet Paint `b = 10 · coverage ÷ strength`
(`dab_route.rs`) → `pressure_dab`: `intensity = min(3, b · 0,5 · 0,69)` e a dureza `min(3, b/2)²` → a célula
`stamp = min(1, queda · grão · intensity)` (`trail/deposit.rs::cell`). **A pressão entrava ANTES do tecto `1`**:
com `pressureOpacity` 1, `intensity = 3,45·p` já satura para `p ≥ 0,29` no miolo — só a borda esmaecia (e a
dureza amaciava). No pó a média por célula é a do contínuo (a gota leva `÷ p`): o mesmo tecto. Nenhum orçamento de
pigmento por dab nem renormalização pelo traço cancela a pressão — é o tecto.

## §2 — A medição (um lote, uma compilação, variantes por env num processo por variante)

Lugar da opacidade × curva `γ` (a pressão sobe a `γ` antes da rampa); «hoje» = a lei de antes. Cena seca:

| caso (raio) | hoje | depósito γ1 | γ1,5 | γ1,75 | γ2 | Rebelle |
|---|---|---|---|---|---|---|
| Wet Flat (17) | 0,765 / 0,525 | 0,794 / 0,600 | 0,794 / 0,473 | **0,765 / 0,411** | 0,765 / 0,356 | 0,83 / 0,42 |
| Gouache Flat (19,5) | 0,821 / 0,619 | 0,846 / 0,674 | 0,821 / 0,538 | 0,795 / 0,468 | **0,769 / 0,419** | 0,87 / 0,43 |
| Airbrush (15; tamanho 0,5) | 0,560 / 0,624 | 0,600 / 0,412 | **0,600 / 0,353** | 0,600 / 0,306 | 0,560 / 0,282 | 0,63 / 0,34 |
| Airbrush, Spray 2 literal (tamanho 0) | 0,920 / 0,801 | 1,000 / 0,524 | 1,000 / 0,400 | 1,000 / 0,353 | 1,000 / 0,294 | 0,63 / 0,34 |
| controlo Wet Round 15 · 17 · 19,5 | 0,448 / 0,673 · 0,455 / 0,637 · 0,462 / 0,688 | | | | | |

(largura / cobertura.) «Depósito» = a opacidade multiplica o que POUSA, depois do portão do papel (como o `sv`).
Também medidos: a opacidade no CARIMBO (depois do tecto, antes do portão) — γ1 na cena seca: Flat 0,765 / 0,373,
Gouache 0,769 / 0,443 (no vale) — e a curva na porta de antes (`b`). No lote 1 (só o gesto, 480 quadros) os dois
deixam de pousar no toque leve com a curva: o Wet Flat a γ ≥ 1,5 e a Gouache a γ 2 dão largura 0 (o portão do papel
come o carimbo fraco: tudo ou nada). ⇒ o lugar é o depósito, que aceita a curva.

- **A linear não basta**: a tinta da pressão é massa de pigmento e a cor dela satura (a claridade é o pigmento);
  a opacidade do Rebelle é a da cor. A curva por pincel compensa: Flat 1,75 · Gouache 2 · Airbrush 1,5.
- **O Airbrush com os números literais do Spray 2** (tamanho 0) esmaece certo com γ 1,75 (0,353) mas NÃO afina
  (largura 1,00 contra 0,63): ficou o tamanho 0,5 + γ 1,5 — o mais perto nas duas (0,60 / 0,35). A ordem do dono era
  «como no Rebelle».

## §3 — As leis

- **A opacidade do dab** (`trail::Dab::opacity`, argumento da porta do pincel `dispatch_pressure_dab_lane_cor`; as
  outras portas pousam com `1`): multiplica o depósito depois do
  portão e do `sv`. A porta do pincel (`dab_route.rs`): com a pressão → opacidade ligada e a ferramenta Paint, o `b`
  é o do toque inteiro (`10 · overlap`) e a opacidade é `coverage ÷ (strength · overlap)`; o resto (Smear, Blend,
  Wet/Dry, a borracha) lê o `b` de sempre. `1` ⇒ o depósito ao bit (`x · 1,0 = x`).
- **O overlap do método** é UM predicado: `StrokeMethod::is_spaced_fill` (o `Stroke` e
  `BrushSpec::method_overlap_factor`).
- **A curva** `pressureOpacityCurve` (Tuning › Paint, 0,25–4, omissão 1 = linear; `KNOB_COUNT` 59):
  `Dynamics::strength_curve`, `coverage_scale(p) = min + (1 − min)·p^γ` (γ = 1 ⇒ o `p` sem `powf`, ao bit).
- **Identidade**: as 19 cenas de `a_lei_da_tinta` sem a pressão → opacidade ao bit (a lei de antes reproduz as 20
  pinadas — verificado com a porta desligada antes de pinar o Wet Flat e a Gouache Flat, `a5134220e5`); Airbrush,
  Wet Flat e Gouache Flat re-pinados por ordem do dono. Regerados só esses três ficheiros.

## §4 — A Shape Color no pincel salvo (B13, dono 17/10: SIM)

- A macro já a levava (`macro_estado.rs`, objecto `shape_color` por nome). Perguntado ao código: a lei passou a ser
  UMA — `EstadoDaShapeColor::para_json`/`ler_json` (`shape_ramp.rs`) — e o preset grava-a em
  `Parametros::shape_color` (opcional). O estado inicial da macro já não a grava no topo (vai no preset dela) e lê
  a das macros de antes.
- **`SCHEMA` fica 1**: um leitor velho guarda a chave no `extra` e regrava-a; lê o pincel sem rampa (sem erro).
- Captura só quando difere da de fábrica ⇒ os 20 de fábrica ao byte. Aplicar SEM a chave repõe a de fábrica (a regra
  dos campos ausentes do P44). A miniatura passa a pintar a Shape Color (o ficheiro devolve-a).

## §5 — Portões

`ph2d-wet-paint` `a_opacidade_do_dab` (meia opacidade = metade da massa no contínuo e no pó; meia intensidade
ainda > 80 %) · `as_formas::a_opacidade_da_pressao_morde_no_po` (Spray 2 literal < 0,75; a lei de antes 1,02) e
`a_curva_esmaece_mais_no_toque_leve` (vermelhos com a lei de antes) · `the_coverage_curve_shapes_the_pressure_only`
· `midias_de_fabrica`: `RampaCobertura` (ruído 0,04, não medido) nos três, `RampaLargura` no Airbrush ·
`wetpaint_preset/shape_color_tests.rs` (4) · `a_miniatura_pinta_a_shape_color` (invertido). Mutação 13/14 (+ M10, que só o
`um_pincel_sem_rampa` vê): o sobrevivente M7 era estado do motor partilhado pelas portas — curado pela forma
(a opacidade passou a argumento), handoff P45 §5.

## §6 — Aberto

- A 20 px o Airbrush lê textura 28,0 contra 22,5 (C28, não juntado).
- `cada_preset_parece_a_sua_midia` / `a_largura_e_a_prometida` passam do prazo de 360 s sob carga.

## ⛔ Recusas MEDIDAS

| proposta | medido | porquê não |
|---|---|---|
| a curva na porta de antes (o `b`) | Flats γ ≥ 1,5: largura 0 no toque leve | o tecto e o portão juntos: tudo ou nada |
| a opacidade no carimbo (antes do portão) | γ1: Flat 0,37 · Gouache 0,44 (no vale); com curva o toque leve não pousa | não se afina: o portão faz o toque leve tudo ou nada; o depósito + curva fica mais perto nos três |
| a opacidade linear no depósito | Flat 0,60 · Gouache 0,67 (Rebelle 0,42/0,43) | a massa satura na cor; precisa da curva |
| os números literais do Spray 2 no Airbrush | largura 1,00 (Rebelle 0,63) | o nosso não afina sem a pressão → tamanho |
