# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, fecho de 2026-09-13

> ⚠️ **A linha NÃO integrou e NÃO pushou** (`CLAUDE.md` §0.7). Ela fecha, entrega isto e para.
> Quem integra é um **agente integrador dedicado**, por ordem explícita do Enio
> (DIRETRIZ §1.5.3–1.5.4).
>
> ⚠️ **Esta é uma linha CLEAN-ROOM** (alvo `blender-cloth`, SKILL_Cleanroom). O §4 conta um
> incidente da parede, tratado pelo protocolo §6 da SKILL. ⛔ **Nada deste documento cita o alvo
> por nome interno** — o integrador não precisa de o ver, e não deve procurá-lo.

---

## §1 — Identidade

| | |
|---|---|
| ramo | `line/sculpt3d` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d` |
| merge-base | `1d43da737` (o `main` enviado na reabertura de 13/09) |
| commits | **8** — 7 até `cb279c0cb`, mais o commit que traz este handoff |
| HEAD | o commit deste handoff (o último do ramo); o último commit de **produto** é `cb279c0cb` |
| smoke do dono | **não há comportamento novo no ecrã** — só gates e comentários (§6) |

### O que a linha entrega, em dois blocos

1. **OS GATES QUE A FAMÍLIA PROMETIA E NUNCA EXISTIRAM** — oito nomes de gate citados em
   comentários sem `fn` em lado nenhum (§3): seis escritos, dois reapontados para o gate que já
   defendia a propriedade com outro nome, sete citações com o nome ANTIGO de um gate vivo, e o
   censo que reprova a recaída. Entre os oito estava a **dívida §2.10 da Fase B** (o empréstimo
   da cena à família), que o cabeçalho do `sculpt3d_host.rs` dava por paga.
2. **O INCIDENTE INC-4 DA PAREDE CLEAN-ROOM, tratado até ao fim** — os comentários do produto na
   família citavam o programa de referência por nomes internos, números de linha do fonte dele,
   fragmentos de uma linha de código e duas citações de comentário. Registado, classificado por um
   R independente, **173 blocos em 59 ficheiros reescritos** em vocabulário do domínio, e a
   vassoura emendada para que o sweep passe a ver esta população (§4).

---

## §2 — Foundational / partilhado tocado, e por que é aditivo

**Nenhum ficheiro foundational.** O diff contra o merge-base:

| onde | o quê | natureza |
|---|---|---|
| `crates/ph2d-sculpt3d` · `-panel-sculpt3d` · `-app-sculpt3d` · `-form-donation` | gates novos + comentários | aditivo (testes) / só comentário |
| `shells/desktop/tests/it/the_sculpt_gesture_is_wired.rs` | **1 linha de comentário** (um nome antigo) | LOC da shell: **±0** |
| `docs/3D/cleanroom/` | `INBOX` (append), `LEDGER` e `VASSOURA` (escritos pelos subagentes R e E), `SPEC_reescrita_…` (novo) | docs |
| `docs/3D/handoffs/` + `CLAUDE.md` §5 | este handoff, o índice, uma linha | docs |

Números do diff contra o merge-base, medidos em `cb279c0cb` (antes do commit deste handoff):
**98 ficheiros** — **94 `.rs`** (`+1 687 −525`: os gates novos e os comentários reescritos) e **4
documentos da parede** (`+760`: `INBOX`, `LEDGER`, `VASSOURA`, `SPEC_reescrita_…`). O commit deste
handoff acrescenta o `CLAUDE.md` (a linha do §5 e uma frase corrigida na entrada da Fase B), o
índice de handoffs e este ficheiro.

⚠️ **Um elo novo que o integrador tem de conhecer:** o gate
`ph2d-app-sculpt3d/src/host_contract_tests.rs` lê por `include_str!` **sete ficheiros de fora da
crate** — `shells/desktop/src/{app_host,chrome_hit,command_palette_input,sculpt3d_host,sculpt3d_absent}.rs`
e `crates/ph2d-app-host/src/{lib,canvas_area}.rs`. Se outra linha os mudar de **sítio**, a crate
da família **não compila os testes** (espécie que falha alto, de propósito). Se outra linha
acrescentar uma **porta ao `AppHost`** ou mudar a **rota** de uma porta, o gate reprova na árvore
combinada **com a instrução escrita**: leia a porta nova e acrescente-a às tabelas
`PORTAS`/`LICENCAS`/`ROTAS` — ⛔ nunca apague o gate.

---

## §3 — Superfície de colisão (`collision-surface.sh`, colada)

```
SUPERFÍCIE DE COLISÃO — line/sculpt3d contra main
  merge-base 1d43da737   ·   2 commit(s)   ·   17 arquivo(s)        ← 1.ª corrida, antes do INC-4
▸ SCHEMAS
    PROJECT_SCHEMA                        128   (base: 128)
      └ tripla do gate               (128, 13, 22)   (base: (128, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
    FIELD_DOC_VERSION                      22   (base: 22)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs                               85   (base: 85)
    ph2d-render (espelho)                  86   (base: 86)
    ph2d-script (espelho)                  86   (base: 86)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR — esta linha não cria ADR
▸ Cargo.lock — nenhum '+name' novo
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

**Re-corrida no ramo acabado** (`cb279c0cb`, 7 commits, 98 ficheiros): **idêntica em tudo** — os
mesmos valores de schema e de registo, os dois contratos intocados, nenhum ADR, nenhum pacote
externo, nenhum marcador de conflito, e nenhum ficheiro da linha acima do tecto (o único que o
script destaca é `crates/ph2d-app-sculpt3d/src/keys_scene.rs`, `135 / 700`, por ter marcador).

**Ids, consts, variants, tokens novos: nenhum.** Nomes de gate novos (todos `#[test]`, sem
colisão possível com código de produto):

| gate | crate / ficheiro |
|---|---|
| `the_host_never_reads_the_borrowed_scene` | `ph2d-app-sculpt3d/src/host_contract_tests.rs` |
| `the_sculpt_host_refuses_without_a_scene` | idem |
| `every_gate_the_sculpt_family_names_exists` | `ph2d-app-sculpt3d/src/named_gates_census_tests.rs` |
| `every_memory_of_a_dead_gate_still_describes_something` | idem |
| `the_shape_of_a_saved_scene_is_pinned` | `ph2d-app-sculpt3d/src/doc_tests.rs` |
| `the_panel_offers_every_retopo_mode_the_engine_has` | `ph2d-panel-sculpt3d/src/censo_das_fileiras_tests.rs` |
| `the_surface_is_the_same_in_any_tangent_frame` | `ph2d-sculpt3d/src/stroke_surface_tests.rs` (novo, `#[path]` do `stroke_surface.rs`) |
| `an_idempotent_dab_does_no_work` | `ph2d-sculpt3d/src/stroke_window_tests.rs` |

---

## §4 — Contratos congelados encostados · e o INCIDENTE da parede

**Contratos congelados: nenhum** (`node.rs` e `tool.rs` intocados, colado acima). Zero schema,
zero registo, zero ADR, zero pacote externo.

### O INC-4 — o que aconteceu, e quem decidiu

1. **Achado.** Ao re-medir os itens abertos, a janela I correu um censo de «nomes de código entre
   crases, em comentário, que não resolvem para a nossa árvore». A população misturava quatro
   espécies: nomes internos do programa de referência · nomes de API **pública** dele · nomes
   antigos NOSSOS · nomes da biblioteca padrão do Rust. Ao imprimir o censo com contexto, a janela
   leu ~30 nomes internos e ~6 fragmentos de uma linha de código do alvo **que já estavam nos
   nossos comentários**.
2. **Separar público de interno sem ler fonte:** o oráculo foi **corrido** sem interface
   (`blender -b --factory-startup`, dump das propriedades RNA, `2>/dev/null`) — 576 nomes
   públicos, e **seis** dos citados são públicos (§4.1.13 da SKILL: ficam).
3. **PARE + registo (§6.1):** entrada no `INBOX_blender-cloth.md` por append cego, a descrever sem
   reproduzir (origem, extensão, instante, o código escrito depois da exposição). A cópia do censo
   foi apagada do scratchpad.
4. **Régua (§6.2) decidida por um R independente, nunca pela janela:** **RELANCE** · quarentena
   **LIVRE** (os commits escritos depois da exposição não tocam as regiões expostas) · sweep da
   família **0 hits** — ⚠️ *e esse zero não provava nada*: **nenhuma das 70 entradas da vassoura
   cobria esta população**. O R escreveu o plano de reescrita
   `docs/3D/cleanroom/SPEC_reescrita_dos_comentarios_com_nomes_do_alvo.md` (94 sítios da população
   + as outras linhas dos mesmos ficheiros + um anexo de 18 ficheiros), varrido e verificado por
   grep a zero identificadores, e registou tudo no ledger (`26ddf4969`).
5. **Reescrita:** quatro subagentes da implementação, em grupos de ficheiros disjuntos, com o
   parágrafo da parede **verbatim** no briefing e contrato de retorno sem expressão do alvo:

   | grupo | ficheiros | blocos | linhas apagadas |
   |---|---:|---:|---:|
   | 1 — pincel, painel, dureza | 19 | 46 | 3 |
   | 2 — filtros de malha | 9 | 61 | 162 |
   | 3 — o traço (plano, alvo, simetria) | 13 | 38 | 2 |
   | 4 — verbos e sondas | 18 | 28 | 72 |
   | **total** | **59** | **173** | — |

   ⭐ **As duas citações de COMENTÁRIO do alvo foram apagadas sem serem lidas** — um visor que
   mascara o conteúdo entre aspas mostrou só a forma das linhas, e o script de remoção afirmava que
   cada linha removida era comentário, sem imprimir o texto.
6. **Verificação pela janela** (sem ver nomes): diff de **77 ficheiros, zero linhas fora de
   comentário** · `cargo fmt --check` limpo · `check --all-targets` das três crates **sem aviso** ·
   censos de nomes de gate verdes · **contagem CEGA** do resíduo (nomes de código de 3+ palavras que
   não resolvem para a nossa árvore, fora dos já classificados como nossos ou públicos, contada sem
   os imprimir): **0**.
7. **Emenda à vassoura + sweep com controlo positivo** (subagente E, ledger `a6f8865da`): a
   vassoura ganhou **+79 entradas (total 149)**, colhidas do estado ANTES e confirmadas no fonte do
   alvo — nomes públicos, nomes nossos e palavras genéricas ficaram de fora. **Controlo positivo:**
   o mesmo sweep sobre o estado ANTES dá **199 hits** (a vassoura antiga dava **0** sobre esse mesmo
   estado — a lacuna do R, agora medida).
   ⛔⛔ **E o sweep da família no HEAD NÃO deu verde: 13 hits, zero falsos positivos** — **12 dentro
   de LITERAIS DE STRING de teste** (os rótulos de uma tabela de curvas, três mensagens de asserção)
   e **1** num comentário de módulo. *A régua do censo, o plano do R e a verificação da janela
   olhavam todos só COMENTÁRIOS* — e uma string não é comentário. ⚠️ **A mensagem do commit do E diz
   «verde» e está errada nisso.** Curados por um subagente da implementação pela mesma regra (os
   nossos nomes de curva, a grandeza sem nome, «o painel de queda»): **13 sítios em 5 ficheiros**, diff só de strings de teste e comentário,
   `check` `exit 0`, os testes afectados **25 + 8 + 3 passaram** (1 sonda ignorada de propósito), e
   ⭐ **o sweep da família no HEAD dá `exit 0` sobre 389 ficheiros** — agora com uma vassoura que
   apanha 199 hits no estado ANTES.

⚠️ **O que ficou de fora, de propósito:** a mesma dívida **noutras famílias** (§9) — ela é das
linhas donas, e a contagem por crate está no ledger.

---

## §5 — O que só o `ship.sh` pega (e o que esta linha já correu)

- **`typos`** sobre os comentários reescritos — corrido no portão (§10).
- **fmt** — limpo; **machete** — nenhuma dependência nova ou retirada.
- ⚠️ **O censo de nomes de gate lê a workspace inteira em runtime.** Se outra linha **renomear ou
  apagar** um gate que um comentário da família cita, o `every_gate_the_sculpt_family_names_exists`
  reprova na árvore combinada **com o nome e o sítio**. A cura é corrigir a citação (ou pô-la em
  `MEMORIAS` com o motivo, se a frase é memória). O controlo positivo dele é o
  `the_ear_does_not_ship_an_edge_across_the_piece` — se esse gate mudar de nome, troque o controlo.
- ⚠️ **O `the_host_never_reads_the_borrowed_scene` lê a shell pelo caminho** — ver §2.

---

## §6 — Ordem, dependências e o que smokar

**Ordem:** nenhuma dependência de outra linha. Os cinco commits de produto são independentes dos
de docs.

**Smoke:** nada muda no ecrã — os commits são gates e comentários. O smoke que vale é o de
sanidade da escultura, na cena das duas peças vizinhas (a última aprovada pelo dono, 10/09):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=39 cargo run -p ph2d-host-desktop --profile smoke
```

O binário fica compilado nesta worktree (§10, última linha).

---

## §7 — ⚠️ SETE coisas que uma leitura rápida do diff entende ao contrário

1. **O diff de 77 ficheiros da reescrita é SÓ comentário**, e não por promessa: o comando
   `git diff -U0 <antes> <depois> | grep -E '^[+-]' | grep -vE '^(\+\+\+|---)' | grep -vE '^[+-]\s*(//|$)'`
   imprime zero linhas.
2. **Os dois gates do empréstimo moram na FAMÍLIA e não nos testes da shell**, de propósito: a
   shell estava a **987** linhas do tecto (`the_shell_only_shrinks`), e um gate lá é crescimento
   da crate que a W2 existiu para encolher. A propriedade é da família: é a escultura que some.
3. **O censo do empréstimo é por LICENÇA, não por palavra proibida:** cada leitura do `AppGfx` e
   cada chamada de cada porta tem de estar nas tabelas. Uma sétima porta, um `self.helper()` novo ou
   uma terceira rota reprovam e mandam alguém LER o que eles leem.
4. **O INC-4 foi RELANCE e a janela não queimou** — quem decidiu foi um R de contexto novo, e a
   quarentena foi comparada por ele. A janela nunca abriu o fonte, o ledger nem a vassoura.
5. **`MEMORIAS` não é uma lista de dívida:** são três frases históricas sobre gates que morreram ou
   foram substituídos, e a tabela tem censo de obsolescência nos dois sentidos (a memória continua
   citada onde a tabela diz; o nome continua sem definição).
6. **O `SPEC_reescrita_…` não é uma espec de comportamento** — é o plano do R, do lado da parede
   que a janela I pode ler, e descreve o que cada comentário deve DIZER.
7. **«Nunca existiu» está provado, não suposto:** `git log -S "fn <nome>"` sem commit para os
   oito, com a mesma busca sobre um gate vizinho a devolver o commit dele (controlo positivo).

---

## §8 — Premissas MINHAS que a medição derrubou

1. **«O próximo item é o 1.º ABERTO do handoff de 10/09 (o caso brando da máscara).»** A
   re-medição contra a árvore nova achou primeiro a dívida da Fase B que o código dava por paga — e
   ela era um de oito.
2. **«`the_hierarchy_has_a_delete_key` não existe.»** Existe — é um **ficheiro** de teste. O
   primeiro censo só lia nomes de `fn`; o gate lê `fn`, `mod` e nomes de ficheiro.
3. **O censo contava DEFINIÇÕES em linhas de comentário**, logo uma prosa «a `fn x` que…» tornava
   `x` definido e calava a citação — o mesmo furo do outro lado da régua. Curado (`b04f9d510`) e
   provado pela mutação M9b.
4. **Uma corrida dirigida com dois filtros correu UM teste** e o resumo dizia «1 falharam · 0
   passaram»: os dois gates do empréstimo não tinham corrido. *O veredito é a contagem, não o exit
   code sozinho.*
5. **A primeira fixtura do golden tinha o detalhe da pilha sem cor nem máscara** — um nível acabado
   de nascer não os guarda. O controlo da própria fixtura reprovou antes de medir; descer e voltar a
   subir um nível escreve-os.
6. **«O censo largo mede nomes do alvo.»** Mede **quatro espécies**, e só correr o oráculo separa
   público de interno sem ler fonte.
7. **«O sweep verde de 10/09 cobre os nomes internos.»** Zero das 70 entradas da vassoura os
   cobria — um instrumento que não conhece uma forma devolve zero sobre ela e lê-se como aprovado.
8. ⛔⛔ **«A reescrita está completa: diff só-comentário e contagem cega a zero.»** As duas réguas
   eram verdadeiras e as duas **só olhavam COMENTÁRIOS** — e 12 das 13 linhas que ficaram estavam
   em **literais de string** de teste. Quem as achou foi o único instrumento que não escolhe onde
   olhar: o sweep com a vassoura, **e só depois de a vassoura ganhar o controlo positivo**. *Uma régua
   que prova o que mudou não prova o que ficou.*
9. **O filtro de teste que eu escrevi no briefing do resíduo casava ZERO testes** (os módulos
   montam como `brush::tests` e `stroke::tests::filter_sharpen_tests`, não pelo nome do ficheiro).
   Quem o apanhou foi o subagente, porque o briefing exigia a contagem ao lado do exit code — sem
   essa exigência, «`exit 0`» sobre nada ter-se-ia lido como verde.

---

## §9 — ABERTO, com o número de cada um

| item | o número | onde |
|---|---|---|
| ⏳ **Gates prometidos que não existem, fora da família** | a régua do censo sobre a workspace: **106 nomes em 43 crates** (2026-09-13) | `named_gates_census_tests.rs` — alarga-se em `FAMILIA` |
| ✅ **Nomes internos do alvo desta população em OUTRAS famílias** | **zero hits fora da família** com a vassoura emendada (as 149 entradas, sobre `crates/**` e `shells/**`) — ⚠️ o que isto NÃO diz: a vassoura só conhece a população DESTA família, e outra família com outros nomes internos do mesmo alvo (o Painter, que tem referência Blender própria) lê zero por construção | ledger `blender-cloth`, INC-4 |
| ⏳ **Nomes do alvo de 1–2 palavras e constantes em maiúsculas** | o censo da família só mede 3+ palavras; esta classe fica coberta **só pela vassoura** | idem |
| ⏳ **O aviso da Fase B §11.3** (um campo sem leitor numa build sem a feature) | ⛔ E sem avisos a erro a shell **não compila sem a feature: `224` erros e `25` avisos** (`cargo check -p ph2d-host-desktop --no-default-features`, medido 13/09) — por isso o aviso da Fase B §11.3 continua **impossível de medir**: um lint de campo morto só corre depois de a verificação de tipos passar | `render_loop` da shell |
| ⛔ **A build SEM a feature por omissão está vermelha com avisos a erro, antes de chegar à shell** | um import não usado sem a feature em `ph2d-panel-registry-init/src/lib.rs:21`, desde `c14fde433` (09/09, outra linha). Nenhum job da CI o vê | a linha dona do painel do esqueleto — **não curado aqui** (fora da família) |
| ⛔⛔ **Sem a feature por omissão a SHELL NÃO COMPILA** (mesmo sem avisos a erro) | **`224` erros e `25` avisos**; **`215`** são `E0433` a uma crate que só entra com a feature — `ph2d_panel_vector` **193** · `ph2d_panel_inspector` **11** · `ph2d_panel_painter_layers` **10** · `ph2d_audio_edit` **1** —, concentrados em `render_loop/fase_bus_clicks.rs` (**86**) e `render_loop/fase_bus_tool_panel.rs` (**44**). É a forma que a descida dos ids para as crates que os lêem deixa numa build sem a feature, e **nenhum job da CI a corre**: a Fase B mediu `14` erros desta espécie em 11/09 e curou-os | a integração / a linha dona da refatoração final — **não curado aqui** (fora da família) |
| ⏳ Os abertos do handoff de 10/09 que não se moveram | o caso brando da máscara (`π/2 < √2`) · o tamanho da ruga · um traço do corpus do tecido fora da barra (**decisão do dono**) · os `docs/**` fora do censo de citações | [`HANDOFF_…_2026-09-10.md` §9](HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-10.md) |

---

## §10 — Portão de fecho corrido NESTA árvore

Todos lidos pelo **exit code** e pela contagem, logo depois do comando:

| gate | resultado |
|---|---|
| `BASE=1d43da737 bash scripts/nextest-impacted.sh` | **`exit 0`** — **13 889 de 13 889** passaram, 11 124 saltados (132 s, load 49,8 no arranque) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | **`exit 0`** (35 s) |
| `cargo clippy --all-targets -- -D warnings` (as 4 crates do diff) | **`exit 0`**, 0 avisos |
| `cargo clippy --all-targets --all-features -- -D warnings` (idem) | **`exit 0`** |
| `cargo machete` | **`exit 0`** |
| `bash scripts/check-standalone-optional.sh` | **`exit 0`** (103 s) |
| `bash scripts/check-workflow-packages.sh` | **`exit 0`** |
| `architecture_workspace_file_loc_cap` · `architecture_the_shell_only_shrinks` · `architecture_no_restricted_source_citations` | **`exit 0`** — 6 passaram, 1 sonda ignorada de propósito |
| `arch_safe_clamp_only` | **`exit 0`** — 2 passaram |
| `shells/desktop/tests/it`: `file_loc_caps` · `fn_loc_caps` · `the_sculpt_gesture_is_wired` | **`exit 0`** — 29 passaram |
| `typos` (as 4 crates + o ficheiro da shell) · `typos` dos docs novos | **`exit 0`** · **`exit 0`** |
| `bash scripts/doc-index.sh --check` | **`exit 0`** — 19 índices em dia |
| `cargo fmt --all -- --check` | **`exit 0`** |
| as corridas dirigidas dos gates novos | 5 de 5 (`ph2d-app-sculpt3d`) · 3 de 3 (`ph2d-sculpt3d`) · 7 de 7 (o censo das fileiras) |
| provas de mutação | **14 de 14 sangram** (lente 1, abaixo) |
| depois do resíduo (`cb279c0cb`): `fmt --check` · `clippy -D warnings` (as 2 crates) · censos de nomes de gate · testes dos 5 ficheiros · **sweep da família** | **`exit 0`** · **`exit 0`**, 0 avisos · 2 de 2 · 25 + 8 + 3 passaram (1 sonda ignorada) · **`exit 0` sobre 389 ficheiros** |
| ⚠️ `CARGO_BUILD_WARNINGS=deny cargo check -p ph2d-host-desktop --no-default-features` | **`exit 101` — VERMELHO PRÉ-EXISTENTE e FORA da linha:** um import que só a feature usa em `crates/ph2d-panel-registry-init/src/lib.rs:21`, presente no merge-base (commit `c14fde433`, 09/09, outra linha). A linha tem **0** linhas nessa crate. Nenhuma build da CI corre sem a feature — é a espécie da armadilha §6.1 da Fase B. ⛔ E sem avisos a erro a shell **não compila sem a feature: `224` erros e `25` avisos** (`cargo check -p ph2d-host-desktop --no-default-features`, medido 13/09) — por isso o aviso da Fase B §11.3 continua **impossível de medir**: um lint de campo morto só corre depois de a verificação de tipos passar |

### O `incremental/` reclamado e o smoke compilado (DIRETRIZ §1.5.9 itens 7 e 9)

`rm -rf target/*/incremental`: o `target/` da worktree foi de **31 GB para 17 GB** (16 GB eram
`incremental/`). Depois, nesta ordem, `cargo build -p ph2d-host-desktop --profile smoke` duas vezes
— a 1.ª em **57,57 s**, e a 2.ª, colada, sem nenhum `Compiling`:

```
    Finished `smoke` profile [optimized] target(s) in 0.20s
```

⚠️ Os commits que vêm depois deste passo são **só documentos** (a vassoura, o ledger, este handoff):
nenhum toca Rust, logo o binário continua válido para o comando do §6.

### Auditoria — as duas lentes

**Lente 1 — «cada gate novo SANGRA quando a propriedade quebra?»** Catorze mutações, cada uma com
UM teste corrido (controlo no filtro) e a árvore restaurada por `git checkout` sobre commit:

| mutação | quem sangra |
|---|---|
| M1 — uma porta passa a consultar a cena | `the_host_never_reads_the_borrowed_scene` («fora da licença») |
| M2 — a rota do chrome passa a consultar a cena | idem («a leitura licenciada mudou de forma») |
| M3 — uma porta nova no trait | idem («o trait mudou de portas») |
| M4 — um invólucro sem cena responde `true` | `the_sculpt_host_refuses_without_a_scene` |
| M5 — um `return` entre o `take` e a devolução | idem («caminho de fuga») |
| M6 — o gémeo sem a feature responde `true` | idem |
| M7 — o monómio `uv` vira `u²` na avaliação | `the_surface_is_the_same_in_any_tangent_frame` (**8,53e-2** contra `2,98e-8` do produto) |
| M8 — um campo novo na peça do documento | `the_shape_of_a_saved_scene_is_pinned` (**1539** contra 1538) |
| M9 — uma citação de gate inexistente | `every_gate_the_sculpt_family_names_exists` |
| M9b — a mesma, com uma `fn` escrita só em comentário | idem (a cura do §8.3) |
| M10 — a memória deixa de ser citada | `every_memory_of_a_dead_gate_still_describes_something` |
| M11 — a memória passa a existir | idem |
| M12 — um chip de motor de retopologia a mais | `the_panel_offers_every_retopo_mode_the_engine_has` |
| M13 — o dab deixa de zerar a janela | `an_idempotent_dab_does_no_work` (61 movidos herdados) |

**Lente 2 — «a parede segurou?»** R independente decidiu a régua · quarentena comparada · plano de
reescrita varrido e verificado por grep · a janela nunca abriu fonte, ledger nem vassoura · diff
só-comentário provado · resíduo contado às cegas · vassoura emendada e sweep **com controlo
positivo** (§4.7).

---

## §11 — Resumo colável

```
line/sculpt3d — fecho 2026-09-13 (reabertura pós-W2)
- 8 gates citados em comentário NUNCA existiram: 6 escritos, 2 reapontados, 7 nomes antigos,
  + censo de recaída (named_gates_census_tests). 14 mutações, 14 sangram.
- a dívida §2.10 da Fase B (o empréstimo da cena) FECHOU: gate por licença das 6 portas.
- INC-4 (clean-room): RELANCE por R independente; 173 blocos em 59 ficheiros reescritos sem
  nomes internos do alvo; 2 citações de comentário apagadas sem leitura; vassoura emendada.
- zero schema, zero registo, zero contrato, zero ADR, zero pacote; shell ±0 LOC.
- smoke: sanidade da cena =39 (nada muda no ecrã). Binário compilado na worktree.
```
