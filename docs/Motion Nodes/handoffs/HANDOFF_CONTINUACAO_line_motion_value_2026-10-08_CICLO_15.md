═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)
Base: os ciclos 13 e 14 fecharam em `line/motion-value` sobre o `main` `2a925859d`, POR INTEGRAR
(handoffs [CICLO_13](HANDOFF_INTEGRACAO_line_motion_value_2026-10-07_CICLO_13.md) e
[CICLO_14](HANDOFF_INTEGRACAO_line_motion_value_2026-10-08_CICLO_14.md); doc [123](../123_ciclo_14_os_oito_abertos.md)).
⚠️ Se a linha já foi INTEGRADA quando você abrir (`git cherry main HEAD | grep -c '^+'` = 0): `git reset --keep
main`; senão `git rebase main` (conflito fora dos seus ficheiros ⇒ PARE e reporte).

⛔ Fale com o Enio SEMPRE em PT-BR, curto e sem jargão (CLAUDE.md §0.8). Smoke de DESEMPENHO sempre em `--release`.
⛔ Ordens que continuam: integrar só por ordem do dono; UM ciclo, UMA rodada de medição intercalada (CLAUDE.md
   §0.10); série só com o porquê escrito antes; NUNCA matar o pai de um processo. Relógio do APP não vale acima de
   `load ~5`; comparar variantes = MESMO processo/binário, intercaladas, o MÍNIMO com a mediana
   ([MEDIR_VELOCIDADE.md](../../DevOps/MEDIR_VELOCIDADE.md)). Smoke do Motion = FORMAS + SIMULAÇÃO, fotografado
   antes de ir ao dono (`docs/Components/ferramentas/fotografa_cena.sh`, `FOTO_PERFIL=release`; a espera conta do
   ARRANQUE do app, `~3 s` — some-os; nunca `spectacle` num kwin virtual).
⛔ Ordens do dono (08/10, depois do ciclo 14): *«o nó de corpo mole deve ter limite de 32 768»* e *«o defeito
   pode consertar conforme padrão ouro»* (o Bug #15). O tecto de `32 768` por nó FICA (decisão de 08/10).

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -4 && git status
        --short --ignored | grep -v target/ ; `pgrep -af 'ph2d|cargo|rustc'` e `fuser -v /dev/dri/*` antes e no fim
        (máquina partilhada; o que for de OUTRA linha não é seu — não toque).
FASE 1 (leia DENTRO da worktree, só os trechos): handoff CICLO_14 §2 e §7 · doc 123 §4 (a família «avança antes de
        cozer» e a tabela dos quatro vermelhos) · BUGS_motion_nodes.md Bug #15 inteiro.

OS ABERTOS (feche TODOS; desenho e kill-criterion escritos ANTES, no doc novo — o número conta-se no
`README.md` do módulo, o último é o 123):

 (1) **O `motion.soft_body` respeita o tecto de `32 768` por nó.** Hoje o lado dele vai a `512`
     (`crates/ph2d-node-motion-soft-body/src/lib.rs:~126`) ⇒ `262 144` pontos num nó. A lei da casa é
     `ph2d_nodegraph::node::MAX_INSTANCIAS_POR_NO` (e `LADO_MAX_DE_GRELHA = 181` para lados de grelha — o
     molde dos distribuidores, `d60a8398c`). Derive o tecto DESSAS constantes (nunca um literal), no
     `ParamHardMax` e no `eval`, nas DUAS rotas (o soft body tem kernel?). Cascata obrigatória: o tutorial 09
     cita «Soft Body 512×512, 2,26 ms» (e o gate `the_cost_table_the_rig_tutorial_prints_is_the_one_that_was_measured`
     em `motion_state_rig_tutorial_tests.rs` prende o `2,26`) — re-meça ao tecto novo e reescreva; cenas que
     usem o soft body acima do tecto (censo pelo roteador de cenas); as figuras/tabelas geradas (o gate
     `as_figuras_dos_tutoriais_sao_as_que_os_geradores_produzem` vai dizer quais) e os PDFs.
     ⭐ **FAMÍLIA (CLAUDE.md §0.10):** o soft body não é o único que escapa — faça o CENSO de todo nó cuja
     contagem de saída pode passar de `MAX_INSTANCIAS_POR_NO` (lado/contagem próprios: `verlet_rope`, `wave`,
     `boids`, `sim.spawn`, `motion.emitter`, `source.table` — que hoje lê um ficheiro de um milhão de linhas e
     o tutorial 08 di-lo —, `motion.duplicator`/`trail`/`drop_shadow`, que MULTIPLICAM linhas…). Separe o que
     é «um nó PRODUZ mais que o tecto» (a lei do dono: cortar) do que é «linhas a jusante de vários nós»
     (outro recurso — meça, não corte por analogia). Barra: um gate de censo que reprova um nó novo cujo
     tecto próprio passe o da casa.
 (2) **Bug #15 — o colisor no laço de um corpo mole não segura a distância mínima.** Medido no ciclo 14
     (fixtura `soft_body_radius` do `ph2d-node-registry-init`, `5 × 5`, `spacing 0,30`, `radius 0,28`): o
     corpo incha (`1,92 → 2,76`), mas à SAÍDA do `motion.collide` o pior par NÃO vizinho chega a `0,21`
     contra o diâmetro `0,56`; o colisor empurra também os vizinhos de malha, que as molas seguram a
     `spacing < 2·radius`, e as `8` iterações não convergem. Padrão-ouro: o que os solvers de partículas
     com restrições fazem (PBD/XPBD, Vellum, Box2D-soft) — **os pares ligados por restrição não colidem**,
     e o contacto resolve-se no MESMO laço de restrições (ou sub-passos) em vez de uma passagem solta.
     ⚠️ Antes de desenhar: (a) OUTRO APP JÁ FAZ ISTO ⇒ é um ORÁCULO que se CORRE (CLAUDE.md §0.9,
     `docs/_ComoInvestigarApps/`): triagem de licença por artefacto, corra-o por script sobre a NOSSA
     fixtura, grave a saída com cabeçalho, compare por passo; (b) meça se a composição já o exprime
     (CLAUDE.md §5.0) — o `motion.collide` sabe quem é vizinho? o `soft_body` publica as arestas da malha
     (uma coluna de adjacência/`id` de malha)? a `motion.rope`/`verlet_rope` resolveu o mesmo problema? —
     ver o gate gémeo `rope_thickness`. Kill-criterion a escrever ANTES: à saída do colisor, pior par não
     vizinho `≥ 2·radius − ε` (ε nomeado e medido), o corpo continua a simular, as molas continuam a segurar
     os vizinhos ao `spacing`, a extensão continua a inchar; a paridade CPU × placa se o nó tiver kernel;
     e o custo medido (o colisor é `O(n²)`? com o tecto do item 1 isso é `32 768²` — meça e diga o recurso).
     O gate `soft_body_radius::the_soft_body_takes_a_particle_radius_through_the_collide_loop` volta a exigir
     a distância (hoje mede só a extensão, a régua que valia); o Bug #15 fecha no BUGS com as lições.

JÁ FECHADOS — não reabra: os oito do ciclo 14 (doc 123) · o tecto de `32 768` (decisão do dono) · o gancho
 `atexit` do SIGSEGV (MEDIDO sem efeito e retirado: a cura é o contexto único, doc 123 §6) · o flake
 `crossing_the_reach_boundary_does_not_step_the_cost` (FLAKES_DE_CARGA). Fora do módulo (só registe ao
 integrador): o `ph2d-render` com o mesmo desenho do SIGSEGV (`~20` `OnceLock` de contexto nos `tests/it`); a
 sonda `FIELD_DOC_VERSION` do `scripts/collision-surface.sh`.

FERRAMENTAS DO CICLO 14 QUE VOCÊ HERDA: `scripts/motion-gate-placa.sh` (o gate de placa do `ph2d-app-motion`,
 `30` testes `placa:`, `~96 s`; `PH2D_GPU=1 bash scripts/ph2d-run.sh bash scripts/motion-gate-placa.sh`) · todo
 `#[ignore]` novo leva `placa:`/`sonda:`/`gerador:` (há censo) · arnês de teste coze e DEPOIS avança o tique
 (há censo) · a lei do dono lê-se da BOMBA (`motion.pump.a_lei()`), nunca do ambiente (há censo) · figuras dos
 tutoriais: corra os escritores `write_the_*`/`dump_*_figures` (`--release --ignored`) e o gate compara.

⚠️ ANTES DE CONSTRUIR (CLAUDE.md §5.0): audite CADA item contra o CÓDIGO de hoje e meça-o. Escreva o doc novo
   do ciclo com o estado MEDIDO, o desenho e os kill-criteria ANTES de construir; nota de estado no doc 103 §5.

DoD: os itens fechados (ou recusados por medição, com o número), no doc novo; nota do doc 103 §5; gate batched
verde (`nextest-impacted` com `BASE=2a925859d`, check com warnings negados, clippy `--all-targets` das crates
tocadas — o clippy da casa recusa `HashMap` (HR-5) —, fmt, censos, `doc-index`, e o gate de placa inteiro);
prova de mutação (uma por lei nova, a sangrar, pré-voo e corrida limpa); handoff de integração com a superfície
de colisão; memória (as lições nas famílias; índice ≤ `22 000` bytes); smoke ao dono FOTOGRAFADO em `--release`,
em passos numerados (uma cena de corpo mole com colisor, que se vê a INCHAR sem as partículas se atravessarem,
e o tutorial 09 com a tabela nova); binário do smoke compilado 2× (zero `Compiling` na 2.ª); NÃO integra, NÃO
pusha. No relatório: ficou ALGUMA coisa em aberto?
═══════════════════════════════════════════════════════════════════
