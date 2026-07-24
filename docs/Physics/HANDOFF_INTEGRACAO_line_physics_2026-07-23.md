# HANDOFF DE INTEGRAÇÃO — `line/physics` (W-AreaFrame + W-AreaFalloff + W-AreaMirror, 2026-07-23)

> Para o **agente integrador**, por ordem do Enio (DIRETRIZ §1.5.9). A linha **não integrou e
> não pushou**: fechou a wave, rodou o gate batched e parou.

---

## 1 — Identidade

| | |
|---|---|
| branch | `line/physics` |
| HEAD | `63fc75df9` (o commit deste próprio doc; o integrador usa `git rev-parse line/physics`) |
| base (merge-base com `main`) | `df91ef6ec` |
| commits à frente | **19** (1 doc herdado + 6 do W-AreaFrame + 6 do W-AreaFalloff + 2 do fix da Dur(s) + 3 do W-AreaMirror) |
| working tree | limpa |

Os commits, em ordem (o 1º já existia quando assumi a linha):

```
1ae4dfef0 docs(physics): handoff de REABERTURA 2026-07-23 -- o proximo agente assume a linha pos-integracao
938c5aaa3 feat(physics): W-AreaFrame -- girar a zona gira o vento
043a345e5 test(physics): os gates do W-AreaFrame -- 13 mutacoes, 11 sangram
8a87ed900 feat(physics): c9 ganha uma zona ROTACIONADA + cena 34 do frame
67c14c5d3 refactor(physics): o overlay separa CONTORNO de ANOTACAO (cap de 600 LOC)
20b0e8797 test(physics): o marcador do frame entra no round-trip de persistencia
cb70bf0d8 docs(physics): W-AreaFrame no tracker/plano/CLAUDE.md + handoff de integracao
baedc8123 feat(physics): W-AreaFalloff -- o empurrao desvanece do centro para a borda
6e4b59b58 test(physics): os gates do W-AreaFalloff -- 14 mutacoes, 13 sangram
7abf8506a refactor(physics): a familia das ZONAS ganha casa propria (caps de LOC)
fd3cc172c docs(physics): W-AreaFalloff no tracker/plano + handoff de integracao das 2 waves
ca2367d50 docs(physics): CLAUDE.md 5 -- o W-AreaFalloff fecha a familia das zonas
96fcafb18 docs(physics): o handoff de integracao pina os commits reais
a0d429305 fix(timeline): a Dur(s) aceita QUALQUER duracao -- o teto de u16::MAX era um palpite
560936090 docs(physics): o handoff nomeia o fix da Dur(s) fora do dominio da linha
2c589f8af feat(physics): W-AreaMirror -- espelhar a zona espelha o vento (e inverte o giro)
b7bd434f9 docs(physics): W-AreaMirror no tracker e no plano
bf9add666 docs(physics): CLAUDE.md 5 + handoff cobrem o W-AreaMirror
(+ este commit)
```

**Duas waves, uma branch.** Este handoff cobre as duas — o integrador lê um documento só.

---

## 1b — ⚠️ UM FIX FORA DO DOMÍNIO DA LINHA (ler antes de integrar)

O último commit toca **`ph2d-panel-timeline`** e **`ph2d-editor-core`**, que não são desta
linha. Foi **pedido direto do Enio** (*"a timeline está limitando a simulação pois não
permite que eu redefina a Duração da Animação/Simulação … permita que eu possa colocar
qualquer valor em Dur"*), e a causa é de física: a duração autorada CORTA o relógio, então
um teto na caixa Dur é um teto na simulação.

**A causa, medida:** os três chips do transporte (Time · Frame · Dur) registravam alcance
`[0, f64::from(u16::MAX)]`. O `65535` não era medido — o doc-comment ao lado já dizia
`[0, ∞)`, ou seja o código contradizia a intenção escrita nele. Mordia por **duas** vias:

1. o **stepper** parava em 65535 s;
2. o **arrasto** ficava inútil — o scrub deste app é *proporcional ao alcance*
   (`DRAG_RANGE_PX_H` = 250 px varrem `[min, max]`), então **um pixel valia 262 segundos**.
   É esta a via que o artista sente como *"a caixa não deixa eu editar"*.

**O fix:** teto **aberto** (`f64::INFINITY`) + um **drag-rate calibrado** igual ao próprio
`step` do chip (um pixel = um clique de stepper). ⚠️ O alcance **não foi apagado**: é ele
que carrega o `step` (sem ele o dispatch cai numa heurística de buffer — `0.01` num valor
com ponto — e o incremento de **0,2 s** que o Enio pediu em 2026-07-23 some em silêncio) e
o piso `0` (duração negativa não é uma coisa; `0` é o gesto de LIMPAR). A mutação M17 prova
isso: apagar o alcance derruba 3 gates.

⚠️ **O doc do `set_number_drag_rate` mandava NÃO combinar os dois** — e o código faz o
oposto há tempo: o rate vence o modelo proporcional *e* dispensa o clamp do arrasto
(`dispatch::pointer_move`, dois sítios). O texto foi corrigido: era exatamente ele que
desencorajava a combinação certa.

**Nenhum schema, nenhum contrato, nenhum id novo.** Gates novos em
`ph2d-panel-timeline/tests/duration_chip_gesture.rs` (um estrutural + um que **arrasta um
ponteiro de verdade** e exige que 20 px pousem em segundos, não em milhares); **3 mutações,
3 sangram**. O `Time` e o `Frame` herdam o mesmo conserto pela MESMA porta (`chip()`), que é
o certo: as três são grandezas sem teto e uma resposta só.

---

## 2 — Foundational / compartilhado tocado, e por quê

Tudo **aditivo**. Nenhum arquivo fora do domínio de física foi reescrito.

| arquivo | o quê | por quê |
|---|---|---|
| `ph2d-editor-core/src/ids/inspector.rs` | +1 `NodeId` de grupo, +1 array de 2 | o toggle da §11 precisa de id |
| `ph2d-editor-core/src/screens/hero/inspector_model_physics.rs` | +1 campo em `InspectorPhysicsInfo`, +1 variant em `PhysicsFieldEdit` | o snapshot que o painel lê e o edit que ele emite |
| `shells/desktop/src/render_loop/physics_overlay*.rs` | **split** por responsabilidade + a seta gira | cap de 600 LOC (detalhe §5) |
| `shells/desktop/src/render_loop/mod.rs` | +1 linha `mod` | declara o módulo novo |
| `shells/desktop/src/render_loop/inspector_physics{,_apply,_markers}.rs` | lê o marcador · roteia o edit · escreve o componente | as 3 pontas da costura |
| `shells/desktop/src/physics_smoke{,_zones}.rs` | cena `=34` + o braço do dispatch | a cena da wave |
| `ph2d-panel-inspector/src/{populate,event_physics,sections/physics{,_rows}}.rs` | registra · despacha · pinta | a row |
| `ph2d-physics-ecs/src/components{,.rs}` | **SPLIT**: a família das ZONAS sai para `components/area.rs` | cap de 700 LOC (§5) |
| `shells/desktop/src/render_loop/inspector_physics_area.rs` | **NOVO**: os braços de ZONA do apply | cap de 600 LOC (§5) |
| `shells/desktop/tests/every_physics_component_is_authorable.rs` | +1 arquivo na lista `WRITERS` | o split moveu 6 escritores |

⚠️ **`ph2d-editor-core/src/ids/inspector.rs` e `inspector_model_physics.rs` são os dois pontos
onde outra linha pode ter mexido na mesma janela.** As adições são **append** no fim dos blocos
de física; o Mergiraf funde isso sozinho, e o §3 abaixo é o que se grepa se não fundir.

---

## 3 — Símbolos novos (é isto que o integrador grepa por mesmo-símbolo)

| símbolo | onde | valor / forma |
|---|---|---|
| `INSP_LIVE_PHYSICS_FORCE_AXES` | `ids/inspector.rs` | `hash_node_id("insp_live_physics_force_axes")` |
| `INSP_PHYS_FORCE_AXES` | idem | `[hash_node_id("insp_phys_force_axes_zone"), hash_node_id("insp_phys_force_axes_world")]` |
| `AreaForceWorldAxes` | `ph2d-physics-ecs` (componente) | marcador; nome canônico **`"ph2d::physics::AreaForceWorldAxes"`** |
| `PhysicsFieldEdit::ForceWorldAxes(bool)` | `inspector_model_physics.rs` | variant **apendado** |
| `AreaEffect.world_axes: bool` | `ph2d-physics` (plain data) | campo novo, **não serializado** |
| `zone_force_world` / `zone_force_world_at` | `ph2d-physics::world::effector` | fns pub, re-exportadas |
| `FORCE_AXES_LABELS` | `physics_rows.rs` | `["Zone", "World"]` |
| `physics_overlay_annotations` / `_annotation_tests` | shell | módulos novos |
| `physics_smoke_force_frame` + cena `"34"` | shell | — |
| `INSP_PHYS_AREA_FALLOFF` | `ids/inspector.rs` | `hash_node_id("insp_phys_area_falloff")` |
| `AreaFalloff` | `ph2d-physics-ecs` (componente) | valuado `f32`; nome canônico **`"ph2d::physics::AreaFalloff"`** |
| `PhysicsFieldEdit::AreaFalloff(f32)` | `inspector_model_physics.rs` | variant **apendado** |
| `AreaEffect.falloff: f32` | `ph2d-physics` (plain data) | campo novo, **não serializado** |
| `ShapeDesc::radial_fraction` | `ph2d-physics::world::shape` | método inerente, `pub` |
| `zone_falloff_scale` | `ph2d-physics::world::effector` | fn `pub`, re-exportada |
| `FALLOFF_RGBA` / `FALLOFF_RING` | shell (annotations) | cor + a fração `0.5` do anel |
| `apply_area_edit` / `area_edit` | shell / painel | os dois helpers dos splits |
| `physics_smoke_falloff` + cena `"35"` | shell | — |
| `AreaEffect.mirror` + `AreaEffect::UNMIRRORED` | `ph2d-physics` (plain data) | campo novo, **não serializado** |
| `zone_spin_sign` | `ph2d-physics::world::effector` | fn `pub`, re-exportada |
| `physics_smoke_mirror` + cena `"36"` | shell | próxima cena livre = **37** |
| `src/bin/physics_ecs_c9/{main,zones}.rs` | `ph2d-physics-ecs` | ⚠️ o bin virou DIRETÓRIO; o `[[bin]].path` do `Cargo.toml` mudou |

**Contadores que SOMAM entre linhas** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]):

- registro de componentes de física **19 → 21** (`register_physics_components` + o
  `assert_eq!(reg.len(), 21)`) — o `AreaForceWorldAxes` e o `AreaFalloff`, uma por wave;
- `physics_ecs_c9` **77 → 83 corpos**, hash **`4e862761…`** (era `7d55a4ab…` no `main`);
- `every_physics_component_is_authorable`: `WRITERS` **3 → 4** e o piso do parse **18 → 21**;
- **`PROJECT_SCHEMA` NÃO mudou — fica em 29.** Componente novo cunha blob-key própria.

⚠️ Se outra linha bumpou o `PROJECT_SCHEMA` nesta janela, **esta wave não disputa o número**
(não o toca). Se outra linha registrou componente de física, o **20 se CONTA**, não se escolhe.

---

## 4 — Contratos congelados encostados

**Nenhum.** `NodeOp`/`OpResolver`/`NodeManifest`, `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` e a superfície do `ph2d-vector-doc` estão intocados — conferido por grep, não por
auto-relato.

---

## 5 — O que só o `ship.sh` pega (o gate de integração NÃO roda)

Rodados **aqui**, verdes: `cargo check --workspace` · `nextest-impacted` (**5321 testes, 0
falhas**) · `clippy --all-targets` nas 5 crates tocadas (**zero warning**) · `file_loc_caps` da
shell · `architecture_workspace_file_loc_cap` · `arch_safe_clamp_only` ·
`architecture_panel_wiring_parity` · `every_physics_component_is_authorable` · suítes de física
em **debug E release** (**69 binários** verdes em cada) · `architecture_panel_loc_cap`.

**Não rodados** (só o ship): `cargo machete` · `cargo deny` · `cargo audit` · `typos` ·
`cargo fmt --check` do repo inteiro (formatei **só os arquivos que toquei**, com `rustfmt`, pela
regra anti-colisão) · doctests.

**Dep nova: nenhuma.** O `libm` já era dep da `ph2d-physics` — com o mesmo raciocínio, escrito no
próprio `Cargo.toml` dela.

---

## 6 — Ordem / dependências, e o que smoke-testar

Os commits são sequenciais e não têm dependência externa. **Nada precisa ser aplicado fora de
ordem.**

### O que o Enio já aprovou (cenas 33 e 34) — não precisam de re-smoke

**`env PH2D_PHYSICS_SMOKE=34 cargo run -p ph2d-host-desktop`** — deixe **Physics** marcado, `B`
liga contorno e seta.

1. **As duas faixas divergem:** esquerda (Zone) as caixas sobem na **diagonal**; direita (World)
   as mesmas caixas andam na **horizontal**. Mesma rotação, mesma força — um bit de diferença.
2. **O gesto:** selecione a zona da **esquerda** → Inspector → *Physics Body* → linha
   **`Force Axes`** → clique **World**. As caixas dela passam a andar na horizontal **e a seta
   laranja gira junto**. Clique **Zone** e o diagonal volta.
3. **A row lembra:** clique noutro objeto e volte — a linha tem de continuar mostrando o lado que
   você escolheu (é a metade *write-only* que mordeu a família de zonas em 22/07).

### O que o Enio ainda não viu

**A caixa `Dur(s)` (§1b), em qualquer cena:** clique nela e **arraste** — o número tem de
acompanhar o dedo em segundos (antes um pixel dava centenas). Digite **300** e dê Enter: a
timeline abre até lá e a simulação corre os 5 minutos, em vez de bater no fim aos 4 s.

**`env PH2D_PHYSICS_SMOKE=36 cargo run -p ph2d-host-desktop`** — o espelho.

Quatro zonas com a MESMA autoria; só o **sinal da escala** difere (as da direita têm
`scale.x = -1`, que é o gesto de virar o sprite). Em cima o vento a 40°: à esquerda as
caixas sobem na diagonal `(2,81, 2,36)`, à direita vão para o lado **refletido**
`(−2,81, −2,36)`. Embaixo o giro: `+117°` à esquerda, **`−117°`** à direita — ao contrário.
Essa última linha É a wave (força é vetor e reflete, torque é pseudoescalar e inverte).

**`env PH2D_PHYSICS_SMOKE=35 cargo run -p ph2d-host-desktop`** — a rajada com centro.

1. **As duas filas divergem:** esquerda (uniforme, como era até hoje) as quatro caixas **voam
   juntas**; direita (Falloff 1) a fila **se abre em leque**, e a mais externa anda 5× menos.
2. **O gesto:** selecione a rajada da **esquerda** → Inspector → *Physics Body* → linha
   **`Falloff`** → digite **1**. O **anel laranja apagado** do meio caminho aparece no overlay
   (é a silhueta encolhida à metade) e a fila dela passa a se abrir também.
3. **A row lembra:** clique noutro objeto e volte — a linha continua mostrando o número.

⚠️ **Já aprovados** (não precisam de re-smoke): `=33` e `=34`.

---

## 7 — O que fica ABERTO (nomeado, não contrabandeado)

- ⚠️ **Espelhar a zona não espelha o vento.** Medido: `scale.x = -1` produz deslocamento
  `(6,73, 0)` — **idêntico** ao da zona não-espelhada; com rotação 45° as duas dão `(4,76,
  4,76)`. O frame honra a **rotação** e ignora a **reflexão**, o que **contradiz o precedente do
  W-Offset** (*"escala SINCADA, não `abs` — offset é POSIÇÃO ⇒ flip espelha"*). A pergunta é de
  produto — *virar o sprite de uma esteira deveria virar a correia?* — e **não foi construída sem
  pedido**.
- ~~**Falloff dentro da área**~~ — **FECHADO** (W-AreaFalloff). A pergunta *"de que ponto se
  mede o raio numa zona que não é redonda?"* tinha resposta melhor que escolher um ponto:
  mede-se a **fração do caminho do centro até a BORDA**, que vale `1` em toda direção e em toda
  forma, e cujas curvas de nível são a própria silhueta escalada.
- **Perfis de falloff** (smoothstep, inverso-quadrado) — outra CURVA sobre a mesma régua; é um
  knob de MODO, não um número novo, e só vale com pedido.
- **3 mutações sobrevivem por projeto** (14+13 rodadas, 26 sangram), todas documentadas no
  fonte — a terceira é a irmã exata da primeira: pôr o `AreaFalloff` no `any` da ponte deixa
  tudo verde, porque `zone_effect` já recusa a zona inerte. As duas do W-AreaFrame: pôr o marcador no `any` da
  ponte (o `zone_effect` já recusa a zona inerte ⇒ é higiene, não correção) e a row ignorar o
  valor autorado (a seleção de um `seg_row` é um **realce na cena**; o testkit expõe valores de
  widget, não estado de pintura — o lado da FONTE está gateado no shell).
