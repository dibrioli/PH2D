# Timeline (módulo geral do app) — Briefing da linha `line/anim` (núcleo de dados, crate `ph2d-anim`)

> **Esta pasta = o módulo Timeline GERAL do app** (transporte + dope-sheet que anima qualquer
> propriedade: sprite, layer do painter, param de motion node, vetor). Ela integra ao Motion Nodes
> mas **não é** a timeline do módulo motion. Este primeiro doc abre a linha que entrega o
> **núcleo de dados** (`ph2d-anim`) — a fundação que a UI da timeline (doc futuro nesta pasta) e o
> Motion Nodes vão consumir. A UI/transporte é módulo à parte, ainda não planejado.

**Data:** 2026-07-07 · **Status:** pronto para abrir em linha (Modo L) · **Regime:** satélite
(fan-out puro — só LÊ contrato congelado) · **Base:** [`00_estudo`](../Motion%20Nodes/00_estudo_estado_da_arte.md)
Camada 1 + [`01_plano`](../Motion%20Nodes/01_plano_modulo_motion_nodes.md) · **Modelo de abertura:**
[`MODELO_ABERTURA_LINHA.md`](../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md) (fiel).

> **Escopo real (Enio, 2026-07-07):** esta linha entrega o **núcleo de dados de animação do APP
> INTEIRO** — o modelo Track/Key/Clip/Curva/Easing que servirá à **timeline geral do app** (um
> módulo próprio, futuro: transporte + dope-sheet que anima QUALQUER propriedade — transform de
> sprite, layer do painter, param de motion node, propriedade vetorial) **e**, pela mesma via, à
> integração com o Motion Nodes. NÃO é a timeline do módulo motion (aquela é um caso de uso; esta
> é o dado compartilhado por baixo de todos). O modelo é **agnóstico de alvo** por construção
> (`AnimValue` é genérico), então a mesma crate serve todos os consumidores sem mudança.

---

## §0 — Por que esta linha (contexto que o agente NÃO precisa, mas o Enio sim)

O motor de animação da PH2D **já está congelado e provado** (ADR-0030..0039). O que falta é a
**camada de produto de animação** — e a peça que TODO o resto dela consome é o **modelo de dados de
keyframes/curvas** (Camada 1 do `00_estudo`). Ele é, ao mesmo tempo, (a) **o núcleo compartilhado
de tudo que anima no app** — a **timeline geral** (dope-sheet app-wide, módulo próprio futuro), o nó
`motion.clip`, a state-machine, e qualquer propriedade animável (sprite/layer/param/vetor) — e (b)
**isolável como biblioteca pura, sem UI, sem shell, sem seam de dispatch** — ou seja, **baixo risco
e rápida**, sem nenhuma das 4 causas da semana perdida no Painter (costura não-testada,
"audit"=compilar, fio órfão, alvo irrefutável). Entregar ESTE núcleo primeiro é o que destrava a
timeline geral e o motion sem construir dois modelos de animação divergentes.

**Gap verificado no código (não em doc):** os traits de animação estão **congelados** em
`ph2d-vector-traits` (ADR-0056), mas as **únicas impls são mocks que retornam constante**:

| Símbolo | Onde (congelado) | Estado hoje |
|---|---|---|
| `trait AttributeEvaluator { fn sample(&self, t: f64) -> AnimValue }` | [`attribute_evaluator.rs:42`](../../crates/ph2d-vector-traits/src/attribute_evaluator.rs#L42) | só mock ([`mocks.rs:35`](../../crates/ph2d-vector-traits/src/mocks.rs#L35), retorna fixo) |
| `trait AnimationCurveSampler { fn at(&self, t: f64) -> AnimValue }` | [`animation_curve.rs:28`](../../crates/ph2d-vector-traits/src/animation_curve.rs#L28) | só mock ([`mocks.rs:96`](../../crates/ph2d-vector-traits/src/mocks.rs#L96), lerp trivial) |
| `enum AnimValue` (6 variantes, `#[non_exhaustive]`, **cap congelado**) | [`anim_value.rs:24`](../../crates/ph2d-vector-traits/src/anim_value.rs#L24) | pronto (+ `LinearInterp` já implementado) |

`crates/ph2d-anim` **não existe**. Esta linha o cria e transforma o contrato de *mockado* em
*real*.

**Disjunção das 4 linhas vivas (por que não colide):**
- **Motion Nodes** (`line/MotionNodes`): **defere a timeline** por decisão de produto ([`01_plano`
  §Decisões #3](../Motion%20Nodes/01_plano_modulo_motion_nodes.md)); só reserva o slot `motion_timeline_slot` + a
  ordem `socket > keyframe > literal`. **Zero código compartilhado agora.**
- **Vector** (`line/Vector`): congelou os traits mas não implementou o store. Esta linha não toca
  `ph2d-vector-*` (só depende de `-traits` como leitor).
- **Painter / Audio:** domínios não relacionados.
- **Timeline geral do app:** é um **módulo próprio futuro** (UI de transporte + dope-sheet), NÃO uma
  linha viva hoje e NÃO parte deste escopo. Esta linha entrega só o **dado** que ela vai consumir.

`ph2d-anim` é **crate-folha** (satélite): nada depende dela ainda; ela só depende de foundational
já congelado. Não é nó, não é tool, não é painel → **nenhum codegen** (`ph2d-{node,tool,panel}-sync`)
a toca, **nenhum edit central**, o glob `crates/*` do workspace a inclui sozinho.

---

## §1 — BLOCO 1: abertura da linha (cole como 1ª mensagem)

> Fiel ao `MODELO_ABERTURA_LINHA.md`. O agente cria a worktree, faz setup, lê DIRETRIZ/DIRETIVA,
> responde **"Linha pronta. Aguardo a tarefa."** e PARA. A tarefa (§2) vem na mensagem seguinte.

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA PARALELA — Modo L        (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é um agente-de-linha. Sua linha: line/anim

O nome após "line/" acima é o NOVO MÓDULO. Todo o resto deste briefing
deriva dele — nos comandos ele aparece como $MODULO: substitua por
"anim" ao executar (env não persiste entre chamadas de shell).
Sua branch:    line/anim
Sua worktree:  Worktrees/line-anim/   (você vai criá-la agora)

FASE 1 — SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. bash scripts/hw-profile.sh
      → tem que dizer `workstation`. Disse `constrained`? PARE:
        esta máquina opera em Modo C, linhas são proibidas aqui.
2. git status -sb
      → você está na RAIZ do repo primário, branch main. Arquivos
        M/?? alheios podem existir (outros agentes): NÃO toque neles.
3. git pull --ff-only origin main
      → falhou (rede/divergência)? Siga com o main local e reporte.
4. mkdir -p Worktrees
   git worktree add -b line/anim Worktrees/line-anim main
      → a branch line/anim já existe (linha reaberta)? Então:
        git worktree add Worktrees/line-anim line/anim
        e em seguida, DENTRO dela: git rebase main
5. cd Worktrees/line-anim
   git branch --show-current        # DEVE imprimir line/anim
6. cargo check -p ph2d-core
      → warm-up do target/ próprio desta worktree; o 1º build é frio
        (minutos). NÃO otimize/investigue a demora — é esperada.
7. bash scripts/mergiraf-setup.sh    # merge sintático p/ foundational (ADR-0107)
      → idempotente, 1× por máquina (config vai no .git comum). Falhou por
        "mergiraf not found"? NÃO é bloqueio: git faz fallback pro merge
        embutido. Reporte a linha do ✗ e siga (Enio instala depois).
8. Leia INTEIRAS (dentro da worktree):
      docs/IntegracaoMultiAgente/DIRETRIZ.md            → §0, §1.5, §2, §6
      docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md  → tudo
        (e RELEIA a cada passo do trabalho, como ela manda)
9. Reporte: "Linha do novo módulo pronta em Worktrees/line-anim.
   Aguardo a tarefa." — e PARE. A tarefa vem na próxima mensagem.

REGRAS PERMANENTES DA SESSÃO (valem até o fim, sem exceção):
A. TODO read/edit/git/cargo acontece DENTRO da sua worktree
   (Worktrees/line-anim/). A raiz do repo é o checkout primário
   compartilhado: o MESMO path relativo existe nas duas árvores —
   editar crates/... na raiz é editar a árvore ERRADA. Na dúvida,
   `pwd` antes de editar.
B. Edite a(s) pasta(s) do novo módulo à vontade. Foundational
   (ph2d-core/editor-core/tokens/host/…) É PERMITIDO sob o protocolo
   testado (ADR-0107): a integração roda scripts/foundational-integrate.sh
   (gate da árvore combinada) e o Mergiraf funde o resíduo textual. PARE
   e reporte ao Enio SÓ se: (a) for contrato congelado (§4, exige ADR),
   ou (b) o rebase conflitar em código FORA dos seus arquivos (colisão de
   mesmo-símbolo com outra linha). Nunca negocie com outra linha.
C. Commits locais frequentes: git commit --no-verify (fast mode).
   NUNCA push. NUNCA --force. NUNCA git add -A.
D. git rebase main no início de cada jornada e antes de integrar.
   Conflito em Cargo.lock ou arquivo GERADO (registry-init): NUNCA
   resolva na mão — regenere (DIRETRIZ §1.5.5). Conflito em código
   fora da sua pasta = você violou a regra B.
E. Fechamento do módulo = gate batched (DIRETRIZ §6.6.A.2: nextest-
   impacted + clippy --all-targets + audit ≥2 lentes + DIRETIVA §3-§5)
   e SÓ ENTÃO a integração (DIRETRIZ §1.5.3) — UM comando:
       bash scripts/foundational-integrate.sh
   Ele faz: rebase main → re-sync (tool/node) → staleness → gate da
   árvore COMBINADA (cargo check --workspace se a linha tocou
   foundational; senão -p das crates mudadas) → nextest-impacted →
   merge --ff-only no primário. Aborta com a orientação certa em cada
   falha. --ff-only falhou = outra linha integrou antes → só RE-RODE o
   script (rebase+retesta). Módulo verde que não integrou NÃO fechou.
F. Ship (ship.sh + push + babysit CI) SÓ se o Enio disser que você
   fecha a ÚLTIMA integração da jornada (DIRETRIZ §1.5.4 + §8).
G. UI canônica sempre: zero hex, zero f32 literal de UI, tudo por
   tokens/i18n (CLAUDE.md §0.3). Contratos congelados (CLAUDE.md §6)
   são intocáveis nesta linha.
═══════════════════════════════════════════════════════════════════
```

---

## §2 — BLOCO 2: a tarefa (cole DEPOIS do "Linha pronta")

```
═══════════════════════════════════════════════════════════════════
TAREFA — line/anim · crate NOVA `ph2d-anim` (keyframe store + curvas)
═══════════════════════════════════════════════════════════════════

OBJETIVO (1 frase):
Criar a crate-biblioteca `crates/ph2d-anim` que dá impls REAIS aos dois
traits de animação HOJE mockados — um keyframe store (`Track`) que
implementa `AttributeEvaluator` e uma curva editável (`AnimCurve`) que
implementa `AnimationCurveSampler` — mais easing e um tempo racional.
Zero UI, zero shell, zero nó/tool/painel. É a Camada 1 do estudo.

──────────────────────────────────────────────────────────────────
PASTA EXCLUSIVA (só aqui você escreve):
  crates/ph2d-anim/
Glob workspace.members cobre — NÃO edite Cargo.toml raiz.
Crate-FOLHA: nada depende dela ainda; ela não é registrada em lugar
nenhum (não é ph2d-node-*/ph2d-tool-*/ph2d-panel-* → nenhum codegen
sync a toca). Ninguém a compila a não ser `-p ph2d-anim`. Isso é
esperado e correto — a fiação no app (nó motion.clip / timeline) é de
LINHAS FUTURAS, fora deste escopo.

──────────────────────────────────────────────────────────────────
CONTRATO QUE VOCÊ IMPLEMENTA (já congelado em ph2d-vector-traits — você
o consome como LEITOR; NÃO edita esse crate):

  trait AttributeEvaluator { fn sample(&self, t: f64) -> AnimValue; }
      crates/ph2d-vector-traits/src/attribute_evaluator.rs:42
  trait AnimationCurveSampler { fn at(&self, t: f64) -> AnimValue; }
      crates/ph2d-vector-traits/src/animation_curve.rs:28
  enum AnimValue { Float(f32), Vec2, Vec3, Color(OklchColor), Bool, Enum(u32) }
      crates/ph2d-vector-traits/src/anim_value.rs:24
      → já traz `impl LinearInterp for AnimValue` (lerp por variante, hue
        OKLCH em arco curto). REUSE — não reimplemente lerp de valor.

Semântica das duas camadas (do doc dos traits):
  • `Track: AttributeEvaluator` = resolve um PARÂMETRO no tempo absoluto
    `t` (segundos). Guarda keys ordenadas; sample(t) acha o segmento,
    normaliza pra u∈[0,1], aplica a função de tempo do segmento, e faz
    LinearInterp::lerp entre os valores das 2 keys.
  • `AnimCurve: AnimationCurveSampler` = uma CURVA editável de 1ª classe
    (o objeto do futuro graph-editor). at(t) amostra a forma.
  Um `Track` pode conter `AnimCurve`s por segmento; entregue os DOIS
  impls (o objetivo é matar os dois mocks).

──────────────────────────────────────────────────────────────────
MODELO DE DADOS A CONSTRUIR (nomes-guia; internals são seus):

  RationalTime { num: i64, den: u32 }   // src/time.rs
    - Storage de tempo SEM drift de float (padrão OpenTimelineIO).
    - Ctors: from_seconds(f64) aproximado, from_frame(frame:i64, fps:u32).
    - to_seconds() -> f64 (só na BORDA, ao chamar sample()).
    - Ord/Eq por valor normalizado. Mantenha MÍNIMO — só o que Track/Clip
      precisam (construir, comparar, to_seconds). NÃO reimplemente
      aritmética racional completa do OTIO (gold-plating).

  Interp (por-segmento):                 // src/curve.rs
    Hold                     // stepped: valor da key anterior até a próxima
    Linear                   // u direto
    Eased(Easing)            // u passa por um preset
    Bezier { x1,y1,x2,y2 }   // cubic-bezier CSS/AE (Linear/Eased são casos)
    → mapa: sample(t_abs) → segmento → u∈[0,1] → timing(u)=v∈[0,1] →
      LinearInterp::lerp(k0.value, k1.value, v). Hold = sem interp.

  Easing (enum + fn eval(u)->f64 + is_deterministic()->bool):  // src/easing.rs
    - Porte ~20 presets de `simple_easing` (MIT) como CÓPIA INTERNA com
      atribuição no header (NÃO adicione dep crates.io — disciplina de
      stack §5 da SKILL; e a DIRETIVA §1 manda portar o algoritmo de ref).
    - POLINOMIAIS (transcendental-free → is_deterministic()=true):
      Linear, Quad, Cubic, Quart, Quint, Back, Bounce (In/Out/InOut).
    - TRANSCENDENTAIS (is_deterministic()=false): Sine, Expo, Circ,
      Elastic. Animação é presentation → ISENTA de HR-5 (membrana
      ADR-0030), mas o flag existe pra um consumidor gameplay futuro
      rejeitar as não-det (espelha `Func::is_deterministic()` do ph2d-expr).
    - cubic-bezier: avalie por subdivisão/Newton (transcendental-free →
      determinístico). Referência de spec: `sampleKeyframeTrack` do
      MiniCavalryV2 (read-only) é a spec executável.

  Key { t: RationalTime, value: AnimValue, interp: Interp }     // src/track.rs
  Track { keys: Vec<Key> (ordenadas por t) }
    - impl AttributeEvaluator: sample(t_sec) → binary search do segmento.
    - Fora de faixa: t < 1ª key → valor da 1ª; t > última → valor da última
      (clamp/hold nas pontas). 0 keys → decida um default explícito e
      documente (NÃO um corpo vazio silencioso — DIRETIVA §2).
    - HR-3 (zero-alloc no hot path): sample NÃO aloca. Além da busca
      binária, mantenha um CURSOR monotônico por track (índice do último
      segmento) pra playback O(1) amortizado. Track/Clip devem ser
      Send+Sync (dados planos) → satisfazem
      Box<dyn AttributeEvaluator + Send + Sync> no bridge futuro.

  AnimCurve { ... } impl AnimationCurveSampler                  // src/curve.rs
    - A curva editável standalone (graph-editor). at(u)->AnimValue.

  AnimTarget(u64)  // src/clip.rs — identidade de alvo OPACA e APP-GERAL
    - A quem um Track escreve: "rotation da sprite X", "opacity da layer Y",
      "param P do nó Z", "propriedade vetorial W". `ph2d-anim` NÃO sabe o que
      é — trata como chave opaca (HR-8: handles opacos; lição de data-binding
      do Rive no 00_estudo §4). Quem MAPEIA chave→setter é o CONSUMIDOR (a
      timeline geral / o motion bridge), fora desta crate. Isso é o que mantém
      `ph2d-anim` app-geral E isolada: o mesmo Clip anima qualquer coisa.
    - (Alternativa aceitável: `Clip<T>` genérico no tipo de alvo. Escolha a
      opaca u64 salvo se o genérico sair mais limpo nos testes — decida e
      documente; não deixe em aberto.)

  Clip { tracks: Vec<(AnimTarget, Track)>, duration: RationalTime } // src/clip.rs
    - Coleção nomeada de tracks + duração. É o que a timeline geral E o futuro
      nó `motion.clip` amostram no playhead (NÃO construa nenhum dos dois aqui).

  Layout sugerido (cada arquivo < 700 LOC — ADR-0105):
    src/lib.rs (#![forbid(unsafe_code)] + re-exports)
    src/time.rs · src/easing.rs · src/curve.rs · src/track.rs · src/clip.rs
    tests/ (golden · zero-alloc dhat · determinismo)

──────────────────────────────────────────────────────────────────
DECISÕES DE DESIGN JÁ TOMADAS (não re-litigar — rationale no 00_estudo):
  1. Tempo de STORAGE = RationalTime; f64 SÓ na borda de sample()/at().
     (evita drift; base de exports/replay futuros — OTIO.)
  2. Cor = reusar `LinearInterp` (OKLCH hue arco-curto já pronto no enum).
  3. Easing = cópia interna MIT (sem dep nova); polinomiais determinísticos.
  4. Híbrido: keyframe é DADO amostrável, não subgrafo ("tudo é nó" perde
     o artista — ADR-0038 princ. 5). Você entrega só o dado + sampler.

Cargo.toml — deps MÍNIMAS (todas já no workspace):
  ph2d-vector-traits  (os traits + AnimValue + LinearInterp)
  ph2d-color          (OklchColor, p/ construir/testar tracks de cor)
  glam                (Vec2/Vec3)
  [dev-dependencies] dhat  (gate zero-alloc; espelhe os tests/*_no_alloc.rs
                            existentes — ex. ph2d-ecs/tests/propagate_no_alloc.rs)

──────────────────────────────────────────────────────────────────
VOCÊ NÃO TOCA (fora de escopo OU congelado):
  • ph2d-vector-traits / -doc / qualquer ph2d-vector-* — CONGELADO
    (gate architecture_vector_contract_surface escaneia -doc + -traits).
    Precisa de uma 7ª variante de AnimValue? PARE e reporte ao Enio
    (amendment ADR-0056 — contrato congelado, regra B).
  • ParamSpec / NodeManifest (só f32) — CONGELADO (ADR-0039). Animar
    Vec2/cor via param é problema da timeline futura, não seu.
  • Playhead / transporte / shell bridge — é o M0 do Motion Nodes (OUTRA
    linha, foundational). Não crie.
  • Nó motion.clip, timeline GERAL do app (UI/dope-sheet/transporte),
    resolução AnimTarget→setter, ph2d-expr vetorizado — módulos/linhas
    FUTURAS. Se sentir vontade de fiar qualquer um, PARE: fora de escopo.
    Você entrega SÓ o dado + o sampler; ninguém escreve na cena por aqui.

──────────────────────────────────────────────────────────────────
NOTA DE COORDENAÇÃO (design, NÃO bloqueio — não há código comum hoje):
Dois consumidores futuros amostram este dado, e o modelo deve servir os DOIS
sem divergir:
  • Timeline GERAL do app (módulo próprio): anima alvos arbitrários por
    AnimTarget (sprite/layer/param/vetor). Por isso o alvo é opaco e app-geral
    (acima) — não assuma nada motion-específico.
  • Motion Nodes: reserva a ordem `socket > keyframe > literal` e um
    `motion_timeline_slot`. O "keyframe" dessa ordem = amostrar um Track/Clip
    por t. Mantenha os shapes Key/Track/Clip + RationalTime CONSISTENTES com
    "keyframe é uma fonte de sinal amostrável por t" pro seam futuro (nó
    motion.clip + ADR do ParamSpec) encaixar limpo.
Não precisa alinhar com nenhuma outra linha agora — só não feche portas
(alvo agnóstico, tempo racional, sampler puro).

──────────────────────────────────────────────────────────────────
DoD — DEFINIÇÃO DE PRONTO (esta crate NÃO tem UI → sem seam/behavioral
test; o gate architecture_interactive_crate_has_behavioral_test NÃO se
aplica. A aceitação é a SUÍTE DE TESTES, no espírito do golden-test de nó):

  [ ] Track constante: sample(qualquer t) = valor fixo (mock virou real).
  [ ] Linear 2-keys: sample no meio = ponto médio exato (Float e Vec2).
  [ ] Eased: preset conhecido (ex. cubic-bezier .42,0,.58,1) em u=0.5 →
      GOLDEN numérico; e todo preset satisfaz e(0)=0, e(1)=1.
  [ ] Hold/stepped: logo antes da próxima key = valor da anterior.
  [ ] Clamp de pontas: t<1ª → 1ª; t>última → última.
  [ ] Track de cor: hue em arco curto (350°→10° = 20°), via LinearInterp.
  [ ] Determinismo: mesma (track, t) → AnimValue bit-idêntico (repeat).
      RationalTime.to_seconds roundtrip estável; from_frame exato.
  [ ] Zero-alloc (dhat): loop de playback sample() em N frames = 0 allocs
      (HR-3). Espelhe o padrão per-crate tests/*_no_alloc.rs de outra crate
      (ex. ph2d-ecs/tests/propagate_no_alloc.rs, ph2d-audio/tests/no_alloc_render.rs).
  [ ] AnimCurve implementa AnimationCurveSampler de verdade (2º mock morto).
  [ ] Doctest/exemplo curto: montar um Track, amostrar, mostrar o valor.
  [ ] clippy --all-targets -D warnings + fmt (rustup run <pin> cargo fmt).

Antes de integrar: gate batched (regra E) + releia DIRETIVA §3 (audit ≠
compilar: preencha o TEMPLATE por claim) e §5. Depois:
  bash scripts/foundational-integrate.sh
(crate-folha, não tocou foundational → ele roda -p ph2d-anim + impacted.)

QUANDO TERMINAR, reporte:
  "ph2d-anim pronto. Commit local: <sha>. Traits reais: AttributeEvaluator
   (Track) + AnimationCurveSampler (AnimCurve). cargo test -p ph2d-anim
   verde (golden + dhat 0-alloc + determinismo). Mocks substituídos.
   Integrado via foundational-integrate.sh: <sim/não>."
═══════════════════════════════════════════════════════════════════
```

---

## §3 — Notas do Enio (fora dos blocos)

- **Ordem de colar:** BLOCO 1 → espere "Linha pronta" → BLOCO 2.
- **Por que é seguro rodar em paralelo às 4 linhas:** crate-folha, satélite, só lê congelado —
  não há símbolo comum com Vector/Painter/Audio/MotionNodes. A integração é `--ff-only` como as
  outras; se outra linha entrar antes, o script re-roda sozinho.
- **O que esta linha DESTRAVA depois:** a **timeline geral do app** (dope-sheet app-wide, módulo
  próprio), o nó `motion.clip` (fan-out) e o `motion.state-machine` passam a ter um store de
  keyframes real por baixo — em vez dos mocks constantes de hoje. Um só modelo de dados serve os
  três, então não há risco de dois sistemas de animação divergentes.
- **Quando fechar de vez:** só quando não houver mais wave de `ph2d-anim` prevista (curvas extras,
  mais easings). Enquanto houver, a linha fica aberta pra próxima jornada.
