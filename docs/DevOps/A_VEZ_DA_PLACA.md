# A vez da placa — todo uso da placa espera a sua vez (18/10)

> Lei curta: **toda árvore de processos que passa pelo `scripts/ph2d-run.sh` tem um PORTEIRO**. Ele pede a
> placa ao cadeado partilhado na **primeira** vez que alguém da árvore precisa dela e segura-a até o comando
> acabar. Ninguém precisa de se lembrar do `PH2D_GPU=1`: o código pede a vez sozinho no ponto onde o processo
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
`pronto` (a vez é da árvore), `recusa` (o porteiro desistiu, com quem segura), `porteiro` (o pid dele). Dois
clientes: [`vez.rs`](../../crates/ph2d-gpu/src/vez.rs) e [`vez-da-placa.sh`](../../scripts/vez-da-placa.sh).

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
- **Bónus (a pausa de 60 s de placa livre da régua, W6c §5 item 4): NÃO encurtar ainda.** Nesta rodada a espera
  foi 61–62 s em todos (ninguém entrou durante ela), mas os intrusos chegaram A MEIO das medições, vindos de linhas
  sem a vez: a pausa não os apanharia de qualquer maneira, e encurtá-la só se mede quando todas as linhas tiverem
  este `main` — a pergunta passa a ser só o assentamento do relógio da placa (`placa.csv` de cada processo).

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
| o porteiro a vigiar o FIM do último uso (soltar quando a árvore pára de usar a placa) | cada teste seguinte da árvore voltaria a esperar uma medição inteira; o ganho só existe para suítes longas, que já são raras (1× no fecho) |

## §5 — O que isto NÃO resolve

- ⚠️ **Cada worktree só ganha a vez quando tiver o `main` com ela** (o `ph2d-run.sh`, o guarda e o `ph2d-gpu` são
  da árvore de cada linha). Até lá uma linha velha continua compatível — o `PH2D_GPU=1` dela tranca o MESMO
  cadeado —, mas os testes dela sem `PH2D_GPU=1` ainda entram sem pedir.
- ⚠️ Um `kill -9` na porta deixa o comando (o scope) a correr até ao prazo, e o porteiro solta a placa em ≤ 5 s —
  como antes, quando o cadeado era do bash da porta.
- ⚠️ O que o dono abre à mão (o app, o Chrome, o Rebelle) está fora por desenho: a medição continua a precisar do
  detector de alheios.
- ⚠️ O guarda vê o comando escrito; um script que lance o kwin sem chamar o `vez-da-placa.sh` escapa — o
  padrão a copiar está nos dois roteiros (§2).
