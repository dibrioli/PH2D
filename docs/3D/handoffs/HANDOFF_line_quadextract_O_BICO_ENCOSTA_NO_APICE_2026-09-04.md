# HANDOFF — `line/quadextract`: a régua POR PONTA, e o BICO que encosta no ápice (2026-09-04)

> **Estado:** a linha entrega a resposta ao report de 04/09 (*«muitas pontas boas no mesmo mesh
> e apenas uma ruim — como tornar uniforme?»*): a régua que diz **qual**, o mecanismo medido, e
> a cura.
> ⛔ Nada foi integrado nem pushado (`CLAUDE.md` §0.7).
> Worktree: `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract` · ramo `line/quadextract`.
> Mecanismo completo com as tabelas: [plano §108](../quad-remesh/PLANO_a_graduacao_da_ponta.md).
> O que veio antes: [a calota e a gravata](HANDOFF_line_quadextract_A_CALOTA_E_A_GRAVATA_2026-09-03.md).

## §1 — O que mudou, numa frase

As três réguas da ponta passam a ter uma **tabela por ápice** ([`ph2d_quadfill::tip_rows`], de
que as duas agregadas são **dobras**), e o acabamento ganha um **remate**
([`ph2d_quadfill::snap_tips`]): o vértice mais próximo de cada bico afiado encosta **no ápice da
escultura** quando está a menos de meia célula dele. Na peça do dono as cinco pontas passam a
ler `gap 0,00` — antes `0,08`–`0,20` conforme a ponta, e `0,47` na realização que ele exportou.

## §2 — Como reproduzir

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract
cargo test -p ph2d-quadfill --lib tip_rows:: tip_snap::                 # 9 gates, < 1 s
cargo test -p ph2d-quadfill --test pontas_do_dono -- --nocapture        # o portão dos dois lados
# a tabela por ponta de QUALQUER par de malhas, sem correr o botão:
cargo run -p ph2d-quadfill --release --example pontas -- \
    --entrada ~/Downloads/sculpt-pre.obj ~/Downloads/sculpt003.obj
# e o botão de ponta a ponta (imprime a tabela em F1 e na SAIDA):
CARGO_INCREMENTAL=0 cargo test -p ph2d-host-desktop --release --bins --no-run
env PH2D_PIECE=/home/enio/Downloads/_base_sculpt.obj PH2D_RECENTER=1 PH2D_DETAIL=1.0 PH2D_ADAPT=1.0 \
    ./target/release/deps/ph2d_host_desktop-<hash> \
    sculpt3d::history::undo::photo_probes::button::the_artists_piece_through_the_button \
    --ignored --nocapture --exact --test-threads=1
```

⚠️ **O binário é o que ESTA linha imprime** no fim do `--no-run`; nunca `ls -t`.

## §3 — A régua: uma linha por ponta

⛔ **Nenhuma das três réguas sabia dizer QUAL.** A [`tip_deviation`] devolve o pior `p50`
**entre** as pontas, a [`tip_density`] a mediana e o pior, a [`tip_survival`] um extremo global
— *as três já mediam ponta a ponta e deitavam fora o índice antes de devolver*. É a **quarta**
vez que esta linha paga a mesma forma (o `edge_max` global, o `χ` cego à almofada, a `ENTREGA`
cega à ponta que engrossou).

`tip_rows(entrada, saída, unit)` devolve, por ápice: `gap` (a amputação), `grade` (a grade do
bico), `cone` (a forma), `dev` (`p50`/`p90`/máximo), `blind` (a ponta comida por inteiro, cujo
número é um **piso**) e o **pólo** (irregulares a `≤ 2 h` e a valência do bico, a coluna do
mecanismo do §101). ⚠️ As duas agregadas são dobras dela e **os números do produto não mudam**.

## §4 — O que ela achou nas malhas do PRÓPRIO dono

`sculpt-pre.obj` → as duas saídas que ele exportou (`h = 0,0167`):

| ápice | raio | `sculpt003` (03/09) | `sculpt-Pos-Remesh` (a anterior) |
|---|---|---|---|
| ⛔ **`8042`** | **`1,808`** | `gap` **`0,47`** · grade `0,79` | ⛔⛔ **comida** (`gap ≥ 3` · grade `3,49`) |
| `9218` | `1,070` | `0,27` · `0,66` | `0,15` · `0,66` |
| `12279` | `1,052` | `0,08` · `0,64` | `0,26` · `0,40` |
| `11108` | `0,924` | `0,08` · `0,76` | `0,05` · `0,62` |
| `15622` | `0,846` | `0,15` · `0,73` | `0,10` · `0,85` |

⭐⭐⭐ **A ponta ruim é sempre a MAIS LONGA**, e é a pior em todas as colunas nas duas saídas. As
agregadas liam `0 de 5` na primeira: *o relatório dizia «perfeito» sobre a ponta da foto.*

## §5 — ⭐⭐⭐ O mecanismo: o defeito nasce no ACABAMENTO, e é FASE, não célula

A **mesma** extracção (`21 914` faces), acabada de duas maneiras (`PH2D_CANDIDATE_DUMP`):

| acabamento | `gap` no `8042` | grade no bico | as outras quatro |
|---|---|---|---|
| ⭐ cerca de viagem apertada (a entregue) | `0,18` | `0,46` | `0,08`–`0,20` |
| ⛔ cerca larga | **`0,51`** | `0,88` | `0,09`–`0,29` |

⭐⭐ **A grade do bico está dentro da barra nas duas** — a calota de 03/09 deu-lhe resolução. O
que muda é onde a última volta da grade **pára**, e a causa tem nome: *o acabamento pousa cada
vértice na escultura, e a projecção ao ponto mais próximo de uma superfície **nunca escolhe o
ápice*** — o bico é um ponto de medida nula e o pé da perpendicular cai no **flanco**. Cada
ronda embota a ponta, e *quanto* depende da cerca que aquela saída calhou de usar. **É por isso
que uma ponta sai boa e a vizinha não.**

## §6 — A cura, e as três cercas que a tornam honesta

[`snap_tips`] corre no fim das **três** saídas do acabamento (gate:
`as_tres_saidas_do_acabamento_desfazem_o_avesso`, que agora conta as três curas):

1. **`gap ≤ TIP_GAP_MAX`** (meia célula) — ⛔ acima da barra o que falta é **célula**, e mudar um
   vértice esconderia do selector um defeito que ele tem de ver;
2. **viagem `≤ 1` célula** — o `gap` mede ponto→FACE e o vértice mais próximo pode estar mais
   longe que ele;
3. **o censo GLOBAL não pode subir** (faces `> 60°` e faces do avesso), **uma ponta de cada
   vez** — aceitar as cinco de uma vez faria uma má vetar quatro boas.

⚠️ **A cerca 3 não é uma precaução: é a diferença entre duas fixturas.** Num bico que é uma
**lasca** (leque de `8` a `15°`) devolver o ápice ao sítio leva as faces péssimas de `8` para
`16` e o remate **recusa**; na malha que o produto entrega o censo fica **intacto** (`>60`
`8 → 8`, avesso `2 → 2` nas cinco pontas). Os dois casos são gate.

## §6-bis — A tabela que autorizou ligar (peça do dono, `Detail 1` · `Curv 1`)

| | sem o remate | ⭐ com o remate |
|---|---|---|
| pior `gap` entre as `5` pontas | `0,18` | ⭐ **`0,00`** (as cinco) |
| grade no bico (pior) | `0,74` | `0,83` (barra `1,0`) |
| aspecto p50 / p99 | `1,15` / `1,88` | ⭐ **`1,09` / `1,57`** |
| enviesamento p50 / p99 | `4,3°` / `29,6°` | ⭐ **`3,0°` / `20,0°`** |
| faces `> 60°` | `8` | ⭐ **`4`** |
| **tentativas do botão** | `9` | ⭐⭐ **`4`** |
| **relógio** | `337 s` | ⭐⭐⭐ **`123 s`** |
| topologia | `χ 2` · `0` bordo · `0` não-manifold · `21 914` quads | **idêntica** |

⭐⭐⭐ **O relógio e a forma são consequência.** A cascata pára quando uma candidata satisfaz as
réguas: com a fase fechada, a **segunda** tentativa já as satisfaz. Antes eram precisas **nove**
para achar uma que ganhava por `0,18` contra `0,51` no bico — e essa pagava a diferença em
forma, porque a cerca de viagem que protege a ponta é a mesma que impede a relaxação de
trabalhar. *O remate tira a ponta da disputa, e o selector passa a escolher pela forma.*

⚠️ `PH2D_TIP_SNAP=0` bissecta (a porta mora no acabamento e alcança a bancada de propósito: a
comparação precisa das duas metades a correr a MESMA cadeia).

## §7 — ⛔ Recusas MEDIDAS desta wave

- **`PH2D_TIP_ALIGN=5` COM a calota** — a célula `(1,1)` que o [§103](../quad-remesh/PLANO_a_graduacao_da_ponta.md)
  encomendou. Corrida: `3` de `7` candidatas com furos (`40` e `26` arestas de bordo), **todas**
  as sete com `p90 3,00` numa ponta, `2`–`5` de `5` com a grade grossa. ⇒ o reforço fica
  **instrumento**, não cura.
- **Rematar acima da barra** (uma ponta comida) — seria trocar um bico chato por um **espeto de
  uma célula**, e apagar do selector um defeito de células. Gate.
- **Rematar um bico-lasca** — ver §6, cerca 3. Gate.

## §8 — ⚠️ O que uma leitura rápida do diff entende ao contrário

1. **`tip_deviation` e `tip_density` mudaram de corpo e NÃO de resultado** — elas são dobras da
   tabela, e há gate que dobra a tabela à mão e exige o mesmo número.
2. **O remate não é o *«puxar o vértice mais avançado»* refutado em 31/08.** Aquele deslocava
   `0,6198` de mundo (**`23` células**) sobre um bico cuja grade estava a `3,85 ×` o alvo, e
   levava o aspecto a `12,11`. Este desloca **meia célula** sobre uma grade que já tem
   resolução, e o censo global tem de ficar igual ou melhor.
3. **A `unit` do remate é a MEDIANA da malha, não o alvo do slider** — quem acaba não conhece o
   slider (a `ph2d-quadchain` e os gates chamam a porta sem ele).
4. **A coluna do `pólo` na tabela não é a régua de nada, ainda** — ela é o instrumento do §101 e
   não discrimina as pontas boas das más nesta peça (a `8042` lê `0,4` como duas pontas sãs).
5. **A fixtura dos gates tem DOIS espinhos de propósito** — com um só, *«a tabela diz qual»* é
   vazio: a única linha é sempre a acusada.

## §9 — O que fica ABERTO, com endereço

- ⏳ A **`sculpt_antes`** (a agulha) continua com `1/4` amputada — ali o que falta é **célula**, e
  o remate recusa-se por desenho.
- ⏳ O **sorteio dos últimos bits** continua (§104): a mesma escultura noutra escala dá outra
  realização, e o remate fecha a fase mas não muda a candidata que vence.
- ⏳ O `pólo` medido (`0,4` no bico da ponta mais longa) diz que a grade **atravessa** o bico em
  vez de fechar nele; a wave que o cura é a do §101–§103 e não esta.

## §10 — O diff

| ficheiro | o quê |
|---|---|
| `crates/ph2d-quadfill/src/tip_rows.rs` + `_tests.rs` | a tabela por ponta e 4 gates |
| `crates/ph2d-quadfill/src/tip_snap.rs` + `_tests.rs` | o remate e 5 gates |
| `crates/ph2d-quadfill/src/tips.rs` | as duas agregadas viram dobras |
| `crates/ph2d-quadfill/src/finish_extract.rs` | o remate nas três saídas + `FinishReport::snapped` |
| `crates/ph2d-quadfill/src/untangle_tests.rs` | o censo das três saídas passa a contar o remate |
| `crates/ph2d-quadfill/examples/pontas.rs` | o instrumento (`--recentrar`, `--unit`, `--rematar`) |
| `shells/desktop/src/sculpt3d_photo_button.rs` | a tabela na sonda, em `F1` e na `SAIDA` |
| `shells/desktop/src/sculpt3d_retopo_one.rs` | o acabamento diz o que fez (`untangled` · `snapped`) |
