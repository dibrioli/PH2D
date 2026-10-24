# A vez da placa — todo uso da placa espera a sua vez (18/10)

> Lei curta: **toda árvore de processos que passa pelo `scripts/ph2d-run.sh` tem um PORTEIRO**. Ele pede a
> placa ao cadeado partilhado quando um processo da árvore precisa dela e segura-a **enquanto um processo que
> a pediu vive** (§6, 23/10 — até então, até o comando acabar). Com uma segunda placa na máquina há **duas
> filas**: os testes vão para a integrada, e a principal fica para as medições, as fotos e a Luz 2D (§7). Ninguém precisa de se lembrar do `PH2D_GPU=1`: o código pede a vez sozinho no ponto onde o processo
> entra na placa. Fora da porta não há porteiro e **nada espera** — o produto, o app aberto à mão pelo dono, a CI.
> Handoff da onda: [HANDOFF_A_VEZ_DA_PLACA_2026-10-18.md](HANDOFF_A_VEZ_DA_PLACA_2026-10-18.md).

## §1 — Porquê

Até 17/10 a vez era OPCIONAL: só `PH2D_GPU=1 bash scripts/ph2d-run.sh …` fazia o `flock` exclusivo em
`$XDG_RUNTIME_DIR/ph2d-gpu.lock`. Na rodada da Luz 2D que SEGURAVA a vez (handoff W6c §2–3, `line/Luz2D`)
entraram na placa sem a pedir binários de teste de outras linhas (`it-*`, `ph2d_render-*`, `ph2d_tool_paint`)
e o smoke de outra linha (`kwin_wayland --virtual` + `target/smoke/ph2d-host-desktop`): ~1 h de rodada sem
valor. O dono não quer parar as outras linhas para medir: quer que esperem a vez (minutos).

⛔ **A exclusão continua exclusiva**: duas linhas na placa ao mesmo tempo penduraram o driver em 14/09
([TETOS §3](TETOS_DE_RECURSO_POR_LINHA.md)).

## §2 — Como funciona

| quem pede a vez | onde |
|---|---|
| todo processo Rust, antes do adaptador | `ph2d_gpu::vez_da_placa()`, chamada pelo `GpuContext::new` ([`vez.rs`](../../crates/ph2d-gpu/src/vez.rs)) |
| teste que pede o adaptador por fora do `GpuContext` | a mesma chamada, à mão — o portão `a_vez_da_placa::todo_pedido_de_adaptador_passa_pela_vez` (`ph2d-gpu`) recusa o pedido sem ela |
| o que não é Rust (o `kwin --virtual` das fotos e das medições, um oráculo) | `bash scripts/vez-da-placa.sh` antes de o lançar — o [`fotografa_cena.sh`](../Components/ferramentas/fotografa_cena.sh) e o `mede_custo_no_app.sh` já o fazem e entram na porta sozinhos |
| uma medição, do princípio ao fim | `PH2D_GPU=1` (a vez já à partida, como antes) |
| um binário cru, o kwin, o wine, o Blender, o Godot por fora da porta | **recusado** pelo guarda (R9, [`tecto-de-recursos.sh`](../../.claude/hooks/tecto-de-recursos.sh)) |

**O protocolo** (uma pasta por árvore, `PH2D_GPU_VEZ`): `porta` (FIFO onde o cliente escreve «pid nome»),
`vez.<pid>` (a vez é desse processo, até ele acabar — §6), `recusa` (o porteiro desistiu, com quem segura),
`porteiro` (o pid dele); a fila de testes tem os mesmos com o sufixo `-testes` (§7). Dois clientes: [`vez.rs`](../../crates/ph2d-gpu/src/vez.rs) e [`vez-da-placa.sh`](../../scripts/vez-da-placa.sh).

**As quatro perguntas do desenho:**

1. **Onde se impõe** — no código, onde o processo entra na placa: um guarda não sabe se um `cargo test` toca a
   placa, e o código sabe. O filho de uma árvore que já tem a vez NÃO a pede de novo (esperaria pelo próprio
   pai): o sinal herdado é a pasta do porteiro no ambiente; uma porta dentro de outra (um portão que re-entra)
   usa o porteiro de fora. Só o porteiro tem o descritor do cadeado (a lição do `sccache`, 29/09).
2. **O app do dono aberto à mão** — fica de fora: sem porteiro, nada espera, nada avisa. Uma vez que
   seguisse o app aberto prenderia todas as linhas durante horas (o «Monitor do Sistema» do dono prendeu a
   rodada da Luz 23:39–07:50, W6c §3). A medição é que tem de o ver: o detector `alheios.py` acusa-o.
3. **Fome** — o `flock` do Linux dá a vez **por ordem de chegada** (medido, §3): os testes que chegam durante
   um processo de medição entram antes do processo seguinte, que entra antes dos testes que chegarem depois.
   Uma rodada e um fluxo de testes alternam; nenhum mata o outro de fome.
4. **O prazo** — o mesmo de sempre: quem segura e pendura é morto pelo `RuntimeMaxSec` do scope, a porta sai e
   o porteiro solta. A vez pedida a meio espera até o prazo (sem `PH2D_GPU_ESPERA`); com `PH2D_GPU=1` desiste
   aos 300 s (`exit 75`) e diz quem segura.

## §3 — Medições

| o quê | resultado |
|---|---|
| quando é que um processo aparece no `nvidia-smi` (sonda wgpu 29, fases de 3 s, amostra a 0,5 s) | `Instance` sozinha: **não**; depois do `request_adapter`: **sim** ⇒ a vez pede-se antes do adaptador |
| `flock`: 5 esperas chegam por ordem, o detentor solta (20 ensaios) | **20/20** pela ordem de chegada |
| `flock`: 1 espera bloqueada; o detentor solta e volta a pedir 0 / 10 / 50 ms depois | a espera entra primeiro **20/20** em cada |
| os dois acima com 48 queimadores na mesma fatia (load 17 → 58) | **20/20** e **20/20** — iguais |
| prova do porteiro (cadeado próprio): [`vez-da-placa.prova.sh`](../../scripts/vez-da-placa.prova.sh) | 14/14 |
| a rodada da Luz com testes e foto em paralelo, e o controlo | §3.1 |

### §3.1 — A prova com a placa real (18/10, 08:45–09:35)

A régua da Luz 2D (W6c: `ronda.sh` com `RENASCE_SE_DEGRADA=0` e `-`, o detector `alheios.py` sobre a tabela
INTEIRA do `nvidia-smi`, amostra a 0,5 s), corrida pelo `ph2d-run` velho da `line/Luz2D` (o mesmo cadeado). Vinte
segundos depois de um processo dela começar a MEDIR, disparados do `main` em paralelo, sem `PH2D_GPU=1`: os 48
testes de placa do `ph2d-flip-render` por `cargo-test-narrow` (um processo), os mesmos por `nextest` (um processo
por teste) e uma foto de cena (`kwin --virtual` + o app). Controlo: o mesmo, com `env -u PH2D_GPU_VEZ`.

| processo da Luz | disparos | amostras dos disparos no `alheios.txt` | esperaram a vez | resultado dos disparos |
|---|---|---|---|---|
| 01 | com a vez | **0** (o processo: limpo) | 103 · 108 · 110 s | 48/48 · 48/48 · foto |
| 08 | com a vez | **0** | 105 · 125 · 120 s | 48/48 · 48/48 · foto |
| 07 | **controlo**, sem a vez | **45** (`it-…:ph2d-flip-render`, `kwin_wayland`, o app do `main`) | 0 s — entraram já | 48/48 · 48/48 · foto |

- **Passa** pelo critério escrito antes: com a vez, nenhum disparo entrou na medição; esperaram ~2 min (o resto do
  processo da Luz) e seguiram sozinhos, nenhum falhou nem pendurou; o controlo é acusado. ⚠️ O `nextest` do
  controlo não era controlo (o `ph2d-run` de dentro criou um porteiro novo e esperou 126 s); o teste e a foto eram.
- ⚠️ **«`alheios.txt` vazio em TODOS os processos» não se cumpriu, e não por esta mudança:** 4 dos 9 processos
  saíram sujos por OUTRAS linhas, que ainda não têm a vez — o app da `line/Macros`, um `godot` de um agente (o
  guarda passou a cobri-lo), testes de placa de `ph2d-light2d-gpu` (11 amostras no 08), `ph2d-app-light2d`,
  `ph2d-app-flip`, `ph2d-app-components`, `ph2d-render`. É o §5 em números: a vez só protege quem tem o `main`.
- **Bónus — a pausa de 60 s de placa livre da régua (W6c §5 item 4): MEDIDA, e os 60 s FICAM.** Rodada própria
  (18/10 10:31–11:42; o app no padrão do produto; 60 s × ~6 s intercalados; 5 limpos de cada; ⚠️ desvio: o
  Spectacle do dono, parado na placa durante toda a rodada, ignorado pelo PID no detector da cópia — igual nas
  duas). Critério escrito ANTES: encurta se o c2 difere ≤ 0,03 ms, o arranque (mediana das leituras 5–60) ≤ 0,10 ms
  e nenhum de 6 s sai da faixa dos de 60 s por > 0,05.

  | | 60 s | ~6 s | diferença | limite |
  |---|---|---|---|---|
  | c2 (mediana de 5) | 11,551 (11,511–11,559) | 11,522 (11,510–11,540) | 0,029 | ≤ 0,03 ✓ |
  | arranque (mediana de 5) | 11,768 (11,595–12,457) | 11,543 (11,444–12,823) | **0,225** | ≤ 0,10 ✗ |

  O arranque falha pela letra — a favor da pausa curta, mas o critério não tinha sentido e a dispersão dele é de
  1,4 ms dentro de cada grupo. Não se muda a régua depois de ver o número: para a encurtar, uma rodada nova com um
  critério UNILATERAL (a pausa curta não pode ser PIOR) escrito antes. Os intrusos desta rodada (2 de 12 processos,
  linhas sem a vez) chegaram a meio da medição — nenhuma pausa os apanharia.

⛔ Kill-criterion de uma fila própria (bilhetes): se uma medição futura mostrar o `flock` fora de ordem, a vez
passa a fila. Até lá a fila é código sem defeito para curar.

## §4 — ⛔ Recusas

(As de 14/09 e de W6c §3 são medidas; as outras são desenho, com o porquê ao lado.)

| o que foi pensado | por que ficou de fora |
|---|---|
| impor no GUARDA (recusar o `cargo test` de placa sem `PH2D_GPU=1`) | o guarda não sabe se um teste toca a placa: muitos testes de placa não são `#[ignore]` e saltam sem adaptador (`let Ok(gpu) = … else { return }`), logo o MESMO comando toca ou não a placa conforme o filtro. O guarda fica só com o que não é Rust e com o binário cru (R9) |
| a vez por PROCESSO (cada processo com o seu `flock`) | o `nextest` corre os testes de uma árvore em processos paralelos: serializá-los-ia entre si, e o 1.º à espera morre no `terminate-after` (a luz: 360 s) |
| o `PH2D_GPU=1` à partida em todo comando de teste | seguraria a placa durante a compilação; a vez a meio só a segura do 1.º uso ao fim |
| uma fila própria (bilhetes) | o `flock` já é por ordem de chegada (§3) |
| partilhar a placa entre árvores (cadeado partilhado) | 14/09 (TETOS §3) |
| o app do dono a esperar a vez | prenderia as linhas pelas horas em que fica aberto (§2.2) |
| ~~o porteiro a vigiar o FIM do último uso (soltar quando a árvore pára de usar a placa)~~ — **REVISTA em 23/10, é a lei do §6** | era desenho, não medição: «cada teste seguinte da árvore voltaria a esperar uma medição inteira; suítes longas já são raras». **Medido 18–23/10, a premissa caiu**: ≥ 36 esperas, ≥ 2 h 45 min, e quem segurava era sobretudo o `nextest-impacted.sh` INTEIRO. O receio (o teste atrás de cada medição) desfaz-se com a fila de testes do §7: os testes livres já não esperam medição nenhuma |

## §5 — O que isto NÃO resolve

- ⚠️ **Cada worktree só ganha a vez quando tiver o `main` com ela** (o `ph2d-run.sh`, o guarda e o `ph2d-gpu` são
  da árvore de cada linha). Até lá uma linha velha continua compatível — o `PH2D_GPU=1` dela tranca o MESMO
  cadeado —, mas os testes dela sem `PH2D_GPU=1` ainda entram sem pedir.
- ⚠️ Um `kill -9` na porta deixa o comando (o scope) a correr até ao prazo, e o porteiro solta a placa em ≤ 0,5 s —
  como antes, quando o cadeado era do bash da porta.
- ⚠️ O que o dono abre à mão (o app, o Chrome, o Rebelle) está fora por desenho: a medição continua a precisar do
  detector de alheios.
- ⚠️ O guarda vê o comando escrito; um script que lance o kwin sem chamar o `vez-da-placa.sh` escapa — o
  padrão a copiar está nos dois roteiros (§2).

## §6 — A vez é de quem a PEDIU, não da árvore (23/10, `line/Velocidade` W1)

**A medição** (transcrições das sessões + saídas das tarefas de fundo, 18–23/10; é um piso, porque o `/tmp`
apaga-se no arranque): **≥ 36 esperas, ≥ 2 h 45 min somadas, a maior de 25 min**. Quem segurava, pelos avisos
«à espera da vez»: o `nextest-impacted.sh` inteiro (PainterWatercolor, MiroClone, Luz 2D, UIUX, CodeEditor),
depois o `fotografa_cena.sh` e os oráculos do CodeEditor. A suíte segurava a placa do 1.º teste de placa ao
FIM, também nos minutos em que só corria testes de CPU.

**A lei:** o porteiro guarda os pids que lhe pediram a vez; segura enquanto um vive e larga quando o último
acaba; o pedido seguinte volta a passar pelo `flock` (por ordem de chegada, §3). Pedintes da mesma árvore
juntam-se à vez que ela já tem — o `nextest` continua a correr testes de placa em paralelo dentro da árvore
(a recusa «a vez por PROCESSO» do §4 continua de pé).

- **Cada pedinte espera o SEU `vez.<pid>`.** O `pronto` da árvore deixaria entrar sem a vez quem pede no
  instante em que ela se larga — o cliente pede SEMPRE, mesmo com a árvore na placa.
- **O `PH2D_GPU=1` não muda:** o pedinte é a própria porta (o `$PPID` do `vez-da-placa.sh`), que vive o comando
  inteiro. O kwin das fotos idem: o pedinte é o roteiro que o lança.
- **`PH2D_VEZ_ARVORE=1`** devolve a lei antiga (a árvore segura até ao fim), sem recompilar: é o interruptor
  da medição A × B.
- **Compatível entre versões:** o porteiro novo e o velho trancam o MESMO cadeado; uma worktree sem o rebase
  continua com a lei velha só para ela.
- ⛔ **O pedido viaja num ÚNICO `write`, e o porteiro guarda o pedaço que um prazo corte.** O `writeln!` num
  `File` escrevia «pid», « nome», «\n» em `write`s separados; sob carga o `read -t` do porteiro vencia a meio,
  largava o pedaço e o pedinte esperava para sempre (23/10, prova de ponta a ponta: 2 de 548 testes a 180 s;
  existia com o prazo velho de 5 s, só mais raro). E o `read` sem `IFS=` come o espaço do começo do resto
  («123» + «resto»). Prova: «um pedido escrito aos pedaços não se perde», vermelha sobre a W1.

**Prova:** [`vez-da-placa.prova.sh`](../../scripts/vez-da-placa.prova.sh) — os casos «a vez larga-se quando o
pedinte acaba», «quem volta a pedir espera a vez de novo» e «dois pedintes da mesma árvore entram juntos»;
os dois primeiros VERMELHOS sobre os scripts do `main` de 23/10 (15/17), verdes com a lei nova. Cliente Rust:
`a_vez_da_placa::protocolo::a_vez_de_outro_pid_nao_dispensa_o_pedido` (`ph2d-gpu`).

## §7 — A fila de testes, na placa integrada (23/10, `line/Velocidade` W2)

**A medição que a autoriza** (23/10, `main` `972f411a6f`, só a Radeon 610M do 9950X visível ao processo, um
amostrador do `nvidia-smi` a 1 s sem nenhum processo da corrida): os testes das 20 crates que pedem adaptador,
**5 573 de 5 580 verdes** na integrada. Os 7 vermelhos são todos da Luz 2D (mudam texels, alocam além do
teto, perdem a lei de custo) e passam na NVIDIA — por isso a Luz 2D está **presa à principal**
(`PRESOS_A_PRINCIPAL` em [`vez.rs`](../../crates/ph2d-gpu/src/vez.rs)); o achado foi para a linha dela. No
tempo, a maior parte igual ou mais rápida que na NVIDIA (contra a suíte do ship a `load ~40`); a Luz 2D ~5,6×
mais lenta.

**Quem vai para a fila de testes** (`ph2d_gpu::vez::escolhe`, só no `GpuContext::new`): um binário de teste
(em `target/*/deps/`, com o `CARGO_PKG_NAME` do cargo/nextest) de um pacote não preso, numa árvore que não
mede (`PH2D_GPU=1`) nem a fechou (`PH2D_PLACA=principal`), e só se o porteiro dela a abriu (a máquina tem
≥ 2 nós `/dev/dri/renderD*`). Tudo o resto fica na principal: o app, o kwin, os oráculos, todo pedido por
fora do `GpuContext` (`vez_da_placa()` pede SEMPRE a principal).

- **Só Vulkan na fila de testes.** Medido em 23/10 (sonda de fases, `nvidia-smi` a 0,25 s, 2/2 cada): com o
  GL na lista, o processo aparece na NVIDIA ao ENUMERAR os adaptadores (o EGL abre um contexto nela);
  enumerando só Vulkan e abrindo o device na integrada, nunca aparece. O `GpuContext` escolhe o
  `IntegratedGpu` de `enumerate_adapters(Backends::VULKAN)`; sem ele, erro alto (nunca a principal sem a vez).
- **Sem tranca entre filas:** quem já tem a principal fica nela (o `GpuContext` não pede a de testes), logo
  ninguém segura a principal à espera da de testes — o ciclo de duas árvores trancadas não se fecha.
- **De ponta a ponta** (23/10, `nextest` de `ph2d-render` + `ph2d-flip-render`, 547 testes, a principal com a
  suíte do CodeEditor e `load ~190`): **547/547 em 2,2 s**, nenhum processo da corrida na tabela do
  `nvidia-smi` (amostra a 0,3 s). ⚠️ O `fuser` no nó da integrada NÃO prova uso: o carregador do Vulkan abre
  todos os nós ao enumerar, também nos processos que ficam na NVIDIA.
- **A integrada também é exclusiva entre árvores** (o mesmo `flock`, `ph2d-gpu-testes.lock`): 14/09 foi na
  NVIDIA, mas ninguém mediu duas árvores juntas na integrada; abrir mais vagas é medição por fazer.
- ⚠️ **O monitor secundário (AOC) está ligado à integrada** ([memória](../../project-memory/reference_display_topology_workstation.md)):
  com testes a correr nela, ele pode perder fluidez. Nos 22 min da corrida de 23/10 nada pendurou.

