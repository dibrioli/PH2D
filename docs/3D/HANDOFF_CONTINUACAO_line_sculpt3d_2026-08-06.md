# HANDOFF DE CONTINUAÇÃO — `line/sculpt3d` (2026-08-06)

> Para o **agente NOVO que assume esta linha viva**. Não é handoff de integração
> (esse é o `HANDOFF_INTEGRACAO_line_sculpt3d_MESTRE_2026-08-04.md`, e ele descreve
> o que **já entrou** no `main`). Este descreve o que está **na linha e ainda não
> entrou**, e o que fazer a seguir.

---

## FASE 0 — ONDE VOCÊ ESTÁ (execute ANTES de abrir qualquer arquivo)

```bash
cd ~/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && pwd && git branch --show-current
```

→ o `pwd` TEM de terminar em `/Worktrees/line-sculpt3d` · a branch TEM de ser
`line/sculpt3d`. ⚠️ **A sua janela abre na RAIZ, que está em `main`, e os MESMOS
paths relativos existem nas duas árvores** — editar a errada compila, testa e
commita sem um único erro, e ninguém descobre até a integração. Esta linha já
tem precedente disso registrado no repo (a `line/Painter`, §5.17-§5.19 do doc 28
dela): *no Modo L, todo comando começa com o `cd` da worktree*.

Depois: `git log --oneline -5 && git status -sb`, e **`git rebase main`** no
início de cada jornada (DIRETRIZ §1.5.2.3).

---

## §1 — O ESTADO, MEDIDO (não afirmado)

| | |
|---|---|
| HEAD | `f3ad37809` |
| Commits à frente do `main` | **23** |
| Árvore | **limpa** |
| Suítes do módulo (6 crates) | **427 verdes em release · 381 em debug**, 0 falhas |
| Suíte do shell | **2 394 verdes**, 0 falhas |
| Gates de GPU | `#[ignore]` — **26/26 na RTX** foi o número da W10.1; **re-rode antes de fechar** |
| `PROJECT_SCHEMA` | **55 na linha E no `main`** ⇒ esta jornada **não bumpa schema** |
| `SCULPT_DOC_VERSION` | **1** (intocado) |

⚠️ **Rode as duas** (release **e** debug). O repo tem precedente registrado de
defeito que só aparece em debug (o `ph2d-flip-colorize` panicava no `wrapping_sub`
e a suíte era rodada só com `--release`), e a `line/Painter` repetiu a política
na integração de 02/08.

### O que o Enio JÁ aprovou nesta jornada, e o que ele não viu

- ✅ **A CAVIDADE + o PAINEL** — *"Smoke OK. Siga"*.
- ✅ **O ALPHA**, depois de **reprovar a 1ª rodada** (*"os poros são gigantescos"*)
  e aprovar o conserto — *"smoke OK … siga"*.
- ⏳ **A W9.2 (a perf do refino, quatro commits) nunca teve rodada própria** — ela é
  **invisível por construção** (byte-idêntica; o que muda é o relógio), e o plano a
  marca `pendente de smoke` nos quatro sub-itens.

⚠️ **Os marcadores do `06.1-Waves-riscos-e-alvos.md` são o REGISTRO, e eles estão
incompletos:** W9.3 · W10.1 · W10.2 carregam `✅` **sem marcador de smoke**, apesar
das duas aprovações acima. *Uma aprovação que não está escrita não aconteceu para o
próximo agente.* **Primeira tarefa de higiene: peça a confirmação ao Enio e carimbe
os marcadores** — não assuma, e não os apague.

---

## §2 — O QUE ESTES 23 COMMITS CONTÊM (para você NÃO reconstruir)

Em ordem cronológica, agrupados por assunto:

**(A) W9.2 — a perf do dyntopo** (`b1331046b` · `5c1013a6b` · `dd1bf6f2e` ·
`8872b5567`): o refino pergunta só pela **REGIÃO** · os anéis aceitam uma **edição**
(o CSR deixa de ser soma de prefixos) · o corte **escreve no lugar** (a malha absorve
a topologia em vez de nascer de novo) · e o corte **deixa de construir o grafo de
arestas da malha inteira**. Tabelas por raio no plano §W9.2c/d.

**(B) W9.3 — o COLAPSO** (`06d0697af` · `8ffc03349`): o pincel **remove** detalhe, a
topologia aprende a encolher, e o traço sobrevive a um colapso (o dab passa a afinar
onde sobra). Histerese **2,05**, com a varredura ao lado (`7f848ecb1`).

**(C) A UI da cena 3D** (`71d2720ae` · `cb997053d` · `07ce02d19` · `0ee23ff64` ·
`0ed1b64ff`): o **painel** (crate nova `ph2d-panel-sculpt3d`, as dezesseis
ferramentas + knobs + o anel do cursor) · as duas opções de **VISTA** (Wireframe e
Matcap) · o fix da **janela que um dab publica** (era da última cópia do espelho, e é
da CHAMADA) · e as duas sondas que **INOCENTAM a matemática do pick** (o defeito era
memória de UI). Doc: `04-Ferramentas/04.4-O-painel.md`.

**(D) W10.1 — a CAVIDADE** (`e0ad8340c` · `91487fe26` · `86b4c9ec8` · `8b40bd9b2`):
a **curvatura por vértice** (o 5º plano) e o termo do shader que faz a fresta
escurecer e a crista clarear. `DEFAULT_CAVITY = 0` ⇒ **o barro liso da W3, ao byte**.
Cena **`=15`**.

**(E) O ACCUMULATE** (`5123a24ec`): a lei do traço troca o envelope por uma
**integral de linha** — o mesmo raciocínio que o doc 20 do Painter registrou e que
lá **não** foi construído.

**(F) W10.2 — o ALPHA** (`9e00285a0` · `5bf19c5c3` · `ed63c06f6` · `bc29a356f`):
seis padrões analíticos que multiplicam o falloff dos dezesseis verbos, o
`recommended_scale` (o seed sai do **MODELO**, não de um literal) e as duas notas de
doc que ainda diziam `0,25`. Cena **`=16`**.

**(G) A sonda do AO** (`f3ad37809`, hoje): medição, sem produto — ver §4.1.

---

## §3 — AS LEIS DESTE MÓDULO QUE JÁ FORAM PAGAS

Cada uma custou uma rodada de smoke ou uma mutação sobrevivente. **Não as
re-derive; e se for mover o número de alguma, reconfira a nota** (`CLAUDE.md` §0).

1. **A lei do traço é um ENVELOPE, não um produto por-dab.**
   `accum[v] ← max(accum[v], w)` e `positions[v] ← lerp(base, target, accum)`.
   É isso que dá independência de espaçamento, idempotência sob re-stamp e undo
   trivial. A `line/Painter` curou a doença oposta **quatro vezes**.

2. **O padrão do alpha é mapeado em 3D, e é o OPOSTO do que o `04.1-Pinceis`
   previa.** Num mapeamento projetado o `max` sobre dezenas de dabs **lava** o
   padrão até a envoltória superior dele — o pincel ficaria mais FORTE, não
   texturizado. A variante direcional entra como **variant novo do enum**, nunca
   como reescrita.

3. **A LEI DAS DEZ ARESTAS.** Uma célula de padrão precisa de ~10 arestas de malha
   para ser amostrada como padrão (correlação entre vizinhos: 1,2 → 0,06 ·
   **10,2 → 0,82** · 20,4 → 0,95).

4. **RESOLVER não é PARECER** — e é o erro que reprovou o 1º smoke do alpha.
   `0,25` era perfeitamente resolvido pela malha da cena **e** punha oito features
   atravessando uma esfera unitária: crateras. **Uma escala absoluta é a unidade
   certa; um literal absoluto não é** ⇒ o seed sai do modelo
   (`max(maior lado ÷ 33, 10 × aresta)`).

5. **A curvatura é DERIVADA, não autorada** (como a normal): fora do undo, fora da
   serialização — mas ela **tem** de viajar pelas quatro portas (`rebuild` ·
   `refresh_region` · `splice_topology` · `shrink_topology`). ⚠️ O comentário do
   splice avisa desde a W6: *o dia em que entrar um quinto plano por-vértice, quem
   esquecer dele é esta função*.

6. **Cobertura volumétrica cai com o CUBO do raio** (as sementes são pontos no
   volume e a superfície o corta) — a primeira calibração dos padrões errou por
   quase 4× por causa disso.

7. **O `rayon` NÃO entra na `ph2d-sdf`**, e a ausência está escrita no `Cargo.toml`
   dela **com o mecanismo**: as caixas de dois triângulos se SOBREPÕEM ⇒ a escrita
   do voxelizador **não é disjunta**, que é a condição do ADR-0109. Paralelizar isto
   exige **ADR próprio** (precedente: as três exceções do repo).

8. **Um gate de costura não pode ENUMERAR os grupos** — a varredura
   *"pintado → clicável"* fazia isso e a lista já tinha apodrecido (matcap,
   Accumulate e Wireframe **nunca foram varridos**). Hoje o conjunto é o que o
   `paint` registrou e a exclusão é por **GESTO** (arrasto), então um controle novo
   nasce **coberto**.

---

## §4 — O QUE ESTÁ ABERTO, NA ORDEM, COM O NÚMERO AO LADO

### 4.1 — O **AO ASSADO** (o irmão da cavidade) — **medido hoje, desenho decidido**

O plano dizia *"o AO assado do §3 é o irmão desta wave e usa o mesmo canal de
vértice"*. A sonda nova (`crates/ph2d-sdf/tests/measure_ao.rs`) mediu o
pré-requisito dele **antes de uma linha ser escrita**, e mudou metade da frase:

| malha | res 64 | res 128 | res 192 |
|---|---|---|---|
| 13 682 verts | 22,3 ms | 52,9 | 126,8 |
| 153 122 verts | 106,6 | 157,7 | 228,2 |
| **425 602 verts** | **231,0** | **300,9** | **386,0** |

**(1) O campo NÃO cabe num pen-up.** Na malha que a cena `=16` abre são
**231-386 ms** — 14× a 23× um quadro de 60 fps, **antes** do primeiro cone ⇒ o AO
assado é um **BOTÃO explícito**, como o REMESH da W7, e a obsolescência é inerente
ao desenho e tem de ser **dita** (a recusa do remesh sob multires é o precedente de
como se diz).

**(2) A decomposição contradiz o palpite:** voxelizar é limitado por **TRIÂNGULO**
(11,3× para 31× a malha; só **1,52×** para **23×** as células) e o flood fill por
**CÉLULA** (quase linear). ⇒ **um campo FINO é barato; uma malha densa é cara.**

⚠️ **O que ainda NÃO foi medido:** o cone tracing em si. Meça-o **antes** de
desenhar o resto — a sonda já existe e o padrão da casa é *o número sai da porta do
PRODUTO*.

### 4.2 — A cavidade **não alcança a DOAÇÃO**

O G-buffer a ignora, como já ignora a máscara: o canal doado é uma **NORMAL** e o
alvo `vec4` não tem canal livre. Levar oclusão à tinta 2D é um **segundo plano**,
portanto uma wave — e ela pesa mais que as outras porque **o objetivo 2 do módulo é
exatamente esse** (`05.2-Doacao-de-sombreamento-para-2D.md`).

### 4.3 — O **SSS pré-integrado** (`05.1` §2a)

Consome **a mesma curvatura** que a W10.1 já publica — *um dado, dois usos*, e foi a
razão de a cavidade vir primeiro. É o item mais caro da W10 e o mais barato de
começar.

### 4.4 — O alpha por **IMAGEM** (a variante direcional)

Traz o frame do dab de volta, e é o caminho para um carimbo autorado. Entra como
**variant novo**, não como reescrita (lei §3.2).

### 4.5 — Abertos herdados, cada um com o motivo escrito

- **O colapso não roda o `dyntopo_flip`** (a valência sobe quando dois anéis se
  fundem) — W9.3.
- **O remesh RECUSA com a pilha de multires montada**, e a recusa é **nomeada no
  log** — a alternativa seria achatar a pilha em silêncio, que é destruir trabalho
  autorado sem dizer.
- A **resolução do remesh não é autorável** (o botão usa o default 150).
- O campo **não carrega cor, material nem a MÁSCARA**.
- `LayerKind::Sculpt3d` segue **não-apendado de propósito** (*um variant que ninguém
  constrói é um variant morto*).
- O **marching cubes** (o Surface Nets entrou antes de propósito: um vértice por
  célula e valência 4 é a topologia que um escultor quer receber).
- **Merge e isolate** — o que a W8.8 deixou (`§W8.8 — o que ainda falta`).

---

## §5 — A SUPERFÍCIE DE COLISÃO (para o handoff de INTEGRAÇÃO, depois)

| Item | Estado nesta linha |
|---|---|
| `PROJECT_SCHEMA` | **55, INTOCADO** ⇒ fora de disputa de número |
| `SCULPT_DOC_VERSION` | 1, intocado |
| Registro do `ph2d-ecs` | **intocado** |
| Contrato congelado (§6) | **intocado** (confira por grep, não por auto-relato) |
| ADR novo | **nenhum** — tudo roda sob o **ADR-0150** |
| Dep externa nova | **nenhuma** |
| Crate NOVA | **`ph2d-panel-sculpt3d`** |
| `Cargo.toml` tocados | **3** (a crate nova · `ph2d-panel-registry-init` · `shells/desktop`) |
| Foundational tocado (**tudo aditivo**) | `ph2d-editor-core` (`ids/chrome/sculpt3d.rs` **novo** + `ids/chrome/mod.rs` · `interaction/dispatch/scroll.rs` · `screens/hero.rs` + `hero/paint.rs` · `widget/mod.rs` + `widget/scrollbar.rs`) · `ph2d-i18n` · `ph2d-panel-registry-init` |

⚠️ **A crate de painel exige os CINCO sítios de fiação** — inclusive a lista
`default` do **shell**. Ligar a feature só na crate de registry **não alcança
ninguém**: foi assim que o painel de física do W2b nasceu invisível, e a
`line/Vector` repetiu a lição em 04/08. O gate
`every_panel_the_shell_drives_is_in_its_registry` está nesta linha por isso.

⚠️ **E o §3 do handoff MESTRE de 04/08 avisou um corte que já quebrou uma linha
vizinha:** esta linha **PARTIU** o `shells/desktop/src/project.rs` (o
`project_load_from` saiu para o irmão `project_load.rs`), e a `line/Vector` teve o
`project_tokens::install` **fundindo limpo para o lado errado do corte**. Se você
tocar aquela família de novo, diga no handoff.

---

## §6 — COMO SMOKAR (o que dizer ao Enio)

```bash
env PH2D_SCULPT3D_SMOKE=<n> cargo run -p ph2d-host-desktop --release
```

- **`=15`** — a **CAVIDADE**: sete sulcos paralelos de profundidades decrescentes.
  A **escada é o oráculo** (com uma profundidade só, ligar o canal daria *uma imagem
  diferente*, e diferente não é a pergunta). A cena **imprime a escada medida** —
  ⚠️ **se o primeiro sulco não passar de ~0,15, PARE.** `Shift+C` cicla
  `0 → 0,35 → 0,70 → 1,00`.
- **`=16`** — o **ALPHA**: abre com **425 k vértices** (35 ms, 0,555 ms/dab) porque
  só a partir de ~800 segmentos o LOOK passa a mandar. A cena **imprime a escala e
  as features**; abaixo de ~20 features o padrão sai como cratera.
- **`=1..=14`** — as cenas herdadas. Elas **têm de continuar iguais**: a cavidade
  nasce em 0 e o alpha em `None`, então o barro liso da W3 é **byte-idêntico**.
- E **rode uma vez SEM a env var** — é a metade do smoke que prova a **inércia** (o
  frame 2D é byte-idêntico sem `PH2D_SCULPT3D_SMOKE`).

---

## §7 — O QUE **NÃO** FAZER

- ⛔ **Não integre e não faça ship.** Você fecha a linha, escreve o handoff de
  integração (DIRETRIZ §1.5.9) e **PARA**. Integração e ship são ordem **explícita**
  do Enio, via agente integrador dedicado (`CLAUDE.md` §0.7).
- ⛔ **Nunca `push`, nunca `--force`, nunca `git add -A`.** Commite por caminho:
  `git commit --no-verify -m "msg" -- <paths>`. Crase em mensagem de commit
  **executa** ⇒ use `-F`.
- ⛔ **Não escreva limite sem medir** (`CLAUDE.md` §0). Todo `MAX_*`, faixa de
  slider e "por ora" desta linha tem tabela ao lado; o próximo também terá.
- ⛔ **Não desfaça mutação com `git checkout`** — use `cp` e depois `touch` (o cargo
  reusa o mutante se o mtime não mudar).
- ⛔ **Não aceite uma nota do plano como medição.** Duas notas deste módulo já foram
  derrubadas pela sonda que as foi verificar (o K2 da W1 apontava para o lugar
  errado; o *"irmão desta wave"* do AO virou **botão** hoje). *Quem move o número
  que tornava algo verdade tem de reconferir a nota.*
