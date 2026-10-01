# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor`, a reabertura depois da rodada 03 (2026-09-26 → 2026-10-01)

> **Este documento é para o AGENTE INTEGRADOR** (que só funde por ordem explícita do dono). Ele diz o
> que a linha toca, onde um merge pode doer, a prova de fecho e os smokes. A linha já tinha sido
> integrada na rodada 03 (o handoff dessa jornada é o
> [`…_A_LINHA_2026-09-25.md`](HANDOFF_INTEGRACAO_line_PainterWatercolor_A_LINHA_2026-09-25.md)); este
> cobre **só** os 26 commits depois disso. O *mecanismo* de cada passo vive em três sítios, e este
> documento aponta para eles em vez de os repetir:
>
> * a **fila** [`44_a_fila_depois_da_linha.md`](../44_a_fila_depois_da_linha.md) — os 12 abertos da
>   linha, um por secção (§1–§10);
> * o **log de performance** [`28_otimizacoes_o_que_funcionou.md`](../28_otimizacoes_o_que_funcionou.md)
>   §4.9 — a pilha do Composite Brush num rabisco, três voltas, até aos 60 fps;
> * o **ADR-0172** e as três emendas de 2026-10-01 (sem número novo).

---

## §1 — Coordenadas

| | |
|---|---|
| ramo | `line/PainterWatercolor` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` |
| base | merge-base **`912a9652e`** = o `main` de hoje (o `main` não andou desde o fork) ⇒ **nada para rebasear** |
| commits | **26** de trabalho + o deste handoff |
| diff | `70` ficheiros, `+7 318 / −838` (antes deste handoff) |
| crates novas | **nenhuma** |
| pacotes EXTERNOS novos | **nenhum** |
| ADR novo | **nenhum** (o 0172 ganhou três emendas) |

**O que a linha fez, numa frase por assunto:**

- **A fila 44 fechou inteira** (12 de 12): o item 1 e os itens 3–5 foram decididos pelo dono; os itens 2, 6–12 foram feitos, com o smoke dele aprovado onde havia o que ver.
  - Aquarela seca mistura pela lei do Wet Paint; com `Charge < 1` o Pigment volta a misturar molhado sobre molhado.
  - O Smudge arrasta e o Rewet redissolve a tinta molhada da sessão.
  - O papel do Wet Paint é feito uma vez por entrada.
  - A pilha do Composite deposita o relevo, o filme e o Tiling do pincel avulso, e a borracha de cima apaga o corpo.
  - O carimbo rectangular foi a `0`.
  - O avental do borrão poupa duas travessias.
- **O Smear da pilha:** deixou de fazer rectângulos de cor e passou a arrastar o corpo com a cor. O impasto só assenta o volume depois da última composição.
- **A pilha a Size 0.5:** report do dono *«pincel com size 0.5 fps cai para 40»*. Hoje mede **60 fps** na app.

---

## §2 — Superfície de colisão (colada do `collision-surface.sh`, 2026-10-01)

```
SUPERFÍCIE DE COLISÃO — line/PainterWatercolor contra main
  merge-base 912a9652e   ·   25 commit(s)   ·   70 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        176   (base: 176)
      └ tripla do gate               (176, 13, 22)   (base: (176, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
    FIELD_DOC_VERSION                      23   (base: 23)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs                              108   (base: 108)
    ph2d-render (espelho)                 109   (base: 109)
    ph2d-script (espelho)                 109   (base: 109)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR
    último no disco: 0175   próximo livre: 0176
    esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock
    nenhum '+name' novo
▸ MARCADORES DE CONFLITO
    nenhum nos arquivos da linha
▸ TETOS DE LOC nos arquivos que a linha tocou
      698 / 700   crates/ph2d-app-painter/src/painter_bridge.rs  (tem marcador/allowlist — confira o valor congelado)
    nenhum arquivo da linha passa do teto
```

⇒ **Todos os contadores partilhados com delta `0`.** O script foi corrido com 25 commits; o 26.º
(`1cc538430`, fmt + um nome de variável num arnês) não toca em nada daquela tabela.

⚠️ **A tabela é REFERÊNCIA, não evidência** (DIRETRIZ §1.5.9 item 3): se outra linha fundir antes
desta, leia os valores do `main` no ficheiro.

---

## §3 — O que está fora da pasta do módulo (e porquê)

| ficheiro | o quê | risco de merge |
|---|---|---|
| `Cargo.toml` (raiz) | **append-only:** `[profile.smoke.package.ph2d-painter-brush] codegen-units = 1`, `incremental = false`, logo a seguir a `[profile.smoke]`, com o comentário da medição | só textual, se outra linha acrescentar uma secção no mesmo sítio — as duas ficam |
| `crates/ph2d-painter-brush/Cargo.toml` | comentário: o **terceiro** uso do `rayon` nesta crate (a cerca diz que cada uso se nomeia) | nenhum |
| `crates/ph2d-app-painter/src/painter_bridge.rs` | **uma linha**: `crate::composite_smoke::rabisca(painter);` antes da drenagem | ⚠️ o ficheiro está em **698/700** — outra linha que lhe acrescente 3 linhas estoura o tecto na árvore combinada |
| `crates/ph2d-app-painter/src/composite_smoke.rs` | o **rabisco automático** (`PH2D_COMPOSITE_RABISCO=<Size>`) | nenhum |
| `docs/architecture/decisions/0172-….md` | três emendas de 2026-10-01 (a primeira sem letra, depois **(b)** e **(c)**), no fim do ficheiro | só se outra linha também emendar o 0172 |

**Superfície nova nas crates do pincel (não é contrato congelado, mas é `pub`):**

- `ph2d-painter-brush`:
  - módulo `blur_peso`: `blur_region_por_peso`, `blur_region_borrado` e `mistura_linha_por_peso`, re-exportados no `lib.rs`;
  - `sculpt::Passeio` (`pub(crate)`): o passeio da pegada linha a linha, que o `walk_dab` passou a usar;
  - os gates do esfregão mudaram-se para `smear_field_tests.rs`.
- `ph2d-tool-painter`:
  - o `smear_dabs_field` partiu-se em `prepara_campo_do_esfregao` → `TrabalhoDoCampo::corre` → `conclui_campo_do_esfregao`;
  - a `BrushTextureImage` guarda a luminância num `Arc` e é `Clone`;
  - `PilhaDoTraco` ganhou `campo_adiantado`;
  - `compoe_a_regiao` ganhou um terceiro argumento: `mudou_ja_medido: Option<Region>`.

---

## §4 — Contratos congelados

**Nenhum.** `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4` e os de nós
intocados (ver o §2).

---

## §5 — O que só o `ship.sh` pega — corrido no fecho, tudo verde

| verificação | resultado |
|---|---|
| `cargo fmt --all -- --check` | verde **depois** de `1cc538430` (o fecho apanhou `ph2d-wet-paint/src/paper_memo_tests.rs`, desvio dos itens 6–7) |
| `typos` sobre os ficheiros da linha | verde **depois** de `1cc538430` (um `SME=` no arnês `muta_o_arrasto_molhado.sh`) |
| `cargo machete` | nenhuma dependência sem uso |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| `scripts/check-standalone-optional.sh` | verde (10 crates) |
| `scripts/check-workflow-packages.sh` | verde |
| `#[cfg(target_os` escrito ou movido | **nenhum** ⇒ nada a cruzar para macOS |
| `cargo clippy -p ph2d-tool-painter -p ph2d-painter-brush -p ph2d-app-painter --all-targets -- -D warnings` | zero |

### §5-bis — A árvore combinada

`bash scripts/censos-da-arvore-combinada.sh` → **127/127**, controlo do filtro `12 de 12`. Como o
`main` não andou, a árvore combinada **é** esta.

---

## §6 — Prova de fecho

- **`nextest-impacted`: 17 757 / 17 758.** O único vermelho é
  `ph2d-physics-ecs::it measure_player_budget::the_cost_of_a_player_is_linear_in_their_number`,
  que já é membro **nomeado** da família de flakes de fan-out (`CLAUDE.md` §5.0). Ele passa sozinho
  4 de 4 vezes, de `load 14` a `load 70`, e a linha tem **zero** linhas de diff naquela crate.
- **Provas de mutação** (o detalhe está no corpo de cada commit):
  - `b11368410`: acumular por quadro e a mistura em linhas;
  - `1915abccb` e `cb9fb0057`: as faixas do pingo — a fatia na linha errada sangra;
  - `bf218ec4b`: **6 de 6** — a força, a dureza, o fluxo, o campo deitado fora, a pergunta depois do empréstimo e o mínimo trocado no paralelo.
- **Byte-identidade:** cada passo de performance de hoje tem um gate que compara a rota nova com a
  antiga **ao byte**:
  - `acumular_por_quadro_da_a_imagem_de_acumular_por_evento`;
  - `a_mistura_do_borrao_em_paralelo_da_o_byte_da_serie`;
  - `as_faixas_do_pingo_dao_o_campo_da_serie`;
  - `quem_le_da_caixa_em_paralelo_da_a_caixa_da_serie`;
  - `o_campo_ao_lado_do_acumulo_da_a_imagem_da_serie`;
  - `o_campo_ao_lado_com_grao_aleatorio_e_impasto_da_a_imagem_da_serie`.

---

## §7 — Smokes

**Aprovados pelo dono nesta jornada:**

- os itens 2, 11 e 12 da fila (aquarela);
- os itens 8b, 9 e 10 e o Smear com corpo (Composite);
- a Size 0.5 duas vezes: *«smoke OK»* e, depois dos 60 fps, *«smoke OK parece mesmo melhor»*.

**Depois da fusão é o `main` que se smoka.** O comando de referência, para o caminho do primário:

```
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_COMPOSITE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

e o instrumento que mede o quadro inteiro sem ninguém a riscar:

```
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_RABISCO=0.5 PH2D_PAINT_PERF=1 cargo run -p ph2d-host-desktop --profile smoke
```

(as linhas `PH2D_COMPOSITE_RABISCO: periodo … ms/quadro (… fps)` dizem o período durante o traço).

---

## §8 — O que uma leitura rápida do diff entende ao contrário

1. **O fluxo aleatório do esfregão MUDOU de dono, e isto é uma mudança de comportamento
   declarada.** Antes, a camada Smear herdava o `tex_rng` da **última camada acumulada**. Hoje usa o
   dela, `rng_camada[pos]`, como toda camada que acumula. A imagem só muda com Shape ou Grain
   **aleatório** no esfregão (sem eles o fluxo não é lido). Foi isto que tornou exacto correr o
   campo ao lado do acúmulo.
2. **O campo do esfregão corre noutra THREAD durante o acúmulo** (`std::thread::scope` com uma
   thread, não `rayon`). Tem duas condições:
   - só quando há **um** esfregão vivo com pingos;
   - a pergunta *«quem lê da caixa»* faz-se **antes** do empréstimo, porque lê o campo do quadro anterior.

   ⚠️ A primeira versão fazia-a depois, e `o_esfregao_nao_deixa_rectangulos_de_cor` leu `255`. Eu
   atribuí esses vermelhos a uma guarda de sessão; a guarda era inobservável e saiu. O registo
   está no doc do `adianta_o_campo`.
3. **A mensagem de `1915abccb` diz «o rayon não ganha» e `cb9fb0057` usa o rayon.** As duas estão
   certas: a primeira mediu o rayon com as **mesmas** 9 fatias grandes; a segunda, com fatias de
   `4` linhas (`227 → 152 µs` por pingo).
4. **O `[profile.smoke.package.ph2d-painter-brush]` não é cosmético.** No `smoke` a pilha custava
   `14,4` ms contra `11,1` no `release`, e era isso que o dono sentia. Ele compila uma crate pequena
   num só pedaço; a shell continua em paralelo.
5. **O `PH2D_COMPOSITE_RABISCO` escreve os eventos ANTES da drenagem.** O custo deles cai na fase
   `preview` do `[paint-perf]`, não na `INPUT`. É um instrumento: sem a env var é um no-op.
6. **O pool do `rayon` no depósito em banda** (`stamp_banded`) foi medido e não ganhou nada. A
   tentativa foi revertida e não está no diff.

---

## §9 — Aberto

- **Nada da fila 44.**
- **Performance:** os 60 fps a Size 0.5 foram medidos numa tela de `1024²` com a pilha do dono.
  Tela maior ou Size maior **não foram medidos** nesta jornada. A sonda
  (`diag_onde_vai_o_quadro_do_rabisco`, `PH2D_DIAG_SIZE`) e o rabisco automático estão prontos
  para isso.

---

## §10 — Para o integrador

- **`CLAUDE.md` §5, módulo Painter:** esta linha não o editou (ordem do dono). Sugestão de UMA
  linha, a acrescentar a seguir à da `line/PainterWatercolor` de 25/09:
  > ⭐ **E reabriu (26/09–01/10): a fila 44 fechou inteira (12/12) e a pilha do Composite a
  > Size 0.5 vai de `~40` para `60` fps** (acumular por quadro · o Blur em linhas · o campo do
  > esfregão em faixas e AO LADO do acúmulo · o motor do pincel num só pedaço no `smoke`).
  > [Handoff](docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-01.md) ·
  > smoke `PH2D_COMPOSITE_RABISCO=<Size>` (risca sozinho e diz o período).
- **Flakes:** nenhuma nova a promover. A que reprovou já está nomeada.
- **Memória:** a linha não escreveu ficheiros em `project-memory/`.
