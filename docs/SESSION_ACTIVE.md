# SESSION_ACTIVE — coordenação leve (DIRETRIZ §1.1)

**Propósito:** post-it compartilhado da posse VIVA da orquestração multi-agente —
quem está escrevendo o quê AGORA, para evitar colisão de git entre agentes
paralelos. **Modelo (DIRETRIZ §6.8):** 1 Coordenador (absorve PR/CI/ship) +
N Implementadores; os Implementadores **leem antes de cada burst** e não escrevem aqui.

> ⚠️ **O MODO é função do hardware, não uma escolha** (CLAUDE.md §0.5 / [ADR-0106](architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)):
> `constrained` = **Modo C** (shared tree + Coordenador, o parágrafo acima) · `workstation` =
> **Modo L** (uma worktree por linha, sem Coordenador; integração e ship por um **agente
> integrador dedicado**, só por ordem explícita do Enio). Rode `bash scripts/hw-profile.sh`
> antes de assumir qual dos dois vale nesta máquina.

> ⚠️ **PARADO desde 2026-08-04, e não é defeito de ninguém: este doc é do MODO C.** O seu
> único escritor autorizado é o **Coordenador**, e o tier `workstation` não tem Coordenador
> (ADR-0106). Medido em 2026-08-18: ele estava intocado há duas semanas enquanto **cinco**
> worktrees estavam vivas.
>
> **No Modo L, o registro de posse é `git worktree list`** — a própria árvore, sempre exata,
> sem ninguém para atualizar. *Um post-it que depende de um papel que não existe mais só pode
> mentir.* Este arquivo fica para quando a máquina for `constrained`.

**Não é log histórico nem fonte de estado.**
- Estado por-módulo (waves/tasks) → **CLAUDE.md §5**.
- Contratos congelados → **CLAUDE.md §6**.
- Histórico (o que já fechou) → **git log** + `docs/HANDOFF_*` / `docs/archive/`.
- Entradas concluídas saem daqui para o git log. **Limpe ao encerrar a sessão.**

---

## Integração — jornada FECHADA (5 linhas), 2026-08-04

**Cinco linhas integraram**, na ordem **MEDIDA** (custo = commits que tocam arquivos quentes
compartilhados; a mais cara primeiro, porque a primeira é fast-forward puro e não paga nada):
**`line/Vector`** (64) → **`line/sculpt3d`** (25) → **`line/physics`** (30) → **`line/motion-value`**
(13) → **`line/Painter`** (8). ⚠️ **Uma inversão deliberada:** o 3D entrou antes da física apesar
dos 25 contra 30, porque a física ainda estava com trabalho não-commitado na worktree — e o vão de
1,2× é ruído contra os 5-8× que separam a cauda.

**Os números, CONTADOS:** `PROJECT_SCHEMA` **50 → 55** (cinco degraus: 51 tokens · 52 sculpt ·
53 baked_forms · 54 joint custom · 55 player) · tripla `(55, 13, 14)` · registro `ph2d-ecs`
**46 → 52** e os **DOIS** espelhos **47 → 53** · `ph2d-physics-ecs` **26 → 28** · ADR máx **0155**
(0154/0155 ficaram como escolhidos — ninguém mais criou ADR) · contrato congelado **intocado**.

⚠️ **A física trouxe DOIS degraus de schema e o handoff dela contou UM** (previa 54; o par certo
era 54+55). *Nem a aritmética escrita pela própria linha se toma pronta.*

⚠️ **Quatro defeitos que só a árvore combinada viu**, todos com o produto CERTO e o gate expirado
ou latente: o arch-gate de tokens lendo `project.rs` depois de a `sculpt3d` ter partido o arquivo
(a linha do `project_tokens::install` **fundiu limpa para o lado errado do corte**) · e três gates
de `ph2d-editor-core/tests/` que a varredura impactada nunca alcança (`paint` em 201/200 · dois
arquivos fora do allowlist de a11y · um literal sem marcador).

**Estado:** `main` local à frente do `origin`; ship + push pendentes de ordem.

---

## Histórico anterior — jornada FECHADA (7 linhas), pushada, **CI VERDE**E**

**Sete linhas integraram** (2026-08-01), na ordem **MEDIDA**: **`line/Vector`** (10 commits —
o corte, o pathfinder, a reivindicação 2-D) → **`line/physics`** (7 — o rig sai da hierarquia,
a solda que cede, o corpo com várias formas) → **`line/Painter`** (28 — a performance do Wet
Paint e o teto de raio herdado do modelo) → **`line/sculpt3d`** (23 — a doação chega à tinta) →
**`line/anim`** (11 — a wave dos fades) → **`line/motion-value`** (10 — o conjunto completo de
idiomas do editor de nós) → **`line/FLIP`** (10 — as pontas do traço). **107 commits.**

> **Como a ordem foi medida** (a sobreposição se MEDE, não se escolhe): as DUAS metades —
> `git diff --name-only main...BRANCH` (net) **e** `git log --diff-filter=AD --name-only`
> (nascidos/mortos por commit, o corolário de 31/07). A segunda pagou-se na hora: a
> `line/Vector` tinha **união 68 > net 66** (o `cut_tool.rs` nasce e morre dentro dela,
> invisível ao diff net). A sobreposição real ficou **concentrada no wiring da shell**
> (`app_state.rs` · `main.rs` · `render_loop/mod.rs`), com duas colisões substantivas:
> `FLIP × physics` no `project.rs` (schema) e `motion-value × anim` no `interaction/`
> (que era **textual pura** — enums diferentes do mesmo arquivo).
>
> O critério de ordem foi o **custo de rebase medido**, não o tamanho: commits que tocam
> arquivo quente (Vector **13** · physics **11** · Painter 5 · os outros 4 · FLIP **2**).
> Quem vai primeiro é fast-forward puro e não paga nada ⇒ **o mais caro primeiro**, e a FLIP
> por último porque o que ela paga é uma CONTAGEM determinística.

**Verde na árvore COMBINADA** (rodado por linha, depois de cada ff-merge): impactados
**7339 · 6958 · 4831 · 4854 · 7073 · 6786 · 6973**. E o **`ship.sh` fechou VERDE 7/7**
(fmt · clippy · machete · deny · audit · typos · nextest `ci-test`) em **4 iterações**.
**Pushado por ordem do Enio** (`25e1f48ef..deebc5550`, 113 commits) e o **CI fechou
`success`**: https://github.com/dibrioli/PH2D/actions/runs/30690240926

> ⚠️ **O primeiro run foi CANCELADO, e nada nele falhou** — lint, MSRV, os três OS com
> check+tests, e o C9 de ubuntu e macOS todos verdes; só o C9 do `windows-latest` bateu
> `timeout-minutes: 15` **no meio do 3º hash** (`fail-fast: false` está setado nessa
> matriz, então não foi cascata: foi o teto). **E o comentário que justificava o 15 media
> a coisa ERRADA** (*"single binary run + hash capture; observed ~3-5 min"*): os três
> RUNS são baratos e não cresceram (physics C9 51s → 61s); quem custa é a **PRIMEIRA
> invocação do cargo**, que paga a compilação inteira, e o preço dela é função de quanto
> o `rust-cache` acertou. Medido no mesmo job, dois dias seguidos: o passo que paga o
> build foi de **2m28 → 10m22 (4,2×)** e o total de **9m00 → 15m (a parede)**. Teto →
> **45** (3× o pior observado, metade do que o job `test` irmão já orça para o MESMO
> build no MESMO OS). No run seguinte ele fechou em **9m06** — com o cache quente, o que
> confirma que o 15 foi batido por um cache FRIO logo depois do merge grande, que é
> exatamente o caso que o teto novo existe para cobrir.

**Gates `#[ignore]` rodados na RTX:** `ph2d-flip-render` **118/118** · `ph2d-mesh-render`
**16/16** · `ph2d-render` os reais verdes. ⚠️ **E aqui uma lição do integrador:** `--ignored`
**não é sinônimo de "os gates de GPU"** — este repo usa `#[ignore]` para TRÊS coisas (precisa
de adapter · scaffold `unimplemented!()` · probe manual que exige env), e a varredura cega
produziu 5 "falhas" que **não eram regressão nenhuma** (as 4 do `smoke_fixture_renderable` são
scaffolds da W2 do Sprite Inspector com **zero commits da jornada**; a do
`write_mobile_to_disk` diz no próprio doc *"it is a probe, not a gate"* e quer `PROBE_OUT`).

**Números no `main` de hoje** (a fonte é o código, não esta linha):
`PROJECT_SCHEMA` **48** · `FLIP_SCHEMA` **13** · `VEC_SCENE` **13** · `DOC_VERSION` **17** ·
**ADR max 0152** (nenhum ADR novo nesta jornada — pela primeira vez em quatro, zero disputa de
número) · registro `ph2d-ecs` **40** (espelhos **41**) · registro `ph2d-physics-ecs` **24** ·
gizmo ids **próximo livre 972** · `physics_ecs_c9` **`556cb652…`, 99 corpos**.

### ⚠️ O que a ÁRVORE COMBINADA pegou, e nenhuma linha podia ver sozinha

**(1) Um PÂNICO real no Wet Paint, escondido pelo `--release`.** O `live_span_cells` (o readout
da poça, doc 28 §5.47) fazia `hi - lo + 1` sobre o par SENTINELA de uma linha sem faixa viva
(`row_lo = i32::MAX`, `row_hi = i32::MIN`) ⇒ **overflow de `i32`**: panica em debug, embrulha em
silêncio em release. O `try_from(..).unwrap_or(0)` que o autor escreveu É a resposta certa para a
linha vazia — ele só nunca RECEBIA o número. A linha fechou em `--release` e ficou verde; é a
MESMA lição que o `ph2d-flip-colorize` pagou em 21/07. **Rode as duas.** Corrigido em `i64`
(byte-idêntico em linha não-vazia), com gate mutação-provado verde→RED→verde.

**(2) O schema que quase EVAPOROU em silêncio.** `line/physics` e `line/FLIP` escreveram **ambas
47** contra o `main` de 30/07, a **TERCEIRA** colisão entre estas duas linhas (30 em 25/07,
32/33/34 em 27/07). ⚠️ **E o `project.rs` NÃO conflitou** — o literal era o mesmo dos dois lados,
e o git não tem opinião sobre o que o número SIGNIFICA: o bump da segunda teria sumido com a
suíte inteira verde. Quem denunciou foi o conflito do `project_schema_tests.rs` **ao lado**.
Contado para **48**, tripla `(48, 13, 13)`.

**(3) Um LOC vermelho-latente que não era conflito.** `motion_bridge_edit_tests.rs` a **615** nas
DUAS árvores (medido antes e depois do rebase): os gates de `shells/desktop/tests/` só correm na
varredura impactada, e um fechamento por `cargo test -p` por crate não os alcança — 4ª jornada
seguida com esta família.

### ⚠️ E o `ship.sh` drenou uma CASCATA cuja causa era ele mesmo

Nenhum dos `✗` era código de linha:

1. **fmt** (12 arquivos) — nenhum estava sujo na própria árvore. O `rustfmt` decide a quebra pelo
   **contexto ao redor**, o rebase mudou o contexto, e o `foundational-integrate.sh` **não roda
   fmt** ⇒ a dívida só aparece no ship. ⚠️ **A edição importa:** `--edition 2021` faz o rustfmt
   **RECUSAR o arquivo inteiro** (*"let chains are only allowed in Rust 2024"*) sem escrever nada.
2. **typos** (5) — e o critério da própria `.typos.toml` (*"allowlistar isto pode esconder um typo
   real?"*) decidiu **diferente para cada palavra**: o pt-BR de *"está a ponto de"* foi REESCRITO
   porque **123 arquivos `.rs` dizem `presets`** e a allowlist cegaria o gate para todos eles; o
   plural errado de `vertex` num nome de teste era typo inglês DE VERDADE; `FULLs` virou regex (o caso
   `PNGs` que a config já explica: a tokenização erra, não o texto); só `instale` entrou na
   allowlist.
3. **`paint()` 207 > 200** — ⚠️ **causado pelo fix de fmt do passo 1**: uma chamada de 7 argumentos
   virou 9 linhas. Cortado em `paint_wires.rs`.
4. **`subgraph_tests` 601 > 600** — o 4º arquivo do mesmo fmt. Cortado pela seção que o **próprio
   arquivo já nomeava** num banner.
5. **`Disk quota exceeded`** — AMBIENTE, não código: a tmpfs do `target/` a 80%, com **27 G de
   `debug/`** acumulados pelas chamadas `cargo test -p` do integrador (o ship usa `ci-test`).


## Estado da orquestração

**Sem sessão multi-agente ativa.** Nenhum slot de implementador aberto; sem posse
reservada. Próximo trabalho parte de CLAUDE.md §5 (planos ativos) + §1 (roteador por tarefa).

> Ao abrir uma sessão: registre aqui um **MAPA DE POSSE** (agente → pasta(s) que vai
> escrever, zero overlap) antes do primeiro burst, e o limite de RAM (≤3 cargos
> simultâneos). Ao fechar: mova o que concluiu para o git log e volte este arquivo
> ao estado idle acima.
