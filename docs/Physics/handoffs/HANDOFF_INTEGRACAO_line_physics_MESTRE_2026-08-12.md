# Handoff de integração MESTRE — `line/physics` (2026-08-12)

> **A linha NÃO integra nem faz ship** (CLAUDE.md §0.7). Este documento é o que o
> integrador precisa para não colidir nem regredir. DIRETRIZ §1.5.9.
>
> ⚠️ **Ele SUPERSEDE o
> [`HANDOFF_INTEGRACAO_line_physics_sensores_2026-08-11.md`](HANDOFF_INTEGRACAO_line_physics_sensores_2026-08-11.md)
> apenas como *o que integrar agora*** — o **detalhe de mecanismo** das sete
> waves de sensores (`W-Swim` · `W-SwimLine` · `W-ZoneForce` · `W-ShapeCast` ·
> `W-Probes` · `W-Probes2` · `W-FootFan`) continua LÁ e **não foi copiado**. Leia
> os dois: este para a superfície de colisão e para a wave nova, aquele para o
> porquê de cada número das anteriores.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/physics` |
| HEAD | **o tip de `line/physics`** ⚠️ ver abaixo |
| merge-base com `main` | `76788440adbabb0e5b12f8fdafecc6f1e1183e1a` |
| commits | **~47** |
| diff | 95 arquivos, +15.096 / −989 |

⚠️ **Todos são pós-integração de 2026-08-10** (a jornada `W-KinMove` / modo
cinemático, que já está no `main`). Nada aqui foi entregue antes.

⚠️ **O HEAD não é escrito aqui de propósito, e a razão é aritmética:** o commit
que o escreve MUDA o HEAD, então um sha nesta tabela é falso no instante em que é
commitado. O que identifica esta entrega é o **merge-base** acima mais *"o tip da
branch"* — que é o que um integrador usa de qualquer forma.

**O assunto é o PLAYER, em três metades.** As duas primeiras estão no handoff de
08-11 (o catálogo do plano 08, e os SENSORES). A terceira, nova aqui, é o **PULO
DO AR**.

---

## 2. A wave nova — `W-MultiJump`

**O `air actions counter` do tnua**, e o item mais pedido do catálogo do plano
08. `JumpConfig` ganhou **`air_jumps`** (a contagem; `0` desliga) +
**`air_jump_height`** (metros). A carga recarrega no CHÃO, no MESMO braço
`if grounded` do coyote — o **terceiro** consumidor da porta única
`JumpState::on_ground` (os outros: o coyote e o ARRANQUE), sem uma 2ª cópia do
predicado, que é exatamente o que o plano exigia.

### O que o plano NÃO previu, e é a parte que precisa de leitura

⚠️ **O proxy do ARRANQUE apodreceu com o terceiro pulo.** O `lib.rs` perguntava
*"a TRANSIÇÃO para o ar"* (`!antes.airborne && depois.airborne`) — exato enquanto
todo pulo começava com o pé em ALGO (chão, parede), e **falso para um pulo do
AR**, que acontece com `airborne` **já verdadeiro**. Ele dizia *não* justamente
no gesto que mais se encadeia com um arranque, contra o que o próprio comentário
de lá promete (*"um pulo de QUALQUER tipo cancela o arranque"*).

Nasceu **`JumpStep::jumped`**, e com ele o **terceiro** gate de cancelamento —
que o comentário do gate irmão da parede **já previa sem o saber** (*"é o gate
seguinte que a apanha, e por isso os dois existem"*).

### As decisões, cada uma com o motivo

⚠️ **A altura do ar é em METROS, não uma fração do primeiro pulo.** Este módulo
tem **três** pulos, e o da parede já é altura absoluta
(`WallConfig::jump_height`) — uma escala aqui faria dois falarem metros e um
falar multiplicador, na mesma seção do painel.

⚠️ **A precedência é a força do APOIO: chão > parede > ar.** Um pulo de parede
**não gasta carga** (a parede é apoio próprio), e o bloco do ar **não tem guard
de *"não estou no chão"***: os dois ramos acima já RETORNARAM em todo caso com
apoio, então chegar ali **é** estar no ar — um `!grounded` seria a 2ª cópia de
uma condição já decidida, e a cópia que envelhece quando um quarto apoio
aparecer.

⚠️ **`next.buffer = 0.0` no ramo do ar é load-bearing:** sem ele o mesmo aperto
re-dispara em tiques consecutivos e queima **as três cargas em ~6 tiques** —
três boosts empilhados, um foguete.

⚠️ **`takeoff: false`, pela mesma física do pulo de parede:** a 3ª lei devolve ao
chão o que o pé nele empurrou, e este pé não empurrou nada. Marcá-lo afundaria
uma jangada com um pulo dado no ar acima dela.

### Medido (`measure_multi_jump`, pela porta do produto)

| gesto | pico acima do repouso |
|---|---|
| um toque | **0,6176 m** |
| dois toques (o 2º no ar) | **1,2326 m** |
| um toque com 0 / 1 / 3 cargas | **0,6176 nos três** |
| duas rodadas com um pouso no meio | **1,2326 nas duas** |
| aperto SEGURADO, um pulo | **1,903 m** |
| aperto SEGURADO, dois pulos | **4,028 m** |

As duas últimas são o que escolhe as prateleiras da cena `=110`.

---

## 3. Superfície de colisão

| item | valor | nota |
|---|---|---|
| `PROJECT_SCHEMA` | **70 → 74** | ⚠️ **quatro degraus**, ver §4 |
| tripla do pin | `(74, 13, 14)` | `project_schema_tests.rs` |
| `physics_ecs_c9` | **`1699123f9ed2844f…`, 117 corpos** | debug ≡ release, medido no tip. ⚠️ **NÃO se move com a wave nova** |
| registro `ph2d-physics-ecs` | **29, INTOCADO** | nenhum componente novo |
| registro `ph2d-ecs` + os 2 espelhos | **INTOCADOS** | |
| gizmo ids | **nenhum novo** (o último segue **973**, próximo livre **974**) | |
| ids novos | **12, todos `hash_node_id`** | ⇒ fora de todo gate de contagem |
| ADR | **nenhum** | ⇒ a linha fica **fora de toda disputa de número** |
| `Cargo.toml` / `Cargo.lock` | **ZERO** | nenhuma crate nova, nenhuma dep nova |
| contrato congelado | **4/4** | rodado, não auto-relatado |
| `PLAYER_ROW_COUNT` | **42 → 44** | as duas rows do pulo do ar |
| cenas de smoke | maior **110** (próxima livre **111**) | ⚠️ o `=84` não existe, de propósito |

⚠️ **O `c9` intocado é a PROVA de que o degrau v74 não move física.** A
capacidade nasce em `air_jumps = 0`, então nenhuma lane do hash a exercita — e o
hash é o único oráculo que não depende de eu afirmar coisa alguma.

---

## 4. Os degraus de schema

Os três primeiros estão no handoff de 08-11 (§4 de lá): **v71** os três campos
do nado · **v72** os quatro números dos sensores · **v73** os dois da perna em
leque.

**v74 (`W-MultiJump`):** o `PlatformPlayer` ganhou `air_jumps` +
`air_jump_height`, **no MEIO do struct** (logo depois do `jump_height`, que é
onde eles se leem), e o postcard é posicional ⇒ **quebra dura**.

⚠️ **Este degrau NÃO move física:** a contagem nasce em `0`, que é a capacidade
DESLIGADA — o precedente do wall slide e do wall jump —, então um projeto salvo
em v73 reabre com o pulo exatamente como estava. É o oposto do v73, que move (e
o handoff de 08-11 diz por quê).

⚠️ **PROVISÓRIO — o valor se CONTA contra o `main` do dia.** Três linhas já
colidiram neste número por o terem *escolhido*, e da última vez o certo não
estava em nenhum dos dois lados. ⚠️ **E a colisão passa MUDA quando as duas
linhas escrevem o MESMO literal:** o `project.rs` não conflita e o git não sabe o
que o número significa — confira nos **DOIS** arquivos (`project.rs` **e**
`project_schema_tests.rs`).

---

## 5. LOC — o corte de `jump.rs`

`jump.rs` estava em **676** e a wave o levou a **796 > 700**. Corte por
**RESPONSABILIDADE**, não por tamanho: **`jump_config.rs`** leva *o que o artista
AUTORA* (o `JumpConfig` e o `STARTING_POINT`, quase inteiramente doc-comments com
as tabelas que escolheram cada número) e o pai fica com *o que acontece num
TIQUE* (`JumpState` / `JumpStep` / `jump_step`).

Re-exportado pelo pai (`pub use jump_config::JumpConfig`) ⇒ **nenhum caminho de
chamador muda**. Os dois arquivos ficam em **584** e **231**.

⚠️ **Um filho de `src/jump.rs` precisa de `#[path]`** — sem ele o compilador
procura `src/jump/jump_config.rs`; é a convenção que `player_leg.rs` e
`height_modes.rs` já seguem.

---

## 6. Gates e mutações

**10 gates novos de lei/produto** + **4 de cena** + as duas rows na varredura de
seam.

- `ph2d-platformer/src/jump_air_tests.rs` (7) — a carga · a altura própria · a
  recarga junto com o coyote · o pulo que não empurra · a parede que não gasta ·
  um aperto uma carga · o controle com `air_jumps = 0`.
- `ph2d-platformer/src/lib_dash_tests.rs` (1) — o **terceiro** cancelamento.
- `ph2d-physics-ecs/tests/player_multi_jump.rs` (3) — pela porta do produto.
- `shells/desktop/src/physics_smoke_multi_jump_tests.rs` (4) — a aritmética em
  tempo de COMPILAÇÃO · o contraste · o controle · **o pouso**.

**7 mutações, 7 sangram:** a feature inteira · o buffer não consumido · a
recarga no ar · a altura errada · empurrar o chão · o ar preceder a parede · o
proxy antigo de volta.

### ⚠️ Duas coisas que as mutações acharam em MIM, e ficam escritas

**(a) O gate novo do arranque nasceu SEM DENTES.** A fixture punha o personagem
a CAIR, e quem nunca pulou entra com `airborne` **falso** — ali o proxy antigo
acerta por acidente. Medido: sem `state.jump.airborne = true` a mutação do proxy
**não sangra**; com ela, sangra sozinha.

**(b) Eu escrevi uma afirmação FALSA num doc de gate.** O gate de PRODUTO dizia
ser o guardião do consumo do buffer, e ele fica **VERDE** sob aquela mutação —
naquele caminho a decolagem do CHÃO já zerou o buffer antes de o personagem
chegar ao ar, então o aperto guardado nunca alcança o ramo do ar. Quem apanha é o
irmão de unidade, cuja fixture entra **já no ar** com uma borda fresca. Doc
corrigido em vez de contrabandeado.

---

## 7. Como rodar o gate

```
cargo test -p ph2d-platformer -p ph2d-physics-ecs -p ph2d-panel-inspector --no-fail-fast
cargo test -p ph2d-host-desktop --no-fail-fast
cargo run -q -p ph2d-physics-ecs --bin physics_ecs_c9 --release   # e sem --release
```

⚠️ **`--no-fail-fast` não é preferência:** sem ele o primeiro binário vermelho
esconde o resto, e na jornada de 08-11 a diferença foi entre *"um gate caiu"* e
*"dez caíram"*.

---

## 8. Smokes

| cena | o quê |
|---|---|
| `PH2D_PHYSICS_SMOKE=105` | **nadar** |
| `=106` | a **correnteza** nos três modos |
| `=107` | as quatro **pedras** que não cabem num raio |
| `=108` | **o que ele vê** — os cinco sensores |
| `=109` | **A FENDA** — o leque, com o controle DENTRO do quadro ✅ **aprovado 2026-08-12** |
| **`=110`** | **A PRATELEIRA ALTA** — o pulo do ar |

⚠️ **A cena 110 em uma frase:** duas raias iguais, dois personagens iguais, e o
teclado dirige **os dois ao mesmo tempo** (`hand_input_to_players` entrega a
todo `PlatformPlayer`) — então um gesto só move os dois lado a lado, e o controle
está **dentro do quadro**. A prateleira **baixa (1,5 m)** cabe num pulo e prova
que os DOIS sabem pular; a **alta (3,0 m)** cai no vão entre 1,903 e 4,028, então
só o da direita a alcança.

**Próxima cena livre: `111`** (o `=84` não existe, de propósito).

---

## 9. Aberto, com o preço ao lado

Os itens do handoff de 08-11 continuam abertos e **não foram tocados**
(`min_float_height` conservadora · a metade *"um degrau íngreme não é chão"* sem
fixture de unidade).

Da wave nova:

- **A carga não é VISÍVEL na tela**, e é decisão, não esquecimento: um contador
  de pulos restantes é um **HUD**, e este app não tem um. A metade visível desta
  wave é o próprio comportamento — o personagem pula uma segunda vez, e a cena
  `=110` é onde isso se julga. Um readout na §14 seria um número que o artista
  não pode usar enquanto joga.
- **Um pulo do ar não zera o `wall_lock`.** Depois de um pulo de parede o
  controle aéreo fica calado por `jump_lockout` (0,2 s), e um pulo do ar dentro
  dessa janela não o encurta. É pequeno (o relógio corre de qualquer forma) e
  **não foi medido**: se o smoke mostrar que atrapalha, o lugar é o mesmo braço
  que já zera o coyote.
- **A fila do plano 08 continua:** `W-Ledge` → `(W-Glide?)` → *o ajuste*.
