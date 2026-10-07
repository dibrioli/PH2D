═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)

⛔ Fale com o Enio SEMPRE em PT-BR, curto e sem jargão (CLAUDE.md §0.8). Smoke de DESEMPENHO sempre em `--release`.
⛔ Ordens que continuam: integrar só por ordem do dono; UM ciclo, UMA rodada de medição (CLAUDE.md §0.10); série só
   com o porquê escrito antes; NUNCA matar o pai de um processo (LOGOUT do dono em 05/10). O relógio do APP não vale
   acima de `load ~5` (`cat /proc/loadavg` antes de cada célula) — mas comparar VELOCIDADE entre variantes NÃO espera
   calma: variantes no MESMO processo, intercaladas, o MÍNIMO ([MEDIR_VELOCIDADE.md](../../DevOps/MEDIR_VELOCIDADE.md)).
⛔ Ordem do dono (07/10): *«resolver num único ciclo o Ciclo 6 completo»* — os abertos que o ciclo 6 (VALOR & PULSO)
   declarou ao fechar em 15/09 e que ninguém retomou. O ciclo está ✅ na fila; o que fecha agora é a LISTA de abertos.

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -5 && git status
        --short --ignored | grep -v target/ ; rebase no main se ele andou (conflito fora dos seus ficheiros ⇒ PARE).
        `pgrep -af 'ph2d|cargo|rustc'` e `fuser -v /dev/dri/*` antes e no fim (a máquina é partilhada).
FASE 1 (leia DENTRO da worktree, só os trechos): doc 110 ([110_ciclo_6_valor_e_pulso.md](../110_ciclo_6_valor_e_pulso.md))
        §3, §6 (inteiro, com «Divergência DECLARADA», «⚠️ Um VERMELHO pré-existente» e o ⏳ da linha ~239), §8.6, §9.7,
        §11.4, §11.5, §13; doc 102 §2 e a W1 (b) (`Compact.complement`, linhas ~197–222); doc 103 §5 a nota de 15/09
        (linha ~196); a memória [chave e texto do mesmo tipo](../../../project-memory/feedback_a_key_and_a_text_of_the_same_type_is_a_defect_waiting.md).

OS ABERTOS (a lista inteira — feche TODOS ou recuse cada um por medição, com o número):
 (1) W1(b) — a porta ≠ 0 ligada ainda derruba o nó para a CPU: é o COMPLEMENTO de um `Compact`
     (`StreamOp::Compact { port, predicate }` + `complement`, doc 102 W1 (b); o caso medido é o `sim.lifetime.pulse`).
     Barra: a cadeia com a porta ≠ 0 ligada fica `fully_gpu`, paridade CPU × placa ao bit, e o censo de rota diz
     quantas cenas isto devolve à placa (ANTES e DEPOIS, pela porta do produto).
 (2) O condutor que é ele próprio um estágio de GPU faz uma volta pela CPU (doc 110 §6, ⏳ linha ~239: «uma cópia de
     4 bytes device-side evitaria»). Barra: zero leituras de volta (readback) por quadro nessa cadeia, com gate.
 (3) W3b — três CHAVES partilhadas por perguntas diferentes (`clamp` em `value.map_range`/`value.mix`, `step` em
     `value.quantize`/`pulse.counter`, `value` em `value.number`/`value.table`). O §9.7 deixou-as «declaradas, não
     curadas» porque renomear faria todo documento gravado perder o valor autorado (override desconhecido cai no
     default, em silêncio). ⚠️ Essa razão expira se a renomeação vier com MIGRAÇÃO ao abrir o projecto (o nome antigo
     lido como o novo, por nó). PRIMEIRO meça: alguma delas ganhou um consumidor PARTILHADO desde 15/09 (então é um
     `substeps` e a cura é obrigatória)? Depois decida pelo padrão-ouro, com o preço escrito: renomear + migrar (gate
     de um projecto ANTIGO gravado a abrir com o valor intacto) ou manter com uma catraca que reprove o primeiro
     consumidor partilhado. ⛔ Nunca renomear sem migração.
 (4) `value.cursor` e `value.table` emitem `102 400` cópias do MESMO número (doc 110 §11.4). A cura é o COMPRIMENTO
     (`1` em vez de `N`), e o §11.4 diz o que a impediu: nem todo consumidor de VALOR difunde um campo de comprimento
     `1` (um `Read` simples no dispositivo julga-o ausente; o `ReadBroadcast` existe por isso). ⇒ CENSO de todos os
     consumidores de valor (do registo, nunca uma lista à mão), cada um provado a difundir na CPU E na placa, e só
     então o comprimento muda. Barra: a costura `102 400 → 1`, a cena `=117` e os tutoriais ao bit.
 (5) O RELÓGIO do ciclo (doc 110 §11.5). ⛔ A sonda `probe_the_price_of_driving_one_param` foi APAGADA (doc 110 §6:
     os dois lados dela corriam o cozedor da CPU — media outro programa e devolvia um `1,21×` plausível); não a
     procure nem a ressuscite. O instrumento novo é device × CPU no MESMO processo, intercalado, o mínimo (CLAUDE.md
     §2), pela porta do produto — e o §6/§11.5 dizem que isso pede um `GpuContext` no arnês, a wave nomeada; construa-a,
     com um controlo que prove que a metade «device» corre MESMO na placa (o registo de rota da cena `=116`).
 (6) O gate `#[ignore]` de paridade que vive a um ULP da barra (doc 110 §6, «Um VERMELHO pré-existente»): meça com o
     autor da barra ao lado (o histórico do ficheiro) e decida a barra pelo vale dos dois lados, ou cure a causa.
 (7) Confira o §8.6 («o que isto ainda NÃO compra»): os `pulse.*` no dispositivo e os consumidores deles — o
     `sim.spawn` (contagem de nascimento dependente de dados) ainda os arrasta para a CPU? Se sim, é deste ciclo.

⚠️ ANTES DE CONSTRUIR (CLAUDE.md §5.0): audite CADA item contra o CÓDIGO de hoje — passaram três semanas e várias
   ondas por estas crates (o `dt` uniform, o motor de contacto, a `ph2d-motion-kit`); algum pode já estar fechado sem
   registo, e o número que a lista diz (`102 400`, `32 de 35`) pode ter mudado. Escreva no doc 110 um §14 «O FECHO DOS
   ABERTOS» com o estado MEDIDO de cada um, o desenho e os kill-criteria de cada um ANTES de construir.

DoD: os sete itens fechados ou recusados por medição (com o número), no doc 110 §14; a nota de 15/09 do doc 103 §5
atualizada; gate batched verde (`nextest-impacted` com a base da linha, check com warnings negados, clippy das crates
tocadas, fmt, censos, `doc-index`); prova de mutação nova (uma por lei nova, a sangrar, pré-voo e corrida limpa); a
cena `=117` e o tutorial 06 continuam ao bit (ou o tutorial refeito, se o que o artista vê mudou); handoff de
integração com uma secção nova (a superfície de colisão — ⚠️ chaves renomeadas e a migração conflitam com qualquer
linha que toque nos mesmos nós); memória (as lições nas famílias, o índice abaixo do tecto); smoke ao dono FOTOGRAFADO
em `--release` (a `=117`, e um projecto antigo gravado a abrir intacto se houver migração), em passos numerados;
NÃO integra, NÃO pusha. No relatório: ficou ALGUMA coisa em aberto?
═══════════════════════════════════════════════════════════════════
