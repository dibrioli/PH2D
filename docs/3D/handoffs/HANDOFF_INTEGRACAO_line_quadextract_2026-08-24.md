# HANDOFF DE INTEGRAÇÃO — `line/quadextract` (2026-08-24)

> A **extracção de malha quad a partir de um mapa de grade inteira** (obra 2) e o
> **arredondamento inteiro** que a alimenta (obra 1), construídos em clean-room a
> partir da espec funcional auditada em [`docs/3D/cleanroom/`](../cleanroom/) mais a
> literatura pública. ⛔ **O fonte de implementação nenhuma entrou na janela que
> escreveu este código** — o protocolo, os atestados e a proveniência vivem naquela
> pasta, e não neste documento.

## 1 — Identidade

| | |
|---|---|
| branch | `line/quadextract` |
| HEAD | ⚠️ **`git rev-parse line/quadextract`** — o último commit é o dos docs, e um sha escrito dentro dele nunca poderia ser o dele próprio |
| último commit | `docs(quadextract): o handoff de integracao, a linha do §5, e o handoff da CORRENTE (R-pos)` |
| ⛔⛔ **base do fork** | **`line/sculpt3d`**, e **NÃO `main`** |
| commits desta linha | **5** |
| merge-base com `main` | `5038249c6` (com os 15 commits da `line/sculpt3d` pelo meio) |

⛔⛔ **A ORDEM DE INTEGRAÇÃO DEIXA DE SER LIVRE.** Esta linha foi aberta sobre
`line/sculpt3d` de propósito: a espec que ela implementa (`docs/3D/cleanroom/`) e os
fixtures verificados **só existem naquela branch**. ⇒ **`line/sculpt3d` entra ANTES,
ou as duas entram juntas.** Rebasear esta linha em `main` antes disso arrastaria os
commits da outra para dentro desta e o integrador veria a mesma obra duas vezes.

## 2 — Foundational / compartilhado tocado, e por quê

| arquivo | o que mudou | por quê |
|---|---|---|
| `crates/ph2d-quadextract/**` | **crate NOVA** (10 módulos, 3 ficheiros de gate, 2 exemplos) | a obra 2. Drop-crate: `members = ["crates/*"]` é glob, zero edit central |
| `crates/ph2d-gridmap/src/round.rs` + `round_tests.rs` | **ficheiros NOVOS** | a obra 1 (G5). ⭐ Módulo **irmão**, não engorda `solve.rs` |
| `crates/ph2d-gridmap/src/corners.rs` | **ficheiro NOVO** | a ponte para a extracção, em **dados simples** — zero aresta nova no grafo de crates |
| `crates/ph2d-gridmap/src/solve.rs` | `Assembly`/`assemble` e `measure` **saem de `run`** para `pub(crate)`; `Tri`/`Partner`/`prepare` viram `pub(crate)` | ⚠️ **É a única edição a código pré-existente desta linha.** O relaxador do arredondamento resolve o **mesmo** sistema; montá-lo por conta própria seria a mesma lei escrita duas vezes. `run` **encolheu**; o ficheiro foi de 566 para 621 LOC (teto 700) |
| `crates/ph2d-gridmap/src/lib.rs` | 2 `pub mod` + 2 `pub use` | aditivo |
| `shells/desktop/src/sculpt3d_history_retopo_extract.rs` | **ficheiro NOVO** (211 LOC) | o caminho novo do botão, **desligado** |
| `shells/desktop/src/sculpt3d_history_retopo_global.rs` | **+7 linhas**: uma bifurcação no topo de `quad_remesh_global` | ⛔ **É o único sítio**, e há gate a contá-lo |
| `shells/desktop/src/sculpt3d_history.rs` | +1 `mod` irmão | aditivo |
| `shells/desktop/src/sculpt3d_remesh_refusal.rs` | +1 variante `Extract` + o braço de `explain` + o de `keeps_the_piece` | ⚠️ `match` exaustivo: a variante nova **obriga** os dois braços |
| `shells/desktop/Cargo.toml` | +2 deps internas (`ph2d-gridmap`, `ph2d-quadextract`) | ⚠️ o `Cargo.lock` só ganha **arestas internas**; nenhum pacote externo novo |

## 3 — Símbolos que podem COLIDIR

Saída de `bash scripts/collision-surface.sh`, corrida **de dentro desta worktree**
em 2026-08-24 (⚠️ **é REFERÊNCIA, não evidência** — re-rode-a antes de fundir):

```
SUPERFÍCIE DE COLISÃO — line/quadextract contra main
  merge-base 5038249c6   ·   19 commit(s)   ·   55 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA  95 (base: 95) · tripla (95,13,14) (base: (95,13,14))
    VEC_SCENE_SCHEMA 14 (base: 14) · FLIP_SCHEMA 13 (base: 13) · DOC_VERSION 18 (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs 69 (base: 69) · ph2d-render 70 (base: 70) · ph2d-script 70 (base: 70)
▸ CONTRATO CONGELADO (§6)
    ph2d-nodegraph/src/node.rs      intocado
    ph2d-editor-core/src/tool.rs    intocado
▸ ADR — último no disco: 0164   próximo livre: 0165
  ⚠ esta linha cria ADR: 0164   — reconte contra o main do dia
▸ Cargo.lock — 1 pacote '+name' novo: "ph2d-quadextract"   (aresta INTERNA)
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⛔⛔ **COLISÃO DE NÚMERO DE ADR, e ela passa MUDA se ninguém a contar.** Medido em
2026-08-24:

| onde | ficheiro |
|---|---|
| nesta linha (herdado da `line/sculpt3d`) | `0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md` |
| ⛔ **na árvore primária, NÃO versionado** | `0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md` |
| ⛔ **idem** | `0165-assets-are-born-inside-the-app-three-level-identity-index-before-browser.md` |

⇒ **duas linhas escreveram o mesmo literal `0164`.** ⚠️ **Não é desta linha** (o ADR
veio da `line/sculpt3d`), e ⛔ **eu não a renegociei com ninguém** — o §5.0 manda
contar, não escolher. Quem integrar decide qual fica com `0164` e qual sobe para
`0166`, **e o `README.md` do índice é derivado** (`bash scripts/adr-index.sh`).

**Ids/consts/variants novos, com os valores literais:**

| símbolo | valor | onde |
|---|---|---|
| `RemeshRefusal::Extract` | variante nova, **no fim** do enum | `sculpt3d_remesh_refusal.rs` |
| `PH2D_RETOPO_EXTRACT` | env nova (`"0"` desliga) | `sculpt3d_history_retopo_extract.rs` |
| `ph2d_gridmap::round::LOCAL_TOL` | `1.0e-2` | medido, ver §6 |
| `ph2d_gridmap::round::LOCAL_CAP` | `20_000` | medido |
| `ph2d_gridmap::round::SWEEPS` | `200` | orçamento do degrau 2 |
| `ph2d_quadextract::exact::COORD_MAX` | `1 << 52` | **derivado**, não escolhido |
| `ph2d_quadextract::exact::Q_HEADROOM` | `11` | derivado |
| `walk::MAX_STEPS` | `256` | tecto de sanidade |
| `cells::MAX_SIDES` | `64` | ⚠️ **medido** — ver §6 |

## 4 — Contratos congelados encostados

**NENHUM.** Nem `NodeOp`/`OpResolver`/`NodeManifest`, nem
`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`. O `collision-surface.sh`
confirma os dois ficheiros **intocados**.

## 5 — O que só o `ship.sh` pega (já corrido nesta worktree)

| verificação | resultado |
|---|---|
| `cargo fmt --all --check` | ✓ limpo |
| `cargo clippy --all-targets -p ph2d-quadextract -p ph2d-gridmap -p ph2d-host-desktop` | ✓ **0 avisos** |
| `cargo machete` nas duas crates | ✓ nenhuma dependência por usar |
| `typos` sobre os ficheiros novos | ✓ limpo |
| deps externas novas | ⭐ **nenhuma** — `miniz_oxide 0.8` entra só como **dev**-dependency, e já era dependência DIRECTA de duas crates da casa |
| `RUSTSEC` | nada novo a auditar (zero pacotes externos novos) |

## 6 — O gate batched, e os ✗ que NÃO são meus

```
CARGO_INCREMENTAL=0 cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast
  ⇒ 18 485 testes correram · 18 483 passaram · 2 ✗
```

⚠️ **Duas corridas, conjuntos DIFERENTES de ✗** — que é a assinatura da família de
flakes de recurso do `CLAUDE.md` §5.0, não a de um defeito de lógica:

| corrida | ✗ | crate | verde sozinho? | o meu diff toca? |
|---|---|---|---|---|
| 1ª (fail-fast) | `flip_smooth::…::orcamento::the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` | `shells/desktop` | ✓ | ⛔ não |
| 2ª | `ph2d-mesh::measure_normals::measure_normals_parallel_speedup` | `ph2d-mesh` | ✓ | ⛔ não |
| 2ª | `ph2d-node-motion-soft-body::cap_gates::the_shape_match_is_linear_in_the_mesh` | `ph2d-node-motion-soft-body` | ✓ | ⛔ não |

⭐ **A família ganha DOIS NOMES NOVOS** (as duas últimas): as duas medem uma **razão
de dois relógios** sob um fan-out de 18 mil testes. O §5.0 diz que a lista nunca
estará completa; estas duas são para lá.

**Gates novos: 15 na `ph2d-quadextract` + 3 na `ph2d-gridmap` + 2 na shell.**

## 7 — O que SMOKAR, e o que ainda não foi smokado

⛔ **Nada disto foi visto pelo Enio.** O caminho novo do botão shipa **desligado**.

```
\
  env PH2D_RETOPO_EXTRACT=1 cargo run -p ph2d-host-desktop --release
```

⇒ abrir o módulo 3D, esculpir qualquer coisa, e carregar em **Quad Retopology**.
Sem a variável, o botão faz o que sempre fez.

Instrumento sem GPU, que é onde os números deste handoff foram medidos:

```
cargo run --release -p ph2d-quadextract --example chain_info -- esfera-fina
cargo run --release -p ph2d-quadextract --example extract_info -- <ficheiro.mapa>
```

## 8 — ⭐⭐⭐ O QUE A MEDIÇÃO DISSE, e é a parte que importa

**A obra 2 fecha nos dois mapas de referência verificados** (`docs/3D/cleanroom/fixtures/`):

| peça | quads | `χ` saída / entrada | bordo | não-manifold | ordem das saídas |
|---|---|---|---|---|---|
| toro (**género 1**) | `2 250` · **100 %** | `0` / `0` ✓ | `0` | `0` | limpa `100 %` |
| gancho (fechado) | `1 637` · **100 %** | `2` / `2` ✓ | `0` | `0` | limpa `100 %` |
| **bordo** (derivado, ⛔ sem oráculo) | `1 624` · **100 %** | `1` ✓ | tem | `0` | limpa `100 %` |

⛔⛔ **E a cadeia da CASA diz outra coisa, que é o achado desta linha.** Com a fase
zero honrada, ponta a ponta (F1 → F2 → F3 → G1–G5 → extracção):

| peça | dobras do mapa | quads | `χ` | aspecto p50 | ⭐ enviesamento p50 |
|---|---|---|---|---|---|
| ⭐ esfera fina | **`0 %`** | `2 102` | ⚠️ `−5` | ⭐ **`1,10`** | ⭐ **`6,8°`** |
| toro | `3,3 %` | `1 495` | ⛔ `−20` | `1,29` | `5,8°` |
| esfera lisa | ⛔ `11 %` | `410` | ⛔ `−14` | `2,02` | `22,1°` |
| — a barra do **oráculo** — | | | | `1,08`–`1,22` | `4,8°`–`7,1°` |

⭐⭐ **A FORMA da esfera fina está DENTRO da barra do oráculo.** ⛔ **O que falta é a
topologia, e a causa está medida e é a MONTANTE desta linha:**

| grandeza | mapas de referência | ⛔ o nosso G3 |
|---|---|---|
| triângulos dobrados no domínio | `0,02 %` · `0,2 %` | **`0 %` a `11 %`**, conforme a peça |
| resíduo da translação de transição | `3,5e-15` | p50 `0` · **máx `0,50`** (uma célula a meio caminho entre dois inteiros) |
| desvio de alinhamento p50 | — | `0,095` (esfera fina) a **`0,53`** (esfera lisa) |

⇒ *a extracção e o arredondamento não são o bloqueador; o solver contínuo é.* A
espec já nomeia a cura numa linha do §5.1 — **«restrições lineares entram eliminando
uma variável por restrição independente, não como equações extra no sistema»** — e o
nosso G3 penaliza a costura em vez de a eliminar (`SEAM_WEIGHT`). **Isto é uma
pergunta para o E emendar a espec, não uma coisa que eu tenha ido corrigir.**

### ⛔⛔ §8-bis — CORREÇÃO DO R-PÓS (2026-08-24): a CURA acima está certa, a CAUSA não

⚠️ **A conclusão de que é preciso ELIMINAR a costura mantém-se — e passa de melhoria a
NECESSIDADE.** O que está errado é a razão que o §8 dá para ela, e a diferença decide se
alguém vai perder tempo a afinar o que está a montante.

**O instrumento** (a mesma cadeia do botão, sem GPU), corrido em 2026-08-24:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract
cargo run --release -p ph2d-quadextract --example chain_info -- esfera-fina
```

| peça | ⭐ dobras do mapa | translação a inteiro | ⛔ **resíduo de COSTURA** (antes → depois) | passo pior do guloso | órfãs | bordo | `χ` |
|---|---|---|---|---|---|---|---|
| ⭐ **esfera fina** | ⭐ **`0,0 %`** | `0` (exacto) | `0,2348` → ⛔ **`1,0000`** | `0,4955` | `5` | `32` | `−5` |
| toro | `3,3 %` | `0` (exacto) | `0,7615` → ⛔ **`1,4142`** | `0,4994` | `75` | `152` | `−20` |
| esfera lisa | `11,1 %` | `0` (exacto) | `0,9054` → ⛔ **`1,0849`** | `0,4913` | `101` | `133` | `−12` |

⛔⛔ **A esfera fina refuta a atribuição do §8.** Ela é a peça cuja FORMA já entra na barra
do oráculo (`6,8°`), o mapa dela **não tem uma única dobra** e as translações dela são
**exactamente** inteiras — e a extracção ainda assim deixa `32` arestas de bordo numa
esfera. ⇒ *«a causa é o G3 entregar até 11 % de dobras» é verdade no toro e na esfera
lisa, e FALSA exactamente onde o resto já está bom.* Nenhuma redução de dobras a montante
fecha esta peça, porque ela já tem zero.

#### ⭐⭐⭐ As DUAS grandezas que estavam a ser lidas como uma

| grandeza | o que mede | depois do G5 |
|---|---|---|
| `RoundReport::shift_frac_max` | quão longe de um inteiro está a **translação** de cada costura | ⭐ **`0`, nas três peças** |
| `SolveReport::seam_max` | quão longe a transição `R(k)·x + t` está de **fazer os dois lados coincidirem**, em células (`1` = uma célula inteira de desacordo) | ⛔ **`1,00` a `1,41`, nas três peças** |

⇒ ⭐⭐ **O G5 torna a costura INTEIRA; ele não a torna FECHADA.** São propriedades
diferentes, e só a segunda é a que a extracção precisa — a primeira é necessária e **não é
suficiente**. Os mapas de referência de `fixtures/` fecham as duas (resíduo `3,5e-15`), e é
por isso que a extracção fecha sobre eles e não sobre a cadeia da casa.

⛔ **E o gate mede a necessária.** `as_translacoes_ficam_todas_inteiras` cobra
`shift_frac_max == 0` — está **verde e correcto** —, e **não existe gate nenhum sobre
`seam_max`**. *Um gate que mede a condição necessária, num sítio onde se lê a suficiente,
fica verde sobre um mapa que não serve.*

#### O mecanismo, e é ele que torna a eliminação obrigatória

O guloso é forçado a pregar variáveis a **`0,49` de célula** (a coluna «passo pior», nas
três peças). Meia célula tem de ir para algum lado. ⛔ Com a costura a ser uma
**penalização** (`SEAM_WEIGHT = 8`), o sistema paga um pouco de energia e **abre a
costura** em vez de o interior absorver o deslocamento — e é literalmente isso que se vê na
esfera fina: `0,2348 → 1,0000`, ⚠️ **o arredondamento TRANSFORMOU um desacordo de um quarto
de célula num desacordo de uma célula inteira.**

⭐ Com a variável **eliminada** (a linha do §5.1), a costura não tem como abrir: ela deixa
de ser um termo do sistema, e o deslocamento só pode ser absorvido pelo interior.

⚠️ **A tabela do [`SEAM_WEIGHT`](../../../crates/ph2d-gridmap/src/solve.rs) já dizia que a
penalização não pode ganhar as duas:** `8` dá `2,9°` de ângulo com costura `0,23`; `512`
fecha a costura para `0,01` e paga **`16,8°`** de enviesamento. *Não há peso que feche a
costura e mantenha o quad — que é a assinatura de uma restrição a fingir-se de termo de
energia.*

#### ⭐⭐⭐ E o rasgo é INVARIANTE aos dois botões da escada — logo não é afinação

A sonda que já existia (`the_rounding_ladder_sweeps_its_two_constants`, `--ignored`) varre
tolerância × tecto. ⚠️ **Ela foi escrita para medir a fracção que fica no degrau barato — e
a coluna que interessa aqui é a do lado**, que ninguém tinha lido:

| tolerância × tecto | degrau 1 | visitas | ⛔ **costura max** |
|---|---|---|---|
| `1e-2` × `2 000` … `200 000` | `98,6 %` … `100 %` | `18 282` … `18 588` | `1,0834` · `1,0849` · `1,0849` |
| `1e-3` × `2 000` … `200 000` | `42,9 %` … `100 %` | `112 585` … `481 365` | `1,0369` · `1,0371` · `1,0369` |
| `1e-4` × `2 000` … `200 000` | ⛔ `1,4 %` … `91,4 %` | `139 908` … ⛔ `6 464 031` | `1,0897` · `1,0370` · `1,0369` |

⇒ ⭐ **A fracção do degrau barato varia de `1,4 %` a `100 %`, as visitas variam 350×, e a
costura fica onde estava: uma célula inteira.** *Um defeito que não se move quando os dois
botões do subsistema varrem toda a gama não é afinação daquele subsistema* — e é a
demonstração de que a cura tem de mudar a FORMA do sistema, não os seus números.

#### ⇒ O que fica prescrito (em termos funcionais — [SKILL_Cleanroom §7.3.d](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md))

1. **O G3 tem de eliminar a variável de costura**, não pesá-la — espec §5.1, e agora com o
   número que o torna obrigatório em vez de preferível.
2. ⭐ **A barra do G5 tem de passar a ser a condição SUFICIENTE:** um gate sobre
   `SolveReport::seam_max` ao lado do que já existe sobre `shift_frac_max`, com a barra
   tirada dos mapas de referência (`3,5e-15`), não do que a cadeia dá hoje.
3. ⛔ **NÃO afinar o `SEAM_WEIGHT`** — está medido que não fecha (a tabela acima), e a
   recusa fica registada aqui para não ser repetida.

⛔⛔ **Quem escreve isto NÃO pode ser a janela R-pós** (`49c94a84-…`): ela leu o laço de
arredondamento da implementação de referência para fazer a revisão estrutural do
[`LEDGER §R-pós.3`](../cleanroom/LEDGER_quadwild.md), e o §5 é exactamente a região que
ela viu. *Escrever a eliminação com aquele laço em contexto converteria em silêncio a rota
do ADR-0164 na rota que ele rejeitou.* ⇒ **janela I nova, que retoma da espec.**

## 9 — Divergências deliberadas da espec (para o R-pós)

1. ⭐ **Sem `num-bigint`/`num-rational`.** O §1 exige um *predicado de orientação
   exacto* e **sugere** a rota bigint+filtro. Entrego o requisito **sem dependência
   nenhuma**: a truncagem do §2.3 numa grade **global** (em vez de por-vértice) faz
   todo o domínio saneado caber em `i64` sem perder um bit, e a orientação vira um
   determinante `i128`, exacto por construção. Os limites são **identidades**
   (`|x| < 2^52`, produtos `< 2^104`), não medições, e a conversão **recusa** um
   mapa que os viole. A grade global é sempre ≥ a que cada vértice exigiria ⇒ ela
   satisfaz o §2.3 de **todos** os vértices ao mesmo tempo.
2. ⚠️ **O gate nº5 muda de forma sem baixar de barra.** Sem filtro não há nada que
   «desista»; o gate passa a provar que o predicado **acerta onde uma avaliação em
   `f64` erra**, com uma construção de cancelamento (`(2^k+1)(2^k−1) − 2^k·2^k = −1`).
3. ⏳ **O degrau 3 da escada (factorização esparsa directa) NÃO foi construído**, e a
   coluna que o justifica é executável: `RoundReport::level2` é **`0`** em todas as
   peças medidas, e há gate a exigi-lo. Construir o degrau caro antes de o barato
   falhar seria construir o que nenhuma medição pede — e o gate acorda no dia em que
   deixar de ser verdade.
4. ⚠️ **O §6.4 (recuperar a aresta em falta) é DETECTADO e contado, não reparado.**
   `ExtractReport::collapsed_fans` é `0` nas duas peças de referência ⇒ nenhuma
   fixtura contém o fenómeno, e a reparação insere uma aresta que nenhum traço
   produziu. *Construir a cura sem uma fixtura que a contenha seria construir código
   que nada mede.*
5. ⚠️ **O gate nº8 (forma por-face) não é medível nos fixtures**, e o teste diz-o
   antes de dizer qualquer número: os dois mapas de referência **não** estão
   remalhados isotropicamente (aspecto de entrada p99 `2,17` e `11,56`, contra a
   assinatura do nosso F1, `1,58`). A barra ali é a da **classe de entrada**, com um
   controlo que reprova se a entrada mudar; a barra do oráculo é medida na cadeia da
   casa (§8).

## 10 — Três defeitos meus, apanhados por medição

Ficam registados porque o **mecanismo** de cada um é reutilizável:

1. ⛔ **Um tecto que eu escolhi apagava células.** `MAX_SIDES = 8` parecia generoso
   (uma célula tem 4 lados); perto de uma dobra as cartas **sobrepõem-se** e a órbita
   fecha com 6, 8 ou 10 lados. Cortar não a torna num quad — **apaga-a**, e com ela as
   saídas que ela já tinha consumido. Era `χ = 5` e 8 arestas de bordo no gancho.
   *Medir a distribuição antes de escolher o tecto* (`ExtractReport::ring_len`).
2. ⛔ **Uma sonda que acusava o caso normal.** A do §6.4 comparava também o par que
   **dá a volta** ao leque — e é aí que a holonomia entra: num nó de valência 5 as
   direcções de referência são `0,1,2,3,0`, e o último contra o primeiro leem iguais
   **por ser uma singularidade**. Lia `5` em cada peça e parecia um achado.
3. ⛔ **Uma congruência com o sinal trocado.** Escrevi `v ≡ 4 − r` para a relação
   valência↔holonomia; a **medição** dá `v ≡ r`. Não dá erro de compilação nem de
   tipo — dá um buraco na malha (os cinco vértices de valência 5 do toro viraram `1`,
   e o gancho perdeu a casca). ⚠️ E o guloso do §5 tinha o irmão disso: a soma dos
   passos saía **idêntica nas nove configurações** da varredura, porque ele lia os
   valores congelados do calibre inicial — *o lote disfarçado*.

## 11 — `CLAUDE.md` §5

Uma linha, na entrada **3D / Sculpt**. A narrativa é este ficheiro.
