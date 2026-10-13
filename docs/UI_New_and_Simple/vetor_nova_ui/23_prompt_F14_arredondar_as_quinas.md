═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F14: ARREDONDAR AS QUINAS EM TODA FORMA QUE AS TEM — ATÉ O SMOKE DO DONO
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie). Modo L.

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: FASE 0 → 1 → 2 do
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com MODULO=UIUX.
Estado em 13/10: F0–F13 no `main` e no GitHub (CI verde, run 38340931430). A linha está IGUAL ao
`main` (`git reset --keep main` feito; cherry 0). Siga sem reset; se o `main` andou: `git rebase main`.

LEIA, NESTA ORDEM (e só isto antes de agir):
  1. docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-12_F13_SELECIONAR.md — o
     que a F13 entregou e mediu (o *Smooth* sobre o contorno JÁ FEITO, o *Cutout*/*Straight* rectos
     sob envergadura, as recusas medidas).
  2. As regras R1–R9 do topo de docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F10.md
     (R1 = ~150 chamadas ⇒ commit + handoff de continuação ≤ 8 KB + PARE).

O PEDIDO DO DONO (13/10) — palavra por palavra:
 *«todas as setas parecem funcionar muito bem! Vamos deixar como estão. Um ajuste legal que ainda está
  ausente em muitas formas é o ajuste que arredonda as quinas. Poderia existir em todas as formas que
  tem quinas, desde que não crie problemas»*

JÁ DECIDIDO (não re-pergunte): as setas ⇕ ficam como estão; a seta dobrada e o ✓ ficam como estão.

O LEVANTAMENTO (13/10, `explorador`, a CONFIRMAR no código antes de agir):
- 49 formas; JÁ arredondam: RoundRect (Radius + desvios + CornerStyle), Polygon (Radius + CornerStyle),
  Star (Tip/Notch round — **8/8 campos, sem vaga**), Pie e Ellipse-fatia (`Corner round`, F12,
  `round::round_slice_corners`), SpeechRect (o corpo), ArrowBent (só o cotovelo), Cloud (o filete do vale).
- CANDIDATAS (quinas vivas no default): Rectangle(?), Segment, ArrowRight, ArrowDouble, ArrowBent (as
  pontas da cabeça), Chevron, Diamond, Parallelogram, Trapezoid, TrapezoidFlip, HexagonFlat, Document
  (topo), Delay (2 à esquerda), Display, PredefinedProcess, OffPage, SpeechRect (o bico), Bolt, Tag,
  Gear (os dentes), Cross, Check (as pontas), Banner, IsoCube, IsoPyramid; a verificar: Shield, Bone,
  RopeSegment, Cylinder.
- DUVIDOSAS (agulha/cúspide — a lei decide, medida): Burst, Drop, Heart (a cúspide), Moon (os chifres),
  Thought, SpeechOval (o bico), IsoCone (o ápice). Abertas/sem quina: Line, Arc, Spiral, Brace,
  NoteBracket, Pill, Junction, Ellipse inteira.
- O motor genérico EXISTE: `ph2d_vec_scene::corner_live::round_authored_corners(verts, closed)` (o tecto
  por quina `min(r/tan(θ/2), ½ corda mais curta)`, G1 junto de curvas, satura sem pânico) — ⚠️ o
  `round_slice_corners` só olha `p.verts`: um genérico novo itera os SUB-CONTORNOS também.
- Capacidade: `MAX_SHAPE_FIELDS = MAX_SHAPE_VALUES = 8`; só a Star está cheia.

ESCOPO — produto final, por esta ordem (cada passo fotografado antes de passar):
  A. ⭐ UMA LEI, UMA PORTA: um campo APENSO, neutro em zero (`radius("Corner round")`, `FieldUnit::Px`, o
     MESMO rótulo da pizza — a tradução já existe), lido por uma função ÚNICA que arredonda as quinas
     VIVAS do contorno cozido de qualquer forma (todos os sub-contornos), por cima do `cook` — o mesmo
     sítio onde o *Smooth* da F13 corre sobre o contorno já feito. Nada de cópia por forma.
     O Rectangle: decida MEDINDO contra o produto do dono (o Affinity: o rectângulo tem raio de canto) e
     contra o RoundRect que já existe — e escreva o porquê.
  B. ⭐ «DESDE QUE NÃO CRIE PROBLEMAS» = um gate: o censo `no_cooked_shape_loops_or_hooks`
     (`stroke_census_tests.rs`) varre o campo novo DE GRAÇA — e mais um censo do catálogo: TODA forma
     com quina viva no default tem quem a arredonde, com a lista de excepções ESCRITA (forma · porquê ·
     a medida). As duvidosas: meça (a agulha a raio 0→máx; a cúspide) e decida — entra com tecto
     honesto, ou fica de fora com a razão na lista. Nunca um raio que crie laço, gancho ou lasca.
  C. A UI: cada campo novo tem o seu ⇕ na fila (`shape_handles_table.rs`, `Spot::Rail` + `Law::Steps`; o
     gate `what_was_on_the_card_is_an_arrow_on_the_side` exige ZERO campos sem alça). Onde houver um
     ponto natural (o ⊙ dentro do canto, como no rectângulo arredondado — `HandleDesc::in_corner`),
     prefira-o ao ⇕ — e UM controlo por campo (o gate `no_field_has_an_arrow_and_another_handle`).
  D. O Motion (`ph2d-node-motion-shape` `ShapeParams`, `motion_shape_gen`) e a persistência: o campo
     apenso lê `0` dos saves antigos — confirme no código e no `PROJECT_SCHEMA`
     (`python3 scripts/schema-recount.py`; um número que SOMA entre linhas CONTA-SE).
  E. A cena de smoke NOVA (o número conta-se em `crates/ph2d-app-vec/src/vector_mode.rs`, hoje a última
     é a `=20`): uma fila das formas candidatas com o raio subido pelo MESMO gesto da shell — a foto
     mostra os cantos redondos e nenhum laço.

FECHO (DIRETRIZ §1.5.9, nesta ordem): mutação de cada cláusula nova (`mutacao`, ANTES do portão) ·
portão batched (`verificador`: `nextest-impacted.sh` + clippy do `ship.sh` + `fmt --all --check`) ·
handoff de INTEGRAÇÃO novo (F14, cita o da F13) · CLAUDE.md §5.1 UI/UX («Último:», smoke `…6..21`,
≤ 700 bytes) · `doc-index.sh` · `agent-loop-profile.sh` · `rm -rf target/*/incremental` · binário
smoke 2× (a 2.ª saída no handoff) · o SMOKE ao dono, curto, sem jargão, em passos numerados.
⚠️ Lições da F13: o `check` estreito NÃO compila os testes — um teste da shell que lia o retorno de uma
função mudada só apareceu no `check --workspace --all-targets` da integração (corra-o antes do fecho);
fotos só com a placa livre; NUNCA `pkill -f`.
NÃO integra, NÃO pusha.
═══════════════════════════════════════════════════════════════════
