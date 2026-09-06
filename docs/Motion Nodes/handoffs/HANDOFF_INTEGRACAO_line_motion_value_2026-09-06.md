# HANDOFF DE INTEGRAÇÃO — `line/motion-value` · 2026-09-06

> **Para o agente integrador.** A linha está FECHADA e não integra nem pusha (CLAUDE.md §0.7).
> Escrito segundo [DIRETRIZ §1.5.9](../../IntegracaoMultiAgente/DIRETRIZ.md).

## §1 — Identidade

| | |
|---|---|
| branch | `line/motion-value` |
| ⚠️ **estado** | **FECHADA e À ESPERA da PRÓXIMA rodada** — decisão do Enio, 2026-09-06 (ver §13) |
| base | **`815555aed`** — a linha foi **rebaseada** sobre o `main` já com as cinco linhas de 06/09 |
| ⚠️ o que se funde | **o tip do branch**: `git rev-parse line/motion-value`, hoje `fd3f34c09` · marco `linha-pronta-para-integrar-2026-09-06`. *Um handoff que cita o próprio SHA persegue o rabo* |
| commits a entrar | **36** — os 35 do trabalho + a cura dos vermelhos da árvore combinada |
| entra em fast-forward? | ✅ **sim**, verificado (`git merge-base --is-ancestor main HEAD`) |
| ficheiros | **130** |
| contrato congelado (§6) | **NENHUM tocado** — `nodegraph/src/node.rs` e `editor-core/src/tool.rs` intactos |
| ADR novo | **nenhum** ⇒ fora de toda disputa de número |

**A jornada em três blocos**, nesta ordem de dependência:

1. **Os estudos (docs 99–102)** — só documentação, sem código. O [102](../102_o_outro_patamar_plano_dos_nos_2026-09-04.md)
   é o plano técnico de que os ciclos saem; o [100](../100_estudo_dos_outputs_2026-09-04.md) mede
   que `value.lfo → motion.drive` é o `motion.oscillator` ao bit.
2. **O SUBSTRATO + o ciclo 1 (ARRANJO)** — os params passam a viver **no cartão** do grafo
   (decisão do Enio, 05/09: *«como no Blender, os parâmetros dos nós devem ser desenhados nos nós
   e vamos retirar o painel lateral»*), mais os 10 nós de arranjo e o **tutorial 1 em PDF**.
   ⚠️ **É este bloco que toca `editor-core`** — os ciclos 2+ só pagam o grupo deles.
3. **O ciclo 2 (ANIMADORES)** — W1..W5, a medição do grupo e o **tutorial 2 em PDF**.

⚠️ **A cena `=110` e o doc 103 entram no meio do bloco 2**, por um report do Enio (o duplicator).

## §2 — Foundational / partilhado tocado, e por quê

Tudo **aditivo**. Nenhuma assinatura existente mudou de forma.

| ficheiro | o quê | por quê |
|---|---|---|
| `ph2d-editor-core/src/interaction/types.rs` | **variante nova** `GraphHitKind::ParamRow { node: u64, row: u16 }` | uma row de param desenhada no cartão precisa de um alvo de gesto; `editor-core` **não a interpreta** (o mesmo *«atravessa como inteiro»* do nó e da aresta) |
| `ph2d-editor-core/src/paint_shapes.rs` | `fill_rounded_rect_srgb8` (função nova) | pintar a **amostra de cor de um param** — cor do DOCUMENTO, não do tema. ⚠️ Não é fuga ao HR-15: a assinatura só aceita bytes vindos de fora, e o tipo `Color` é privado à crate |
| `ph2d-editor-core/src/text_elide.rs` | `title_elided_width` (função nova) | **cura de um defeito shipado**: media-se em `NORMAL` e pintava-se em `SEMI_BOLD`, e o número cortava (`0....`). A porta mede no peso em que pinta |
| `ph2d-editor-core/tests/hr12_widgets_a11y.rs` | +2 entradas na lista de pintores puros | `paint_card_params.rs` e `paint_socket.rs` — quem regista os alvos deles é o `hits.rs` |
| `ph2d-editor-core/tests/architecture_motion_chrome_never_wraps_a_row_label.rs` | catraca `ELIDED_TODAY` **28 → 30** | as 4 rows novas do cartão que cortam |
| `ph2d-ui-testkit/src/lib.rs` | `paint_and_count_geometry_with_layout` | contar geometria de um painel do **split**; a variante `for_viewport` dá rect de área zero ao Motion e o gate ficaria **vácuo** |
| `ph2d-motion-region/src/lib.rs` | `Region::half_extents` | a grelha de vizinhança do `motion.scatter` precisa de indexar o espaço da região sem reconstruir a caixa |
| `shells/desktop/src/main.rs` | +2 `mod` `#[cfg(test)]` | as duas sondas do ciclo 2 — não entram no binário |
| `scripts/tutorial-pdf.sh` | **ficheiro novo** | HTML → Chrome headless → PDF (a única ferramenta medida nesta máquina) |
| `CLAUDE.md` | **1 linha** (a `Ler:` do Motion) | ponteiros para os docs 100–105 e para os tutoriais |
| `project-memory/` | 9 ficheiros novos + `MEMORY.md` | lições da jornada |

⛔ **Nenhuma crate nova, nenhuma dependência nova** (`Cargo.lock` sem `+name`).

## §3 — Símbolos que podem COLIDIR (a saída do `collision-surface.sh`, não de memória)

> ⛔ **A tabela abaixo mede contra `53832c884`, que era o `main` ANTES da integração de 06/09.**
> Depois dela a linha foi rebaseada sobre `815555aed` e a tabela ficou histórica. Ela continua
> útil para saber *o que a linha achava que estava a tocar* — **re-corra o script** antes de
> fundir (é o que a própria §1.5.9 manda, e agora há um motivo a mais: a base mudou).

```text
SUPERFÍCIE DE COLISÃO — line/motion-value contra main
  merge-base 53832c884   ·   33 commit(s)   ·   131 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        114   (base: 114)
      └ tripla do gate               (114, 13, 18)   (base: (114, 13, 18))
    VEC_SCENE_SCHEMA                       18   (base: 18)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)                  80   (base: 80)
    ph2d-script (espelho)                  80   (base: 80)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR — último no disco: 0168   próximo livre: 0169
    esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — nenhum '+name' novo
▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⚠️ **PRAZO DE VALIDADE:** a tabela acima mede contra o `main` de **2026-09-06** (`53832c884`).
**Re-corra `collision-surface.sh` nesta worktree imediatamente antes de fundir** — se outra linha
entrar no meio, todo número da coluna `base` muda e esta tabela descreve um `main` que já não
existe. Use-a só para saber *o que a linha ACHAVA que estava a tocar*.

**Os quatro pontos que um `grep` de mesmo-símbolo tem de olhar** (nenhum aparece na tabela acima,
porque nenhum é schema):

1. ⚠️ **`GraphHitKind::ParamRow`** — variante nova num enum **foundational**. Outra linha que
   acrescente uma variante ao mesmo enum funde limpo e as duas sobrevivem; o que **não** sobrevive
   é um `match` exaustivo noutra crate que passe a faltar um braço. Grepe `GraphHitKind`.
2. ⚠️ **`ELIDED_TODAY` = 30** em `editor-core/tests/architecture_motion_chrome_never_wraps_a_row_label.rs`
   — é uma **catraca**, e duas linhas que a subam escrevem o mesmo tipo de literal em sítios
   próximos. O valor certo depois da fusão é o **contado**, nunca o de nenhum dos dois lados.
3. ⚠️ **`MAX_DEMO_LEVEL: u32 = 110`** e o braço `Some("110")` em
   [`motion_state_demo_router.rs`](../../../shells/desktop/src/motion_state_demo_router.rs) — o
   número da próxima cena **CONTA-SE do roteador**. O gate `no_two_smoke_scenes_claim_the_same_level`
   mede o **piso**, não o teto: duas cenas com o mesmo número passam sem o acordar.
4. ⚠️ **`ParamSpec` apendados** em dois manifestos: `motion.spring` (+`mode`, `duration`, `bounce`)
   e `motion.stagger` (+`order`, `seed`). São **apêndices no fim**, então todo documento autorado
   lê o mesmo índice — mas se outra linha apendar aos MESMOS dois nós, a ordem depois da fusão tem
   de ser reconferida contra os gates de byte-identidade das duas.

## §4 — Contratos congelados

**Nenhum.** Confirmado pelo `collision-surface.sh` e pelo gate `architecture_contract_surface`.

## §5 — O que só o `ship.sh` apanha (o gate de integração não roda)

- **`typos`** — 2 tutoriais em HTML e ~6 docs novos em português, mais 9 memórias. O corpo do texto
  nunca passou pelo `typos`. ⚠️ Os **PDF são binários** (`01_arranjo.pdf` 464 KiB,
  `02_animadores.pdf` 786 KiB) — o `typos` não os lê, então uma gralha lá dentro só sai
  regenerando do HTML.
- **`cargo machete`** — nenhuma dependência nova, mas duas crates ganharam módulos.
- **`cargo deny` / RUSTSEC** — nada novo no `Cargo.lock`.
- **`fmt`/`clippy` pré-fork** — a linha correu os dois sobre o **seu** diff, não sobre o `main`
  fundido.
- ⚠️ **`doc-index.sh --check`** — `docs/Motion Nodes/` ganhou 6 docs e a pasta `tutoriais/`.
  Índice regenerado nesta linha; re-corra depois de fundir.

## §6 — Ordem, dependências, e o que ainda NÃO foi smokado

**Ordem:** os 34 commits são sequenciais e cada um compila. Não há reordenação possível — o bloco 2
(o cartão) é pré-requisito do bloco 3 (o ciclo 2 mede e documenta cartões).

| smoke | estado |
|---|---|
| Tutorial 1 «Do primeiro objeto ao milhão» (ciclo 1) | ✅ **feito pelo Enio, 2026-09-05/06** — 3 reports, os 3 curados |
| Cena `=110` (todo o duplicator, 13 bandas) | ✅ **feito pelo Enio, 2026-09-06** |
| **Tutorial 2 «O tempo entra no grafo» (ciclo 2)** | ⏳ **PARCIAL — «parece ok, mas ainda não fiz completamente»** (Enio, 2026-09-06) |

⛔ **O que NÃO foi smokado, nomeado:** as seções **5 a 12** do tutorial 2 (Spring · Wiggle/Noise ·
Orbit/Delay · LFO) não foram percorridas de ponta a ponta por ele. Elas estão **medidas por sonda**
e as figuras são geradas com gate — o que falta é a mão dele no app.

**O comando do smoke** (uma env var não é um build: **um** binário cobre todas as cenas):

```
env PH2D_GPU_COOK_DEMO=1 cargo run -p ph2d-host-desktop --release
```

## §7 — Portão de fecho (batched, 1×)

| gate | resultado |
|---|---|
| suíte do shell (`--release --no-fail-fast`) | **212 binários, 0 falhas** (4 473 + 308 ignorados no principal) |
| `clippy -p ph2d-host-desktop --all-targets` | limpo |
| tectos de LOC (`--test file_loc_caps`) | verde — **dois cortes por responsabilidade**, nenhuma isenção |
| `cargo fmt --all -- --check` | limpo |
| `collision-surface.sh` | acima |

⚠️ **Os dois cortes de LOC não foram «partir ao meio»:** a **curva no tempo** saiu da **nuvem num
instante** (`animadores_spring_fig`), e os seis instrumentos dos ciclos saíram do `motion_bridge`
para um irmão de declarações (`ciclos_mods`) — o mesmo molde do `test_mods` que já vivia ao lado.

## §8 — As sete coisas que uma leitura rápida do diff entende ao CONTRÁRIO

1. **`+17 640` linhas não é código novo em maioria** — ~40 % são docs (6 planos/estudos, 2
   tutoriais em HTML) e 2 PDF binários.
2. **O painel lateral de params ainda EXISTE.** A decisão é retirá-lo, e o portão que o autoriza a
   sair já foi escrito (`no_param_the_panel_offers_falls_off_the_card`) — mas a remoção não está
   nesta linha. ⛔ Apagar o painel agora não é *«terminar o que a linha começou»*.
3. **`motion.scatter` ficou `22×` mais rápido AO BIT.** A nuvem não mudou; quem mudou foi a
   consulta de vizinhança (era `O(n²)`). Um golden de outra linha sobre aquele nó **não** muda.
4. **`Physics` continua a ser o default da mola, e é byte-idêntico.** O modo `Time` é aditivo:
   nenhum grafo autorado se mexe.
5. **O `reverse` do stagger NÃO foi substituído** pelo `order`. Eles **compõem** — três ordens ×
   dois sentidos —, e há gate a provar que `From Center` espelhado **é** o *das pontas para o meio*
   (por isso ele não é uma quarta entrada da lista).
6. **O ganho de `7,5×` do `delay` é só do modo `Blend`.** Os outros dois modos são byte-idênticos e
   continuam a pagar o anel de 32 fatias — de propósito: encolhê-lo com o `ticks` faria a
   profundidade mudar **durante o arrasto**.
7. **`GraphHitKind::ParamRow` é registada DEPOIS do corpo do cartão**, e é isso que a faz ganhar o
   gesto. O efeito colateral é de produto e está no tutorial: **arrastar o nó passa a ser pelo
   cabeçalho**.

## §9 — As premissas que a MEDIÇÃO derrubou (minhas, todas)

1. ⛔ *«O `motion.delay` lê o passado de OUTRO elemento»* — **falso**. Cada um lê o **seu**. Ia
   mandar a wave atrás de um `gather` que não existe.
2. ⛔ *«Falta MASSA à mola»* — **falso duas vezes**: a equação absorve-a (`m·x″+c·x′+k·x = 0`
   dividido por `m`), e ela já existe **por elemento** (`inv_mass`).
3. ⛔ *«O termo do atrito nunca é o menor»* — **falso** (`tension 0,5` com `friction 4,08`). O que
   não muda é a **contagem** de sub-passos, em 1 323 células — com **um** canto nomeado.
4. ⛔ **A coluna do dispositivo da auditoria** saiu do `NodeManifest::lowerings` e imprimiu `NÃO`
   para os **oito**; o caminho do device é side-metadata (`register_gpu_kernel`). *Teria aberto o
   ciclo a escrever oito kernels que já existem.*
5. ⛔ **O censo do cartão acusou `motion.noise::rotation`/`::uniform`** de inalcançáveis: eles
   vivem numa **secção que nasce fechada**, a um clique.
6. ⛔ **A régua das figuras errou QUATRO vezes** (doc 105 §5.1) — a última porque uma *norma* é
   cega ao sinal, e a onda quadrada é exactamente o caso uniforme em magnitude.
7. ⛔ **Duas sondas mediram nós DESLIGADOS**: sem `Cook::advance_tick` a mola e o atraso devolvem
   o primeiro tique, que é a identidade por desenho
   ([memória](../../../project-memory/feedback_a_delayed_edge_does_not_carry_itself_advance_tick_does.md)).

## §10 — O que fica ABERTO (para o §5 do CLAUDE.md, não para esta linha)

1. ⏳ **O passo 7 do ciclo 2** — o smoke do tutorial 2, que é a **aceitação** do ciclo. É do Enio.
2. ⏳ **Retirar o painel lateral de params** — autorizado por gate, não feito.
3. ⏳ **Os ciclos 3–9** ([doc 103 §5](../103_dinamica_dos_ciclos.md)), e depois os três que o Enio
   mandou pôr no fim: **10** o carimbo no dispositivo · **11** a avaliação geral de performance ·
   **12** os tectos confortáveis de objectos.
4. ⏳ **`motion.delay` no device** — depois do `7,5×` ele deixou de ser a alavanca que era, mas com
   **duas** colunas de estado o `Blend` passou a ser um mapa por-elemento, logo é candidato.
5. ⏳ **`motion.voronoi` custa `19,68 ms` para 2 000** (doc 104 §3) — está no device e é caro; medir
   **onde** antes de tocar.
6. ⏳ Os nós do ciclo 1 que ficam fora do device, cada um com o mecanismo nomeado (doc 104 §3.2).

## §11 — Limpeza e binário quente

- `rm -rf target/*/incremental` — feito (§1.5.9 item 7).
- Binário do smoke **compilado e quente** — a prova (2ª corrida) está no §12.

## §12 — Prova do binário quente (§1.5.9 item 9)

O Enio não espera build. O binário do smoke está compilado **nesta worktree**, com a **mesma**
linha de comando que o handoff entrega (mesmo pacote, mesmo perfil, mesma árvore):

```text
$ cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value
$ cargo build -p ph2d-host-desktop --release
    Finished `release` profile [optimized] target(s) in 2m 29s
$ cargo build -p ph2d-host-desktop --release          # a PROVA
    Finished `release` profile [optimized] target(s) in 0.19s
```

**Zero linhas `Compiling` na 2.ª corrida.** ⚠️ Uma env var não é um build: **um** binário cobre
todas as cenas (`PH2D_GPU_COOK_DEMO`, `PH2D_MOTION_*`, …). E o `target/*/incremental` foi
reclamado antes disto — **19 GB** devolvidos, e o perfil `release` não o usa.

---

## §13 — A LINHA ESPERA A PRÓXIMA RODADA, e as cinco providências

**O que aconteceu:** a integração de 2026-09-06 levou **cinco** das seis linhas — `sculpt3d`,
`Vector`, `components`, `3DModeling`, `UIUX` — e esta ficou de fora. O commit de preparação do
integrador diz *«as lições das SEIS linhas»* (15:01) e o de fecho diz *«nenhuma das seis linhas
os podia ver»* (16:26); a linha estava pronta às **14:39**.

⚠️ **Não se confirma isto pelo reflog:** ele tem um `merge line/motion-value` em **04/09 15:58**,
que é a jornada **anterior** — e é por isso que o L-System está no `main`. A pergunta certa é
pelo **conteúdo desta jornada**:

```bash
git cat-file -e main:crates/ph2d-node-motion-spring/src/law.rs   # ausente ⇒ não integrada
```

**Decisão do Enio (2026-09-06): esperar a próxima rodada.** O custo foi medido antes de decidir —
na rodada que passou as cinco linhas tocaram **7 dos 130 ficheiros** desta (5%), e esses 7
custaram **três** defeitos que nenhuma linha via sozinha (§8). À data da decisão o encosto VIVO
era **zero**: `3DModeling`·`components`·`sculpt3d` tinham 2·2·5 ficheiros novos, nenhum meu.

**As cinco providências em vigor enquanto ela espera:**

| # | providência | porquê |
|---|---|---|
| 1 | **`git rebase main` no INÍCIO de cada sessão**, não só no fecho | a distância nunca passa de um dia, e um conflito de um dia resolve-se; um de uma semana reescreve-se |
| 2 | **`rerere` ligado** (`rerere.enabled` + `autoupdate`), com **16** resoluções em cache — a do `paint_socket.rs` entre elas | o mesmo conflito não se resolve duas vezes |
| 3 | ⛔ **não tocar nos três ficheiros-hub** até integrar: `CLAUDE.md` · `shells/desktop/src/main.rs` · `crates/ph2d-panel-motion-graph/src/paint.rs` | são os que **toda** linha edita; a edição do §5 que o protocolo exige **já está feita** |
| 4 | **`collision-surface.sh` no início da sessão também** | o handoff é referência e envelhece; a corrida é a evidência |
| 5 | **marcos**: `linha-pronta-para-integrar-2026-09-06` (o tip verde) e `pre-rebase-2026-09-06` | nada se perde num rebase que corra mal |

⚠️⚠️ **O risco que NÃO é conflito de texto, e é o maior:** esta linha carrega o **substrato do
cartão**, e o **tutorial em PDF é o smoke do produto** — ele descreve o app passo a passo. Se
outra linha mudar a UI do cartão antes desta entrar, o tutorial passa a **ensinar o que não
acontece**, que é a família que o [`CLAUDE.md §5.0`](../../../CLAUDE.md) regista com o preço
pago. A vizinha perigosa é a **`line/UIUX`**, que acabou de passar o app inteiro para cartões e
que já colidiu com esta **duas vezes na mesma pasta** (§8 e o commit `fd3f34c09`).
⇒ **ao reabrir esta linha, o passo 1 é conferir se a `UIUX` mexeu no painel do Motion.**
