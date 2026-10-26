# Continuação — `line/BlenderConstraints`, onda W4 (CAD no vetor), 26/10/2026 — A MEIO

> Anterior: [`W3`](HANDOFF_CONTINUACAO_W3_2026-10-25.md) (smoke da cena 5 OK). Doc da onda: [`04_W4_cad.md`](../04_W4_cad.md)
> (medição §1, W4.1 §1.4, desenho e ordem §3, **W4.2 em detalhe §3.1**, gates §4, cena 6 §5) — **não duplicado aqui**.

## Identidade

Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4` (o `main` não andou).
Commits da onda: `97fb9dc4c4` W4.0 (ezpz × fiksi + oráculo FreeCAD do arrasto) · `0820a1a991` W4.1 (solução única e conflito) ·
o commit deste handoff (desenho da W4.2). Árvore limpa. `PROJECT_SCHEMA` desta linha = 194 (a W4.2 sobe a 195; reconta-se na
integração).

## Decidido

- **Solver = `kcl-ezpz` `=0.2.18`** — o dono disse «pode seguir» (26/10) depois de ver a medição: 0,68–0,81 ms no pior passo a
  200 pontos (< 2 ms), violação ≤ 9e-10 px, determinista; bate o FreeCAD ao 1e-8 px nas 11 cenas de solução única e dá a
  MESMA lista de conflito. fiksi recusada por medição (§1.1).
- REF = teste de posto nosso (a ezpz não avisa redundância coerente); «apague uma destas» = `unsatisfied` da ezpz.
- O id do vértice mora DENTRO do `VecVertex` (lei do `corner_radius`); atribui-se no gancho antes da captura de undo (§3.1).
- O `PontoDeForma` da W3 troca de índice para `VertId` sem migração (a W1–W3 não está no main).

## Próximo passo (W4.2, pela ordem do §3.1)

1. Censo de todo portador gravado de `VecPath`/`VecVertex` (delegar ao `explorador`).
2. Gates vermelhos primeiro: `a_vertex_keeps_its_id_when_a_neighbour_is_deleted`, `a_v23_scene_opens_with_ids_in_order` (sobre
   BYTES gravados pelo código v23 — gravar a fixture ANTES de mudar o tipo), `copy_point_follows_its_vertex_after_an_insert_before_it`,
   `each_vertex_mutation_site_keeps_or_assigns_ids`.
3. `VertId`, campos, `normaliza_ids`, âmbito `GeometryLayoutV23`, degrau 24, leitores, regravação dos blobs, PROJECT_SCHEMA 195
   (a nota do degrau em `project_schema.rs` segue o formato das anteriores), literais da shell → construtores (a shell encolhe).
4. Depois W4.3 (crate `ph2d-vec-constraints`), W4.4 (editor), W4.5 (medida animada + cena 6) — §3.

## Ferramentas desta onda

- Bancada: `bash docs/BlenderConstraints/ferramentas/bancada_solvers/mede.sh` (uma cena por processo; `W4_SO="…"` filtra) ·
  `bancada_solvers_w4 bem <cenas> <freecad.json> <saida>` · `diag <cenas> <cena> <variante>`.
- Oráculo: `oraculo_freecad.py` (todo: ~17 min por causa do misto_800; `W4_SO_BEM=1` só as cenas W4.1, segundos).

## Armadilhas

- `SketchObject.movePoint` não existe no FreeCAD 1.x (`moveGeometry`, devolve `None`); o sketch só tem geometria depois do 1.º `solve()`.
- Um solver pode pendurar DENTRO de uma chamada (fiksi L-BFGS > 25 min): uma cena por processo com `timeout`.
- `git checkout --` de um hack temporário apaga também o não comitado: comite o WIP antes.
- Dois agentes no mesmo worktree: só UM comita, no fim.
