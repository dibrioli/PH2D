# Tectos de recurso por linha — o que uma linha pode tomar da máquina

> **Gatilho (Enio, 2026-09-15):** *«seus testes deixaram o PC lento por muito tempo
> prejudicando os smokes das outras linhas… veja se é possível definir uma regra que
> nunca permita que uma única linha consuma mais de 50 % da GPU ou da CPU.»*
>
> **Resposta curta:** para a **CPU sim**, e está a valer, medido e obrigado por um
> guarda. Para a **GPU não** — e a impossibilidade é do hardware, não uma folga:
> a regra honesta ali é **exclusão com prazo**, que é mais forte do que 50 % para o
> defeito que de facto aconteceu. As duas metades estão abaixo, com as medições.

---

## §1 — A porta

Todo comando pesado de uma linha passa por **`scripts/ph2d-run.sh`**:

```bash
bash scripts/ph2d-run.sh cargo test -p ph2d-timeline
PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-field-gpu -- --ignored
```

Ela põe o comando numa **fatia (cgroup) que é da LINHA**, derivada da worktree
(`git rev-parse --show-toplevel`), com quatro tectos:

| tecto | valor | de que RECURSO é | onde se sobe |
|---|---|---|---|
| CPU | `50 %` dos núcleos (`1600 %` de 32) | tempo de CPU da máquina | `PH2D_CPU_PCT=75` |
| prioridade | `CPUWeight=20` | quem CEDE quando a máquina enche | — |
| memória | `24G`, `MemorySwapMax=0` | RAM (⛔ não swap: o zram vive dentro da RAM) | `PH2D_MEM_MAX=48G` |
| prazo | `1800 s` | a ATENÇÃO de quem devia colher o resultado | `PH2D_PRAZO=5400` |
| placa | exclusão | ver §3 | `PH2D_GPU_ESPERA=600` |

⚠️ **§0.0: quem sobe um tecto mede antes e escreve o número.** Os três portões do
repo já o fazem à vista — `cargo-test-narrow.sh` re-entra com `900 s`,
`nextest-impacted.sh` com `5400 s`, `ship.sh` com `7200 s`, cada um com o número
escrito no próprio ficheiro.

---

## §2 — Por que a fatia é da LINHA e não do comando (medido 2026-09-15)

Um tecto por **comando** não compõe — duas corridas somam dois tectos. Medido com
queimadores de 32 threads, lendo o `cpu.stat` da própria fatia (⚠️ a 1.ª leitura
saiu por `/proc/stat` e veio contaminada pelo smoke que o dono corria ao lado —
*uma régua da máquina inteira não mede uma linha*):

| arranjo | comandos | núcleos ocupados |
|---|---|---|
| fatia **sem** tecto (controlo) | 2 | **22,32** de 32 |
| tecto no **COMANDO** | 2 | **30,05** ⛔ soma |
| tecto na **FATIA da linha** | 2 | **16,03** |
| tecto na **FATIA da linha** | 4 | **16,01** ⭐ compõe |

⭐ *«nunca mais de 50 %» é uma afirmação sobre a LINHA, e só um cgroup que a linha
inteira partilha a pode fazer.* Um `nice`, um `-j`, um `--test-threads` são todos
por-processo — nenhum deles consegue exprimir esta frase.

### §2.1 — O tecto não chega: falta a PRIORIDADE

Um tecto de 50 % limita **uma** linha; ele não impede que **duas** linhas encham a
máquina, e não foi para isso que o Enio pediu. O que protege o smoke do dono é um
**peso**, que é outra grandeza: ele não desperdiça máquina ociosa e cede quando
alguém precisa. Medido, com um smoke fingido de 8 threads:

| situação | o smoke leva | a linha leva |
|---|---|---|
| máquina livre (referência) | **6,8 s** | — |
| linha a 32 queimadores, `CPUWeight=20` | **7,0 s** (+3 %) | 17,7 núcleos |

⇒ os dois coexistem na fatia de propósito: a **quota** responde ao pedido do dono,
o **peso** responde ao problema dele.

---

## §3 — A placa: 50 % NÃO é exprimível, e isso mede-se

As quatro portas foram perguntadas a esta máquina (RTX 5060 Ti, driver proprietário):

| porta | resposta medida |
|---|---|
| **MIG** (partição de placa) | `[N/A]` — é placa de consumidor, não particiona |
| controlador cgroup **`dmem`** | existe no kernel e **não está delegado** ao utilizador; e o driver proprietário não implementa contabilidade DRM por cgroup |
| **compute mode** `EXCLUSIVE_PROCESS` | só cobre contextos **CUDA**; este repo desenha por **Vulkan/wgpu** |
| alguma quota em `sysfs` | nenhuma |

⛔ **Não há como dar 50 % da placa a uma linha.** Escrever a regra à mesma seria um
limite que não diz de que recurso é — exactamente o que o §0.0 proíbe.

⭐ **E a regra honesta é mais forte do que a pedida.** O defeito real de 14/09 não
foi partilhar a placa: foi **segurá-la** — uma sonda pendurada ficou `56 minutos`
com o driver e `2,6 GB`, em `S (sleeping)`, com o tempo de CPU **parado**. Uma
quota de 50 % não teria ajudado em nada. O que ajuda é **exclusão + prazo**, e as
duas metades estão provadas:

| ensaio | resultado |
|---|---|
| segunda linha tenta entrar com um detentor vivo | **recusada**, e a mensagem diz **quem** segura (linha, pid, desde quando, comando) |
| detentor **pendurado**, prazo a expirar | morto (`exit 124`), a fechadura **liberta-se sozinha** |

A recusa sai com **`exit 75` (`EX_TEMPFAIL`)** — *tente outra vez*, não *o seu
código está errado*. ⛔ **Não force**: duas linhas na placa ao mesmo tempo é o que
pendurou o driver.

---

## §4 — Por que o prazo é do SCOPE e não de um `timeout` (medido 2026-09-15)

Um binário de teste **reparenta-se ao `systemd --user`** e sobrevive a quem o
lançou — o repo já pagou isto duas vezes (dois binários de `ph2d-poly2d` queimaram
`1 h 55 m` a ~6 núcleos depois de o lançador morrer). Ensaio com um neto `setsid`
a fingir esse órfão, **com controlo** (a régua VÊ `1` neto vivo antes do ensaio):

| mecanismo | netos vivos depois |
|---|---|
| scope com `RuntimeMaxSec=4s` | **0** ⭐ o cgroup alcança |
| `timeout --kill-after` sozinho | **1** ⛔ não alcança |

⚠️ **A 1.ª corrida deste ensaio deu `0` nos DOIS** e quase fechou a pergunta ao
contrário: o script do neto acabava em `exec sleep 900`, que **substitui a linha de
comando**, e o `pgrep -f` deixou de o ver. *Um zero de «morreu» e um de «a régua
não vê» são o mesmo byte* — foi o controlo que os separou.

---

## §5 — O guarda: por que isto é um hook e não uma linha num doc

`.claude/hooks/tecto-de-recursos.sh`, ligado em `.claude/settings.json`, corre
**antes de cada Bash de cada agente deste repo**.

⛔ **Sem ele, isto seria mais um script que ninguém chama** — e o repo tem a
medição: o `cargo-check-narrow.sh` está no roteador há semanas, faz exactamente a
coisa certa, e foi invocado **5 vezes em 101 sessões** contra **13 791** `cargo
check` digitados à mão; o `git-stage-guard.sh` tem 5 docs a apontá-lo e **zero**
invocações. *Ponteiro não é adoção.*

Ele recusa **duas** formas e dá a linha corrigida:

- **R1 — comando pesado fora da porta.** `cargo test|nextest|build|run|bench|clippy`
  sem `ph2d-run.sh`.
  ⚠️ **O laço interno fica de fora de propósito:** `cargo check`, `fmt`, `tree`,
  `metadata` custam 1–3 s e são a resposta certa a *«a minha edição entrou?»*.
  Encarecê-los empurraria os agentes de volta ao `cargo test`, que é o defeito
  **4,3:1** que o repo já mediu.
- **R2 — vigia de fundo sem prazo.** Um `until …; do sleep 45; done` em segundo
  plano nunca termina sozinho, e **o silêncio dele lê-se igual a «ainda a
  trabalhar»**. Em 14/09 nove destes ficaram a girar e um deles escondeu a sonda
  pendurada durante quase uma hora.

⚠️ **Ele FALHA ABERTO por desenho** — `jq` ausente, JSON ilegível, estado que não
reconhece: devolve `0` e o comando passa. *Um guarda que se engana a fechar pára
seis linhas; um que se engana a abrir volta ao que havia antes dele.*

**Prova versionada:** `bash .claude/hooks/tecto-de-recursos.prova.sh` — 18 casos,
e os de *«tem de passar»* valem tanto como os de *«recusado»*.

---

## §6 — ⛔ Recusas MEDIDAS — não as reconstrua

| o que foi pensado | por que ficou de fora |
|---|---|
| **quota de 50 % na GPU** | não é exprimível nesta máquina (§3); e não cobre o defeito real, que é segurar, não partilhar |
| **tecto por COMANDO** (`-p CPUQuota` no scope) | **não compõe**: 2 comandos = 30,05 núcleos (§2) |
| **`timeout` como único prazo** | não alcança o binário reparentado: `1` órfão vivo contra `0` do scope (§4) |
| **`nice` / `--test-threads` / `-j`** | são por-processo; nenhum exprime *«esta LINHA ≤ 50 %»* |
| **quota no `ph2d.slice` (todas as linhas juntas)** | seria um número que eu não medi — não consegui medir o smoke do dono (ele terminou antes). O **peso** (§2.1) responde à mesma preocupação **com** medição. ⏳ Fica aberto: medir de quantos núcleos um smoke precisa e decidir se o tecto global vale a pena. |
| **`runner` no `.cargo/config.toml` do repo** | alcançaria TODO binário de teste sem depender de ninguém se lembrar — mas é global, atinge a CI (onde não há `systemd-run`) e o `CLAUDE.md` §2 já proíbe mexer ali pelo mesmo motivo (o linker). ⏳ **Decisão do Enio**, com o preço ao lado. |
| **`IOWeight` na fatia** | não medido. Escrever um número sem o medir é o que o §0.0 chama palpite. ⏳ aberto |

---

## §7 — O que isto NÃO resolve

- ⚠️ **Duas linhas a 50 % enchem a máquina.** O pedido era sobre **uma** linha, e é
  isso que está garantido. O que protege o dono de N linhas é o `CPUWeight`, que é
  prioridade e não tecto.
- ⚠️ **O guarda só vê o comando EXTERNO.** Um `cargo` nascido dentro de um script
  passa — é por isso que os três portões pesados **re-entram pela porta sozinhos**.
  Um portão novo tem de fazer o mesmo, e o bloco a copiar está nos três.
- ⚠️ **Fora deste repo o guarda não existe** (ele vive no `.claude/` do projecto).
- ⏳ **A fatia é criada em `~/.config/systemd/user/` na primeira corrida.** Numa
  máquina sem systemd de utilizador (CI, contentor, macOS) a porta **avisa e
  continua** — com prazo, sem tecto de CPU.
