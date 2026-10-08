═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)
Base: os ciclos 13, 14 e 15 fecharam em `line/motion-value` sobre o `main` `2a925859d`, POR INTEGRAR
(handoffs [CICLO_13](HANDOFF_INTEGRACAO_line_motion_value_2026-10-07_CICLO_13.md),
[CICLO_14](HANDOFF_INTEGRACAO_line_motion_value_2026-10-08_CICLO_14.md) e
[CICLO_15](HANDOFF_INTEGRACAO_line_motion_value_2026-10-08_CICLO_15.md); doc
[124](../124_ciclo_15_o_tecto_dos_criadores_e_o_colisor_do_corpo_mole.md)). Smoke do ciclo 15 APROVADO
pelo dono (cena `=129`).
⚠️ Se a linha já foi INTEGRADA quando você abrir (`git cherry main HEAD | grep -c '^+'` = 0): `git reset --keep
main`; senão `git rebase main` (conflito fora dos seus ficheiros ⇒ PARE e reporte).

⛔ Fale com o Enio SEMPRE em PT-BR, curto e sem jargão (CLAUDE.md §0.8). Smoke de DESEMPENHO sempre em `--release`.
⛔ Ordens que continuam: integrar só por ordem EXPLÍCITA do dono (este prompt NÃO é essa ordem); UM ciclo, UMA
   rodada de medição intercalada (§0.10); série só com o porquê escrito antes; NUNCA matar o pai de um processo, e
   NUNCA `pkill -f`/`P=$(pgrep -f …)` — eles apanham o PRÓPRIO shell (aconteceu DUAS vezes no ciclo 15); use
   `pgrep -x <comm>` e o PID. Relógio do APP não vale acima de `load ~5`; variantes = MESMO processo, intercaladas,
   o MÍNIMO. Smoke do Motion = FORMAS + SIMULAÇÃO, FOTOGRAFADO antes (`docs/Components/ferramentas/fotografa_cena.sh`,
   `FOTO_PERFIL=release`; a espera conta do arranque, `~3 s`). ⭐ Antes de mandar uma cena, pergunte *«o que o
   dono vai TOCAR para obter isto?»* e MEÇA esse gesto (a lição do ciclo 15: a cena segurava com um cartão fora
   do catálogo e os botões que ele via estavam mudos — `reference_topic_control_design_hazards`).
⛔ Ordem do dono (08/10, depois do smoke aprovado): *«vamos corrigir tudo»* — os abertos abaixo, num ciclo.

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -6 && git status
        --short --ignored | grep -v target/ ; `pgrep -af 'ph2d|cargo|rustc'` e `fuser -v /dev/dri/*` antes e no fim.
FASE 1 (leia DENTRO da worktree, só os trechos): handoff CICLO_15 §2, §6, §7 · doc 124 §2.3, §6 e §6.1 · BUGS
        Bug #16 inteiro · doc 115 §15.3 (porque o passe do fim não segura um solver).

OS ABERTOS (feche TODOS; desenho e kill-criterion escritos ANTES, no doc novo — conte o número no `README.md`
do módulo, o último é o 124):

 (1) **Bug #16 — a corda com colisor atravessa-se num chicote a 60 Hz** (`rope_thickness`: pose final `0,811·D`,
     mas `0,0497` na 2.ª metade). Medido no ciclo 15: com sub-passos do GRAFO ela segura `0,52`/`0,81`/`0,80` a
     `2`/`4`/`8`, mas a corda NÃO declara o relógio (o `solver_substeps` é LOCAL por uma decisão medida — um
     `substeps` de grafo fazia uma `sim.zone` vizinha custar `8×`, ver o comentário do manifesto). E a decisão do
     dono de 08/10 é que **o `Collide` do Shape segura um corpo** (o corpo mole já o faz: lê o colisor das peças
     da porta `shape`, `crates/ph2d-node-motion-soft-body/src/contacto.rs`). ⇒ o desenho a decidir pelo
     padrão-ouro, MEDIDO numa rodada: (a) a corda LER o colisor declarado das suas peças pelo mesmo caminho do
     corpo mole, com o contacto DENTRO do laço de restrições dela (intercalado com as `iterations` — o que o
     Vellum faz), (b) a corda declarar o relógio do grafo (reabrir a decisão com a medição do vizinho), (c) um
     limite de deslocamento por sub-passo (o `maxSpeed` do FleX). ⚠️ OUTRO APP JÁ FAZ ISTO: estenda o oráculo
     `docs/Motion Nodes/ferramentas/godot_oracle_corpo_mole/` a uma CORRENTE de contas (a nossa fixtura
     `rope_thickness`: `24` nós, raio `0,5`, chicote `±3,0`/`0,6 s`) e grave a saída. Kill-criterion: pares
     distantes `≥ 0,9·D` na 2.ª metade, comprimento `≤ 1,10 ×` a corda nua, o controlo do artista ALCANÇÁVEL (nada
     fora do catálogo), e a cena/tutorial que o mostram fotografados.
 (2) **O `Collide` do Shape DEPOIS do corpo** (`Soft Body → Duplicator(Shape) → Output`): hoje só arruma o
     DESENHO, e só com o `Collide` do Output ligado — sozinho é um botão mudo (medido no ciclo 15, doc 124 §6).
     O tutorial 09 ensina a ordem que segura (as peças ENTRAM no corpo), mas o artista que montar ao contrário
     vê um botão que não faz nada. Padrão-ouro a decidir por medição (o doc 115 §15.3 já refutou o passe do fim
     como lei de contacto): ou o grafo DIZ ao artista o que fazer (um conselho do diagnóstico no cartão do Shape,
     pela porta `motion_bridge_heal` / `ph2d_motion_diagnose`, como os outros), ou o corpo passa a alcançar o
     colisor das formas que o vestem. Kill-criterion: nenhum `Collide` visível fica mudo sem o artista saber
     porquê e o que fazer — gate que liga cada botão sozinho e exige efeito OU conselho.
 (3) **`motion.scatter` com o `count` dirigido por fio acima do digitável** — o digitável é `3 000` (um tecto de
     RECURSO medido: `O(count²)`), mas o `eval` aceita até ao tecto da casa e um `value.*` ligado ao `count`
     congela o app por minutos (o censo do ciclo 15 mediu-o: não acabou em `10` min a `32 768`). Meça (há rota de
     placa? o custo a `3 000`/`10 000`/`32 768`) e cure pela lei da casa (§0.0: o tecto é o do recurso, e um
     controlo que aceita e mente é defeito).
 (4) **O `ph2d-render` com o desenho do SIGSEGV** (`~20` `static OnceLock<Option<GpuContext>>` nos `tests/it`,
     a família que o ciclo 14 curou no `ph2d-gpu-cook` com UM contexto partilhado — `8/8` contra `0/16`, doc 123
     §6). Fora do módulo, mas Modo L pode tocar foundational. MEÇA primeiro (N corridas intercaladas, `rc` e
     carga ao lado): se reproduz, a mesma cura; se não, registe o número e feche.
 (5) **A sonda `FIELD_DOC_VERSION` do `scripts/collision-surface.sh`** — os handoffs 13/14 dão-na como aberta,
     mas o `main` já a tirou (`69b28daa5`, a `ph2d-field` saiu com o 3D). Confirme no ficheiro e feche a
     referência nos handoffs/doc (não reconstrua).

JÁ FECHADOS — não reabra: tudo do ciclo 15 (doc 124: o tecto em todo criador, o Bug #15, o `Collide` do Shape no
 corpo mole de peças, a cena `=129`) e dos ciclos 13/14.

FERRAMENTAS QUE VOCÊ HERDA: `scripts/motion-gate-placa.sh` (placa, `30` testes) · o censo
 `nenhum_criador_passa_o_tecto_com_a_contagem_no_extremo` · o oráculo Godot (`corre.sh` grava a `saida.txt` que
 o gate do Bug #15 LÊ — mantenha esse padrão: a saída do oráculo é fixtura de gate) · `marcha_com` (e o gate
 do gesto `turning_on_the_left_shapes_collide_makes_it_hold`) na `motion_state_corpo_com_colisor_demo_tests.rs` (a bomba do app com as membranas publicadas —
 sem `publish_all` o `source.shape` não existe e um teste de `Cook` cru mede outra coisa).

⚠️ ANTES DE CONSTRUIR (CLAUDE.md §5.0): audite CADA item contra o CÓDIGO de hoje e meça-o. Doc novo do ciclo com o
   estado MEDIDO, o desenho e os kill-criteria ANTES; nota de estado no doc 103 §5.

DoD: os itens fechados (ou recusados por medição, com o número), no doc novo; BUGS (#16 fechado com as lições);
nota do doc 103 §5; gate batched verde (`nextest-impacted` com `BASE=2a925859d`, check com warnings negados, clippy
`--all-targets` das crates tocadas, fmt, censos, `doc-index`, gate de placa inteiro); prova de mutação (uma por lei
nova, a sangrar, pré-voo e corrida limpa); handoff de integração com a superfície de colisão; memória (lições nas
famílias, índice ≤ `22 000` bytes); smoke ao dono FOTOGRAFADO em `--release`, em passos numerados (uma cena com a
corda que não se atravessa, pelo controlo que o artista alcança; e o gesto do item 2); binário compilado 2× (zero
`Compiling` na 2.ª); NÃO integra, NÃO pusha. No relatório: ficou ALGUMA coisa em aberto?
═══════════════════════════════════════════════════════════════════
