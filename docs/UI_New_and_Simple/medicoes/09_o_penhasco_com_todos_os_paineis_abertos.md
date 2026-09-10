# 09 — O penhasco com **todos** os painéis abertos

> Medido em 2026-09-09 pela `line/UIUX`, a caçar o *«travou por um minuto»* do report do dono.
> ⛔ **Não é a causa daquele report** — ver §4. É outra coisa, e ela existe.
> ⭐⭐⭐ **CURADO em 2026-09-10 (w50): `157,91 ms` → `7,81`** — a causa era a cache de moldagem de
> texto a transbordar, e **nenhum** dos três suspeitos que esta página listava (§3).

## §1 — A medição

Um quadro do `paint_hero_screen` sobre `1366 × 1024`, em **debug**, com a cena resetada por quadro
(como no produto), média de 8–20 quadros depois do primeiro:

| painéis abertos | ms por quadro |
|---|---|
| 1 (qualquer) | `0,5` – `3,8` |
| 20 | `3,0` |
| 21 · 22 · 23 · 24 · 25 | `6,1` · `3,0` · `4,2` · `6,1` · `8,0` |
| **26 (todos)** | **`182`** |

⚠️ **A soma das partes é ~`30 ms`**, e nem essa devia acontecer: com vários painéis num encaixe só
o da frente pinta. *O total é `6×` a soma, e o salto é `23×` entre 25 e 26.*

## §2 — O que a eliminação já excluiu

| hipótese | medição |
|---|---|
| um painel caro sozinho | o pior é o `tokens`, com `3,8 ms` |
| um painel caro **acrescentado** aos outros 20 | nenhum passa de `+3 ms` |
| um PAR (`widget_lab` + X) | nenhum par sai da soma dos dois |
| todos os **docados** + `widget_lab` + `widget_gallery` | `4,9 ms` |
| só os **7 flutuantes** | `6,6 ms` |
| os 7 flutuantes, cumulativos | máximo `23 ms` |
| **tudo** | **`182 ms`** |

⇒ não é um painel, nem um par, nem «os flutuantes»: é **a população inteira junta**, e o custo
não é a soma dela.

⛔ **E uma medição anterior desta mesma sonda foi DEITADA FORA:** a 1.ª versão pintava sempre na
**mesma** `VectorScene` sem `reset()`, logo media uma cena a crescer. Os números acima são os da
sonda corrigida — e as duas séries são parecidas, o que só se soube depois de corrigir.

## §3 — ⭐⭐⭐ A CAUSA foi achada, e não era nenhum dos três suspeitos

> Medido e curado em **2026-09-10** (w50). A §3 desta página dizia *«suspeitos por ordem de
> barateza: clips ANINHADOS · o `HitIndex` · a medição de texto das abas»* — **os três estão
> refutados**, e o que ficou de pé é a quarta coisa, que ninguém tinha listado.

O penhasco é a **cache de moldagem de texto** a cair de um penhasco de tamanho, não um passe caro:

| painéis | textos distintos | moldagens em REGIME | ms/quadro (debug) |
|---|---|---|---|
| 24 | `775` | `0` | `4,87` |
| 26 | `1 110` | **`1 109`** | **`157,91`** |
| 26, **depois da cura** | `1 106` | **`0`** | **`7,81`** |

⚠️ **O mecanismo:** a cache tinha `LAYOUT_CACHE_CAP = 1024` entradas e um `clear()` no transbordo.
Enquanto o conjunto de trabalho **por quadro** cabe no tecto, ninguém molda nada em regime; quando
ele o passa, o `clear()` cai **a meio do quadro** e o quadro seguinte volta a moldar **tudo**, para
transbordar outra vez. *Não é um custo por painel — é um degrau na população inteira*, que é
exactamente a forma que a §2 tinha medido e não sabia nomear.

⭐ **A cura é ROTAÇÃO, nunca um número maior:** duas gerações (`hot`/`cold`), o transbordo promove a
quente a fria em vez de a deitar fora, e um acerto na fria **promove** a entrada de volta. O
residente fica entre `CAP` e `2 × CAP` — logo um conjunto de trabalho de `1 106` cabe. Subir o
tecto só teria mudado **onde** fica o penhasco.

⚠️ **A régua é um CONTADOR, não um relógio** (`TextSystem::shapes()`): moldagens são determinísticas
e um portão de tempo entraria na família das flakes de carga (`CLAUDE.md` §5.0).

⇒ o que ficou de pé de perfil, e as duas metades gateadas:

| metade | onde |
|---|---|
| mecanismo (a cache roda em vez de se deitar fora) | [`ph2d-text/tests/a_still_screen_never_reshapes_its_text.rs`](../../../crates/ph2d-text/tests/a_still_screen_never_reshapes_its_text.rs) |
| produto (o ecrã que o artista abre não paga moldagem nenhuma) | [`shells/desktop/tests/the_app_never_reshapes_a_still_screen.rs`](../../../shells/desktop/tests/the_app_never_reshapes_a_still_screen.rs) |

⚠️ **O gate de produto mora no SHELL de propósito:** o conjunto de trabalho que produz o fenómeno é
o do app inteiro, e a `ph2d-panel-registry-init` liga **22 dos 26** painéis (as features pobres do
`CLAUDE.md` §2) — ali o penhasco não existe, e o gate sairia verde sem medir nada.

⚠️ **Fica por medir em RELEASE.** Debug corre 20–50× mais devagar; os `7,81 ms` de hoje são um
dígito lá. *Isto continua a ser uma dívida com número, não um incêndio* — mas o penhasco em si
desapareceu, e o gate impede que ele volte em silêncio.

## §4 — ⛔ Por que isto NÃO é o report do dono

> *«quando colapsei arrastando e abri no menu da barra superior travou por um minuto»* — Enio,
> 2026-09-09.

Um minuto é **três ordens de grandeza** acima do que esta medição encontra, e o caminho dele
(fechar por arrasto, reabrir pelo menu) não abre 26 painéis. *Uma explicação que não bate na ordem
de grandeza não é a explicação* — e chamar-lhe a causa fecharia a caça a um defeito que continua
solto. O gesto que ele usou saiu do produto na w49; o minuto **fica sem causa conhecida**, e o
que falta para o caçar é uma reprodução.

## ⛔ Recusas MEDIDAS

| o que | por que não |
|---|---|
| atribuir o «minuto» a este penhasco | três ordens de grandeza de diferença, e a rota dele não abre 26 painéis |
| curar antes de perfilar | ⚠️ **respondida, e a resposta refutou a própria lista**: os 182 ms não estavam em passe nenhum dos três suspeitos, e sim num `clear()` de cache (§3) |
| subir o `LAYOUT_CACHE_CAP` | só muda **onde** fica o penhasco; a cura é rotação entre duas gerações, e o residente passa a ser `CAP..2×CAP` (§3) |
| um portão de TEMPO sobre o quadro | moldagens são determinísticas e um relógio entraria na família das flakes de carga (§5.0) — a régua é o contador `TextSystem::shapes()` |
