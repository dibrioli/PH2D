# HANDOFF de CONTINUAÇÃO — `line/sculpt3d` · etapa 4, CAMADAS na peça: a W1 fechada (2026-10-02)

> **Para quem é:** o agente que assume a linha para a **W2** (pintar na camada activa) — pelo bloco
> do [MODELO_TROCA_DE_AGENTE_NA_LINHA](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md).
> **Não é** handoff de integração: o de integração em vigor é o
> [A_INCLINACAO de 02/10](HANDOFF_INTEGRACAO_line_sculpt3d_A_INCLINACAO_2026-10-02.md), e o da etapa 4
> escreve-se no fecho da linha (depois da W7, ou quando o dono mandar integrar).

## §0. Estado

| | |
|---|---|
| ramo · worktree | `line/sculpt3d` · `Worktrees/line-sculpt3d` |
| merge-base | `1ad60a1ce` |
| plano | [`docs/3D/30`](../30_plano_camadas_e_efeitos_na_peca.md) — decisões do dono §8, **o que a W1 fez e as premissas derrubadas: §10** |
| `SCULPT_DOC_VERSION` | **5 → 6** (degrau: um v5 abre ao bit; regravado é UMA camada opaca, cor a ≤ ½ degrau sRGB8, relevo ao bit) |
| `PROJECT_SCHEMA` · contratos §6 | intocados |
| fora da família | `ph2d-painter-effects` (`AdjustmentKind::reads_the_image_layout` + gate) · `ph2d-tool-painter` (`has_spatial_adjustment` lê essa pergunta — mesmo conjunto, comportamento idêntico) · `Cargo.toml` da família ganha `ph2d-color` |
| smoke do dono | ✅ o da §9 do relevo (handoff A_INCLINACAO §7) APROVADO a 03/10; a W1 não tem smoke próprio — nada muda na tela |

## §1. Ficheiros da W1

- `crates/ph2d-app-sculpt3d/src/pilha_da_peca.rs` (+ `_tests.rs`, 6 gates)
- `crates/ph2d-app-sculpt3d/src/doc_camadas.rs` (+ `doc_camadas_tests.rs`, 5 gates) · `doc.rs` (v6, `ate_v5`,
  `tinta_das_camadas`, `plano_vazio`, `uniforme`) · `doc_tinta.rs` (`Amostra for [u8; 4]`)
- `tinta_da_peca.rs`: `semente(mesh, k)` — a porta única da semente (a `garante` e o fundo da pilha)
- gates antigos com régua mudada (cor ao bit → meio degrau; v5 via `encode_v5`): ver `docs/3D/30` §10.3

**Verde (02–03/10, perfil de teste, `load < 4`):** `ph2d-app-sculpt3d` 382 · `ph2d-i18n` 23 ·
`ph2d-tool-painter -- compositor` 27 · `ph2d-painter-effects` (o gate do conjunto espacial).
**Mutação** (10 mutações nas portas `dobra`, `compor_faixa`, `apaga`, `achata`, `relevo_composto`,
`pinta_tinta`, `nova_mascara`, `byte_de`, `sincronizada`, leitor do v6): **10 / 10 sangram** — a M10
(plano repetido no ficheiro) sobreviveu à 1.ª corrida porque o caso do gate era recusado pela
CONTAGEM antes de chegar à repetição; curado. Pré-voo dos 17 `muta_*.sh`: limpos depois de re-ancorar
a P17 (`muta_o_plano_no_ficheiro.sh`: o escritor do relevo mudou-se para o `doc_camadas.rs`).
⚠️ Não há arnês versionado para estas 10 — ficaram na corrida do agente `mutacao`; o fecho da linha
deve escrevê-las num `docs/3D/ferramentas/muta_a_pilha_da_peca.sh`.

`cargo clippy -p ph2d-app-sculpt3d -p ph2d-painter-effects -p ph2d-tool-painter --all-targets -- -D warnings`
limpo (curou também um `chunks_exact(4)` latente na `sonda_camadas.rs`, de antes desta onda). O binário
do smoke (`--profile smoke`) está compilado nesta worktree no HEAD da onda: 2.ª corrida `Finished … in 0.23s`.

## §2. O que a W2 tem de fazer (a ordem importa)

1. **Pôr a pilha na `SceneObject`** (`pilha: Option<PilhaDaPeca>` ao lado de `tinta`), nascida pela
   `PilhaDaPeca::de_tinta` onde o plano nasce (`garante_no_orcamento`) e instalada pelo `decode`
   (hoje ele compõe e DEITA a pilha fora — `LoadedPiece` só leva o `Tinta`).
2. **Redireccionar os SEIS escritores do `Tinta`** para a camada activa no MESMO passo (senão o
   `Ctrl+S` grava uma pilha velha): `ph2d_sculpt3d::tinta_fina` (dab + tela do Painter),
   `preenche`, `tela_semente`, `uniformiza` (remesh — é a W7, mas tem de reamostrar CADA camada),
   `history_tinta_fina` (desfazer por camada), e o `doc`.
3. O `encode` passa a gravar a pilha da peça (e não a `de_tinta`).
4. Precisão: o traço acumula em `f32` no gesto e quantiza à camada no FIM (§10, «Para a W2»).
5. Critério de desistência da W2 (doc 30 §7): o incremental passar de `1 ms` por quadro a `8x`.
6. ⏳ Medir a RAM da pilha com o histórico de desfazer (doc 30 §6 prometeu-o à W1; sem pilha na
   `SceneObject` não havia o que medir).

## §3. Armadilhas pagas nesta onda

- A cauda da dobra (`1024·H − N` amostras) tem de ficar a ZERO em toda camada nova, senão o ficheiro
  não volta igual (`nova_mascara` pintava-a de branco).
- O `cargo-check-narrow.sh` só mostra ERROS: avisos (`dead_code`) só aparecem num `cargo check`
  cru. As operações da porta estão em `#[cfg(test)]` até a W3 (o painel) as chamar.
- O metadado da pilha custa `46 B` fixos por plano: fixtura de «plano por pintar» precisa de `N`
  grande (o gate `as_duas_formas…` passou ao degrau `5`).
