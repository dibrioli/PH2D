# 09 — O penhasco com **todos** os painéis abertos

> Medido em 2026-09-09 pela `line/UIUX`, a caçar o *«travou por um minuto»* do report do dono.
> ⛔ **Não é a causa daquele report** — ver §4. É outra coisa, e ela existe.

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

## §3 — O que ainda não foi medido (e é por onde se continua)

- **Onde** os `182 ms` são gastos. A sonda mede o quadro inteiro; falta um perfil por passe
  (`panel_walk` · `reserve_slot_tabs` · o `HitIndex` · os clips do Vello).
- Suspeitos por ordem de barateza: **clips ANINHADOS** (cada flutuante empurra o seu, e uma pilha
  de camadas do Vello não é linear), o **`HitIndex`** com todas as superfícies registadas, e a
  medição de texto das abas (`prefix_width` por ocupante por encaixe por quadro).
- ⚠️ **Em RELEASE isto não foi medido.** Debug corre 20–50× mais devagar, logo `182 ms` ali é
  plausivelmente um dígito em release — *o que faz disto uma dívida, não um incêndio*.

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
| curar antes de perfilar | não se sabe **onde** os 182 ms são gastos; a lista da §3 são suspeitos, não causas |
