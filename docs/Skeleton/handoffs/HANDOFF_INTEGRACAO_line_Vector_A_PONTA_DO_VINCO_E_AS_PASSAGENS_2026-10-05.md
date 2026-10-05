# HANDOFF DE INTEGRAÇÃO — `line/Vector`: a ponta do vinco, as passagens que o traço enche e a pele mais barata (2026-10-05)

> Leitor: o agente integrador (e a próxima LLM da linha). Smoke = do dono. Nada aqui foi integrado nem
> enviado.

## 0. Onde está e o que fazer

- Worktree `Worktrees/line-Vector`, ramo `line/Vector`. Base desta onda: `f4a5594a4` (o `main` na
  integração de 04/10; a linha foi reaberta por fast-forward ao `main` de 05/10, `5d596eaaf`, com a
  poda do 3D — nada da linha dependia do 3D).
- Commits da onda: `1ab7e6999` … `HEAD` (`git log --oneline 5d596eaaf..line/Vector`).
- Integrar: `/pd-integracao line/Vector` (DIRETRIZ §1.5.3). Só por ordem do dono.

## 1. Superfície de colisão

| sítio | o quê | natureza |
|---|---|---|
| `crates/ph2d-skeleton/src/lib.rs`, `centro.rs` | `Skin` ganha dois campos PRIVADOS de cache (`angulos`, `juntas: OnceLock`); `PartialEq` passa a manual (ossos + lei) | API pública igual, resultado igual AO BIT (gate) |
| `crates/ph2d-skeleton/src/centro_cache_tests.rs`, `centro_tests.rs` (`cadeia3` → `pub(super)`) | gate + sonda | novos |
| `crates/ph2d-skeleton-live/src/skin_desenho_frente*.rs` | a malha fina dos cobridores (A10); `posa_em`/`posa_com`/`posa_no` | interna |
| `crates/ph2d-skeleton-live/src/skin_desenho_frente_fina{,_tests}.rs`, `skin_desenho_camadas_vinco_tests.rs` | novos | — |
| `crates/ph2d-skeleton-live/src/skin_desenho_fendas{,_tests}.rs`, `skin_desenho_efeitos.rs` (`uniao_dos_fechados` chama a lei) | as passagens (A12) | interna |
| `docs/Skeleton/01_a_fila.md` (F59–F63), `docs/Skeleton/handoffs/` | docs | — |

- **Nenhum** esquema (`PROJECT_SCHEMA`, `VEC_SCENE_SCHEMA`), registo, ADR, contrato congelado (§6) ou
  pacote novo. Cargo.lock intocado.

## 2. As ondas (números MEDIDOS; detalhe na fila)

- **F60 — a ponta do vinco (A10).** Quem decide «tapado» passa a ser a malha dos COBRIDORES posada
  pela pele exacta, partida sob pedido até `0,1` largura (`skin_desenho_frente_fina.rs`). Régua da
  convergência contra o lado 32: hoje `2,11` larg. a 160° → lei `≤ 0,019` (150–175°); varredura
  100–180° de 2,5° `≤ 0,043`. Preço (mesmo processo, mínimo de 7×20, loadavg ~3): 110↔150° `959 → 1 090`
  µs/forma/quadro, 160↔175° `928 → 1 405`; sem dobra igual (saída rápida). Gate
  `a_ponta_do_vinco_converge_para_a_pele_exacta` (controlos: campo `> 0,5`, 1.º degrau `> 0,03`).
- **F61 — as passagens (A12, «sim» do dono).** Na união dos efeitos cozidos, a passagem NOVA (o arco
  tem um cruzamento) mais estreita que o traço — fenda ou ponta — corta-se pela corda; buracos e
  ilhotas que o traço engole inteiros ficam (F59-b). FOTOGRAFADO na `=5` a 100/105/110/120°. Preço
  `~12 → 0,38–0,58 ms` por chamada. 9 gates (`skin_desenho_fendas_tests.rs`).
- **F62 — a cache da pele.** `108,1 → 34,0 ns` por ponto (3,2×), ao bit (gate
  `a_cache_da_pele_da_o_ponto_de_antes_ao_bit`). Vale para todo o desenho preso.
- **F63 — o efeito animado numa forma presa:** fechado no código (só as de deslocamento de camada
  ficam vivas, e o solver delas corre em fundo).
- **Mutação:** A10 8 (3 sangram, 3 ganharam o controlo do 1.º degrau e sangram, M8 equivalente, M2
  sobrevive sem caso — margem declarada); A12 9 + 2.ª ronda (todos sangram menos N8, equivalente e
  retirado); pele 3/3.

## 3. Prova de fecho

Gate batched 1× (agente `verificador`, base `f4a5594a4` — o diff apanha também o `main` de 05/10 que a
linha recebeu por fast-forward; tudo verde):

| portão | resultado |
|---|---|
| `nextest-impacted.sh` (BASE `f4a5594a4`) | ✅ 19 927 / 19 927 |
| `cargo clippy --workspace --all-targets -D warnings` | ❌ 9 avisos, todos desta onda (tipo complexo ×2, `arco` só usado em teste, `div_ceil` ×4, laço indexado, `if` colapsável) → corrigidos em `…` e o clippy das duas crates limpo |
| `cargo fmt --all --check` | ✅ |
| `file_loc_caps` (4/4) · `architecture` da shell (3/3) · `architecture` do editor-core (102) | ✅ |
| `censos-da-arvore-combinada.sh` | ✅ 114/114, 12 de 12 censos |
| ficheiros da onda acima de 700 linhas | nenhum |

Depois das curas: `ph2d-skeleton-live` 301 ✅, `ph2d-skeleton` 97 ✅, `ph2d-app-vec smoke_bone` 60 ✅.
Mutação: §2. Fotos: as cenas `=6` e `=5` a abrir nas telas novas (painel **Bones** com
**Transform**; na Hierarchy **Copias bone 2**, **Zig Zag bone 2**).

## 4. ABERTO

- **A5-a** — a cúspide da imagem junto à tampa: a hipótese *marching squares* foi MEDIDA e
  REFUTADA (fila §F59, tabela das três leis; ramo `exp/a5a-marching`, `50328de56`). Sem lei
  candidata: falta perceber porque a costura sobre a tinta remenda sobre OUTRO membro.

## 5. A linha do `CLAUDE.md` §5 (para o integrador)

O «Último» do módulo Vector + Esqueleto passa a apontar para ESTE handoff.

## 6. Smoke (o dono)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=6 cargo run -p ph2d-host-desktop --profile smoke
```
(1) duas barras de cópias dobradas em S; (2) Hierarchy → **Copias bone 2**, painel **Bones** →
**Transform**, dobrar quase sobre si; (3) na ponta da dobra o contorno de trás acaba onde a parte da
frente começa, sem gancho; (4) errado = um pedaço de contorno a passar por cima da parte da frente
junto à dobra. Depois `=5`: **Zig Zag bone 2** → **Transform**, dobrar até ~100°: as duas marquinhas
escuras saem, os buracos pequenos ficam. FOTOGRAFADAS as duas cenas a abrir (telas novas).
