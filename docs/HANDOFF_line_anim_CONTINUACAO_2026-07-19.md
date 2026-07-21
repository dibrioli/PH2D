# Handoff de CONTINUAÇÃO — `line/anim-fixes` (novo agente assume)

> Para o agente que assume a linha **depois da integração do nesting** (2026-07-19).
> Este doc é o "onde paramos + para onde ir". A **mecânica de reabrir a linha** é o BLOCO de
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
> — leia-o, não o resumo daqui; duas cópias da mesma regra divergem.

---

## 0. Reabrir a linha (a mecânica, curta)

Você começa na **raiz do repo, que está em `main`**. Os mesmos paths existem nas duas árvores;
editar o da raiz compila e commita sem erro e só aparece na integração. Então, **antes de ler
qualquer código:**

```
cd Worktrees/line-anim && pwd && git branch --show-current
      → tem de terminar em /Worktrees/line-anim, branch line/anim-fixes
git log --oneline -5 && git status -sb
git rebase main            # OBRIGATÓRIO no início da jornada (DIRETRIZ §1.5.2.3)
cargo check -p ph2d-timeline
```

⚠️ **O rebase vai trazer ~88 commits de outras linhas** (Painter, FLIP, GPU-nodes, audio…) que
integraram enquanto o nesting rodava. É esperado — a linha está **0 commits à frente do main e
88 atrás** (integrou limpa). O 1º build pós-rebase é frio (minutos): não investigue.
Conflito em `Cargo.lock` ou arquivo GERADO (registry-init, `chrome/mod.rs`) → **regenere, nunca
resolva na mão**. Conflito em código fora dos seus arquivos → colisão de mesmo-símbolo → **PARE
e reporte ao Enio** (DIRETRIZ §1.5.5).

⚠️ **A worktree ainda está no HEAD pré-integração** (`8b4847dd`). O `git rebase main` é o que a
alinha. Não estranhe o log velho antes de rebasear.

---

## 1. Onde paramos — o nesting FECHOU e INTEGROU

A linha entregou o **nesting da timeline**
([ADR-0133](architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md)):
um objeto que por dentro é uma cena animada inteira e por fora é um objeto, com o relógio do
**pai**. Pesquisa → ADR → plano → Fatias 0–3f, **smokado e aprovado pelo Enio**, e **integrado
ao main** (commits do nesting em `main`; handoff de integração em
[`HANDOFF_INTEGRACAO_line_anim_nesting_2026-07-19.md`](HANDOFF_INTEGRACAO_line_anim_nesting_2026-07-19.md)).

⚠️ **A entrada do CLAUDE.md §5 (Timeline) no main ainda diz "aguardando integração"** para o
nesting — está desatualizada (a integração aconteceu). Não é bug seu; se for mexer no roteador,
corrija de passagem, mas não é obrigatório.

**A ideia central, para você não reconstruir o raciocínio:** o nesting **não precisou de
mecanismo novo**. O relógio (outer-then-inner + recusa nomeada) já existia; o z-order já estava
respondido (`SortingGroup`, Unity); o custo O(bindings²) já tinha sido pago (`ClockIndex`); e o
strip já carregava o mínimo universal de override por-instância. O trabalho foi **estender uma
cadeia**, não inventar uma segunda. Onde um número não podia ser invertido (o relógio do
container), a resposta foi publicar a **relação** (`container_map`), que serve às duas direções.

---

## 2. O que já está PRONTO (não reconstrua, não re-litigue)

A timeline está madura. O que **já landou** e você pode tratar como dado:

- **Transporte, régua, lanes, zoom/pan, box-select, copy/paste, `F`=fit** (W2).
- **Curva por-faixa, handles bézier, weighted tangents, speed graph, presets de easing** (W3/W5).
- **Time remap por-objeto** (o relógio do AE) — e os 3 bugs dele já fechados.
- **Roving keys, Performing/Record, simplificação de keyframes (Schneider)** (W5).
- **Seletor de clips + composição de clips** (ADR-0115: faixas, crossfade por sobreposição,
  canais esparsos, alça de fade, read-only onde o vizinho define a janela).
- **Relógio único** (W4.T7 — `MotionTransport` morreu; o Motion cozinha no tick do `Playhead`).
- **Save de projeto cena+timeline** (W4.T6/B5 — o `TimelineDoc` viaja no `ProjectFile`).
- **Undo global** cobrindo objetos+hierarquia+canvas.
- **Nesting** (ADR-0133) — esta linha, agora integrada.

**Regra que quebrou este módulo três vezes** ([[feedback_derived_coordinate_seed_must_match_sample]]):
autoria e leitura de uma coordenada derivada usam a **mesma** transform. O nesting a aplicou
recursivamente; a próxima feature de tempo vai encostar nela de novo.

---

## 3. Os planos a seguir (ranqueados, com o gatilho de cada um)

**Nada disto está em andamento.** A linha está fechada contra o seu plano — o próximo passo é
**ordem do Enio**. Abaixo, o que está aberto, honesto e com o critério que o acorda.

### 3.1 — Cauda da Timeline (o escopo natural desta linha)

Fonte: `HANDOFF_line_anim_integracao_2026-07-11.md` §"Cauda da W4" + CLAUDE.md §5. Já filtrei o
que **já landou** (T6/T7, composição de clips, nesting):

| Item | O que é | Estado / gatilho |
|---|---|---|
| **W4.T4 — docar a timeline** no `motion_timeline_slot` quando o split do Motion está ativo | coordenação leve com Motion | **DESBLOQUEADO** (Motion integrou). Pronto para ser pego. |
| **markers → signals** | markers da régua viram eventos disparáveis | ordem do Enio |
| **bake curves → keyframes** | assar uma curva procedural em keys editáveis | ordem do Enio |
| **export** | sair da timeline para um formato (Lottie/vídeo?) — desenho ainda não feito | precisa de ADR antes de código |
| **MCP / Luau** | dirigir a timeline por script | ordem do Enio |
| **Refinamentos do fit de keyframes** | corner broken-tangents · overshoot clamp p/ opacity · rotation unwrap · low-pass | deferidos no topo de `curve_fit.rs`; gatilho = smoke reclamar |

### 3.2 — Cauda do próprio NESTING (deferido de propósito, ver ADR-0133 §6 e plano §7)

| Item | Por que NÃO foi feito | O gatilho que o acorda |
|---|---|---|
| **Relógio próprio por container** (*movie clip*) | maior fonte de confusão documentada; mata o scrub determinístico | máquina de estados / interação em runtime — e aí é **opt-in por instância**, nunca default |
| **Cache de saída do container** | sem consumidor, cachear é escolher um modo de falha antes de ter o problema | o kill-criterion ser excedido por instâncias IDÊNTICAS (desconto = *Master Pose Component* do Unreal) |
| **Teto de profundidade** | **não temos o recurso que o justifique** (§0.0 — medido: custo linear); ninguém no mercado publica um medido | alguém medir memória/recursão de pilha e escrever o número |
| **Instância que dá a VOLTA tem mapa** | sob loop/pingpong um segundo do interior está em vários da timeline; escolher um em silêncio é o palpite que o módulo recusa | desenho: "a ocorrência mais próxima do playhead" — e aí precisa de gate próprio |
| **Loop próprio do container** | pegar emprestado o loop do documento seria um loop de outro relógio embrulhando uma pilha que ele não conhece | quando o container precisar repetir por dentro sem depender do pai |
| **Vetor/pintura DENTRO do container** | o interior deste ADR é **animação** | ordem do Enio |

### 3.3 — Dívida de teste conhecida (não bloqueia, mas está medida)

- **O lado do SAVE não tem gate.** `project_save` exige `gfx` (janela + GPU): o mundo mora
  dentro do `AppGfx`, e é isso que trava o harness headless do shell. O **load** já é gateado
  (`project_tests.rs`, o `App` nasce sem janela). Se for mexer em persistência, esse é o buraco.

---

## 4. Armadilhas ESPECÍFICAS desta linha (as que ela pagou caro)

- **`stack.rs`/`stack_eval.rs`/`snapshot.rs` vivem perto do cap de LOC (700).** O nesting já fez
  3 splits (`nest.rs`, `stack_frames.rs`, `ruler_clock.rs`, `nest_map.rs`). O gate de LOC mora na
  `ph2d-editor-core` e **não roda** com `cargo test -p ph2d-timeline` — rode na árvore combinada,
  e `cargo fmt` **antes** de medir (o fmt re-expande).
- **A ponte painel→shell é um thread-local carimbado ANTES de drenar intents** (o padrão
  `keys_mode` / `edit_path`). Toda feature nova de estado-de-vista do painel copia esse padrão;
  não invente um segundo canal.
- **`DOC_VERSION` é rejeição pura, sem migração** (postcard posicional, política da casa). Bumpou
  o doc? A versão velha é recusada no load, com toast. Não escreva maquinário de migração.
- **`PROJECT_SCHEMA` não bumpa quando só o blob interno muda de versão** — a forma do
  `ProjectFile` é que manda. Dois números para uma incompatibilidade seriam duas portas.
- **Gates que CLICAM, não que compilam** ([[feedback_widget_is_done_when_a_test_clicks_it]]): o
  seam do nesting arrasta a régua e clica a trilha de verdade. Copie esse rigor — "pintado ≠
  populado", e a 1ª rodada de mutação do nesting estava MENTINDO (o filtro `--test` não rodava as
  suites do painel).

---

## 5. Ao assumir, reporte e PARE

> *"Assumi `line/anim-fixes` em `Worktrees/line-anim` (HEAD `<sha>` após rebase). O nesting
> fechou e integrou; a linha está limpa contra o plano. Aguardo a tarefa."*

Não integre, não pushe (§0.7). A tarefa vem do Enio.

**Mapa de leitura (nesta ordem, DENTRO da worktree):**
1. este doc;
2. [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md) (a mecânica);
3. [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) (releia a cada passo);
4. as **REGRAS PERMANENTES A–H** do [`MODELO_ABERTURA_LINHA.md`](IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md);
5. se a tarefa for de timeline: `docs/Timeline/` (00 briefing · 01 UI · 02 clips · 03 pesquisa nesting · 04 plano nesting) e os `HANDOFF_line_anim_*` por data.

---

## Sessão 2026-07-20 (pós-abertura) — fades das strips, teclado por hover, exemplo de container, e os 4 fixes do smoke do container

Commits locais desta sessão (todos na `line/anim-fixes`, sem push):
`a369ab1d..9381a9c4` (loop wrap out · roteamento de teclado por hover · lead_out completo ·
alças grandes + hover + alça fora da strip) · `c29c12ca` (`PH2D_NEST_SMOKE=2`, o exemplo
completo do container: 3 clips/2 lanes/3 instâncias + gate `nesting_leads.rs`) · o commit
desta entrada (os 4 fixes abaixo).

**O smoke do exemplo derrubou 4 coisas (Enio), todas fechadas — detalhe na emenda 2026-07-20
do ADR-0133:**

1. **Panic na aba Keys dentro de container** — o rebuild pulava o prime em `keys_mode` e o
   braço do host lia o scratch assim mesmo (`debug_assert_scratch_at`). Host readouts agora
   são Arrange-only e PUROS. Gate: `nesting_view.rs::the_keys_tab_inside_a_container_does_not_panic`
   (nasceu com o panic exato).
2. **Régua não arrastava dentro do container** — o mapa vinha de "o único strip tocando
   AGORA" (piscava nos vãos; recusava com 2+ instâncias). Agora o caminho de entrada lembra o
   STRIP (`EnterStep {container, lane, strip}`) e o mapa é `entry_map` — puro, total, sem
   scratch. O par scratch-based (`container_playhead`/`container_map` + `sole_container_strip_of`
   + `ActiveStrip.id`) morreu sem consumidores. Marca do playhead: só quando o relógio da cena
   está dentro da janela da instância entrada.
3. **Loop não se ajustava ao entrar** — `timeline_bridge::on_nav_change` (edge-trigger no
   carimbo do `edit_path`, espelho do sync de troca de aba): entrar abraça
   `entry_reach` (janela + leads) e busca o início se o playhead está fora; sair reinstala o
   loop do DOCUMENTO. Navegação nunca escreve no doc.
4. **Loop automático cortava o lead_out** — `stack_end_seconds` agora mapeia `lead_end()`;
   o Loop da barra e o go-to-end passam a cobrir a fade final.

Gates: `nesting_map.rs` reescrito para `entry_map`/`entry_reach` (o gate "toca 2× não tem
mapa" DISSOLVEU por desenho — virou "o mapa é o da instância entrada") · `nesting_view.rs`
novo · 3 gates de navegação no `timeline_bridge_tests` · extent no `lead_in.rs`. Mutações:
6/6 sangram no gate certo; extent nasceu vermelho. Suítes ph2d-timeline (28 alvos) +
ph2d-panel-timeline + bridge (37) verdes.

**Smoke sugerido:** o MESMO `PH2D_NEST_SMOKE=2` — agora: entrar no Jump não pode panicar na
aba Keys; a régua arrasta em qualquer instante; o loop vira a janela da instância entrada
(com as fades) e volta ao da cena ao sair; o Loop da barra na cena cobre até 10,5 s.

### Adendo (mesma sessão): a aba Keys dentro do container — 3 fixes do re-smoke

1. **"em keys não consigo voltar direto para Jump"** — clicar em QUALQUER segmento da
   trilha agora aterrissa no **Arrange** daquele nível (o pop do segmento final é no-op de
   propósito; sem a troca de aba o clique não fazia nada). Todo lugar que a trilha nomeia é
   um lugar do Arrange.
2. **"ao pular de jump para keys, não consigo mover playhead"** — a conversão do scrub da
   régua perguntava só à trilha+mapa; na Keys dentro de um container (crumbs cheios,
   `host_map` None desde o fix do panic) o gesto era ENGOLIDO. A pergunta é da ABA: na Keys
   a régua é o relógio do clip e o valor cru já é o eixo. ⚠️ Dois gates antigos do scrub
   passavam com `TimelinePanelState::default()` (aba default = Keys) só porque o código
   nunca perguntava a aba — as fixtures agora declaram `Tab::Arrange`.
3. **O readout "not playing here" MENTIA na Keys** (screenshot do Enio) — `status()` é um
   fato da régua do ARRANGE; na Keys cala (a trilha fica: é navegação).

Gates: 2 novos no `nesting_seam.rs` + 1 no `breadcrumb.rs`; mutações 3/3 sangram.

### Adendo 2 (mesma sessão): '+ Container' esmagado + o loop DENTRO do container

- **"+ Container some se o painel for estreito"** — duas causas: o header dos ADDs rachava
  50/50 (o rótulo 2× maior perdia) e o piso da coluna no Arrange perguntava "há lanes?"
  quando o header vive na ABA (é como se cria a primeira lane). `stack_add_header.rs`
  (split de LOC, 609→555) racha pela fração de caracteres dos MESMOS strings pintados;
  piso por aba flipado red-first. Commits `83a7d762`/`052063a5`.
- **"o loop dentro do container deve se ajustar automaticamente quando ligado às strips"**
  — dentro, Loop/PingPong agem SÓ no TRANSPORTE (`TimelineIntent::SetTransportLoop`,
  apendado): ligar abraça o `entry_reach` da instância entrada (a MESMA porta do enter),
  desligar limpa; o loop do DOC é o da CENA e nunca é tocado de dentro (sair o reinstala,
  `on_nav_change`). GO_START/GO_END dentro vão às bordas da instância. E a VISTA: as
  chaves + o estado do toggle dentro do Arrange leem o PLAYHEAD mapeado pro interior — a
  screenshot mostrava o loop da cena desenhado CRU no eixo do interior (0..10,8 sobre 2 s).
  `Playhead::is_ping_pong` novo (ph2d-core, apendado). ⚠️ A mutação "vista lê o doc"
  SOBREVIVEU à 1ª rodada — o decoy clampado colidia com o valor certo; o fixture virou
  subconjunto próprio da janela ([[feedback_a_green_gate_may_be_green_by_accident]]).
  Gates: 2 no bridge + 1 apply + 1 display; mutações 3/3 sangram.
