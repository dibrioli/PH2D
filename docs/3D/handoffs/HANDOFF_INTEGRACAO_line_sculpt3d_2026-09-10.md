# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, fecho de 2026-09-10

> ⚠️ **A linha NÃO integrou e NÃO pushou** (`CLAUDE.md` §0.7). Ela fecha, entrega
> isto e para. A ordem de integrar é do Enio e foi dada em 2026-09-10; quem a
> executa é um **agente integrador dedicado** munido deste documento
> (DIRETRIZ §1.5.3–1.5.4).

---

## §1 — Identidade

| | |
|---|---|
| ramo | `line/sculpt3d` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d` |
| merge-base | `39d48cd76` |
| commits | **63** |
| ficheiros | **274** |
| HEAD | `45dd59ff8` |
| smokes aprovados pelo dono | `=37` (filtro de tecido, passos 12 e 13) · `=39` (as duas peças vizinhas) |

### O que a linha entrega, em cinco blocos

1. **O FILTRO DE TECIDO** — o solver do pano na peça INTEIRA, com os oito
   controlos que o alvo oferece, bancada de oráculo própria e a cena `=37`.
2. **AS QUATRO VIEWPORTS, o gizmo de navegação e o gizmo de TRANSFORMAÇÃO** —
   `ph2d-mesh-render` passa a desenhar num sub-rectângulo do alvo, e a cena
   ganha vistas nomeadas com botão.
3. **O PANO DEIXA DE SER UM ELÁSTICO** — tecto de esticão, âncoras de longo
   alcance, volume, dobra, e o material a atravessar gestos.
4. **A DÍVIDA DE CITAÇÕES A ZERO** — `351 → 0`, com o instrumento que faltava.
5. **O `Connected Only`** — o carimbo deixa de saltar para o que a superfície
   não liga, com as duas opções no painel e a curada por omissão.

⚠️ **Os blocos 1–3 são anteriores à janela que escreve este handoff.** O que está
aqui sobre eles sai do log de commits e dos docs que eles próprios escreveram
(`docs/3D/cloth/`, `docs/3D/cleanroom/`), **não** de medição refeita nesta
janela — e isso está dito onde importa. Os blocos 4 e 5 foram medidos aqui, e os
números deles são desta árvore.

---

## §2 — Foundational / partilhado tocado, e por quê é ADITIVO

| crate | o que entrou | por que é aditivo |
|---|---|---|
| **`ph2d-mesh-render`** | `ScreenRect` — a malha desenha num sub-rectângulo do alvo em vez do alvo inteiro (`2ea127ef9`) | o caminho de um viewport só passa o rectângulo cheio; ⚠️ **e `d0e0a93e6` é a cura de um defeito que só as 4 vistas revelam** — era **UM** uniform para quatro passes, então as quatro desenhavam a MESMA câmera |
| **`ph2d-editor-core`** | 4 ficheiros: `ids/chrome/sculpt3d.rs` (ids novos + a cura de citações), **`ids/chrome/sculpt3d_cloth.rs` NOVO** (147 linhas), `ids/chrome/flip.rs` (1 linha de comentário), **`tests/architecture_no_restricted_source_citations.rs` NOVO** (575 linhas) | ids são `hash_node_id` e **append-only** ⇒ nenhum contador de contagem; os dois ficheiros novos não existem no `main` ⇒ fusão sem hunk |
| **`ph2d-i18n`** | **14 rótulos, todos APENDADOS** (13 do filtro de tecido + `"Connected Only"`) | tabela de tradução é append-only por construção |
| **`ph2d-cloth`** | as leis do filtro (`5b1a140bf`), tecto/âncoras/volume/dobra (`ab6140fb1`), material entre gestos (`1d276083b`) | crate-folha desta linha; nenhum outro consumidor no `main` |

### ⭐⭐ As CINCO crates de OUTRAS linhas mudam **zero linhas fora de comentário**

Medido, não afirmado (`git diff main..HEAD -- <crate>`, contando `+/-` e
subtraindo as que abrem com `//`):

| crate | linhas `+/-` | fora de comentário |
|---|---:|---:|
| `ph2d-painter-brush` | `113` | **`0`** |
| `ph2d-flip` | `40` | **`0`** |
| `ph2d-flip-reshape` | `45` | **`0`** |
| `ph2d-flip-render` | `22` | **`0`** |
| `ph2d-tool-flip` | `2` | **`0`** |
| `ph2d-panel-painter-layers` | `2` | `2`, e as duas são **a MESMA linha de código** com o comentário de fim de linha reescrito — o código é byte-idêntico |

⇒ **risco de fusão semântica nessas seis: nenhum.** Elas foram tocadas apenas
pela cura de citações (§4.2 da `SKILL_Cleanroom`), que re-diz o FACTO em
vocabulário do domínio e não altera uma expressão de código. ⚠️ **Um conflito
textual ali é resolúvel a favor de qualquer lado sem perder comportamento** — mas
⛔ perde-se a cura, e a catraca do gate novo volta a acusar. *Prefira o lado desta
linha nos comentários e o lado do `main` no código.*

---

## §3 — Superfície de colisão (`collision-surface.sh`, colada)

Corrida com o caminho ABSOLUTO do primário, como a DIRETRIZ manda:

```
SUPERFÍCIE DE COLISÃO — line/sculpt3d contra main
  merge-base 39d48cd76   ·   63 commit(s)   ·   274 arquivo(s)

▸ SCHEMAS
    PROJECT_SCHEMA                        123   (base: 123)
      └ tripla do gate               (123, 13, 22)   (base: (123, 13, 22))
    VEC_SCENE_SCHEMA                        —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)

▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)                  80   (base: 80)
    ph2d-script (espelho)                  80   (base: 80)

▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — último no disco: 0169   próximo livre: 0170
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — nenhum '+name' novo
▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⭐⭐⭐ **É o perfil de integração mais barato que esta linha já teve: nenhum contador
partilhado se move.** ⚠️ E isso é uma AFIRMAÇÃO SOBRE O DIA — o valor de um
schema conta-se contra o `main` no momento de integrar, nunca se copia de um doc
(`CLAUDE.md` §5.0). Se outra linha subir o `PROJECT_SCHEMA` antes desta entrar,
**esta linha continua a não ter opinião sobre ele** e a fusão fica muda e
correcta.

### ⚠️ O `Brush` NÃO é serializado, e é isso que dispensa o schema

O bloco 5 acrescenta `Brush::surface_only`. O `Brush` deriva
`Clone, Debug, PartialEq` e **não** `Serialize` ⇒ zero `PROJECT_SCHEMA`, zero
migração, zero degrau. *A prova é o `derive`, não uma promessa.*

---

## §4 — Contratos congelados encostados

**Nenhum.** `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4`
intocados; `NodeOp` / `OpResolver` / `NodeManifest` intocados; a superfície do
`ph2d-vector-doc` intocada. ⇒ **nenhum ADR devido por esta linha.**

⚠️ A navegação orbital, as quatro viewports e os dois gizmos moram no **SHELL**,
nunca numa `Tool` — é isso que mantém o `Tool=12` fora do caminho, e é a lei que
o §5 do `CLAUDE.md` já escreve para este módulo.

---

## §5 — O que só o `ship.sh` pega (o gate desta árvore NÃO roda)

1. **A matriz de 3 OS e o `physics_ecs_c9`** — o `spike.yml` compara os três
   sistemas ENTRE SI, então o risco real é eles discordarem, e só o CI o mede.
   Esta linha não toca física; o risco é o de sempre.
2. **`cargo machete` · `cargo deny` · `cargo audit`** — nenhum corre aqui.
   ⭐ **O `typos` FOI CORRIDO nesta árvore e devolve `exit 0`.** Eu nomeei-o como o
   candidato real e depois medi-o em vez de o deixar para a fusão — e ele **tinha
   um vermelho**, desta linha: `catch-alls` em
   `shells/desktop/src/sculpt3d/keys_view_tests.rs` (ele quer `all` ou `falls`).
   ⚠️ **E a 1.ª cura falhou duas vezes de maneira instrutiva:** a nota que
   explicava a cura **continha a própria palavra** (*um comentário sobre um lint
   passa pelo lint*), e a 2.ª redacção trouxe `portugues` sem acento, que ele
   também marca. A cura final é a grafia correcta, `português` — ⛔ e **não** uma
   entrada nova no `.typos.toml`, que seria dívida onde bastava um acento.
3. **`cargo nextest --workspace --cargo-profile ci-test`** — a varredura desta
   árvore é a **impactada** (`nextest-impacted.sh`), que corre as crates do diff
   e os dependentes reversos. ⛔ Ela é mais estreita que a do `ship`.
4. **Os gates de GPU** são `#[ignore]` e precisam de adapter; *skip gracioso não
   é verde*.

---

## §6 — Ordem, dependências e o que smokar

### Ordem

Esta linha **não tem dependência de ordem** contra nenhuma outra: ela não move
schema, não cria ADR, não toca contrato e não acrescenta pacote externo. ⇒ pode
entrar em qualquer posição da rodada.

⚠️ **A única adjacência a vigiar é textual**, e está no §2: as seis crates de
outras linhas em que ela mudou **só comentários**.

### Os smokes, com o comando exacto

**(a) O FILTRO DE TECIDO — aprovado pelo dono em 2026-09-10:**

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=37 cargo run -p ph2d-host-desktop --release
```

**(b) AS DUAS PEÇAS VIZINHAS — aprovado pelo dono em 2026-09-10:**

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=39 cargo run -p ph2d-host-desktop --release
```

**(c) AS QUATRO VIEWPORTS e os dois gizmos** — cena `=38`
(`sculpt3d/scenes_viewports.rs`):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=38 cargo run -p ph2d-host-desktop --release
```

⚠️ **O roteiro de cada uma é impresso pela PRÓPRIA cena** — ⛔ não o duplique
aqui, senão as duas cópias divergem e a que o artista lê é a que envelhece.

⚠️ **Rode uma vez SEM a env var** — é a metade que prova a inércia: sem ela o
`AppGfx.sculpt3d` é `None` e o frame 2D é byte-idêntico.

⚠️ **Preferência fora do repo:** um `reduced_motion=1` esquecido em
`~/.ph2d/prefs.txt` reprova smokes sobre produto correcto.

---

## §7 — ⚠️ OITO coisas que uma leitura rápida do diff entende ao contrário

1. **O `Connected Only` nasce LIGADO, e a justificação NÃO é a que eu tinha
   dado.** Eu argumentei *«numa peça convexa a máscara corta `0,00 %`, logo o
   caminho de omissão é byte-idêntico»* e um gate de arquitectura desmentiu-o (a
   orelha, `56,0 %` contra `20 %`). Ele nasce ligado por **decisão do dono**, e a
   cura do vermelho foi **PREGAR a fixtura**, não mexer no default.

2. **`eared_sphere()` prega `surface_only: false`, e isso NÃO é afrouxar o gate
   dela.** Aquela fixtura é **esculpida**, e o trabalho dela é reproduzir uma
   geometria ESPECÍFICA (a que o dono fotografou em 22/08, sobre a qual o
   `the_ear_does_not_ship_an_edge_across_the_piece` foi calibrado). Deixá-la
   seguir o default do pincel faria o gate medir **outra peça** a cada mudança de
   pincel, em silêncio — *e um gate cujo sujeito muda não afirma nada.*

3. **A máscara de alcance NÃO troca a distância do falloff.** O peso continua
   `falloff(|p − c| / R)` **ao bit**; ela só **corta da pegada** quem a
   superfície não liga. É isso que mantém toda a paridade com os oráculos intacta
   **por construção**, e que mantém o viés `√2` do passeio por arestas FORA de
   qualquer peso.

4. **O `Verb::Cloth` não oferece a caixa por RAZÃO ESTRUTURAL, não por escolha.**
   Ele **desvia antes do `dab_core`** (`stroke_symmetry.rs`: é dono da própria
   expansão de simetria, porque cada cópia tem a sua região) ⇒ a máscara nunca
   corre nele. Pintar a caixa ali seria um interruptor de coisa nenhuma. ⭐ E é a
   mesma razão que garante que os **86 traços do corpus do tecido não podem ser
   tocados** pela máscara: é roteamento, não uma cerca a lembrar.

5. **A catraca `POR_CLASSIFICAR` está VAZIA, e a lista vazia é o FUNDO, não um
   gate desligado.** Com ela vazia, **qualquer** citação nova a ficheiro-fonte
   externo reprova. Uma entrada nova exige a licença **lida no artefacto**.

6. **`ALVO_PERMISSIVO` tem `layout.py` e `metrics.py`, e eles são NOSSOS.** São a
   bancada do PH2D, que vive **fora da árvore do repo** (`ph2d-quadbench/`, ao
   lado do oráculo) ⇒ o discriminador «é da nossa árvore?» não os alcança. ⛔ Não
   os leia como isenção a alvo alheio.

7. **A resposta do filtro é CÚBICA no arrasto**, e dois sítios diziam
   «quadrática». A fórmula da espec §7 (`Σ_j (k−j+1)·S_j·Δt`) sempre esteve
   certa; a leitura ao lado dela supunha `S_j` **constante**, e ele é o arrasto
   ACUMULADO. ⭐ A mutação que congela o `S_j` lê exactamente `4,083×` — **o doc
   descrevia o mutante**.

8. **O `Expand` obedecer ao tecto NÃO mudou uma comparação de oráculo.** A cura
   entra no topo do `limitar_esticao`, que já sai cedo quando `estica_max` é `∞`
   ⇒ a paridade é byte-idêntica **por construção**, e as 113 comparações
   continuam iguais (`0c10b1831`).

---

## §8 — Premissas MINHAS que a medição derrubou

1. **«A cura é trocar a régua de distância do pincel.»** Falso e caro. A classe
   que decide é a **INALCANÇÁVEL**, e responder a *«a superfície liga isto?»* não
   precisa de distância nenhuma ⇒ uma **máscara**, muito mais barata e sem risco
   para a paridade.

2. **«O chão de ruído da régua é pequeno.»** Pus a barra da classe *LONGE* em
   `1,25` e o controlo mediu **`1,41×` = `√2`** — numa grelha de quads o passeio
   vai por dois lados onde a superfície atravessa a diagonal. Com a barra abaixo
   do viés, a coluna lia `22 %`–`62 %` numa esfera CONVEXA. *Uma barra abaixo do
   viés do próprio instrumento mede a grelha, não a peça.*

3. **«Duas pontas ligadas mostram o defeito.»** Não mostram num raio usável
   (`6,9 %` do carimbo). Para o defeito aparecer nelas, elas têm de ser tão finas
   que o carimbo toca `9`–`39` vértices — um alfinete, não um gesto.

4. **`SEMENTE_RECUO`: uma constante construída, medida e APAGADA.** A mutação
   `0,25 → 0,0` sobreviveu a tudo, e a razão é geométrica: o centro do dab está
   SOBRE a folha que o raio atingiu, e todo vértice da folha de lá é um da de cá
   mais a espessura ⇒ **a folha de cá ganha sempre**.

5. **«A dívida de citações são 145.»** Eram `103` depois de a triagem passar a
   ser pelo **artefacto citado**, e `0` depois de o detector aprender as suas
   **sete** formas cegas — quatro delas só visíveis depois de a população
   encolher, porque *uma dívida grande esconde os erros do instrumento no meio do
   trabalho legítimo*.

6. **«`ph2d-sculpt3d` está a zero citações.»** Estava a zero **para as formas que
   o detector conhecia**. A forma `ficheiro.cc::simbolo` — a mais específica de
   todas — media zero em **seis** sítios, um deles nessa crate.

7. **TRÊS leituras minhas de suíte estavam erradas por FERRAMENTA, não por
   defeito de teste:** um `awk` mal escrito somou `failed 1` sobre uma corrida
   limpa · um `cargo test --test <nome>` devolveu `exit 101` por **não achar o
   alvo** · um `--lib` sobre a shell (que **não tem** biblioteca) devolveu o
   mesmo `101`. ⇒ *o veredito é o EXIT CODE do comando, lido logo depois dele e
   nunca depois de um pipe* — e ⚠️ **`exit 0` com `0 passed` é a outra metade**:
   um filtro que não casa nada devolve verde sobre teste nenhum.

8. **E o meu esquema de backup para mutações usava o NOME-BASE do ficheiro.**
   `ph2d-panel-sculpt3d/src/paint/brush.rs` e `ph2d-sculpt3d/src/brush.rs` têm o
   mesmo ⇒ o segundo `cp` esmagou o backup do primeiro e a restauração **trocou
   os dois ficheiros**, deixando a árvore vermelha por um motivo sem relação com
   a mutação em curso. Recuperado pelo `git` (o ficheiro era rastreado e a
   alteração dele estava no meu próprio script). ⇒ **commitar ANTES de mutar**, e
   restaurar por `git checkout` sobre árvore limpa — nunca um `cp` com chave
   ambígua.

---

## §9 — ABERTO, com o número de cada um

| item | o número | onde ler |
|---|---|---|
| ⏳ **O caso BRANDO da máscara** — duas partes LIGADAS por parede fina | razão `superfície/ar = π/2 ≈ 1,57`, **menor** que o viés `√2` do passeio ⇒ este instrumento não a mede. Pede geodésica a sério (método do calor, ou MMP) | [`dab_alcance.rs`](../../../crates/ph2d-sculpt3d/src/dab_alcance.rs) |
| ⏳ **O tamanho da ruga** ainda segue a densidade da malha | `8`–`32` passagens levam a onda de `10` para `20`–`27` arestas numa malha fina, e **achatam** a peça numa grossa (`0,329 → 0,0016`). ⛔ *«mais varreduras»* é **recusa medida** (o arrasto vai de `0,071` a `0,709`) | [`cloth/11 §5`](../cloth/11_o_material_a_ruga_e_a_memoria.md) |
| ⏳ **UM traço do corpus do tecido fora da barra** | `plano_apertar_ponto_plano_local`, `2,04` da barra pelo `p95`. É o regime §5.2-ter em que o próprio alvo inverte a malha e a ORDEM decide — **decisão do dono**, e a saída (b) foi **REFUTADA** com número (`b0a1e615c`) | [`oraculo_do_pincel.rs`](../../../crates/ph2d-cloth/tests/it/oraculo_do_pincel.rs) |
| ⏳ **Nomes de SÍMBOLO internos** do alvo | §4.2 pela mesma linha da SKILL, população maior e mais subtil (parte é API pública, permitida pelo §4.1.13). **O gate não os mede** | [`architecture_no_restricted_source_citations.rs`](../../../crates/ph2d-editor-core/tests/it/architecture_no_restricted_source_citations.rs) |
| ⏳ **Os `docs/**` ficam fora do censo de citações** por construção | o `docs/Flip/02_referencia_*.md` declara-se *«a referência comentada»* | idem |
| ⏳ **Uma linha de doc DUPLICADA, pré-existente** | `crates/ph2d-flip-render/tests/it/gpu_render.rs:1298` tem a mesma frase duas vezes na mesma linha física (`149` colunas). Está no `main`, não é desta linha, e não a corrigi para não poluir o diff | — |

---

## §10 — Portão de fecho corrido NESTA árvore

Todos lidos pelo **exit code**, logo depois do comando:

| gate | resultado |
|---|---|
| `cargo fmt --all -- --check` | **`exit 0`** |
| `cargo clippy --all-targets -D warnings` (as 5 crates do diff + shell) | **`exit 0`** |
| `architecture_workspace_file_loc_cap` | **`exit 0`** |
| `architecture_widget_loc_cap` | **`exit 0`** |
| `architecture_panel_loc_cap` | **`exit 0`** |
| `arch_safe_clamp_only` | **`exit 0`** |
| `shells/desktop/tests/it/file_loc_caps.rs` | **`exit 0`** (2 passed) |
| `BASE=main scripts/nextest-impacted.sh` | **`exit 0`** — **14 720 de 14 720 passaram**, 1 722 skipped, 513 s |
| `cleanroom-sweep.sh` × **as duas vassouras** | **✓ limpo** (70 e 94 entradas) |
| `typos` (⚠️ é gate de `ship`, corrido aqui de propósito) | **`exit 0`** — depois de curar um vermelho desta linha |
| `doc-index.sh --check` | **`exit 0`** — 18 índices em dia |
| `collision-surface.sh` | **nenhum contador movido** (§3) |

⚠️ **O `file_loc_caps.rs` da shell é corrido À PARTE de propósito** — o
`architecture_workspace_file_loc_cap` cobre só `crates/`, e essa lacuna já deixou
vermelho latente duas vezes.

### Auditoria — as duas lentes

**Lente 1 — «o controlo novo está VIVO em todos os passos?»** Um controlo tem
quatro maneiras de estar morto (pintado · registado · o clique drena · o valor
chega a um consumidor), e as quatro foram mutadas:

| mutação | quem sangra |
|---|---|
| o id sai do registo | `every_painted_control_is_clickable_where_it_is_drawn` |
| o dreno do clique sai | `the_connected_only_switch_flips_the_brush_field` ⭐ **gate NOVO, pedido pela mutação** |
| o `dab_core` deixa de ler o campo | `the_ear_does_not_ship_an_edge_across_the_piece` — ⭐ a fixtura PREGADA é o sentinela |
| o Cloth passa a oferecer a caixa | `the_connected_only_switch_exists_for_every_verb_but_the_cloth` ⭐ gate NOVO |
| o default volta a `false` | `o_connected_only_nasce_ligado_por_decisao_do_dono` ⭐ gate NOVO |

⇒ **três dos cinco gates desta wave nasceram de mutações SOBREVIVENTES**, ou seja
de buracos reais que a primeira redacção deixara. ⛔ A que mais importa: apagar o
dreno do clique deixava a caixa **pintada, registada e MUDA** com a suíte inteira
verde — a família do *«dreno de um braço só»* do `CLAUDE.md` §5.0.

**Lente 2 — «a fixtura contém o fenómeno?»** Esta jornada pagou **duas vezes**
por fixturas que não o continham (o `shapes::cylinder` sem vértices a meia
altura, e o gate da época a medir uma esfera convexa onde *«parou de cortar»* é
invisível). ⇒ a cena `=39` tem gates sobre a **própria fixtura**: dois
componentes (`a_cena_tem_duas_pecas_soltas_e_nao_uma`) e o vão `≤ 0,20`
(`as_duas_pecas_estao_perto_o_bastante_...`), cada um morto pela mutação do
outro.

---

## §11 — Resumo colável

```
line/sculpt3d — fecho 2026-09-10
  merge-base 39d48cd76 · 63 commits · 274 ficheiros · HEAD 45dd59ff8

  SCHEMAS      nenhum movido (PROJECT 123=123 · FLIP 13=13 · DOC 18=18 · registo 80=80)
  CONTRATOS    nenhum encostado ⇒ nenhum ADR devido
  Cargo.lock   nenhum pacote externo novo
  ORDEM        livre — sem dependência contra nenhuma outra linha

  ADITIVO      ph2d-mesh-render (ScreenRect) · ph2d-editor-core (ids append-only
               + 2 ficheiros NOVOS) · ph2d-i18n (14 rótulos apendados) · ph2d-cloth

  RISCO ZERO   ph2d-painter-brush · ph2d-flip · ph2d-flip-reshape · ph2d-flip-render
               · ph2d-tool-flip · ph2d-panel-painter-layers
               → 0 linhas fora de comentário (medido). Conflito textual: prefira
                 o lado desta linha nos COMENTÁRIOS, o do main no CÓDIGO.

  SMOKES OK    =37 (filtro de tecido) · =39 (as duas peças vizinhas)
  SMOKE NOVO   =39 — ⚠️ ela nasceu =38 e o gate do roteador apanhou a colisão

  PORTÃO       fmt 0 · clippy -D warnings 0 · 4 arch-gates 0 · file_loc_caps 0
               · sweep limpo nas 2 vassouras · collision-surface limpo
```

**Ler a seguir:** [`docs/3D/cloth/`](../cloth/) (o tecido, com as recusas medidas)
· [`docs/3D/cleanroom/`](../cleanroom/) (a espec atestada, o INBOX e o LEDGER) ·
[`dab_alcance.rs`](../../../crates/ph2d-sculpt3d/src/dab_alcance.rs) (a máscara, a
constante derivada e a constante apagada).
