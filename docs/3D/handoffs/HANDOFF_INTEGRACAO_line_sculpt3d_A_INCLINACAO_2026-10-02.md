# HANDOFF de INTEGRAÇÃO — `line/sculpt3d` · a INCLINAÇÃO por amostra: a encosta deixa de ler-se em degraus (2026-10-02)

> **Para quem é:** o agente INTEGRADOR (só por ordem do dono — CLAUDE.md §0.7). Superfície de
> colisão MEDIDA, o que mudou fora da família, números e smoke. A narrativa e as tabelas de medição
> vivem em [`29`](../29_plano_o_relevo_do_impasto_na_peca.md) **§9**.
>
> ⚠️ Ele **SUPERSEDE** o [handoff de 01/10](HANDOFF_INTEGRACAO_line_sculpt3d_O_RELEVO_2026-10-01.md)
> como documento de integração desta linha: aquele já está no `main` (a linha foi reaberta a 02/10
> por `git reset --keep main`; `git cherry main HEAD` = 0 antes dos 3 commits abaixo).

## §0. Estado em uma tabela

| | |
|---|---|
| ramo | `line/sculpt3d` · worktree `Worktrees/line-sculpt3d` |
| merge-base | `1ad60a1ce` (= o `main` de agora: **0 commits atrás**, rebase **desnecessário**) |
| commits | **3** — `9f26085a1` (a feature) · `c79e53b7b` (doc 29 §9) · `b88d9462d` (fmt + clippy + âncoras do arnês); **13** ficheiros, `~+1 569 −182` antes do fmt |
| smoke do dono | ⏳ **por fazer** (§7) |
| contrato congelado (§6) | **intocado** |
| `PROJECT_SCHEMA` · `VEC_SCENE` · `FLIP` · `DOC_VERSION` · `FIELD_DOC_VERSION` · registos de componentes | **0** (`collision-surface.sh`) |
| `SCULPT_DOC_VERSION` | **0** (continua `5`): a inclinação é DERIVADA — não vai ao documento nem ao undo |
| ADR · `Cargo.lock` | nenhum · nenhum |
| tectos de LOC | nenhum passa · sem marcadores de conflito · **a shell não foi tocada** |

## §1. O que a linha traz

A normal do relevo (doc 29 §7) usava o gradiente EXACTO **por célula**: constante no sub-triângulo,
salta na aresta da retícula ⇒ **degraus visíveis em encosta inclinada** (sonda
`diag_o_relevo_visto_inclinado`, de lado e de cima). Cura: a inclinação passa a ser **por AMOSTRA**,
como a normal por vértice.

- **`ph2d_mesh_colors::Inclinacoes`** (`inclinacao.rs`, novo): cada célula dá o seu gradiente exacto
  (constante no sub-triângulo; derivada bilinear no centro da célula do quad, usando a metade que o
  contém, células diagonais fazem a média das duas) aos seus cantos, **pesado pela área no espaço do
  objecto**. O fragmento interpola com os pesos da cor (`Tinta::inclinacao_tri/_quad`) ⇒ contínua
  entre faces (amostras partilhadas).
- `atualiza(sujas)`: faces das amostras sujas por inteiro + faces do anel (por vértices) **só a
  borda**. `nova()` é esparsa (só faces com altura ≠ 0; recomputa inteiro se > metade).
- Paralelo acima de `LIMIAR_PARALELO = 32 768` células (`std::thread::scope`), *scratch* por face
  fundido **na ordem das faces** ⇒ **ao bit para qualquer nº de fios** (gate). A crate continua sem deps.
- **GPU** (`tinta.wgsl`): `@binding(8) tinta_inclinacoes: array<f32>` (3 f32/amostra), acumulada em
  `tinta_soma` por `tinta_inclinacao(i)` (zero sem o bit do relevo); o código de derivada por célula
  saiu de `tinta_le_tri/le_quad/tinta_no_ponto4`. ⚠️ O fragmento passa a **7** buffers de
  armazenamento (piso WebGPU `8`).
- **`tinta_gpu.rs`**: `TintaGpu` ganha o buffer `inclinacoes`, `inc: Option<Inclinacoes>` e
  `inc_foto: FotoDasInclinacoes` (carga, posições, alturas). Subida completa com a MESMA topologia
  recalcula só o que mudou (vértice movido entra como amostra suja: o índice da amostra é o id do
  vértice); topologia mudou ⇒ `nova`.

## §2. O que mudou FORA da família

| ficheiro | o quê | aditivo porque |
|---|---|---|
| `ph2d-mesh-colors` (`inclinacao.rs` + `inclinacao_tests.rs` **novos**, `lib.rs` +módulo/+`pub use Inclinacoes`) | o tipo novo | módulo irmão; nada existente muda |
| `ph2d-mesh-render` `shaders/tinta.wgsl` | `@binding(8)`; derivada por célula removida | só corre com o bit do relevo; sem ele a normal nem é tocada |
| `ph2d-mesh-render` `tinta_gpu.rs` | buffer + estado da inclinação | ⚠️ **ASSINATURA mudou**: `upload_tinta_amostras_at` ganha `mesh: &Mesh` |
| `ph2d-mesh-render` `tests/it/` (`tinta_paridade.rs`, `tinta_relevo_no_device.rs`, `tinta_no_device.rs`) | harnesses | ver §4.3 |

Família: `ph2d-app-sculpt3d` `slots.rs` (passa a malha), `tinta_no_produto_painter.rs` (+3 linhas de
`mod`), `tinta_no_produto_subida.rs` (**novo**, duas sondas).

## §3. Formato do documento

**Inalterado.** `SCULPT_DOC_VERSION` fica em `5`; a inclinação reconstrói-se da altura ao abrir.

## §4. O que uma leitura rápida do diff entende ao contrário

1. **A altura NÃO mudou nem a forma da lei — mudou ONDE o gradiente vive** (célula → amostra). Quem
   procurar a altura de novo no `MeshData` não a acha; a inclinação não é guardada em lado nenhum.
2. **`upload_tinta_amostras_at` recusa** (devolve `false` ⇒ o chamador faz subida completa) se há
   relevo e o `inc` não serve o plano. É a porta de segurança, não um erro.
3. **A paridade GPU/CPU mudou de régua** (`tinta_paridade.rs`): a 3.ª palavra compara a GPU com a CPU
   `inclinacao_*` em **TODAS** as sondas (a lei antiga só longe das bordas de célula), barra `1e-4`. As
   sondas e a saída fundiram-se num buffer `read_write` **porque o arnês batia em 9 > 8 buffers**
   (`Too many bindings of type StorageBuffers in Stage COMPUTE, limit is 8, count was 9`).
4. **O gate `o_relevo_inclina_a_luz_so_onde_ha_degrau`** tem agora a banda = rampa + **uma coluna de
   cada lado** (o alisamento espalha uma célula; corpo `0` fora da tinta não acende).
5. **O custo de uma pincelada subiu** (0,04 → 0,10 ms de CPU) mas o critério do doc 29 §4 passa (§5).

## §5. O portão desta linha (02/10)

| passo | resultado |
|---|---|
| `nextest-impacted.sh` (contra o merge-base) | **17 594 / 17 594** verdes (13 295 fora do alcance) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| `cargo clippy -D warnings` nas 3 crates | limpo (depois de curar um `needless_range_loop`) |
| `cargo fmt --all --check` | limpo (depois do fmt) |
| `censos-da-arvore-combinada.sh` | **127 / 127**, controlo do filtro `12 de 12` |
| GPU com adaptador (`--ignored relevo tinta`) | **9 / 9** |

Gates novos — `ph2d-mesh-colors` (**8**): `a_inclinacao_lida_e_continua_e_a_lei_por_celula_reprova_a_mesma_regua`
(salto máx. `1,1e-4` contra `4,2e-2` do controlo por célula, **~380×**) ·
`a_inclinacao_por_amostra_e_a_da_superficie` (grelha não alinhada aos eixos: erro `1,6 %` nível 3,
`0,4 %` nível 4, ordem dois) · `os_dois_lados_de_uma_aresta_leem_a_mesma_inclinacao` ·
`a_atualizacao_por_pedacos_da_a_inteira_e_diz_o_que_mudou` · `mover_vertices_e_atualizar_por_eles_da_a_inteira` ·
`um_plano_com_pouca_altura_nasce_igual_ao_inteiro` · `as_faces_em_paralelo_dao_o_mesmo_ao_bit` ·
`sem_relevo_a_inclinacao_e_nula`. GPU (`tests/it`): `esculpir_com_relevo_desenha_o_que_uma_subida_do_zero_desenha`
· `o_relevo_subido_por_pedacos_desenha_o_que_uma_subida_do_zero_desenha`.

**Medido** (perfil `smoke`, `load < 4`, peça de fábrica, cena 52) — o critério do doc 29 §4
(*subida incremental do relevo ≤ 1 ms/quadro durante um traço*), nunca medido antes:

| | CPU escreve (mediana / pior) | submit+espera p95 / pior |
|---|---|---|
| ANTES do §9 | `0,039` / `0,064` ms | `0,079` / `0,190` ms |
| COM inclinações | `0,10` / `0,20` ms | `0,16–0,29` / `0,6–0,9` ms |

⇒ **passa**. Recálculo inteiro, serial → paralelo: 8× (47 k amostras) `1,0 → 0,6` ms · 16× (188 k)
`4,7 → 1,6` · 32× (754 k) `17 → 6,8` · 64× (3,0 M) `68 → 27`. Quadro de escultura (subida completa,
mesma topologia), 8×/16×/32×/64×: pincelada de 11 vértices `0,23 / 0,6 / 2,1 / 8,1` ms; grande (40)
`0,39 / 1,2 / 3,3 / 12,2`; a MESMA subida SEM relevo (custo pré-existente) `0,06 / 0,18 / 0,7 / 6,1`.

## §6. Mutação

`docs/3D/ferramentas/muta_a_normal_do_relevo.sh` **reescrito** (as N1–N4 antigas miravam a lei por
célula; 4 âncoras mortas): **12 de 13 sangram**, o C1 (CONTROLO) sobrevive. Depois do fmt as N10/N12
re-ancoraram-se e re-correram: **2 / 2** sangram; pré-voo **13 / 13**. Os outros 16 arneses de
escultura passam o pré-voo (`MUTA_SO_ANCORAS=1`) limpos.

## §7. Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=52 cargo run -p ph2d-host-desktop --profile smoke
```

1. `Paint Detail` → `8x`; `IMG` → `PNTR`; meio `Impasto`; um traço grosso.
2. Rode a vista com o botão direito: a luz na encosta tem de ser **lisa**, sem degraus de quadrícula.
3. A ponta junto ao contorno da bola continua **sem meia-lua**.
4. Deu errado se vir a quadrícula em escada na encosta, ou a meia-lua de volta.

Prova de que o binário do comando acima está COMPILADO nesta worktree (a corrida depois do
`rm -rf target/*/incremental`, que reclamou `24 G` + `5,4 G`; a 1.ª build foi de `37,8 s`):

```
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

## §8. A UMA linha que proponho para o `CLAUDE.md` §5 (3D / Sculpt)

> **3D / Sculpt** — malha que doa a normal ([ADR-0150](docs/architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md)); pincéis, Quad Retopology, tinta fina e o Painter na peça, crate `ph2d-app-sculpt3d`. Smoke `PH2D_SCULPT3D_SMOKE=<n>`. Último: [handoff 02/10](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_A_INCLINACAO_2026-10-02.md) · [docs](docs/3D/README.md) · [BUGS](docs/3D/BUGS_sculpt3d.md) · [retopologia](docs/3D/quad-retopology/README.md) · [história](docs/archive/estado-2026-10-02/sculpt3d.md)

## §9. ABERTO

- ⏳ O caminho da escultura **re-sobe o plano INTEIRO** em todo quadro em que a peça muda de forma
  (`6,1` ms a 64×; pré-existente, independente do relevo). Próximo ganho: subir só as faces dos
  vértices movidos.
- ⏳ A etapa 4 do Painter na peça (camadas e efeitos) espera a decisão do dono.
- ✅ **FECHADOS** por esta linha: a *faceta da retícula* (handoff de 01/10 §9) e o *relógio do upload
  incremental com relevo* (medido, §5).
