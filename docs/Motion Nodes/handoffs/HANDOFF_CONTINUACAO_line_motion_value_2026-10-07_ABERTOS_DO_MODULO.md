═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)
Base: `main` em `2a925859d` (07/10, depois da rodada de seis linhas; a linha foi avançada por `--ff-only`
e o `ph2d-app-motion` compila). A ciclo 6 está INTEGRADA ([handoff §19](HANDOFF_INTEGRACAO_line_motion_value_2026-10-05.md)).

⛔ Fale com o Enio SEMPRE em PT-BR, curto e sem jargão (CLAUDE.md §0.8). Smoke de DESEMPENHO sempre em `--release`.
⛔ Ordens que continuam: integrar só por ordem do dono; UM ciclo, UMA rodada de medição intercalada (CLAUDE.md
   §0.10); série só com o porquê escrito antes; NUNCA matar o pai de um processo (LOGOUT do dono em 05/10). O
   relógio do APP não vale acima de `load ~5` (`cat /proc/loadavg` antes de cada célula); comparar VELOCIDADE
   entre variantes não espera calma: MESMO processo, intercaladas, o MÍNIMO com a mediana ao lado
   ([MEDIR_VELOCIDADE.md](../../DevOps/MEDIR_VELOCIDADE.md)). Smoke do Motion = FORMAS + SIMULAÇÃO, fotografado
   antes de ir ao dono (`docs/Components/ferramentas/fotografa_cena.sh`; nunca `spectacle` num kwin virtual).
⛔ Ordem do dono (07/10): *«resolver tudo que está em aberto em um único ciclo»* — a lista abaixo é TODA a
   lista do módulo, auditada contra os docs em 07/10 (o que já fechou está no fim, com o endereço — não reabra).

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -3 && git status
        --short --ignored | grep -v target/ ; rebase no main se ele andou (conflito fora dos seus ficheiros ⇒ PARE).
        `pgrep -af 'ph2d|cargo|rustc'` e `fuser -v /dev/dri/*` antes e no fim (a máquina é partilhada).
FASE 1 (leia DENTRO da worktree, só os trechos): [BUGS Bug #14](../BUGS_motion_nodes.md) (linha ~1355, inteiro);
        doc 110 [§14.2](../110_ciclo_6_valor_e_pulso.md) (linhas ~995–1062: o fio da navalha do `pulse.beat` e o
        custo de preparar `value.attribute`+`value.reduce`); doc 120 [§4.1, a nota «a decisão é do dono» (~205) e o
        «nomeado e não perseguido» (~400)](../120_ciclo_12_os_tectos_confortaveis.md); doc 103 §5, a linha do ciclo 12.

OS ABERTOS (feche TODOS, ou recuse cada um por MEDIÇÃO com o número; desenho e kill-criterion escritos ANTES):

 (1) ⛔ **Bug #14 — formas carimbadas da SAÍDA de uma zona de simulação não aparecem no ecrã.** O `coze_o_quadro`
     produz as estrelas nos 180 quadros, nas duas rotas e ao elemento; o app não as desenha (nem a cruz da posição),
     com `PH2D_GPU_COOK=0` também; o log `[formas] 3 copias … pela PLACA` sai UMA vez; `PH2D_PAN_DIAG=1` diz que nos
     quadros seguintes o app não tem forma para desenhar. O mesmo carimbo ANTES da zona desenha. ⇒ a perda está
     entre o cozimento e o desenho (o `dispatch` à volta de uma zona — o pump, o recuo/scrub, a publicação dos
     externals das formas — ou o passe das formas), não no cozimento.
     Barra: um gate pela porta que o APP usa (o `dispatch` inteiro, com `HeroScreen` ou o equivalente da casa) que
     reprova hoje e passa com a cura; a `=27` volta a ter ESTRELAS (é a cena que o bug deixou só com posições) e o
     gate `the_fireworks_draw_the_same_stars_on_both_routes` continua verde. Varra a FAMÍLIA antes de curar
     (CLAUDE.md §0.10): toda cena com `source.shape`/`motion.duplicator` a jusante de um `sim.zone` (censo pelo
     registo de cenas, nunca à mão) — quantas desenham hoje e quantas depois.
 (2) **O pulso do metrónomo (`pulse.beat`) diverge ±1 tique entre CPU e placa nos empates batida-tique** (a CPU usa
     o playhead `f64`, a placa `f32`; tiques 36 e 72 da fixtura decimal), e o gate de paridade dele
     (`parity_pulse_beat`, [`gpu_cpu_parity_pulse.rs:669`](../../../crates/ph2d-gpu-cook/tests/it/gpu_cpu_parity_pulse.rs))
     só corre 6 tiques e nunca chega lá; a paridade do `sim.spawn` SALTA esses 4 de 91. Padrão-ouro: curar a
     CAUSA — as duas rotas decidirem a batida pela MESMA aritmética (p. ex. o índice do tique inteiro e o `dt`, em
     vez de um tempo em `f32`) — e então o gate corre a fixtura inteira e o spawn deixa de saltar tiques. Se a cura
     for recusada por medição, o gate passa a correr a fixtura inteira e afirma a divergência DECLARADA nos empates
     (e paridade exacta fora deles). Mutação: a lei antiga a sangrar no tique 36.
 (3) **Preparar os passes de `value.attribute` + `value.reduce` na placa custa ~`0,09` ms de CPU por quadro**
     (`0,125` contra `0,033` do condutor constante; doc 110 §14.2) — é o que come o ganho do «condutor na placa».
     Meça primeiro ONDE (o que se refaz por quadro que podia ficar: bind groups, uniforms, buffers, o plano).
     Kill-criterion: ≤ `0,05` ms, medido intercalado no mesmo processo pela porta do produto
     (`motion_ciclo6_relogio_probe.rs`), com a paridade ao bit.
 (4) **O tecto de `32 768` por nó foi medido com o carimbo na CPU, e o doc 120 diz que «nesse dia esta tabela
     morre»** — o carimbo FOI à placa (doc 120 §8.4–§8.6: `7,30 → 0,72` ms no proxy de telemóvel). CLAUDE.md §0.0:
     quem move o número que tornava algo inalcançável reconfere a nota. Meça de novo, com FORMA e SIMULAÇÃO, nas
     duas placas (a integrada é o proxy de telemóvel), a cena `PH2D_MOTION_OBJ_SMOKE=17`, até ao penhasco, e
     escreva a tabela nova com o recurso que limita. ⛔ O NÚMERO é do dono (ele mandou manter em 24/09): leve-lhe a
     tabela e a recomendação; não mude o tecto sem a ordem dele.
 (5) **O custo de submissão da leitura de volta na NVIDIA** (`0,34`–`0,42` ms, doc 120 ~400, «nomeado e não
     perseguido»: empacotar os params não o moveu; a cura nomeada é enfileirar a leitura no MESMO submit do
     cozimento). Audite contra o código de hoje (a ponte mudou muito desde 24/09) e meça na iGPU também — o alvo é
     o telemóvel. Feche ou recuse com o número.

⚠️ ANTES DE CONSTRUIR (CLAUDE.md §5.0): audite CADA item contra o CÓDIGO de hoje — seis linhas acabaram de entrar
   no `main` (Vector, UIUX, MiroClone, components, PainterWatercolor, esta). Escreva um doc novo do ciclo
   (o número conta-se no `README.md` do módulo — o último é o 121) com o estado MEDIDO de cada item, o desenho e
   os kill-criteria ANTES de construir; a nota de estado no doc 103 §5 aponta para ele.

JÁ FECHADOS — não reabra (auditados em 07/10):
 · os sete do ciclo 6 → doc 110 §14 · o painel lateral de params → apagado em 17/09 (doc 103 §5, nota «FECHADO»)
 · as estrelas grandes esticadas na iGPU → `0,88` ms ≤ `1,0` (doc 121 §9.12) · a emissão tracejada por cópia →
   recusa medida · a mordida do traço rente e o SIGSEGV da RTX → curados (handoff de integração §6/§9.18)
 · o contacto a `16 384` e o p95 a `4 096` → fechados pelo rapier `parallel` (handoff §14) · o rolamento nas pilhas
   de discos → fechado pelo oráculo Box2D (doc 121 §9.24; a rampa perto do limiar é divergência REGISTADA)
 · o `fk.rs` duplicado (Bug #11) → a `ph2d-rig-kinematics` + portão · o `orient_world` do `rig.skeleton` → o dono
   viu o mundo como a resposta certa; nomeado, não pedido.
 Fora do módulo (só registe ao integrador, não é desta linha): a sonda `FIELD_DOC_VERSION` do
 `scripts/collision-surface.sh` está CEGA (a const saiu com o 3D, ADR-0179) e imprime `✗` em toda integração.

DoD: os cinco itens fechados ou recusados por medição, no doc novo; a nota do doc 103 §5 atualizada; o Bug #14
com a cura e as lições no BUGS; gate batched verde (`nextest-impacted` com a base da linha, check com warnings
negados, clippy das crates tocadas, fmt, censos, `doc-index`, e a suíte da placa); prova de mutação (uma por lei
nova, a sangrar, pré-voo e corrida limpa); a `=27` com ESTRELAS e a `=117` ao bit; handoff de integração com uma
secção nova (a superfície de colisão); memória (as lições nas famílias; o índice ≤ `22 000` bytes — está a `~21 9xx`
no `main`, e outras sessões têm entradas por comitar no primário); smoke ao dono FOTOGRAFADO em `--release`, em
passos numerados (a `=27` com as estrelas a estourar, e a tabela do tecto do item 4 em palavras dele); NÃO integra,
NÃO pusha. No relatório: ficou ALGUMA coisa em aberto?
═══════════════════════════════════════════════════════════════════
