# Handoff de integração MESTRE — `line/physics` (2026-08-15)

> **A linha NÃO integra nem faz ship** (CLAUDE.md §0.7). Este documento é o que o
> integrador precisa para não colidir nem regredir. DIRETRIZ §1.5.9.
>
> ⚠️ **Ele SUPERSEDE o
> [`HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md`](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md)
> apenas como *o que integrar agora*.** O **detalhe de mecanismo** das waves até
> a ÂNCORA dos gizmos continua LÁ (e o das sete waves de sensores continua no de
> 08-11), e **nada disso foi copiado**. O que é NOVO aqui é a **fila da auditoria
> 09**: sete waves construídas e **duas recusadas por medição**.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/physics` |
| HEAD | **o tip de `line/physics`** ⚠️ ver abaixo |
| merge-base com `main` | `76788440adbabb0e5b12f8fdafecc6f1e1183e1a` |
| commits | **96** |
| diff | 204 arquivos, **+37.940 / −1.980** |

⚠️ **O HEAD não é escrito aqui de propósito, e a razão é aritmética:** o commit
que o escreve MUDA o HEAD, então um sha nesta tabela é falso no instante em que é
commitado. O que identifica a entrega é o **merge-base** acima mais *"o tip da
branch"*.

⚠️ **O `main` local está 5 commits à frente do `origin/main` e a linha parte
DELE** (`76788440a` é o tip do `main` local). O `origin/main` **não moveu** desde
o fork (`refs/heads/main..origin/main` = 0), então o rebase é trivial — mas
confira no dia, porque esta frase envelhece.

---

## 2. O que é NOVO nesta entrega

**A fila da [auditoria 09](../09_auditoria_engines.md), desenhada no [plano
10](../10_plano_fila_da_auditoria.md) e executada na ordem que ele fixou.** O
detalhe de cada wave — o desenho, o que a medição REFUTOU do plano, os gates e as
mutações — vive no próprio plano 10, em seções `⟨FECHADA⟩` ao lado do desenho
original. Aqui só o mapa:

| | wave | o que ela é | cena |
|---|---|---|---|
| **A** | `W-PlayerOut` | o jogador **PUBLICA** estado e transições; a §14 mostra e o evento vira **SINAL** | `=113` |
| **C** | `W-Brake` | **frear ≠ acelerar** — a fração do orçamento gasta com o eixo solto | `=114` |
| **B** | `W-Surface` | a superfície **FALA** com a lei: tração (gelo) e **esteira** | `=115` |
| **D** | `W-Fall` | a queda tem **TETO** (não havia velocidade terminal: **142,57 m/s aos 8 s** e a crescer) | `=116` |
| **E** | `W-Launch` | o **mundo empurra** o personagem — a explosão passa a alcançar os três modos | `=117` |
| **J** | `W-Leave` | a sonda do Snap fechou **VERDE** e o buraco era outro: a altura autorada era medida **contra a plataforma** | `=118` |
| **G** | `W-Brink` | `bCanWalkOffLedges` — ele **PARA na quina**, com o alcance **DERIVADO** | `=119` |
| **H** | ⛔ | **voar/noclip — RECUSADA por medição** | — |
| **I** | ⛔ | **air control boost — RECUSADA por medição** | — |

### 2b. As duas recusas, porque elas são metade da entrega

⚠️ **Uma recusa medida vale tanto quanto uma wave construída: ela é o que impede
o item de voltar.** As duas sondas ficam no repo (`--ignored`, imprimem e não
afirmam, cada uma com o seu **CONTROLE**).

**I — `AirControlBoostMultiplier`.** A auditoria descreve o item por um SINTOMA
(*"não consigo sair do lugar no topo de um pulo vertical"*) e a §0 manda medir o
fenómeno antes da cura. Medido (`measure_air_control`): **no ápice ele já corre à
velocidade de CRUZEIRO** — `5,9999` contra `speed = 6,0`, em toda a varredura —,
e **8× o `air_acceleration` compra 8,5% de deriva e move a velocidade do ápice em
ZERO**. O regime em que o sintoma EXISTE também está medido (abaixo de
`air_accel ≈ 10`: 67% / 29% / 15% / 7% do cruzeiro), e **o knob que o cura já está
no painel**. É por isso que o Unreal precisa do multiplicador e nós não: lá o
`AirControl` é uma **FRAÇÃO da velocidade de caminhada** (5% por default); aqui é
uma **aceleração própria** que alcança o cruzeiro em 18 dos 73 tiques de voo. Um
`air_control_boost` seria a **segunda porta** para a mesma pergunta.

**H — `MOVE_Flying` / noclip.** A capacidade **já existe pelos gestos que o editor
tem**, e a sonda mede isso em vez de o supor (`measure_noclip`): com o toggle
**Physics** desmarcado o artista põe o personagem **dentro de uma parede sólida**
(`6,0000 · 4,0000`), **do outro lado dela** (`12,0000 · 0,9000`) e **20 m acima**
(`6,0000 · 20,0000`) — exato a quatro casas —, e o Play **retoma dali** (deriva
lateral **0,0000 m**). Com **CONTROLE**: empurrado contra a parede com o relógio a
andar ele **pára em x = 3,8011** contra uma parede que começa em 4,0. ⚠️ **O que a
recusa NÃO cobre está dito por inteiro:** voar **com as teclas durante o play** não
existe aqui (a MÃO agarra por mola *através do solver*, logo colide) — o gesto
existente dá **teleporte com o relógio parado**, que é exactamente o caso de uso
que a auditoria nomeia.

⚠️ **E a fixture do noclip nasceu ERRADA, com a primeira tabela a MENTIR:** o
gesto do toggle desmarcado é **`PhysicsBridge::hold`**, e **não**
`dispatch(playing = false, …)` — aquela porta, com o alvo a CRESCER, entra no braço
`Greater` e **DÁ PASSO** (o doc dela chama-lhe *"um scrub para a FRENTE enquanto
pausado"*, porque o estado da sim é função do TIQUE e não do botão de play), e o
`readback` do passo seguinte devolvia a pose escrita à mão.

---

## 3. Superfície de colisão

| | |
|---|---|
| `PROJECT_SCHEMA` | **70 → 82** ⚠️ **PROVISÓRIO — CONTE contra o `main` do dia** |
| a tripla do pin | **`(70, 13, 14)` → `(82, 13, 14)`** em `shells/desktop/src/project_schema_tests.rs` ⚠️ *é `src/`, não `tests/`* |
| registro `ph2d-physics-ecs` | **29 → 31** (`PlayerSignals` · `WalkSurface`) |
| registro `ph2d-ecs` + os **dois** espelhos | **INTOCADOS** (`git diff` vazio em `crates/ph2d-ecs/`) |
| gizmo ids | **nenhum novo** — o último segue **973**, próximo livre **974** |
| ids novos | **todos `hash_node_id`** ⇒ fora de todo gate de contagem |
| scrollbar ids | nenhum novo |
| ADR | **NENHUM** ⇒ a linha fica **fora de toda disputa de número** |
| `Cargo.toml` / `Cargo.lock` | **ZERO** — nenhuma crate nova, nenhuma dep nova |
| `ph2d-i18n` | **INTOCADO** |
| contrato congelado | **3/3 + 4/4 + 11/11**, rodados (nós · tools · vector) |
| cenas de smoke | **105 → 119** (quinze novas); **próxima livre: 120** |

### 3a. ⚠️ O `PROJECT_SCHEMA` são DOZE degraus, e é aqui que a colisão mora

⚠️ **ESTA LINHA PARTIU O `project.rs`, e é o ponto de merge mais sensível da
entrega.** A escada inteira **e a constante** saíram para o irmão **NOVO**
`shells/desktop/src/project_schema.rs` (498 linhas), e o `project.rs` perdeu 374
linhas de doc-header no mesmo corte. *Uma linha que acrescente um degrau dentro do
`project.rs` funde **limpa** contra um arquivo de onde a escada saiu* — o modo de
falha exacto que o corte do `project.rs` de 04/08 produziu na `line/Vector`, e a
razão de esta caixa existir.

⚠️ **O valor se CONTA, nunca se escolhe** — e esta colisão **passa MUDA quando
duas linhas escrevem o MESMO literal**: o git não sabe o que o número significa,
e o bump da segunda evapora com a suíte inteira verde. São **TRÊS** sítios a
conferir, não um: o literal (`project_schema.rs`), **a escada ao lado dele**, e a
**tripla** do `project_schema_tests.rs`. Escreva a entrada da escada no MESMO
commit que renumera (a lição do degrau v69, que chegou ao `main` com a linha
AUSENTE).

Os doze degraus, em uma linha cada: **v71** o nado · **v72–v73** os sensores
editáveis · **v74** o pulo do ar · **v75** a beirada · **v76** o planeio ·
**v77–v78** o sensor da beirada (posição e extensão; ⚠️ o v78 existe porque
*`reach_y` é TAMANHO e não POSIÇÃO*, e o v77 tinha mapeado os dois no mesmo
número) · **v79** o freio · **v80** o teto de queda · **v81** o que a plataforma
dá ao pulo · **v82** a trava de beirada.

⚠️ **Todos nascem NEUTROS** — cada rung documenta que o projeto salvo na versão
anterior reabre exactamente como estava. O bump é pelo caminho **INVERSO** (o
postcard é posicional: um leitor velho tem de RECUSAR em vez de ler lixo
bem-formado).

### 3b. O `physics_ecs_c9`

**`1699123f9ed2844fa5159bc842a4e583f0675cdd88bb8895e2654ac706053787`**,
**117 corpos**, **debug ≡ release** (medido nesta árvore, nos dois perfis).

⚠️ **Ele MOVE contra o `main`** (`fb27f676…`, os mesmos 117 corpos) **e a
atribuição é medida, não suposta:** `git diff main...HEAD` sobre
`crates/ph2d-physics-ecs/src/bin/physics_ecs_c9/` é **VAZIO** ⇒ nenhuma lane
nasceu ou morreu, e o que moveu foi a **LEI do player** — a cena carrega quatro
lanes com `PlatformPlayer::default()`, e esta linha mudou defaults (o mais visível
é o **leque de pés**, que nasce com **três** raios contra o raio único do `main`).
Cada degrau v79..v82 declara byte-identidade individualmente; o movimento
acumulado vem das waves anteriores à fila, cujo detalhe está nos handoffs de
08-11 / 08-12.

---

## 4. Mudanças de comportamento, nomeadas

1. **Um player publica estado e transições** (`W-PlayerOut`) — o readout vivo na
   §14 e, **com o opt-in autorado ligado**, sinais no barramento. ⚠️ **Nasce
   DESLIGADO**, e é decisão de custo: sem isso toda cena de smoke com um
   personagem passaria a cuspir toasts.
2. **Frear deixou de ser acelerar** (`W-Brake`) — o campo nasce em `1`, que é o
   mundo de antes da wave, ao bit.
3. **A superfície fala com a lei** (`W-Surface`) — `WalkSurface` é componente
   OPCIONAL; ausente, é o neutro. ⚠️ **Oferecido em TODO collider**, porque a
   superfície que importa é quase sempre um chão **estático**.
4. **A queda tem teto** (`W-Fall`) — nasce em `0`, que **desliga** a lei.
5. **O mundo empurra o personagem** (`W-Launch`) — a explosão passa a alcançar os
   três modos; antes ela não tinha canal para um player de pose própria.
6. **O pulo numa plataforma** (`W-Leave`) — a política nasce em `Full`, onde a
   porta devolve o valor VERBATIM.
7. **Ele pode parar na quina** (`W-Brink`) — os dois campos nascem em `true` (a
   CAPACIDADE, nunca a trava), e o sensor **nem sequer casta** com a trava
   desarmada. ⚠️ **O alcance NÃO é um degrau de schema**: ele é **DERIVADO**
   (`v²/2a` da lei + meia-largura da ponte), porque o knob que ele substituiu
   tinha o valor certo **em função de outros dois** — medido, a 8 m/s um `0,30`
   deixava CAIR e um `0,60` segurava, com a fronteira exactamente em `0,533`.

---

## 5. O gate de fechamento — o que foi rodado, e o resultado

Tudo abaixo nesta árvore, com a máquina em `load average 1,8` (⚠️ *nenhum kill de
relógio deste repo significa coisa nenhuma com o load alto*).

| gate | resultado |
|---|---|
| `cargo test -p ph2d-physics-ecs -p ph2d-physics -p ph2d-platformer --release` | **verde**, 0 falhas |
| `cargo test -p ph2d-host-desktop --release` | **verde**, **2897 passados / 0 falhas** (inclui o `file_loc_caps` da shell) |
| `cargo test -p ph2d-panel-inspector -p ph2d-editor-core --release` | **verde**, 0 falhas |
| `arch_safe_clamp_only` | 2/2 |
| `architecture_workspace_file_loc_cap` | 2/2 |
| `architecture_contract_surface` (nós) | 3/3 |
| `architecture_tool_contract_surface` | 4/4 |
| `architecture_vector_contract_surface` | 11/11 |
| `cargo clippy --workspace --all-targets --release` | **limpo** |
| `cargo fmt --all -- --check` | **limpo** ⚠️ *depois de dois arquivos da wave D* |
| `typos` | **limpo** |
| `physics_ecs_c9` | hash acima, **debug ≡ release** |

⚠️ **O `fmt` pegou dois arquivos VERMELHOS herdados da wave D**
(`measure_terminal.rs` · `player_terminal.rs`) — corrigidos no último commit. É a
**terceira vez nesta linha** que uma varredura de fecho acha fmt latente: um
`cargo test -p` por crate **não roda o `fmt`**, e o arquivo fica vermelho até
alguém varrer a árvore inteira.

---

## 6. Smoke

**Nada nesta entrega está pendente de smoke** — as sete waves foram aprovadas pelo
Enio à medida que fecharam, e as duas recusas não têm o que smokar (elas são
medição).

Rodar, se o integrador quiser reconferir:
`env PH2D_PHYSICS_SMOKE=<n> cargo run -p ph2d-host-desktop --release`

* **`=113`** a saída do player · **`=114`** o freio · **`=115`** a superfície ·
  **`=116`** o teto de queda · **`=117`** o empurrão · **`=118`** o pulo na
  plataforma · **`=119`** a trava de beirada.
* ⚠️ **`=84` não existe, de propósito** — o roteador é um `match` de literais e o
  compilador é o gate (um segundo braço com o mesmo literal é `unreachable`).
* ⚠️ **O número da próxima cena se CONTA lendo o `match`** em
  `shells/desktop/src/physics_smoke.rs`, **nunca uma nota** — a nota da §5 do
  `CLAUDE.md` já envelheceu em onze cenas uma vez.

As sondas que decidiram H e I rodam assim:

```
ph2d-run cargo test -p ph2d-physics-ecs --release --test measure_noclip      -- --ignored --nocapture --test-threads=1
ph2d-run cargo test -p ph2d-physics-ecs --release --test measure_air_control -- --ignored --nocapture
```

---

## 7. Aberto, com o preço ao lado

* **O campo de ATRAÇÃO ainda não alcança um player de pose própria** — ele é
  sustentado, não um evento, e pede um canal **por-tique**, não a porta do
  `W-Launch`.
* **O `bXYOverride` do Unreal** (substituir em vez de somar no empurrão) segue
  fora; entra quando houver quem o peça.
* **A trava de beirada não tem gesto de canvas** — ela é autorada por chip na
  §14, como as irmãs. Um marcador visual da quina seria overlay, e o overlay do
  player já tem dono.
* **O resto da lista aberta do módulo** (o horizonte do plano 02 §8, a paridade
  de arrasto do modo cinemático, o bobbing na poça) está onde estava: nada nesta
  entrega o toca.

---

## 8. Ordem de leitura para quem integrar

1. **§3 deste doc** — a superfície de colisão, e em particular o `PROJECT_SCHEMA`
   e o corte do `project.rs`.
2. **[plano 10](../10_plano_fila_da_auditoria.md)** — o mecanismo de cada wave,
   com as seções `⟨FECHADA⟩` que dizem **onde o plano errou** e o que a medição
   refutou.
3. **[08-12](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-12.md)** e
   **[08-11](HANDOFF_INTEGRACAO_line_physics_sensores_2026-08-11.md)** — para o
   porquê de cada número das waves anteriores à fila.
