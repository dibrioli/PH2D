# 15 — Handoff de CONTINUAÇÃO: próxima linha do Motion (domínio de valor + resto do M2)

**Data:** 2026-07-11 · **De:** agente que fechou o domínio de valor (docs 12–14, integrado no main
`1c7c9a22`) · **Para:** o próximo agente-de-linha do Motion Nodes. **Você NÃO vai reusar a linha
anterior — você abre uma linha nova do main** e segue a implementação. Este doc te diz onde
estamos, o plano, o próximo passo e as convenções pra você não re-derivar nada.

---

## 0. LEIA PRIMEIRO (nesta ordem)

**Processo (canônicas — o Enio pediu explicitamente):**
1. [`DIRETIVA_IMPLEMENTACAO.md`](../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) — o antídoto das 4 causas da semana perdida. **A CADA passo.** Regra-mãe: *verde-de-compilação é velocidade; no audit vale ZERO.*
2. [`DIRETRIZ.md`](../IntegracaoMultiAgente/DIRETRIZ.md) — §0 (sanity), §1.5 (Modo L / worktrees), §2 (triagem), §3.A (drop-crate/fan-out), §6 (velocidade), §7 (git). **Não leia inteiro** — use o roteador.
3. [`GUIA_JORNADA_MODO_L.md`](../IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md) — como rodar a jornada (abrir a linha, fechar, quem faz o ship).
4. [`MODELO_ABERTURA_LINHA.md`](../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md) — o bloco de setup da worktree (você mesmo cria a sua).

**Técnico (deste módulo):**
5. [`01_plano_modulo_motion_nodes.md`](01_plano_modulo_motion_nodes.md) — **O PLANO** (M0→M5, roadmap §3, contrato §4).
6. [`12`](12_dominio_de_valor_nota_adr.md) · [`13`](13_lfo_map_range_nota_adr.md) · [`14`](14_sample_hold_instance_field_nota_adr.md) — as 3 fatias do domínio de valor já feitas (leia as 3; a §5 de cada uma tem o follow-up nomeado).
7. `SKILL_Stack_PH2D_Definitiva.md §11.13` (Motion) + `CLAUDE.md §5` (estado por-módulo) + `§6` (contratos congelados).
8. Referência viva **read-only:** `/home/enio/Documentos/Recursos/Nodes/MiniCavalryV2` (semântica/UX do MVP).

---

## 1. ⭐ A REGRA DE OURO (inegociável — vale ANTES de escrever qualquer nó)

**Antes de implementar QUALQUER nó, você faz duas descobertas:**

1. **Padrão-ouro da indústria + melhor algoritmo.** Varra as fontes primárias (TouchDesigner CHOP,
   Houdini VEX/VOP, Cavalry, Nuke, Max/MSP, vvvv, Faust, Rive, papers) e descubra **a semântica
   correta e o melhor algoritmo** — não invente, não chute. Depois porte **por semântica**, com os
   melhoramentos embutidos, **transcendental-free (HR-5)** na produção. As 3 fatias anteriores
   fizeram exatamente isso (ex.: `map_range` = o `fit` do Houdini com clamp-no-`t`; a onda do `lfo` =
   aproximação parabólica do oscillator, ~0.09% de erro, só multiply+abs; o Random do `instance_field`
   = hash stateless Jarzynski/Olano). **Documente a pesquisa na nota-ADR** (o "porquê", com as fontes).

2. **O melhor NOME pro nó — o mais expressivo e intuitivo.** O nome é uma decisão de design, não um
   detalhe. Pesquise como as ferramentas maduras chamam a coisa, e escolha o nome que um artista
   entende de cara. Siga a **convenção de prefixo (doc 09 §4.2):**
   - `pulse.*` = plumbing abstrato de pulso/valor (produtor/redutor/ponte que NÃO desenha nada visível): `pulse.beat`, `pulse.counter`, `pulse.threshold`, `pulse.sample_hold`.
   - `value.*` = produtor/transformador de **valor** (campo escalar): `value.lfo`, `value.map_range`, `value.instance_field`.
   - `motion.*` = behaviour **VISÍVEL** que escreve um canal de transform: `motion.drive`, `motion.strobe`, `motion.step`.
   - Nomes por extenso e claros (`sample_hold`, não `sah`; `instance_field`, não `id_field`; `map_range`, não `fit`).

> Essas duas descobertas são o **primeiro passo** de cada nó. Sem elas, pare — você está prestes a
> reimplementar algo errado com um nome ruim.

---

## 2. ONDE ESTAMOS (estado do módulo no main `1c7c9a22`)

O módulo Motion está **vivo e usável** (M0/M1 + neck do M2 fechados, editor de grafo funcionando,
transporte, ~34 nós registrados). O que a linha anterior acabou de integrar:

- **Neck M2.N2/N3/N4 (doc 11):** `Cook::checkpoint()/restore()` + `CheckpointRing` + `scrub_to_scoped`/
  `advance_or_scrub_scoped` no pump → **scrub para trás determinístico** (o pré-requisito que faltava).
  Contrato de nós **intacto** (só métodos inerentes em `cook.rs`).
- **Domínio de VALOR (docs 12–14), 3 fatias:**
  - **Fatia 1 (12):** `pulse.counter` (redutor PURO pulse→value) + `motion.drive` (consumidor value→canal, com a **regra de broadcast 1→N**).
  - **Fatia 2 (13):** `value.lfo` (produtor contínuo) + `value.map_range` (o `fit` — cola universal).
  - **Fatia 3 (14):** `pulse.sample_hold` (o `sah~`, fecha o combo `LFO→SampleHold→drive` do doc 09) + `value.instance_field` (o ÚNICO que MINTA campo len-N da identidade — Index/Ramp/Random).
- **Cena boot** (`shells/desktop/src/motion_demo_strobe.rs`, 15 nós): 3 cadeias de valor demonstradas
  — X broadcast (beat→counter→drive), Y sample-and-hold (lfo→sample_hold→map_range→drive), Size
  gradiente por-elemento (instance_field→map_range→drive), tudo piscando no beat. **Smoke aprovado.**

**O tipo de valor já existe e é usado:** `PortType(Instances, Scalar, Frame)`, coluna `v` (dual
contínuo do pulse `(…, Event)`). **Contratos congelados (§6) intocados** — o domínio de valor é
100% **fan-out aditivo (caminho A)**; o gate `architecture_contract_surface` conta só
`NodeOp`=2/`OpResolver`=1/`NodeManifest`=8.

---

## 3. O PLANO (doc 01 §3) e onde ele nos coloca

Roadmap: **M0** (foundational+editor) → **M1** (editor usável + ~30 nós) → **M2** (tempo+dinâmica) →
**M3** (distribuições/deformers/polish) → **M4** (rig+FX) → **M5** (GPU CookPlan).

**Estamos na cauda do M2** (o fan-out de nós de dinâmica). O neck do M2 está fechado; o domínio de
valor é uma sub-linha coerente dentro do fan-out do M2. Já landaram (M2): spring, trail, integrate,
forças (attractor/vortex/wind/drag/curl), wiggle, noise, time-remap, emitter, pulse.threshold, +
o domínio de valor acima. **Faltam do M2:** os últimos nós de valor/pulse (abaixo) e alguns
utilitários (`motion.delay`, `pulse.on_change`).

---

## 4. PRÓXIMO PASSO (recomendação — mas a decisão final é sua, depois da pesquisa)

**Complete o vocabulário-núcleo do domínio de valor** (doc 14 §5 — os follow-ups nomeados). Recomendo
a próxima fatia como o par que fecha a composabilidade:

- **`value.math`** — o **primeiro combinador de DOIS campos de valor** (add/sub/mul/min/max/…),
  exercendo a regra de broadcast entre dois campos (hoje só o `motion.drive` a exerce, e só 1→N
  contra o stream). É o que desbloqueia `instance_field × lfo → drive` = **gradiente espacial
  modulado no tempo** — o ganho generativo que ainda falta. *(Pesquise: TD Math CHOP, Houdini VOP
  add/multiply, Nuke Merge(math), Cavalry Math — e decida se é UM nó multi-op ou nós por-op, e o
  nome mais claro.)*
- **`pulse.compare`** — a ponte **valor→pulse GENUÍNA** (dual do `sample_hold`), com histerese
  (2 thresholds, Schmitt). **NÃO é duplicata do `pulse.threshold`**: aquele lê um *canal de
  transform* (`INST_VEC2`, o input do "clock hack" que o doc 09 matou), este lê o *campo de valor*
  (`v`). Fecha o round-trip contínuo↔discreto (agora um grafo de valor pode realimentar o de pulso).
  *(Pesquise: Max `>~`/`edge~`, Reaktor compare, Pd `moses`/`threshold~`.)*

Depois: **`value.switch`/`gate`** (roteia um de N por seletor) · os utilitários do M2
(`motion.delay`, `pulse.on_change`) · então **M3** (distribuições avançadas + deformers). O doc 01
§3 tem a lista exaustiva por fase.

> Escolha o tamanho da fatia como as anteriores: **2 nós que provam uma capacidade end-to-end**, com
> a cena boot atualizada pra você smokear. Se a pesquisa apontar um nome/algoritmo melhor que o que
> sugeri, **siga a pesquisa** — o nome acima é um ponto de partida, não um mandato.

---

## 5. CONVENÇÕES TÉCNICAS DO DOMÍNIO DE VALOR (não re-derive — copie das crates existentes)

Leia `crates/ph2d-node-value-lfo/` e `crates/ph2d-node-pulse-sample-hold/` como **template fresco**
(mais atual que o `ph2d-node-debug-wave` do plano). O essencial:

- **Tipos (mirror local por leaf-crate — o compartilhado é a PORTA, não um símbolo):**
  `const VALUE = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame)` (coluna `v`);
  `const PULSE = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event)` (coluna `pulse`);
  `const INST_VEC2 = (Instances, Vec2, Frame)` (o stream de transform; `motion.drive` lê `P`/`rot`/`size`).
- **Regra de broadcast 1→N (doc 12):** len-1 faz *hold* pra todos os N; len-N element-wise; desiguais
  ambos >1 = mismatch (`debug_assert`). Vive em `motion.drive::channel::value_at` e
  `pulse.sample_hold::pulse_at` — **copie o helper** no seu nó combinador.
- **Nó sequencial (tem estado):** feedback `pre` no porto **`state`** (o nome `state` faz o editor
  auto-plumbar o self-loop `state_out --pre--> state_in`). `Effect::Pure` (o tick entra no
  fingerprint pela aresta `pre`; NÃO use Temporal aqui). **Prime na 1ª tick** (amostre/inicialize
  imediatamente pra a saída nascer viva, não um 0 morto — como `pulse.beat`/`sample_hold`).
- **Nó que lê o playhead (LFO, beat):** `Effect::Temporal` (só Temporal põe o playhead no
  fingerprint do memo; Pure serviria valor stale num re-cook do mesmo tick).
- **HR-5 (transcendental-free na produção):** `sin/cos/tan/exp/pow/log` PROIBIDOS; `.sqrt` é
  IEEE-determinístico e OK. Aleatoriedade = **reusar o hash do `motion.emitter`** (`hash3`/`rand01`,
  splitmix, já copiado em `value-instance-field/src/hash.rs`). Onda periódica = o wave core parabólico
  do `motion.oscillator`/`value.lfo` (`wave.rs`). Grep de fechamento:
  `\.(sin|cos|tan|atan2|exp|ln|log|powf|powi)\b`.
- **Fan-out (drop-crate):** crie `crates/ph2d-node-<prefixo>-<nome>/` (Cargo.toml + src/lib.rs;
  deps só `ph2d-nodegraph` + `ph2d-node-registry`), depois **`cargo run -p ph2d-node-sync`**
  regenera `ph2d-node-registry-init` (o gate `staleness` cobra). **Cuidado (gotcha):** crate NÃO-nó
  na área de nós não pode começar com `ph2d-node-` (o sync gera `::register` inexistente).
- **UI:** `register_ui` (display_name, `NodeUiCategory::{Utility/Transform/Fx}`, silhueta) +
  `register_param_ui(PARAM_HINTS)` (`ParamUiHint{param,label,min,max,step,widget: Slider/Enum{labels}}`).
  O teste `every_row_range_contains_its_value_for_every_node_and_param` **exige que o range contenha
  o default** de todo param — se não contém, quebra.
- **Cena boot pro smoke (regra "exemplo pronto"):** wire seus nós novos em
  `shells/desktop/src/motion_demo_strobe.rs` (auto-play), atualize a contagem de nós +
  doc-comments em `motion_state.rs`/`motion_state_tests.rs`, e escreva ≥1 teste de integração
  headless **falsificado dos dois lados** (o que prova que está vivo E correto).
- **LOC cap:** 700/arquivo (workspace), 600 (shell). Split em módulo-irmão, nunca allowlist. Rode
  `fmt` (pin 1.95) ANTES de medir (fmt re-expande multi-arg).
- **Gate batched no fechamento (1× sobre o diff):** nextest impacted · `architecture_contract_surface`
  (2/1/8) · staleness registry-init · clippy `--all-targets` · typos · machete · HR-5 grep · LOC ·
  fmt (pin 1.95, `rustfmt --edition 2024` nos arquivos do shell pra não reformatar a crate inteira).

---

## 6. O PROCESSO MODO L (você abre a linha, fecha e PARA — não integra, não pusha)

- **Outras linhas estão VIVAS** agora (Painter, Vector, anim, audio — `git worktree list`). Você roda
  em paralelo. O **worktree antigo `line/MotionNodes` (já merjado) precisa de limpeza** antes de você
  abrir a sua: `git worktree remove Worktrees/line-MotionNodes` (ou reutilize a pasta resetando pro
  main). *"Uma linha por módulo, nunca duas."*
- **Abra a SUA worktree do main** (bloco do MODELO_ABERTURA_LINHA; sugiro branch `line/motion-value`):
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D
  git pull --ff-only origin main
  git worktree add -b line/motion-value Worktrees/line-motion-value main
  cd Worktrees/line-motion-value        # TODO o trabalho a partir daqui
  ```
- **⚠️ A ARMADILHA DO CWD (custou uma investigação na linha anterior):** o cwd do Bash **reseta pro
  repo primário (`main`) na fronteira de contexto** (ex.: pós-/compact). Um comando com caminho
  **relativo** então atinge o `main`, não o seu worktree. **Regra:** todo Bash começa com
  `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && …`; toda mutação de arquivo
  por caminho **absoluto**; e antes de qualquer git/cargo, um fence:
  `test "$(git branch --show-current)" = "line/motion-value"`. Vide a memória
  `feedback_sed_relative_path_hits_primary_cwd`.
- **Isolamento:** edite a pasta do seu módulo (as crates `ph2d-node-*` novas + o shell do Motion).
  Foundational você PODE tocar sob o protocolo testado (ADR-0107), mas o domínio de valor é
  aditivo-puro — provavelmente você não precisa. Contrato congelado (§6) = PARE e reporte ao Enio.
- **Velocidade:** inner loop = **só `cargo check -p <crate>`**. Teste/clippy/auditoria **1× no
  fechamento**, nunca por task. `bash scripts/hw-profile.sh` primeiro (esta é uma workstation → Modo L).
- **Fechamento:** feche o módulo, **escreva o handoff de integração (DIRETRIZ §1.5.9)** — um doc
  novo (modele pelo `docs/HANDOFF_line_MotionNodes_integracao_2026-07-10.md`, seção "BRIEFING DO
  INTEGRADOR"), e **PARE**. Você **NÃO integra nem pusha** — quem funde é o agente integrador
  dedicado, por ordem EXPLÍCITA do Enio. Anote no handoff: crates/símbolos novos, o ponto de merge
  do codegen (`registry-init` → `ph2d-node-sync`), contratos encostados (deve ser nenhum), e o que
  smoke-testar.
- **Smoke (você entrega o comando pronto, com o `cd`):**
  `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop`
  → tool Motion → sua cadeia nova ao vivo.

---

## 7. Resumo de uma linha

*Domínio de valor: 3 fatias feitas (counter+drive, lfo+map_range, sample_hold+instance_field),
integradas no main. Próximo: completar o vocabulário — recomendo `value.math` (1º combinador de 2
campos) + `pulse.compare` (ponte valor→pulse), mas **pesquise o padrão-ouro e o melhor nome antes**.
Abra uma linha nova do main (Modo L), feche com handoff §1.5.9, não integre. A regra de ouro (§1) e
a armadilha do cwd (§6) são o que mais importa.*
