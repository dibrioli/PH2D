# Handoff de integração — `line/physics`, **O PLAYER DE PLATAFORMA** (2026-08-04)

> **A linha está FECHADA e PARADA.** Ela não integra e não pusha — DIRETRIZ §1.5.9,
> CLAUDE.md §0.7. Este documento é o que um agente integrador precisa para fundir.
>
> **12 commits**, de `aff4516c2` (o handoff de troca) a `4277be065` (W8).
> ⚠️ O `main` local desta worktree está **atrasado**; conte a jornada a partir de
> `aff4516c2`, não de `main..HEAD` (que arrasta a jornada já integrada de 03/08).

---

## 1. O que o Enio pediu, e o que decidiu

> *"Vamos fazer uma pesquisa profunda sobre mais uma feature importante da física: **o
> player Plataforma**. Vamos criar uma **seção no inspector para Comportamentos** e o
> primeiro comportamento será o de Plataforma. Como temos uma engine moderna com uma
> física bastante funcional, vamos tentar criar **a partir de um Dynamic e não de um
> Kinematic**. Sei que em Rust há gente trabalhando nisso: um player de plataforma com
> **perfeita compatibilidade com a física, podendo interagir com todo o sistema, inclusive
> Joints**, e manter comportamento coerente e correto. É isso que vamos buscar. **Todas as
> features que fazem dos games de plataforma tão precisos, mas sem perder a interação com
> os objetos físicos.**"*

E, no seguimento: ele não tem resposta melhor do que a que a pesquisa em artigos e código
de especialistas dá; um Player Kinematic virá um dia, mas **hoje é Dynamic**; sem
milagres, mas o melhor possível — **procure o estado da arte, o padrão-ouro ou melhor, e
TOME AS DECISÕES**.

As três fases pedidas (pesquisa → plano → implementação) estão em
[`05_pesquisa_player_plataforma.md`](05_pesquisa_player_plataforma.md) e
[`06_plano_player_plataforma.md`](06_plano_player_plataforma.md).

---

## 2. A espinha: **a cápsula FLUTUA, e é isso que a mantém Dynamic**

O achado da pesquisa que decidiu a arquitetura inteira (referência: `bevy-tnua` e o
*Very Very Valet* da GDC): um personagem de plataforma **preciso** e um corpo **Dynamic**
só coexistem se o personagem **não encostar no chão**. A cápsula paira a uma altura de
flutuação e **a perna é uma MOLA** — um ray cast para baixo, uma força proporcional ao
erro de altura, um amortecedor contra a velocidade **RELATIVA ao chão**.

É isso que dá, **de graça**, tudo o que o pedido nomeia:

- **degraus e rampas** — a mola sobe sozinha, sem `move_and_slide` nem code de step-up;
- **plataformas móveis e joints** — o amortecedor mede contra a velocidade do PONTO do
  chão (`point_velocity`), então uma plataforma que gira leva o personagem mesmo com o
  centro parado, e um corpo pendurado num joint é chão como qualquer outro;
- **o solver decide as colisões** — nada aqui teleporta um corpo, então empurrar caixas,
  ser empurrado, cair numa poça de empuxo e prender-se a uma corda continuam sendo o que a
  física já fazia.

O motor tem **dois canais** e a distinção é load-bearente: `accel` (→ força × massa,
mediada pelo solver, ao lado dos contatos e dos joints) e `boost` (escrita direta de
velocidade). A W6 mediu o preço de confundi-los — ver §4.

---

## 3. As waves

| Wave | O que entrega | Smoke |
|---|---|---|
| **W1** | O wrapper ganha consultas de cena: `cast_ray` (a perna) e `point_velocity` (a velocidade do PONTO) | — |
| **W2** | A crate nova **`ph2d-platformer`** — a cápsula flutuante, a mola e o amortecedor | `=80` |
| **W3** | **ANDAR** — a caminhada com aceleração assimétrica, a rampa e o vagão; o dedo chega à ponte | `=81` |
| **W5** | A **§14 do Inspector** — o comportamento vira autorável (o pedido do Enio) | `=82` |
| **W4** | **O PULO**, parametrizado por ALTURA, com o arco de quatro fases e a altura variável | `=83` |
| **W6** | **A REAÇÃO** — a 3ª lei: a jangada afunda e INCLINA | `=85` |
| **W7** | **A FITA** — a entrada vira função do TIQUE, e o replay de um scrub passa a dirigir o player | `=86` |
| **W8** | **O PERDÃO** — coyote time e jump buffer | `=87` |

⚠️ **A ordem dos commits não é a das waves** (a W5 vem antes da W4): foi a ordem em que a
autoria ficou alcançável, porque afinar o pulo sem UI é afinar no escuro.

---

## 4. Os quatro achados que valem mais que o código

### 4.1 Uma escrita de velocidade por tique NÃO é uma força, e não volta (W6)

A 3ª lei é o que separa este controlador do `bevy-tnua`: uma cápsula que só empurra a si
mesma é um **fantasma** — ela paira sobre uma jangada sem a afundar. A reação devolve ao
chão o que a perna tira dele.

⚠️ **Mas o `boost` do amortecedor NÃO pode voltar**, e o preço está medido: ele é
calculado a partir da velocidade RELATIVA ao chão, então devolvê-lo fecha um laço no nível
da VELOCIDADE que o solver nunca media (nenhum dos dois lados é uma força que ele resolva).
Numa jangada pendurada por molas ela disparava para **−0,946 m**, ultrapassava o
personagem (folga 2,48 m contra o alcance de 1,4 da perna), ele perdia o chão, caía até o
piso, e ela ficava a **DERIVAR para cima sem ninguém em cima**. Excluindo esse boost ela
assenta em **−0,194** e fica lá, com o personagem a bordo e inclinada 2°.

⚠️ **O boost da DECOLAGEM continua voltando** — um pulo é **um** tique, não há laço a
fechar, e pular de uma jangada tem de a afundar. *O que é excluído é o que se repete todo
tique.*

**A cura não foi desligar a feature** (que era o kill-criterion escrito no plano): foi
**separar os canais**, e é por isso que `reaction()` recebe `support`/`impulse`/`movement`
separados em vez de um motor já somado.

### 4.2 O replay de um scrub não dirigia o player (W7)

O laço de replay do `rewind` dirige as poses da cena e **nunca chamou `drive_players`**:
um scrub para trás replayava as plataformas e deixava o personagem **sem perna e sem
caminhada** — ele caía pelos tiques replayados e parava onde a gravidade o deixasse. A
trajetória de um scrub e a de um play discordavam sobre o mesmo tique.

A cura é a **fita**: `PlayerInputAtTick`, porta IRMÃ da `SceneAtTick`, no mesmo ponto do
laço. E o **estado de pulo viaja com o checkpoint** — é o mesmo argumento que pôs o
`pulley_payout` no checkpoint do rapier, um nível acima; ele não cabe naquele checkpoint
(é chaveado por `Entity`, do ECS e não do solver), então a ponte guarda o dela em
paralelo, **nos mesmos tiques âncora**.

### 4.3 A ordem pouso-antes-de-decolagem é o jump buffer (W8)

Com a decolagem primeiro, ela lê o `airborne` que ainda não caiu, recusa, e o pulo
bufferizado sai **um tique depois** — 16 ms de atraso exatamente no gesto que o buffer
existe para adiantar.

### 4.4 Os números que tornam os defaults julgáveis

- **Coyote/buffer 0,1 s** (a janela do Celeste, 5-6 quadros): a 5 m/s são **0,5 m além da
  beirada**, e a queda dentro dela é **4,9 cm** — um vigésimo da altura do personagem. É
  por isso que ela lê como *"eu ainda estava na borda"*.
- **O teto de 0,5 s é MEDIDO e o recurso é a QUEDA:** ali o personagem já desceu
  **1,23 m**, mais de uma altura de corpo (a cápsula tem 0,9 m).
- **Reação: defaults OPOSTOS** — `support = 1.0` (o peso é físico, desligá-lo é que seria
  estranho) e `movement = 0.0` (senão a plataforma **escorrega como um tapete** quando o
  personagem anda em cima; é atrito honesto e péssimo de jogar).

---

## 5. Números que a integração precisa conferir

| Coisa | Valor | Observação |
|---|---|---|
| `PROJECT_SCHEMA` | **INTOCADO** | A jornada não abre `project.rs` (`git diff aff4516c2..HEAD` vazio ali). ⚠️ Esta linha fica **FORA da disputa de número** desta janela. |
| Registro do `ph2d-physics-ecs` | **27 → 28** | Um componente novo: `PlatformPlayer`. |
| `physics_ecs_c9` | **101 → 103 corpos**, `8c7ba624…` → **`21628fe5…`** | debug ≡ release, conferido. A W7 acrescenta a lane do player; a **W8 não move o hash** (ver §7). |
| Gizmo ids | **nenhum novo** | O último segue **973**, próximo livre **974**. |
| ADR | **nenhum** | Tudo sob o **ADR-0131**. |
| Contrato congelado | **4/4 verde** | Rodado, não auto-relatado. |
| `Cargo.toml` | **2** | A crate nova + a aresta de path na ponte. |
| Dep externa nova | **NENHUMA** | O `Cargo.lock` ganha só `ph2d-platformer`. |

**Crate nova: `ph2d-platformer`** — folha, **uma** dependência (`libm`, o pin do repo), sem
rapier, sem ECS, sem shell. Ela é a LEI; quem traduz é a ponte. É isso que torna o coyote
gateável sem GPU e a fita testável sem janela.

---

## 6. Como rodar os smokes

```
env PH2D_PHYSICS_SMOKE=<n> cargo run -p ph2d-host-desktop --release
```

`=80` a perna · `=81` a caminhada · `=82` a autoria · `=83` o pulo · `=85` a reação ·
`=86` a fita · `=87` o perdão.

⚠️ **Todo smoke começa marcando `Physics` na barra de transporte** — ele nasce
DESMARCADO de propósito (W4b), e uma cena parada lê como *"a física quebrou"*.

⚠️ **A `=86` e a `=87` pedem um GESTO, não um olhar.** A `=86` só significa alguma coisa
se você **soltar as teclas** antes de arrastar a régua (é isso que separa *"a fita
dirige"* de *"meu dedo dirige"*), e a `=87` só se você **zerar os dois knobs** e repetir —
é o contraste que diz que a assistência está agindo, e não que o jogo é fácil.

---

## 7. Aberto, com o preço ao lado — **nada disto é dívida escondida**

1. **⚠️ A cobertura do C9 para os dois relógios do perdão é um vão NOMEADO.** O roteiro da
   fita pula com o pé no chão, então nenhuma das duas janelas dispara e a **W8 deixa o
   hash intocado**. Os contadores são `x - dt` com clamp — aritmética sem transcendental e
   sem chamada de plataforma —, e quem os cobre são os gates de unidade. Fechá-lo custa
   reescrever o roteiro para correr para fora de uma beirada, e **move o hash**.
2. **Corner correction (D8)** — o item do plano §W8 que **não** foi construído. Ele é o
   único da wave que o estado da arte **não tem em Dynamic** (varrido: tnua, wanderlust e
   avian não o têm; a literatura só o resolve em kinematic com
   `OverlapCapsule`+`ComputePenetration`), e o desenho está escrito no plano: **preditivo,
   nunca reativo** — um shapecast curto para cima enquanto sobe, e um boost lateral no
   tique **anterior** ao contato. Reativo devolveria a velocidade que a quina já comeu, o
   que é **inventar energia** e brigar com o solver que acabou de resolver.
3. **Lift momentum** — sair de uma plataforma preservando a velocidade dela por uma
   janela. Também do §W8, também não construído.
4. **Um dedo, todos os players.** `hand_input_to_players` entrega a MESMA entrada a todo
   player da cena, e é honesto hoje: há um teclado, logo um dedo. Um segundo jogador muda
   a FONTE (uma fita por-entidade), não a lei nem o laço — e nada no desenho a impede.
5. **A fita não é persistida.** Ela é estado de JANELA (a classe do
   `TimelineFlags::performing`); um replay que sobrevive a fechar o app é wave posterior.
6. **A gravidade lateral não alcança o player.** O `UP` da ponte é uma constante, numa
   porta só; derivá-lo da gravidade é uma linha, no lugar já nomeado.
7. **⚠️ `rebuild_from_rest` limpar o estado VIVO de pulo é MEDIDO INERTE hoje**
   (dy = 0,000000), porque `jump_step` re-deriva `airborne` da amostra de chão. A linha
   fica porque a inércia **morre** com qualquer contador futuro que não se auto-corrija —
   e os dois da W8 são exatamente isso.

---

## 8. O que a integração deve rodar

- `cargo test -p ph2d-platformer -p ph2d-physics-ecs -p ph2d-panel-inspector -p ph2d-editor-core -p ph2d-host-desktop` — **em release E em debug** (esta linha tem precedente: um gate de wall-clock que só reprovava em debug, e o `ph2d-flip-colorize` que só panicava lá).
- `cargo run -p ph2d-physics-ecs --bin physics_ecs_c9 --release` **e sem `--release`** — os dois têm de imprimir `21628fe5…` com 103 corpos.
- Os gates de LOC das **duas** casas (`architecture_workspace_file_loc_cap` na `editor-core` **e** `file_loc_caps` na shell — o segundo não é coberto pelo primeiro, e esta linha já foi mordida por isso).
- Clippy `--all-targets` **incluindo a shell**: um fechamento por `cargo test -p` por crate **não** o alcança, e esta jornada fechou um vermelho-latente da W6 exatamente assim.

**Estado no fechamento:** 0 falhas nas cinco crates, **2080 verdes na shell**, clippy **0
warnings**, LOC verde nas duas casas, `typos` limpo, contrato congelado 4/4.
