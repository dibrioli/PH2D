═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)
Base: o ciclo 13 fechou em `line/motion-value` sobre o `main` `2a925859d` (handoff de integração
[CICLO_13](HANDOFF_INTEGRACAO_line_motion_value_2026-10-07_CICLO_13.md), doc [122](../122_ciclo_13_os_abertos_do_modulo.md)).
⚠️ Se a linha já foi INTEGRADA quando você abrir (`git cherry main HEAD | grep -c '^+'` = 0): `git reset --keep
main`; senão `git rebase main` (conflito fora dos seus ficheiros ⇒ PARE e reporte).

⛔ Fale com o Enio SEMPRE em PT-BR, curto e sem jargão (CLAUDE.md §0.8). Smoke de DESEMPENHO sempre em `--release`.
⛔ Ordens que continuam: integrar só por ordem do dono; UM ciclo, UMA rodada de medição intercalada (CLAUDE.md
   §0.10); série só com o porquê escrito antes; NUNCA matar o pai de um processo (LOGOUT do dono em 05/10). O
   relógio do APP não vale acima de `load ~5` (`cat /proc/loadavg` antes de cada célula); comparar VELOCIDADE
   entre variantes não espera calma: MESMO processo, intercaladas, o MÍNIMO com a mediana ao lado
   ([MEDIR_VELOCIDADE.md](../../DevOps/MEDIR_VELOCIDADE.md)). Smoke do Motion = FORMAS + SIMULAÇÃO, fotografado
   antes de ir ao dono (`docs/Components/ferramentas/fotografa_cena.sh`; nunca `spectacle` num kwin virtual).
⛔ Ordem do dono (08/10): *«para os demais em aberto … resolvê-los em um único ciclo»*. E a DECISÃO dele no
   mesmo dia: **o tecto de `32 768` por nó FICA** — não o mexa, e o tecto por dispositivo está FORA.

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -3 && git status
        --short --ignored | grep -v target/ ; `pgrep -af 'ph2d|cargo|rustc'` e `fuser -v /dev/dri/*` antes e no
        fim (a máquina é partilhada; um `ph2d-host-desktop` de OUTRA linha na tela do dono NÃO é seu — não toque).
FASE 1 (leia DENTRO da worktree, só os trechos): handoff CICLO_13 §5–§7 · doc 122 §1, §2 (o `Compasso` e o
        FMA), §6 (o SIGSEGV e a ablação) · BUGS Bug #14 (lições).

OS ABERTOS (feche TODOS, ou recuse cada um por MEDIÇÃO com o número; desenho e kill-criterion escritos ANTES):

 (1) **A imunidade do `pulse.beat` a FMA não tem régua.** O `sc` leva 7 bits para `i·sc` ser exacto e o
     arredondamento fundido igualar o não fundido (doc 122 §2) — e a mutação «`quantiza` = identidade»
     SOBREVIVEU na RTX (a placa não fundiu). Padrão-ouro: uma régua que não dependa de a placa fundir —
     p. ex. um gate de CPU que compara `frac − i·sc` com `(−i).mul_add(sc, frac)` sobre as fixturas e uma
     varredura (todo `i < 2^17`, fracções nos empates) e AFIRMA igualdade com o `sc` quantizado e
     DESIGUALDADE encontrada sem ele; e corra a paridade sem quantização na iGPU (`radv`), que pode fundir.
     Barra: a mutação 3 do handoff CICLO_13 §5 passa a SANGRAR.
 (2) **O gerador `write_the_value_figures` (tutorial 6) está partido**: coze a `=117` com um `Cook` cru, sem
     publicar as formas, e lê `0` valores desde que a `=117` ganhou estrelas. Cure pela porta do produto
     (`MotionState` + `coze_o_quadro`, ou a corrente ANTES do carimbo) e compare a figura nova com a comitada.
 (3) **As figuras de parâmetros dos tutoriais estão VELHAS contra o código** — o gerador escreve hoje
     `tutoriais/fig/params.html` com «digitável até 181» onde o ficheiro diz `1000000`, e o `motion.grid`
     perdeu a linha `Shape` (também `params_animadores.html` e `params_sim.html`). Varra a FAMÍLIA: TODOS os
     geradores de figuras dos tutoriais (censo pelo nome/atributo, nunca à mão), corra-os, diff contra o
     disco, regenere, e refaça os PDFs que as usam. Barra: um gate que reprova quando a figura no disco
     difere do que o gerador produz hoje — sem ele isto volta a envelhecer em silêncio.
 (4) **As sondas `#[ignore]` que reprovam SOZINHAS** (fora do diff do ciclo 13; confira cada uma também na
     base para separar velho de novo): `audit_the_corner_radius_cost` e `audit_the_stamp_frame_split`
     (pedem as `102 400` linhas recusadas no ciclo 6 — hoje `32 761`); `audit_what_the_camera_can_see`
     (`23,4` contra `~39`); `measure_the_source_group` (só passa com `PH2D_MOTION_SO_COM_FORMA=0` — é a
     omissão que mudou?); e `the_ceiling_is_honoured_on_every_tick_including_the_turn`, que é um DEFEITO
     declarado no próprio `#[ignore]` (`0,1678` contra o tecto `0,0800`) — esse cura-se, não se recusa sem
     número. Cada uma: curada pela lei de hoje ou aposentada com o porquê medido.
 (5) **A suíte da placa do `ph2d-app-motion --lib` não cabe na fatia da linha**: com `--include-ignored` bate
     o prazo de `1800 s` (sondas lentas) e o fecho não tem `test result`. Separe as SONDAS dos GATES de modo
     que o gate de placa do fecho corra inteiro dentro da fatia (meça o antes e o depois), sem perder gate.
 (6) **O SIGSEGV da RTX ao SAIR do binário `it` do `ph2d-gpu-cook`** (`3/3`, dentro de `libnvidia-glcore`,
     depois dos testes; a base morre igual — não é do ciclo 13). É a família do doc 121 §9.18 (F): o
     `OnceLock<Option<GpuContext>>` dos testes segura o dispositivo até ao `exit`. Cure (o dispositivo solto
     antes do `exit`) e prove-o com `N` corridas intercaladas antes/depois, carga ao lado.
 (7) **A `=27` esvazia a tela em `~4 s`**: a cascata sai do quadro (o `curl` puxa-a para baixo e espalha-a
     a `|x| > 10` aos `3 s`, medido), e uma foto ou um olhar tardio não vê nada. Desenhe a cena para a
     cascata FICAR à vista (câmera, forças, laço do tempo — a escolha é técnica) sem partir os gates dela
     (nascimento só por morte, `3 → 6` no 1.º estouro, irmãs a separarem-se, estrelas legíveis), e
     fotografe-a a `2`, `4`, `6` e `8 s`.
 (8) **A sonda `PH2D_PAN_DIAG` diz a rota da placa mas não lê o MUNDO das cópias do dispositivo.** Desenho
     sugerido: com a sonda ligada, a 1.ª cópia vem pela leitura dos cartões que já existe (nenhuma descarga
     nova). Kill-criterion: custo ZERO com a sonda desligada (régua de texto, como a do `tap_sem_espera`).

JÁ FECHADOS — não reabra: os cinco do ciclo 13 (doc 122) · o tecto de `32 768` (decisão do dono, 08/10) ·
 o tecto por dispositivo (fora, pelo dono) · o flake `crossing_the_reach_boundary_does_not_step_the_cost`
 (registado em FLAKES_DE_CARGA). Fora do módulo (só registe ao integrador): a sonda `FIELD_DOC_VERSION` do
 `scripts/collision-surface.sh`.

⚠️ ANTES DE CONSTRUIR (CLAUDE.md §5.0): audite CADA item contra o CÓDIGO de hoje e meça-o. Escreva o doc
   novo do ciclo (o número conta-se no `README.md` do módulo — o último é o 122) com o estado MEDIDO de cada
   item, o desenho e os kill-criteria ANTES de construir; a nota de estado no doc 103 §5 aponta para ele.

DoD: os oito itens fechados ou recusados por medição, no doc novo; a nota do doc 103 §5; gate batched verde
(`nextest-impacted` com a base da linha, check com warnings negados, clippy `--all-targets` das crates
tocadas — ⚠️ o clippy da casa recusa `HashMap` (HR-5) —, fmt, censos, `doc-index`, e a suíte da placa
INTEIRA dentro da fatia, item 5); prova de mutação (uma por lei nova, a sangrar, pré-voo e corrida limpa);
a `=27` e a `=117` ao bit nas duas rotas; handoff de integração com a superfície de colisão; memória (as
lições nas famílias; o índice ≤ `22 000` bytes); smoke ao dono FOTOGRAFADO em `--release`, em passos
numerados (a `=27` com a cascata à vista, e um tutorial com as figuras novas); binário do smoke compilado
2× (zero `Compiling` na 2.ª); NÃO integra, NÃO pusha. No relatório: ficou ALGUMA coisa em aberto?
═══════════════════════════════════════════════════════════════════
