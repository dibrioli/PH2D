# 29 — A SIM FORA DA THREAD DO FRAME

> **Estado: CONSTRUÍDO e verde** (2026-07-29). 902 gates do tool, suíte do motor
> nos DOIS perfis, `check --workspace` limpo, fingerprint intacto.
> Pendente de **smoke**.

## 1. Por que a wave existe (os números, não a intuição)

O agendamento estava **esgotado** — seis rodadas de smoke fecharam realimentação
(§5.31), orçamento fixo (§5.32), atribuição (§5.33), catraca (§5.34), régua
pregada (§5.35) e o passo atômico (§5.37). O que sobrava era o **custo por
célula**, e ele é o piso declarado da física: **16 ns por visita de célula-passe**,
zero transcendental no laço quente, faixa por-linha já justa.

⇒ **A taxa VISUAL da água é a taxa de PASSOS** (o composite roda quando um passo
completa), e enquanto a sim dividia a thread do frame essa taxa era
`orçamento ÷ custo do passo` ≈ **15 Hz** numa poça de 4K. Foi isso que o Enio viu
como *"lenta e truncada"* com o FPS intacto.

## 2. O desenho: o engine VIAJA, e o slot é um `Deref`

⚠️ **Sem mutex, e a escolha é de SEGURANÇA.** Um `Arc<Mutex<Engine>>` com 87
sítios de acesso tem um modo de falha silencioso — travar duas vezes no mesmo
escopo é *deadlock*, não erro de compilação.

⚠️ **E o slot é `Deref`/`DerefMut`, não um método** — foi isto que fez a wave
caber numa sessão: `sess.engine.foo()` continua compilando nos **87 sítios**, e o
*field splitting* de que o composite depende (ele precisa das camadas do motor
**e** do `pigment`/`base` ao mesmo tempo) sobrevive. Um `fn eng(&mut self)`
emprestaria a sessão inteira e cascatearia por toda a suíte — foi exactamente o
que consumiu a tentativa anterior sem fechar.

O preço do `Deref` é que ele **panica** quando o motor está com o worker, então a
regra é que **toda porta que alcança a sessão chama `bring_home()` primeiro**.
São dez no produto, e o pânico nomeia o conserto no sítio exato.

| peça | o que faz |
|---|---|
| `EngineSlot::{Here, Away}` + `Deref` | onde o motor está; os 87 sítios não sabem que ele viaja |
| `SimWorker` | os dois canais + `want: AtomicBool` + `steps: AtomicU64` |
| `worker_loop` | recebe o motor, roda **estágios** no ritmo de 40 Hz, devolve em fronteira de estágio |
| `bring_home()` | bloqueia ≤ um estágio — a porta de quem vai ESCREVER (dab, ações, commit) |
| `try_bring_home()` | espera `TICK_WAIT` e desiste — a porta do TICK |
| `hand_off_sim()` | devolve ao worker; nunca com traço aberto |
| `sim_steps()` / `seen_steps` | *há algo novo a compositar?* — respondido pelo ATÔMICO, sem trazer o motor |

O tick deixou de dar passos: ele **mostra** o que o worker fez.

## 3. `TICK_WAIT = 4 ms` — medido, com mecanismo

O tick pode esperar pouco (perde taxa) ou o estágio inteiro (perde o frame). A
varredura, 4096², 120 frames a 60 Hz:

```text
  espera      0 ms    1 ms    2 ms    4 ms    8 ms
  sim         23,4    26,9    28,9    33,4    33,9   Hz   (nominal 40)
  pior tick    7,4     8,2     7,7     9,0    10,7   ms
```

⚠️ **O joelho em 4 ms é o `IDLE_SLEEP`**: quando o worker não tem o que simular
ele dorme, e só olha o `want` ao acordar — a granularidade da resposta dele **é**
o sono. Esperar menos erra um worker adormecido; esperar mais compra o resto de um
estágio, e aí o frame volta a pagar trabalho de sim (a versão bloqueante media
**60,6 ms** de pior tick na poça de três traços).

## 4. O resultado

| | antes | depois |
|---|---|---|
| taxa da água (um traço, 4K) | ~15 Hz | **33,4 Hz** (84% do nominal) |
| tick p50 | ~2 ms | **0,049 ms** |
| pior tick | 55-73 ms | **9,0 ms** |

O frame não paga mais passo nenhum — só o composite.

## 5. O que os gates afirmam (e as duas defesas que não são observáveis)

Seis gates em `wetpaint/offthread_tests.rs`: o frame não roda estágio (arch-gate
com controle positivo nas duas pontas) · o tick termina com o motor fora de casa ·
a água avança **sem tick nenhum** · a taxa é utilizável sob um laço de frames
REAL · o tick não espera um estágio · uma ação de canvas alcança o motor.

⚠️ **Duas defesas ficam documentadas em vez de gateadas**, porque no regime que
shipa elas não são observáveis: o `hand_off_sim` desarmar o `want` (só morde com
espera ZERO — foi assim que eu produzi o colapso) e não haver `return` no ramo de
falha do pedido (o desenho se autocura). Detalhe e as mutações no próprio arquivo.

⚠️ **E três lições de gate desta sessão, todas sobre mim:**

1. **A atribuição errada custou três gates.** Eu culpei o `return` pelo colapso de
   **33,4 → 0,5 Hz** *antes de reler a minha própria medição*, que já dizia o
   contrário (consertar o `return` deixou 0,5; desarmar o `want` levou a 23,9).
2. **A fixture é parte do gate, nas DUAS direções.** Encolher a poça para matar
   uma flake tirou os dentes do gate do `want` (o defeito só existe quando o
   worker responde mais devagar que a espera); e o gate da espera precisava da
   poça PESADA (com um traço, a versão bloqueante media 10,5 ms e passaria).
3. **`reconcile_facts` não era porta.** Ele sai antes do `bring_home` quando os
   facts não mudaram — o caso comum —, e uma porta que só às vezes abre não é
   porta. O gate da ação de canvas pegou isso ao vivo, com o pânico certo.

## 6. A frente seguinte — FEITA (ADR-0145)

O Enio autorizou (*"rayon"*, 2026-07-29) e os passes row-disjuntos foram
paralelizados: **[ADR-0145](../architecture/decisions/0145-wet-paint-solver-row-parallel-passes-rayon-exception.md)**,
a 2ª exceção sancionada ao "sem rayon" do repo. Detalhe, tabelas e a prova de
byte-identidade moram lá; o resumo é:

* **três** passes entraram, não dois — o `rebuild_active_region` tem **3 de 4**
  sub-passadas row-disjuntas (só a saia fica serial), e esta seção não tinha
  visto isso;
* **um passo inteiro: 16,08 → 10,34 ms (1,56×)**, pior caso 26,43 → 19,07,
  medido pela porta do produto no mesmo binário;
* `advect`, `build_flow_field`, `drying_pass` e a saia do rebuild ficam
  **seriais por semântica**, cada um com o mecanismo escrito no `src/par.rs`.

⚠️ **A afirmação do repo que caiu no caminho:** o header do
`measure_wetpaint_tick.rs` dizia *"o solver é Gauss-Seidel em toda parte"* — o
ADR-0134 nomeia **dois** mecanismos sequenciais e eles somam **34%** do passo, não
o passo inteiro. O `project` é **Jacobi** e o `smooth_velocity` é gather puro.

⚠️ **E o que sobra não tem caminho de CPU:** os 60% restantes são os quatro
sequenciais. A próxima alavanca é a **GPU**, que quebra o port 1:1 e o
fingerprint pinado — ADR próprio, decisão do Enio.

## 7. Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
env PH2D_WETPAINT_SMOKE=1 PH2D_FLUID_PROFILE=1 cargo run -p ph2d-host-desktop --release
```

Canvas **4096**, pincel grande, Wet Paint no dropdown. O que olhar:

1. **A água tem de correr LISA** — é a entrega. No log, `agua: sim media` deve
   sumir (o frame não simula mais) e `tool-tick` deve ficar em frações de ms.
   ⚠️ **Desde o ADR-0145 a TAXA também subiu:** conte os `composite ... xN` na
   janela de 2 s — `N/2` é a taxa da água em Hz. O passo ficou 1,56× mais barato
   numa poça de 5 M células, então o número tem de subir na MESMA poça (pincel
   grande, várias faixas sobrepostas), não numa risca fina.
2. **Pintar durante o escorrido** — o dab traz o motor para casa; o traço não pode
   engasgar.
3. **Undo (Ctrl+Z) leva o escorrido** — os gates do drip agora dirigem o motor
   direto, então esta metade é do smoke.
4. **Wet canvas / Dry canvas / Fast dry** com a água correndo — as portas
   bloqueantes.
5. **Sair do Wet Paint e voltar** — a sessão morre e nasce; nenhuma thread deve
   sobrar (a do worker morre com o canal).
