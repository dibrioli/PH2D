# Integração da rodada 04 — 2026-10-02 (sete linhas, em série)

> Ordem explícita do Enio (CLAUDE.md §0.7): integrar em série, cada integrador espera o sinal do
> anterior em `~/.ph2d/integracao-2026-10-02/<n>-<linha>.feito` (o sha do `main`) ou `.falhou`; o
> último verifica a rodada e faz o SHIP + CI. Este registo é escrito pelo integrador da posição 7
> (`line/components`).

## §1 — A ordem e os sinais

| # | linha | `main` depois (sinal) | §5 |
|---|---|---|---|
| — | (antes da rodada) | `912a9652e` | — |
| 1 | `line/UIUX` | `35ada67ca` | `35ada67ca` |
| 2 | `line/3DModeling` | `2d2c60b31` | `2d2c60b31` |
| 3 | `line/PainterWatercolor` | `85044ff93` | `85044ff93` |
| 4 | `line/Vector` | `9ec276d55` | `9ec276d55` |
| 5 | `line/motion-value` | `8aa225ce6` | `8aa225ce6` |
| 6 | `line/sculpt3d` | `4022091e5` | `4022091e5` |
| 7 | `line/components` | `bfce2fe7d` | `bfce2fe7d` |

Verificado no fim: os sete `.feito` existem e não há `.falhou`; `git merge-base --is-ancestor
line/<x> main` é verdade para as sete; o `CLAUDE.md §5` tem a linha de cada uma (oito, contando as
duas da `line/components`: VIDA W6/W7, que nunca tinha entrado, e NAVEGAÇÃO); zero marcadores de
conflito (incluindo `|||||||`) em `*.rs` e `*.md`; o bloco temporário `merge=text` de
`.git/info/attributes` foi **retirado** (o ficheiro já não existe).

## §2 — Contadores, antes e depois (contados no ficheiro, nunca na coluna `base:`)

| contador | `912a9652e` | `main` final | quem o moveu |
|---|---|---|---|
| `PROJECT_SCHEMA` | `176` | `178` | `line/components` (+1 VIDA W6, +1 NAVEGAÇÃO) |
| `VEC_SCENE_SCHEMA_VERSION` · `FLIP_SCHEMA_VERSION` · `DOC_VERSION` (timeline) · `FIELD_DOC_VERSION` | `22` · `13` · `18` · `23` | iguais | ninguém |
| `SCULPT_DOC_VERSION` (dentro do blob, não no `ProjectFile`) | `3` | `5` | `line/sculpt3d` (o relevo do impasto) |
| registo do `ph2d-ecs` · espelhos `render`/`script` | `108` · `109` · `109` | iguais | ninguém |
| registo da FÍSICA (`ph2d-physics-ecs`, o `collision-surface.sh` não o mostra) | `40` | `42` | `line/components` (`NavRegion`, `NavAgent`) |
| `LIVE_SECTIONS` | `43` | `45` | `line/components` (NAV REGION, NAV AGENT) |
| catraca de comandos do Inspector (`quantas_entradas_tem_cada_painel`) | `81` | `85` | ver §3.1 — a `line/UIUX` deixou `78`, que era um **recorte** |
| catracas de cortes · letras do Inspector (`nenhum_rotulo_do_app_pinta_nada`) | `87` · `83` | `89` · `85` | `line/components` (VIDA W6) |
| índice da memória (`project-memory/MEMORY.md`, tecto `22 000` B) | — | `21 983` B | ver §3.4 |
| membros em `crates/` | `381` | `384` | — |
| ADR | `0175` | `0175` | ninguém criou ADR |

## §3 — O que só a árvore COMBINADA mostrou (posição 7)

### §3.1 — A catraca do Inspector media a borda da janela, não o painel (UIUX × components)

A `line/UIUX` baixou a catraca de comandos do Inspector de `81` para `78`. Isso **não** era o painel
a encolher: o Inspector armado do `main` media `16 203 px` e a `VIEWPORT` do censo parava nos
`16 000` ⇒ três comandos do fundo ficavam fora do índice de acerto. A folga de `4×` escrita ao lado
da `VIEWPORT` tinha **envelhecido sem ninguém a medir**: o Inspector já estava a `15 968 px` no
`main` da UIUX, e as secções da VIDA W6 e da NAVEGAÇÃO empurraram-no para lá da borda. Quem acusou
foi a igualdade da `a_ordem_das_seccoes_e_a_da_paleta` (*«24 declaradas, 21 pintadas»*) e a catraca
(`79` contra `82`); o piso de população **não** acusou.

Cura: a `VIEWPORT` vai a `32 000` nos três censos que a partilham (`quantas_entradas_tem_cada_painel`,
`a_marca_tem_a_altura_da_linha`, `onde_comeca_o_valor`). O `main` medido com a janela nova (por uma
edição temporária, revertida, na worktree da UIUX) dá `81`; a VIDA W6 soma o DELTA `+4` ⇒ `85`. A
cura foi **dobrada no commit W6 rebaseado** (edit-rebase), não deixada num commit por cima.

⚠️ Os censos do `ph2d-panel-registry-init` só são honestos em âmbito de **workspace**
(`cargo nextest run --workspace -E "package(ph2d-panel-registry-init)"`): painéis como `painter_layers`,
`wet_tuning` e `flip` só entram por unificação de features, e um `-p` sozinho dá vermelhos de «âmbito
pobre» sobre produto certo.

### §3.2 — As secções da VIDA e da NAVEGAÇÃO no idioma da UIUX (UIUX × components)

A UIUX trocou a API do Inspector (`begin_section`/`finish_section`/`close_section`/`section_tops_y`
morreram) pelo `Plano` + `emoldurada`, e os ids `INSP_LIVE_*_COLOR` por `INSP_LIVE_*_GRIP`. As duas
secções NAV (`paint_optional_nav.rs`, reescrito) e a VIDA (`push_vida_sections`, com o
`resistencia_aberta` da W6) foram portadas para esse idioma no próprio rebase; o piso de um campo é
`number_input_min_w_px()`, nunca a `const`. O Mergiraf disse «Solved» no `paint.rs` e no
`ids/inspector_camera.rs`, e os dois foram revistos à mão.

### §3.3 — O mesmo defeito achado duas vezes sem as linhas se verem (3DModeling × components)

As duas linhas acharam o servidor do `sccache` a segurar o cadeado da placa: a `line/components` em
29/09 (três linhas presas 21 min, `.dono` vazio) e a `line/3DModeling` em 01/10 (14 min). As curas
são **complementares** e ficaram as duas no `scripts/ph2d-run.sh` com **uma** explicação só:
`9>&-` no lançamento do comando (a cura geral: nenhum daemon herda o fd) e o servidor arrancado
ANTES do `exec 9>` e fora da fatia (a cura do `sccache`: nascido dentro do `systemd-run --scope` ele
morreria com a fatia de uma linha), mais o `fuser` na mensagem de espera quando o `.dono` está vazio.
As duas memórias fundiram-se numa
([`feedback_a_daemon_born_under_the_gpu_door_inherits_its_lock.md`](../../../project-memory/feedback_a_daemon_born_under_the_gpu_door_inherits_its_lock.md)),
com um ponteiro só no índice; a da `line/components` foi apagada e a entrada dela no tópico de ship
retirada.

### §3.4 — O índice da memória passou o tecto por uma linha alongada

O 1.º `foundational-integrate.sh` reprovou **um** teste de `19 078`:
`o_indice_cabe_no_orcamento_do_carregador` — `22 037` B contra `22 000`. A fusão das memórias (§3.3)
cabia (`21 992`); quem passou foi o commit VIDA W7, que alongou a linha das provas de mutação em
`45` B. Encurtada **no próprio commit** (fixup + autosquash, comparação multiconjunto a dar só essa
linha), o índice fica em `21 983` B e o 2.º portão deu `19 078 / 19 078`.

### §3.5 — Das posições 1–6

O único commit de integração além dos do §5 é o `ed0297889` (posição 3): dois comentários da
`field3d` citavam o arnês `muta_perto.py` e a régua das citações a fonte restrito lia-o como fonte
alheio. O resto do que cada integrador anterior achou está nos commits e handoffs da linha dele.

## §4 — A prova da posição 7

| passo | resultado |
|---|---|
| rebase da `line/components` (16 commits) sobre `4022091e5`, com `merge=text` nas listas e catracas | 1 conflito (`project-memory/MEMORY.md`, resolvido pelos estágios); `scripts/ph2d-run.sh` fundiu limpo com as duas curas; saída inteira gravada |
| comparação multiconjunto `+/-` de cada commit rebaseado com o original (controlo positivo) | só diferem os dois commits resolvidos à mão (o do `ph2d-run.sh` e o W7 da memória) |
| `Cargo.lock` | só `+ph2d-nav` e `+ph2d-navmesh` contra o `main` (zero pacote externo novo) |
| `scripts/censos-da-arvore-combinada.sh` | `127 / 127`, controlo do filtro `12 de 12` |
| `scripts/foundational-integrate.sh` | 1.ª corrida `19 077 / 19 078` (§3.4); 2.ª `19 078 / 19 078`, `--ff-only` aterrou |
| cada commit compila sozinho (`cargo check --workspace --all-targets`, avisos = erro) | `13 / 13` verdes, **zero** vermelhos (os 3 commits só de docs/script saltados por não terem `.rs`/`.toml`/`Cargo.lock`) |

**Faixas a saltar num `git bisect`:** **nenhuma** na `line/components` — as duas curas (a `VIEWPORT` e a linha do índice) foram dobradas nos commits que as causaram, não deixadas por cima.

## §5 — Flake promovida ao §5.0

`text_path_smoke::perf::riding_the_path_costs_about_twice_the_straight_layout` (shell, gate de razão
de dois relógios), a pedido da `line/components`: reprovou uma vez em `18 735` com outra linha na
placa, 3/3 verde sozinho a `load 7,3`, zero linhas de diff daquela linha no módulo.
