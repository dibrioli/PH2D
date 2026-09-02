# HANDOFF — `line/quadextract` · **A RÉGUA QUE CONCORDA COM O OLHO DO DONO**

> **Data:** 2026-09-02 · **Branch:** `line/quadextract` · **Base:** `066b4f92e` · **HEAD anterior:** `d7cefcbf0`
> **Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract`
> **Ordem recebida:** *«o remesh embota e amputa as pontas dos espinhos […] Comece pelo §0 do
> handoff — a régua que concorde com a foto — e NÃO toque no algoritmo antes de a ter.»*
> **Estado:** a régua existe, está gateada nos dois lados que o dono julgou, e o produto lê-a.
> ⛔ **O algoritmo NÃO foi tocado.** ⛔ Não integrado, não pushado (`CLAUDE.md` §0.7).
> Mecanismo e tabelas: [`PLANO_a_graduacao_da_ponta.md` Parte XII](../quad-remesh/PLANO_a_graduacao_da_ponta.md).

---

## §0 — O que esta janela provou, em três frases

1. **As réguas separavam a malha aprovada das reprovadas — o que nunca fora feito era corrê-las no
   lado aprovado.** Corridas, mostraram dois defeitos de calibração, não de algoritmo: o **piso do
   ápice** (`0,55` do raio) escondia as pontas da foto (`0,43`–`0,51` do raio), e a **barra da
   grade** (`1,5`) fora calibrada só com a nossa saída (a aprovada entrega `≤ 0,79`).
2. **A saída do botão não muda; o veredito muda.** Na peça do dono a `Detail 1,00` a mesma malha
   de sempre passa a ler **RED** no espinho `3138` (grade `1,36`), e nenhuma das nove candidatas o
   cura sem amputar outra. Na entrada da malha aprovada, **todas as nove** comem o espinho
   principal. *O selector já não tem onde escolher.*
3. **⛔ `Sculpt_Blender.obj` é a retopologia de `sculpt_antes.obj`, não de `_base_sculpt.obj`** —
   o handoff anterior emparelhava peças diferentes.

## §1 — O que ficou construído (tudo com gate, `cargo test -p ph2d-quadfill` verde: 67 · shell 4 786)

| o quê | onde | o que muda |
|---|---|---|
| **Lei do ápice** com piso `0,25`, filtro de FORMA (`CONE_MAX = 1,0`) e unidade | `ph2d_quadfill::apex::{apices, cone_of, path_ball, adjacency, median_edge}` (módulo novo, cortado de `local.rs` pelo tecto de 700 LOC) | as três réguas da ponta medem os espinhos afiados e **não** as bossas nem os botões aprovados grossos |
| **Amputação no ÁPICE** | `TipDeviation::{apex_max, cut}`, `TIP_GAP_MAX = 0,5` | a `p50` deixava passar a agulha `15909` (`0,84` com o bico a `1,11`) |
| **Barra da grade** | `TIP_DENSITY_MAX = 1,0` (era `1,5`) | vazio `0,88…1,10`, calibrado nos DOIS lados |
| **Selector** | `decide::worse`: chave `cut` antes de `over`; `rulers::still_broken` arma por `cut` | a amputação decide antes da mediana |
| **Unidade** | produto = **alvo** (`one.rs`); bancada/sondas = **mediana** (`median_edge`) | censo idêntico entre candidatas; comparável com outra ferramenta |
| **Portão dos dois lados** | `ph2d-quadfill/tests/pontas_do_dono.rs` + `tests/fixtures/pontas/` (5 `.obj.gz`, README com proveniência) | RED nas reprovadas, GREEN na aprovada, margens exigidas |
| **Dijkstra com heap** | `apex::path_ball` | `71 s → 0,5 s` numa fixtura |
| Sondas | `photo_rulers::tips` usa a lei da casa (tinha cópia própria); `photo_button` imprime `AMPUTADAS`/`GRADE` | |
| Instrumentos (scratch, fora da árvore) | `regua_ponta.py` (bateria por ponta sobre pares `.obj`), `render_ponta.py` (arame de cada ponta, lado e frente, PNG sem dependências) | ⚠️ vivem em `/tmp/claude-1000/…/scratchpad/`; se forem precisos, recriar a partir do plano §92/§96 |

## §2 — Como reproduzir

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract
# o portão dos dois lados (0,8 s em debug)
cargo test -p ph2d-quadfill --test pontas_do_dono -- --nocapture
# a peça recentrada (o botão vê a peça RECENTRADA — Parte VII)
python3 - <<'PY'
src="/home/enio/Downloads/_base_sculpt.obj"; dst="/tmp/base_recentrada.obj"
L=open(src).read().splitlines(); V=[tuple(map(float,l.split()[1:4])) for l in L if l.startswith("v ")]
c=[(min(p[k] for p in V)+max(p[k] for p in V))/2 for k in range(3)]; it=iter(V); out=[]
for l in L:
    if l.startswith("v "): p=next(it); out.append(f"v {p[0]-c[0]:.6f} {p[1]-c[1]:.6f} {p[2]-c[2]:.6f}")
    else: out.append(l)
open(dst,"w").write("\n".join(out)+"\n")
PY
cargo test -p ph2d-host-desktop --release --bins --no-run   # 1×, e NÃO reconstrua durante uma medição
env PH2D_PIECE=/tmp/base_recentrada.obj PH2D_DETAIL=1.0 PH2D_ADAPT=1.0 PH2D_DUMP=/tmp/saida.obj \
    ./target/release/deps/ph2d_host_desktop-<hash> the_artists_piece_through_the_button --ignored --nocapture --test-threads=1
```

A linha a ler é `AMPUTADAS: … | GRADE NA PONTA: pior …` no fim, e a `GRADE NA PONTA` de cada
`candidata` no meio. `sculpt_antes.obj` recentrada corre da mesma maneira.

## §3 — O que o produto lê HOJE (mesma saída de antes)

| peça · `Detail` | escolhido | amputadas | grade pior | relógio |
|---|---|---|---|---|
| `_base_sculpt` · `1,00` | `21 747` quads | `0/5` (gap `0,31`) | **`1,36`** (`3138`) ⇒ RED | `251 s` (era `103`; o socorro arma) |
| `_base_sculpt` · `0,75` | `5 287` | `1/6` (`15909`, gap `1,02`) | `0,96` | `142 s` |
| `sculpt_antes` · `1,00` | `19 154` | `1/4` (`4849`, gap `3,00` = piso) | `5,41` | `262 s` |

## §4 — ⛔ Recusas medidas (plano §99) — não as reconstrua

`TIP_DENSITY_MAX = 1,5` · piso `0,55` com corte `12` · cone sem `h` · razão de área · faixa
`2,5`–`4,5 h` · `CONE_MAX = 1,5` (ou `1,2`: mete o botão `7328` aprovado dentro por `0,02`) ·
unidade = mediana da candidata no produto · alinhamento ao meridiano como discriminador · Dijkstra
por pilha.

## §5 — ⭐ Por onde eu começaria (a obra seguinte é ALGORITMO, e agora tem régua)

1. **Meça o campo cruzado junto de cada ápice contra as direcções principais** (num cone:
   meridiano e anel), nas duas peças e no campo do oráculo (`ph2d-quadbench/ref/*.rosy`, fora da
   árvore). O arame (plano §96) mostra a grade a atravessar o espinho em **diagonal** onde termina e
   a seguir o meridiano onde converge — é a única pista de mecanismo que esta janela tem, e ainda
   **não é uma medição do campo**.
2. O alvo mais alto é `sculpt_antes` / `4849`: a fase zero já corta `−3,0 %` (`ALVO/F1 = 0,39×`)
   e a extracção perde o resto. É a peça de que o dono guardou a retopologia aprovada — o portão
   `pontas_do_dono` é o critério de aceitação: **a nossa saída sobre `sculpt_antes` tem de passar a
   mesma asserção que a `Sculpt_Blender.obj` passa.** Uma fixtura com a nossa saída pode entrar no
   mesmo portão quando ela passar.
3. Não reconstrua o que o §3 do handoff de 01/09 e o §99 do plano já recusaram.

## §6 — Higiene

- ⛔ **Não integre, não pushe.** Commits locais sobre `d7cefcbf0`.
- ⛔ **Clean-room activo** (ADR-0162/0167): as cinco fixturas e os ficheiros novos passaram
  `bash scripts/cleanroom-sweep.sh docs/3D/cleanroom/VASSOURA_quadwild.txt …` ⇒ limpo (94 entradas).
  `Sculpt_Blender.obj` é **saída** de programa (dados), com a proveniência no README das fixturas.
- **Contratos congelados encostados:** nenhum. **Foundational tocado:** `ph2d-quadfill` (módulo
  `apex` novo; `apices` passa a `pub` com parâmetro `unit`; `tip_deviation`/`tip_density` mudam a
  semântica do 3.º argumento; `TipDeviation` ganha dois campos — `..Default::default()` nos
  literais do shell já os cobre).
- **Ids/consts novos** (colisão na integração): `CONE_MAX` · `TIP_GAP_MAX` · `apex::*` ·
  `median_edge` · `TipDeviation::{apex_max, cut}` · dev-dep `miniz_oxide` em `ph2d-quadfill`.
- **Portões corridos 1× no fecho:** `cargo test -p ph2d-quadfill` (67 · 19 ignorados) ·
  `cargo test -p ph2d-host-desktop` (4 786 · 268 ignorados) · `cargo clippy -p ph2d-quadfill
  -p ph2d-host-desktop --all-targets` · `cargo fmt --all --check` · `cleanroom-sweep.sh`.
- **Ficheiros no tecto de LOC:** `sculpt3d_history_retopo_extract.rs` (`598/600`, intocado).
  `local.rs` foi a `819` e voltou a `420` pelo corte do `apex.rs` (`417`).
