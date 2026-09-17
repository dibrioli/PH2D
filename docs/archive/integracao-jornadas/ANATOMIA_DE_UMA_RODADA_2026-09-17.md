# ⏱️ Anatomia de uma rodada de integração — **onde as horas foram, medido**

> Rodada de **2026-09-17**: seis linhas (`UIUX` · `components` · `Vector` · `sculpt3d` ·
> `3DModeling` · `motion-value`) integradas no `main` por um integrador dedicado.
> Pergunta do dono, no fim: *«porque foram necessárias tantas horas, e que estratégia evita isso?»*
>
> ⚠️ **Este doc é MEDIÇÃO, não impressão.** Os números saem do `git log` (hora de cada merge), dos
> `mtime` dos registos de cada corrida de portão, e da contagem das falhas por família. O que não
> foi medido está marcado como não medido.

---

## §1 — A cronologia, do `git log`

| linha | aterrou | Δ desde a anterior |
|---|---|---|
| `line/UIUX` | 16/09 **23:08** | — |
| `line/components` | 17/09 **01:07** | 1 h 59 |
| `line/Vector` | 17/09 **01:24** | **17 min** |
| `line/sculpt3d` | 17/09 **07:00** | **5 h 36** |
| `line/3DModeling` | 17/09 **07:15** | **15 min** |
| `line/motion-value` | 17/09 **07:26** | **11 min** |

⭐⭐⭐ **As duas últimas linhas custaram 26 minutos JUNTAS**, e não são mais pequenas que as outras:
a `3DModeling` levou **92 commits e sete conflitos** (quatro degraus de schema, um painel que duas
linhas mudaram, seis ficheiros de memória) e a `motion-value` **77 commits**.

⇒ *o custo não é proporcional ao tamanho da linha.* Ele é **o custo da PRIMEIRA vez que cada
família de falha aparece**.

⚠️ **O que não foi medido:** a janela `01:55 → 07:02` contém uma compactação de contexto do agente,
e não há instrumento que separe, dentro dela, o tempo de diagnóstico do tempo de espera. O que se
pode afirmar é o que está abaixo: o que foi corrido, quantas vezes, e porquê.

---

## §2 — O portão não é o gargalo. Medido.

Seis corridas completas de `foundational-integrate.sh` (duas por linha: uma vermelha, uma verde).

| corrida | fase de testes | falhas | veredito |
|---|---|---|---|
| `sculpt3d` #1 | 120,8 s | 1 | ✗ |
| `sculpt3d` #2 | 121,2 s | 0 | ✓ |
| `3DModeling` #1 | 99,9 s | 2 | ✗ |
| `3DModeling` #2 | 100,5 s | 0 | ✓ |
| `motion-value` #1 | 40,4 s | 2 | ✗ |
| `motion-value` #2 | 41,3 s | 0 | ✓ |

**A fase de testes custa entre 40 e 121 segundos.** Mesmo com a compilação da árvore combinada à
frente, uma corrida inteira são minutos. **Seis corridas não explicam horas.**

⇒ o relógio está no **ciclo entre corridas**: ler o vermelho, decidir a cura, escrevê-la, prová-la.

---

## §3 — A taxonomia das falhas, que é onde a estratégia nasce

**Oito falhas** ao todo, nas três linhas que este integrador correu:

| família | nº | podia a LINHA tê-la visto? |
|---|---|---|
| **Censo de texto (HR-15)** | **5** | ⛔ **NÃO** — por construção |
| **Tecto de LOC por ACUMULAÇÃO** | 2 | ⛔ **NÃO** — por construção |
| Partida pela CURA do próprio integrador | 1 | (auto-infligida) |

### §3.1 — Porque a linha não pode vê-las: as duas são **propriedades da SOMA**

- **O censo** é escrito pela linha `X` e o literal que o acorda é escrito pela linha `Y`. **Nenhuma
  das duas árvores contém as duas coisas.** As duas fecham verdes de boa-fé, e a falha só existe
  depois do merge. Foi assim nas cinco: o censo é da `line/UIUX` e os literais eram da `sculpt3d`
  (14), da `Vector` (3), da `3DModeling` (1, um shader WGSL) e da `motion-value` (1, um diagnóstico
  que mudou de fase).
- **O tecto de LOC** é *a única grandeza deste repo que SOMA entre linhas sem ninguém a contar* —
  o `CLAUDE.md` §5.0 já o diz. Nesta rodada ele mordeu duas vezes: a catraca da shell
  (`197 376` contra `196 990`) e o **ficheiro da escada do schema** (`651` contra `600`), este
  último porque a rodada lhe acrescentou **dezasseis** parágrafos.

⚠️⚠️ **E o CI não os corre.** O job de teste do `spike.yml` é um `-p` de ~25 pacotes; estes gates
vivem em `tests/it/` e são alcançados pelo `nextest-impacted` e pelo `ship.sh`. ⇒ *não há nenhum
sítio, em todo o processo, onde alguém que não seja o integrador os veja.*

### §3.2 — A única falha auto-infligida, e a lição dela

A catraca da shell pedia **711** linhas (386 de excesso + ~326 do que faltava vir). O corte feito
levou **4 470** — seis vezes mais. O excesso não foi de graça: ele apanhou um gate que lia um
ficheiro **por caminho em tempo de execução** e quatro ficheiros de teste que o `git mv` deixou
para trás, o que custou **uma corrida inteira de portão a mais**.

⇒ **dimensione o corte pela MEDIÇÃO** (a catraca imprime o excesso), não pela generosidade.

---

## §4 — A prova de que o custo é a PRIMEIRA vez

As duas últimas linhas custaram 26 minutos porque, quando chegaram:

1. as famílias de falha já eram conhecidas (censo · LOC · memória · schema);
2. **existiam dois scripts** escritos a meio da rodada — a recontagem do degrau de schema e a fusão
   append-only dos docs de memória —, e os conflitos passaram a ser mecânicos;
3. o padrão de arquivo da escada já tinha sido descoberto (⚠️ *depois* de eu ter escrito por cima
   de um ficheiro de história de 490 linhas que já existia — ver §5.3).

*Uma rodada em que o integrador aprende as famílias enquanto integra paga cada aprendizagem uma vez
por linha, em série.*

---

## §5 — A estratégia, por ordem de alavanca medida

### §5.1 — ⭐⭐⭐ **A linha fecha contra o `main` de HOJE, e corre os censos**

**O maior lever, e o mais barato.** Antes de escrever o handoff (DIRETRIZ §1.5.9), a linha:

```bash
git rebase main                      # a árvore combinada, do lado da linha
bash scripts/censos-da-arvore-combinada.sh
```

⇒ isto converte **N descobertas em série do integrador** em **N descobertas em paralelo das
linhas** — e cada linha cura o SEU texto, que é quem sabe se ele chega ao ecrã ou não. ⚠️ **A
decisão é de quem escreveu o literal, não de quem integra:** foi o integrador que teve de decidir,
sobre código alheio, se `"Bool"` numa fixtura é língua e se um shader WGSL é texto de interface.

⚠️ **Não é grátis e o preço está medido:** o censo da shell sozinho custa **77 s** (ele varre 742
ficheiros três vezes; já é memoizado). Os das famílias custam 4–6 s cada. Com a árvore quente, o
conjunto são ~2 minutos.

⛔ **E isto NÃO substitui o portão da árvore combinada** — o `--ff-only` continua a ser a única
prova de que ninguém aterrou no meio. O que muda é que ele deixa de ser o sítio onde estas falhas
são **descobertas**.

### §5.2 — ⭐⭐ **A recontagem de um contador partilhado é um SCRIPT, não um juízo**

O degrau do `PROJECT_SCHEMA` reconta-se por uma regra mecânica: *pega na escada do `main`, extrai o
degrau que a linha acrescentou, renumera-o para `main+1`, sobe a tripla do ficheiro irmão*. Foi
escrito a meio desta rodada (depois de dois degraus feitos à mão) e vive em
[`scripts/schema-recount.py`](../../scripts/schema-recount.py).

⚠️ **Cada passo dele tem `assert`** — a 1.ª redacção supôs que a âncora era igual dos dois lados
(verdade na escada, falso na tripla, que CONTÉM o número) e **parou alto** em vez de escrever lixo.

### §5.3 — ⭐ **Antes de inventar uma cura de arquivo, `grep` o padrão**

A cura do tecto da escada (arquivar uma faixa de degraus) **já existia com três irmãos**
(`project_schema_history{,_v83,_v99}.rs`). A 1.ª tentativa escreveu por cima do primeiro deles —
490 linhas de história verbatim — e só o `grep 'mod project_schema_history'` no `main.rs` o
mostrou. *Uma cura que parece óbvia ao integrador costuma já ter sido paga.*

### §5.4 — ⏳ **Não medido, e por isso não recomendado ainda: a fusão de ensaio**

Fundir as seis num ramo descartável e correr os censos **uma vez** daria a lista COMPLETA de
antemão, em vez de uma lista por linha. ⚠️ Mas os conflitos de rebase teriam de ser resolvidos
**duas** vezes (uma no ensaio, outra a sério) e isso **não foi medido** — nesta rodada foram sete
conflitos só na `3DModeling`. *Fica nomeado como hipótese, não como recomendação.*

---

## §6 — O que esta rodada mostrou sobre os próprios instrumentos

- ⭐ **As duas metades de um censo (o intruso + a isenção órfã) a acusar na MESMA corrida são o que
  distingue uma MUDANÇA DE ENDEREÇO de texto novo.** Cada uma sozinha mente, e as duas curas
  seriam erradas. Aconteceu três vezes.
- ⛔⛔ **Um literal que aparece num `_tests.rs` pode ser um ÓRFÃO, não texto novo:** a régua salta o
  que está declarado sob `#[cfg(test)]`, logo um ficheiro que o `git mv` deixou para trás **deixa
  de ser saltado**. Escrever-lhe uma isenção calaria o instrumento para sempre.
- ⛔ **`cargo check --all-targets` é cego aos testes de uma DEPENDÊNCIA** — quatro ficheiros de teste
  ficaram para trás com a árvore verde. Quem os apanhou foi o `cargo nextest list`.
