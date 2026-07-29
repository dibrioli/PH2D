# 29 — A SIM FORA DA THREAD DO FRAME: o desenho pronto, o que foi construído, e a UMA decisão que falta

> **Estado: desenhado, construído até o produto COMPILAR, e RECUADO deliberadamente** (2026-07-29).
> O código do produto está guardado verbatim em
> [`29_offthread_sim_desenho_pronto.rs.txt`](29_offthread_sim_desenho_pronto.rs.txt) — ele **não é
> código morto na árvore**, é texto, porque um módulo sem chamador é uma segunda resposta esperando
> alguém chamá-la.

## 1. Por que a wave existe (os números, não a intuição)

O agendamento está **esgotado** — seis rodadas de smoke fecharam realimentação (§5.31), orçamento
fixo (§5.32), atribuição (§5.33), catraca (§5.34), régua pregada (§5.35) e o passo atômico (§5.37).
O que sobra é o **custo por célula**, e ele é o piso declarado da física:

- **16 ns por visita de célula-passe**, medido (`measure_how_many_cells_a_pass_actually_visits`);
- **zero transcendental** no laço quente (o mecanismo do doc 24 não se aplica aqui);
- a faixa por-linha **já está justa** num traço só (1,5× as células ativas);
- ADR-0134, textual: *"o ~7,5 ms determinístico É o teto escalar serial desta física com aritmética
  f64-espelho-do-JS"*.

⇒ **A taxa VISUAL da água é a taxa de PASSOS** (o composite roda quando um passo completa), e enquanto
a sim divide a thread do frame essa taxa é `orçamento ÷ custo do passo` ≈ **15 Hz** numa poça de 4K.
Foi isso que o Enio viu como *"lenta e truncada"* com o FPS intacto.

Num núcleo próprio a água ganha os 1000 ms/s inteiros ⇒ `1000 ÷ 33 ≈ 30 Hz`, **o dobro**, e o frame
para de pagar qualquer coisa além do composite (1,9 ms medidos).

## 2. O desenho: o engine VIAJA, não é compartilhado

⚠️ **Sem mutex, e a escolha é de SEGURANÇA.** Um `Arc<Mutex<Engine>>` com os 76 sítios de acesso tem
um modo de falha silencioso — travar duas vezes no mesmo escopo é *deadlock*, não erro de compilação.

No desenho construído o engine viaja por canal e `WetSession::eng()` devolve `&mut Engine`: **o borrow
checker garante um acesso por vez**, e o pior caso de um acesso é *esperar um ESTÁGIO* (3-10 ms), não
um passo (33-38) — que é exatamente o que o passo retomável da §5.37 comprou.

⚠️ **E o borrow checker JÁ SE PAGOU durante a construção:** ele exigiu a porta
`engine_and_pigment()` (`E0499`, em compilação) no composite, que precisa das camadas do engine E do
rascunho ao mesmo tempo. **Sob um mutex, essa mesma forma seria um deadlock em runtime.**

Peças, todas no texto guardado:

| peça | o que faz |
|---|---|
| `EngineSlot::{Here, Away}` | onde o engine está; os canais moram no worker |
| `SimWorker` | os dois canais + `want: AtomicBool` + `steps: AtomicU64` |
| `worker_loop` | recebe o engine, roda **estágios** no ritmo de 40 Hz, devolve em fronteira de estágio quando `want` |
| `WetSession::eng()` | o engine aqui e agora (bloqueia ≤ um estágio) |
| `WetSession::hand_off_sim()` | devolve ao worker — o tick faz no FIM, uma vez |
| `sim_steps()` / `seen_steps` | *há algo novo a compositar?* — sem isto o tick traria o engine a cada frame e o worker perderia ~30% do núcleo |

O tick deixa de dar passos: ele **mostra** o que o worker fez.

Mais dois fatos descobertos construindo, que a próxima sessão não precisa re-descobrir:

- **`Engine` precisa de `on_dab: Option<Box<dyn FnMut(f64, &Dab) + Send>>`** — sem o `+ Send` o motor
  inteiro não é `Send`. Nenhum caminho de produto preenche esse campo (é sonda), então o bound não
  custa nada.
- **O `Engine` é `Send` de resto** (dados planos); nada mais barra a viagem.

## 3. A UMA decisão que falta — e é de produto, não de código

⚠️ **Off-thread troca *"a sim avança por TICK"* por *"a sim avança em TEMPO DE PAREDE"*.** As duas não
coexistem: se a UI espera o trabalho devido para manter o determinismo, o frame volta a pagar o passo,
e o ganho inteiro evapora.

**Tempo de parede é o correto para uma ferramenta interativa** (é o que o app de referência faz), mas
a consequência é concreta e está medida: **das 903 gates do tool, 9 falham** — 5 são leitura do motor
depois de um tick (mecânicas: trazer para casa) e **4 encodam a promessa antiga** (*"N ticks ⇒ N
passos"*), entre elas os gates do escorrido do undo (§ o drip de 2026-07-26).

A saída é a que a suíte de aceitação do engine já usa: **quem precisa de determinismo dirige o motor
direto.** Duas portas `#[cfg(test)]` foram escritas e funcionam:

- `wet_step_sync(t, n)` — traz o engine para casa, roda `n` passos, composita. **Os 4 gates do drip
  passaram com ela.**
- `wet_bring_home(t)` — o `sess.eng()` explícito para gates que LEEM o motor.

**O que resta é plumbing de assinatura** (os helpers de leitura dos gates tomam `&PainterTool` e
precisam de `&mut`, o que cascateia), e é isso que consumiu a sessão sem fechar.

## 4. O que a próxima sessão faz, em ordem

1. Reinstalar `offthread.rs` do texto guardado + o `+ Send` no `on_dab`.
2. `WetSession`: `engine: EngineSlot`, `worker: Option<SimWorker>`, `seen_steps: u64`; `acc` e o
   módulo `budget` **saem** (o frame não paga mais passo nenhum ⇒ 7 gates do orçamento e 2 sondas
   morrem com ele — **é o resultado honesto, não desperdício**).
3. Reescrever os 76 sítios `sess.engine.` → `sess.eng()`; leitura imutável → `sess.engine.here()`.
4. O tick novo: `fresh || facts_moved` → reconcile → composite → `hand_off_sim`.
5. Os gates: `wet_step_sync` nos 4 do drip; `&mut` nos helpers de leitura.
6. **Medir** (`measure_what_the_sim_time_budget_buys` re-apontado): a taxa da sim e o custo do frame.
7. **Gates NOVOS da wave** (nenhum existe ainda — é o buraco real deste recuo):
   - o frame **não paga passo** (o tick não chama `step_stage`);
   - `eng()` espera **no máximo um estágio** (razão, não wall-clock absoluto);
   - a água **avança sem tick** (o worker é o dono do relógio);
   - uma ação de canvas **drena** o passo em voo (o `Engine::drain_step` já existe e já é gateado).

## 5. Por que recuar foi a decisão certa

O produto compilava; a suíte não. Shipar um refactor de **posse do motor** com 9 gates vermelhos
contradiz tudo que esta linha fez em 44 commits — e a taxa de erro mecânico das minhas próprias edições
em massa estava subindo (três regex excessivos em sequência, um deles deixando o arquivo com delimitador
aberto). *Um recuo com o desenho guardado e a decisão nomeada custa uma sessão; um merge meio-feito na
posse do grid custa a confiança em tudo que o módulo afirma.*
