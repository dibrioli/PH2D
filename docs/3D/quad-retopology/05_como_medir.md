# 05 — Como MEDIR: os instrumentos, com os comandos copiáveis

> ⚠️ **Uma ferramenta que nenhum passo escrito chama pelo NOME morre** (`CLAUDE.md` §2). Esta
> página é o passo escrito. Todos os comandos assumem a worktree da linha:
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract`.

## §1 — A sonda que corre a PORTA DO PRODUTO

Ela chama o mesmo `Sculpt3dScene::quad_remesh_global` que o botão chama — ⛔ não uma cópia da
ordem dele.

```bash
CARGO_INCREMENTAL=0 cargo test -p ph2d-host-desktop --release --bins --no-run
# ⚠️ o binário é o que ESTA linha imprime; NUNCA `ls -t` (apanha o executável do PROGRAMA)
env PH2D_PIECE=/home/enio/Downloads/_base_sculpt.obj PH2D_RECENTER=1 \
    PH2D_DETAIL=1.0 PH2D_ADAPT=1.0 \
    PH2D_DUMP=/var/tmp/ph2d-medicoes/saida.obj \
    PH2D_DUMP_F1=/var/tmp/ph2d-medicoes/f1.obj \
    ./target/release/deps/ph2d_host_desktop-<hash> \
    sculpt3d::history::undo::photo_probes::button::the_artists_piece_through_the_button \
    --ignored --nocapture --exact --test-threads=1
```

⛔⛔ **`PH2D_RECENTER=1` não é opcional.** O importador **recentra** a malha e põe escala e
posição numa `Pose` que só desenha e exporta ⇒ *o botão vê SEMPRE a peça centrada*. Uma sonda
que alimente o ficheiro cru mede **outro programa** — isto mordeu **quatro** vezes em dois dias.

O relatório imprime, em ordem: o censo da `ENTRADA`, o alvo do slider ao lado da aresta do `F1`
(a razão `ALVO/F1`), a `F1 CALOTA`, **a tabela por ponta em `F1` e na `SAIDA`**, cada candidata
numa linha, e as réguas da saída.

## §2 — A tabela por ponta, de QUALQUER par de malhas

Não precisa de correr o botão: compara ficheiros `.obj` já exportados.

```bash
cargo run -p ph2d-quadfill --release --example pontas -- \
    --entrada ~/Downloads/sculpt-pre.obj --bandas \
    ~/Downloads/sculpt003.obj ~/Downloads/sculpt-Pos-Remesh.obj
```

| bandeira | o que faz |
|---|---|
| `--entrada <obj>` | a **escultura** (a lei do ápice corre sobre ela) |
| `--recentrar` | passa a entrada pela porta do importador — use sempre que a comparar com dumps do botão |
| `--unit <h>` | fixa a unidade (o alvo do slider); sem ela, a mediana da 1.ª malha |
| `--bandas` | varre `0–3`, `3–6`, `6–12`, `12–24 h` **e conta o anel** (quantas faces dão a volta) |
| `--rematar` | aplica o [`snap_tips`] antes de medir — responde *«o remate pega nesta peça?»* sem correr o botão |

⚠️ **A unidade é a MESMA em todas as malhas comparadas**: a lei do ápice decide *o que é um
espinho* à escala da unidade, então duas unidades dariam dois **censos** e as tabelas não se
leriam lado a lado.

## §3 — As faces do avesso, de qualquer malha

```bash
cargo run -p ph2d-quadfill --release --example dobras -- <malha.obj> [outra.obj …]
cargo run -p ph2d-quadfill --release --example dobras -- --curar <escultura.obj> <malha.obj>
```

Imprime, por ficheiro: as faces que apontam contra a vizinhança, **o maior GRUPO delas** (é o
grupo que se vê como fenda, não a contagem), as gravatas e a forma mediana.

## §4 — As portas de bissecção

| env | o que ela devolve |
|---|---|
| `PH2D_RETOPO_EXTRACT=0` | o motor de patch (o de antes da extracção) |
| `PH2D_RETOPO_LEGACY=1` | o motor **local** (Instant Meshes) |
| `PH2D_ISO_ADAPT=0` | a fase zero **uniforme**, sem graduação |
| `PH2D_TIP_CAP=0` · `PH2D_TIP_CAP_R=<x>` | sem calota · com outro alcance |
| `PH2D_TIP_SNAP=0` | sem o remate do bico |
| `PH2D_EXTRACT_FINISH=0` | sem acabamento nenhum |
| `PH2D_EXTRACT_TRAVEL=<x>` | a cerca de viagem do acabamento |
| `PH2D_GRIDMAP_WELD=0` | a costura penalizada (o de antes da eliminação) |
| `PH2D_EXTRACT_MIRROR=0` | não descarta as almofadas |
| `PH2D_RETOPO_TIPKEY=0` | o selector sem a chave da ponta |
| `PH2D_SIZING_RATIO=<n>` · `PH2D_SIZING_SMOOTH=<n>` | a faixa e o alisamento do campo de tamanho |
| `PH2D_CANDIDATE_DUMP=<dir>` · `PH2D_SING_DUMP=<dir>` | grava **cada candidata** · as singularidades do campo |
| `PH2D_ISO_LOG=1` | a coluna `NO PISO` do campo de tamanho |

⚠️ **A cerca de viagem é escolhida pelo CHAMADOR** (o botão), não lida dentro da biblioteca —
uma env lida lá dentro alcançava a bancada, os gates e o produto de uma vez. O
`PH2D_TIP_SNAP=0` é a excepção **declarada**: ele é diagnóstico, e a comparação precisa das duas
metades a correr a MESMA cadeia.

## §5 — Os portões (o que corre no fecho da linha)

```bash
bash scripts/cargo-test-narrow.sh ph2d-quadfill          # a crate das réguas e do acabamento
cargo test -p ph2d-quadfill --test pontas_do_dono -- --nocapture   # o portão dos DOIS lados
cargo test -p ph2d-host-desktop --bins --tests           # 4 452 + os binários de integração
cargo clippy -p ph2d-quadfill -p ph2d-host-desktop --all-targets -- -D warnings
```

⭐ **O portão `pontas_do_dono` é o que impede a régua de deslizar:** ele mede as fixturas dos
**dois lados** — a retopologia que o dono APROVOU (que tem de passar) e as duas que ele
REPROVOU (que têm de falhar) —, com as margens exigidas.

## §6 — Onde as medições ficam

`/var/tmp/ph2d-medicoes/` — ⛔ **nunca** no scratchpad da sessão: ele foi limpo a meio de uma
jornada **três** vezes, levando scripts e logs. O que for reutilizável vira **exemplo in-tree**
(`examples/pontas.rs`, `examples/dobras.rs`), que é o que sobrevive à limpeza e ao fim da sessão.
