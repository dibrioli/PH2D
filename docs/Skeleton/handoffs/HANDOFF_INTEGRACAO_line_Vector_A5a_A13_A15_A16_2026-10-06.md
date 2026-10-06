# HANDOFF DE INTEGRAÇÃO — `line/Vector`: A5-a, A13 (cor), A15 e A16 numa rodada (2026-10-06)

> Leitor: o agente integrador (e a próxima LLM da linha). Smoke = do dono, ainda NÃO feito. Nada aqui
> foi integrado nem enviado. Mecanismo, medições e recusas por item: fila
> [`01_a_fila.md`](../01_a_fila.md) §F65. Continuação que esta fecha:
> [`…_CONTINUACAO_A5a_A13_A15_A16_2026-10-06.md`](HANDOFF_line_Vector_CONTINUACAO_A5a_A13_A15_A16_2026-10-06.md).

## 0. Onde está e o que fazer

- Worktree `Worktrees/line-Vector`, ramo `line/Vector`, HEAD `12a9c7c97`; commits da rodada
  `16abdf7cb`…`12a9c7c97` (`git log --oneline 0910f5315..line/Vector`), base `0910f5315`.
- Sobre DUAS ondas ainda por integrar, que entram juntas: F60–F63
  [`…_A_PONTA_DO_VINCO_E_AS_PASSAGENS_2026-10-05.md`](HANDOFF_INTEGRACAO_line_Vector_A_PONTA_DO_VINCO_E_AS_PASSAGENS_2026-10-05.md)
  e F64 [`…_O_ESQUELETO_E_UM_OBJECTO_2026-10-05.md`](HANDOFF_INTEGRACAO_line_Vector_O_ESQUELETO_E_UM_OBJECTO_2026-10-05.md)
  (as duas com smoke aprovado). ⚠️ **EMPILHADA sobre a `line/UIUX`** (ainda não no `main`): a UIUX entra
  ANTES; os commits dela saem do rebase por igualdade.
- Commits: `16abdf7cb` (gate vermelho A16 + sonda A15) · `7a95107d5` (A16) · `7fd187694` (A15) ·
  `5b97431cf` (sonda A13) · `5d9acc3c2` (A13) · `496af6cdd` (A5-a; cherry-pick de `3b7923028` do ramo
  `a5a-lei`; o ramo de experiência `exp/a5a-lado` `ea5d82f8e`/`d7de451fb`/`9ee84c693` guarda as sondas) ·
  `cbae7cd25` (fecho: ficheiros só de teste da fresta renomeados `*_tests.rs` — o censo
  `architecture_who_reads_the_posed_skin_mesh` separa teste de produto pelo sufixo; fmt) · `12a9c7c97`
  (gates dos 12 sobreviventes da mutação).
- Integrar: `/pd-integracao line/Vector`, só por ordem do dono.

## 1. Superfície de colisão

| sítio | o quê | natureza |
|---|---|---|
| `Cargo.lock` | 1 linha | dependência de teste |
| `crates/ph2d-app-vec/Cargo.toml` | dev-dep `ph2d-vec-skin` + `ph2d-skeleton-live` (test-support) | teste |
| **`crates/ph2d-editor-core/src/screens/hero/mode_drive.rs`** | `ModeFamily::keeps_parts_selected` (omissão `false`) + `restore_parts` (+29 linhas) | ⚠️ FUNDAÇÃO da `line/UIUX`, só ANEXO; outras famílias ao bit |
| `crates/ph2d-app-skeleton/` | `loose.rs` (`adopt_loose_roots(sim, &TimelineDoc)`), `skeleton_mode.rs` (`true`) + testes | família |
| `crates/ph2d-skeleton-live/` (19 ficheiros) | `skin_image_costura.rs` (novo, 497), `skin_image_arte.rs` (novo), `skin_image_fecho.rs` (288), `skinned_mesh.rs` (`mascara`, `le_malha`), `skin_bake_cache.rs`, `esqueletos.rs`/`peso_a_mao.rs` (leitores por `le_malha`) | lei |
| `crates/ph2d-vec-skin/` (4) | `curva_segundo_corpo_refino.rs` (novo), `curva_segundo_corpo.rs`, `curva.rs` | lei |
| `crates/ph2d-app-vec/` (7) | cenas/sondas `smoke_bone_copias_*`, `smoke_bone_par_fresta_*` (renomeadas `*_tests.rs`) | teste |
| `shells/desktop/` | `render_loop/fase_object_mode.rs` (passa `&self.timeline.doc`), `tests/it/the_loose_bone_roots_get_a_skeleton.rs` (agulha do gate textual) | shell |

- Contrato congelado (§6): **nenhum** tocado. **`PROJECT_SCHEMA`: SEM degrau** (a máscara viaja no
  `postcard` do `SkinBind.source`; `le_malha` cai para `SkinnedMeshV1` em binds antigos — mesmo precedente
  do `le` do `SkinnedPath`; `scripts/schema-recount.py` só renumera em rebase conflituoso, não detecta).
- Contadores como DELTA: nenhum registo de componentes, ADR, nem cena nova (cenas `=3,4,6,7` existentes).
- O que um merge pode partir: (1) a UIUX mexer no `mode_drive.rs` (o anexo é em `Step::Enter` e passo 0);
  (2) outra linha que leia a struct da malha sem `le_malha` — o censo
  `ninguem_le_a_malha_sem_a_porta_dos_binds_anteriores` apanha-o; (3) ficheiros novos da fresta que não
  acabem em `_tests.rs` — o censo da malha posada trata-os como produto.

## 2. O que mudou para o artista

- **A5-a** numa imagem presa dobrada, o encontro dos membros fecha o fio fino junto à tampa redonda sem
  tinta de outra cor por cima (cúspide, 27 poses: fio `3114` sem costura → `2368` antes → `1852`).
- **A13 (só a cor)** com as duas juntas muito dobradas (`170/140`, `170/170`) a cor chega à borda de
  fora da dobra (vão `0,27 → 0,04`).
- **A15** o anel de mover de cada esqueleto das cenas antigas fica na cabeça do 1.º osso, não no centro.
- **A16** `Tab` / seletor *Mode* com um osso escolhido mantém o osso escolhido (o pai do próximo osso).

## 3. Prova de fecho

Gate batched 1× (BASE `0910f5315`):

| portão | resultado |
|---|---|
| `nextest-impacted.sh` | 15 764/15 765 — o ÚNICO vermelho era o censo da malha posada (sufixo), curado em `cbae7cd25` |
| `cargo clippy --workspace --all-targets -D warnings` | ✅ |
| `cargo fmt --all --check` | ✅ (depois do fmt de `cbae7cd25`) |
| `file_loc_caps` · `the_shell_only_shrinks` | ✅ |
| `censos-da-arvore-combinada.sh` | ✅ 114/114 |
| censos do `ph2d-panel-registry-init` (`--workspace`) | ✅ 125/125 |

- **Mutação:** 46 mutações, 30 sangraram; 16 sobreviventes → 12 ganharam gate em `12a9c7c97` (vermelho
  visto em cada); o `>=` do teste da divisão e a guarda `l != 0` são equivalentes (a guarda saiu); o piso
  de precisão é coberto pelo teste de terminação.
- **Fotos** (`kwin --virtual`, binário de smoke de `cbae7cd25`; scratch, não versionadas): `=3` (anéis na
  cabeça do 1.º osso), `=4` com `PH2D_VEC_BONE_DOBRA=36 PH2D_VEC_BONE_DOBRA2=-144` (cúspide da imagem
  fechada), `=6` com `PH2D_VEC_BONE_DOBRA=170,140` (cor enche as duas barras), `=7`.
  ⚠️ Fotografado no binário de `12a9c7c97`: a `170/170` e `170/140` ficam dois **tiques** soltos do traço
  (`~0,5` larguras) junto à dobra de dentro — ver §4.
- Binário de smoke compilado 2×; `rm -rf target/*/incremental` por fazer na janela de integração.

## 4. ABERTO

- ⏳ **A13 parcial:** a cor ✅ `5d9acc3c2`. **A18 (novo)** as PONTAS LIVRES do traço na ABA da dobra
  extrema (`≥ ~170°`, já a `110°` uma) ficam: duas leis medidas e recusadas (L1 exclui coberturas
  viradas/mesma folha; L2 estende a ponta ao cruzamento) — fila §F65, ramo `exp/a13-tiques`
  (`2038739fb`), sonda `d831c8d72`. **Decisão de produto pedida ao dono:** deixar, ou desenhar o VINCO da
  aba como linha (papel dobrado), por medir.
- **A17 (novo)** raiz com posição ANIMADA: o anel fica na origem (A15 só move onde é exacto). Re-alvejar
  a faixa de translação para o esqueleto seria exacto mas muda as linhas da timeline que o artista vê —
  pergunta de PRODUTO, não feita.
- Flip com ossos: excluído pelo dono (06/10).
- O precedente divergente do olho (sprites por entidade × esqueleto e Flip pela árvore): herdado do
  handoff do esqueleto, ainda aberto.

## 5. A linha do `CLAUDE.md` §5 (para o integrador; não toquei no ficheiro)

```
- **Vector + Esqueleto** — motor vetorial nativo kurbo/Vello ([ADR-0108](docs/architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md)), crates `ph2d-app-vec`/`ph2d-app-skeleton`; o esqueleto é um objecto (Object·Edit·Pose). Smokes `PH2D_BUILD_SMOKE=<n>` · `PH2D_VEC_BONE_SMOKE=<n>`. Último: [handoff 06/10](docs/Skeleton/handoffs/HANDOFF_INTEGRACAO_line_Vector_A5a_A13_A15_A16_2026-10-06.md) · [docs](docs/Vector%20Module/README.md) · [BUGS](docs/Vector%20Module/BUGS_vector.md) · [história](docs/archive/estado-2026-10-02/vector.md)
```

## 6. Smoke (o dono) — por fazer

Comando base, igual nos cinco: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && cargo run --profile smoke -p ph2d-host-desktop`
(com as variáveis à frente, ver cada um).

**6.1 A junta de duas imagens fecha sem fio**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=4 PH2D_VEC_BONE_DOBRA=36 PH2D_VEC_BONE_DOBRA2=-144 cargo run --profile smoke -p ph2d-host-desktop`
2. Olhe a ponta redonda da barra de imagem de baixo, onde ela encontra a outra parte.
3. Tem de acontecer: o encontro está cheio, sem fio branco fininho e sem pedaço de outra cor por cima.
4. Deu errado se: vê uma linha branca fina no encontro, ou uma mancha com a cor da outra parte.

**6.2 A cor da dobra apertada** (as pontas de traço abaixo são conhecidas)
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=6 PH2D_VEC_BONE_DOBRA=170,140 cargo run --profile smoke -p ph2d-host-desktop`; depois o mesmo com `PH2D_VEC_BONE_DOBRA=170,170`.
2. Olhe a dobra de fora das barras.
3. Tem de acontecer: a cor enche as duas barras até à borda de fora da dobra, sem faixa vazia.
4. Deu errado se: sobra uma zona sem cor na curva de fora. Nas dobras mais apertadas ficam duas pontinhas
   de traço soltas dentro da cor: já sabemos, não conte como erro desta rodada — vou fazer-lhe uma pergunta
   sobre elas.

**6.3 O ponto de mover do esqueleto**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=3 cargo run --profile smoke -p ph2d-host-desktop`
2. Olhe o círculo de cada **Skeleton**; clique nele e arraste (*Move*).
3. Tem de acontecer: o círculo está em cima do primeiro osso, não no meio da tela; ao arrastar, a forma segue.
4. Deu errado se: o círculo continua no centro da tela, ou a forma fica para trás / muda de sítio ao clicar.

**6.4 Tab com um osso escolhido**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=7 cargo run --profile smoke -p ph2d-host-desktop`
2. Na Hierarquia clique num osso (**Bone**); carregue em `Tab`. Depois menu **Mode ▸ Pose Mode**.
3. Tem de acontecer: o mesmo osso continua aceso depois do `Tab` e depois do Pose Mode. De volta ao Edit,
   a ferramenta **Create** a partir da ponta desse osso faz nascer o osso novo preso a ele.
4. Deu errado se: depois do `Tab` acende o **Skeleton** em vez do osso, ou o osso novo nasce solto.

## 7. LISTA VIVA dos abertos (copiada da continuação, actualizada)

> Regra: ao fim da janela cada item fica com o estado novo; aberto novo entra com o próximo número. O
> registo de antes dos itens JÁ fechados antes desta rodada (A1–A12, A14) vive na
> [continuação](HANDOFF_line_Vector_CONTINUACAO_A5a_A13_A15_A16_2026-10-06.md) §1 e não se repete aqui.

- **A1** ✅ DECIDIDO pelo dono (04/10): o osso mais PERTO — nada a construir.
- **A2** ✅ F52 · **A3** ✅ F53 · **A4** ✅ F54 · **A6** ✅ F55/F55-b · **A7** ✅ F56 · **A8** ✅ F58 (sem
  cura) · **A9** ✅ F57 · **A10** ✅ F60 · **A11** ✅ `eaa53edb1` · **A12** ✅ F61 · **A14** ✅ F64.
- **A5** — (b) ⛔ recusado pelo dono (`501daabf4`). **(a) ✅ FEITO `496af6cdd` (F65, 06/10):** a 5.ª lei é
  ORDEM + ANEL — anel da arte por marcha axial + cada triângulo de costura inserido a seguir ao da malha
  de que a aresta vem; a «hipótese refutada» de 05/10 estava superada: o que remendava sobre outro membro
  era a ordem de pintura. Régua de vão fixo: cúspide `3114 / 2368 / 1852`, nenhuma pose pior. *Registo de
  antes:* a cúspide da arte junto a uma tampa redonda (limite da F49); no *Zig Zag* a `~110°` os dentes de
  dentro encavalam-se e fecham buraquinhos reais.
- **A13** — ⏳ **parcial: a cor ✅ `5d9acc3c2`; as pontas livres do traço na aba da dobra extrema ficam
  (A18).** A cor: o refino local do bake (`k = 2`) cura a tampa
  cortada pelo contorno desenhado (`0,27 → 0,036`); a hipótese *NonZero* caiu. *Registo de antes:* a
  `=6` com as duas juntas `≥ 140°` (três camadas) tinha zonas sem cor, traço sem cor por baixo e
  tiques soltos; igual sem F60/F61 e com `TOL = 0`.
- **A15** ✅ FEITO `7fd187694` (F65): o esqueleto adoptado nasce com a origem na cabeça da raiz, pose ao
  bit (0/12 000 fora do bit; rebasear para `L−T` dava 5 055/12 000). *Registo de antes:* o anel de objecto
  vazio dos esqueletos adoptados caía na origem (visto nas fotos do fecho da A14).
- **A16** ✅ FEITO `7a95107d5` (F65): `keeps_parts_selected` + `restore_parts` no `mode_drive`, só anexo.
  *Registo de antes:* `Tab`→Edit do esqueleto com um osso escolhido trocava a selecção pelo esqueleto.
- **A18 (NOVO, 06/10)** as PONTAS LIVRES do traço na ABA da dobra extrema (`≥ ~170°`, já a `110°` uma):
  `Posada::tapado` conta como cobertura triângulos virados e vizinhos da mesma folha; L1 e L2 medidas e
  recusadas (fila §F65). ⏳ aguarda o dono: deixar, ou desenhar o vinco da aba como linha — por medir.
- **A17 (NOVO, 06/10)** raiz com posição animada: o anel fica na origem (A15 só move onde é exacto);
  medir se re-alvejar a faixa de translação para o esqueleto serve o artista — pergunta de produto.
