# HANDOFF de CONTINUAÇÃO — `line/sculpt3d` · etapa 4, CAMADAS na peça: a W2 fechada (2026-10-03)

> **Para quem é:** o agente que assume a linha para a **W3** (o painel de Layers mostra a pilha da
> peça) — pelo bloco do [MODELO_TROCA_DE_AGENTE_NA_LINHA](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md).
> **SUPERSEDE** o [de W1](HANDOFF_CONTINUACAO_line_sculpt3d_CAMADAS_W1_2026-10-02.md). Não é handoff de
> integração: o de integração em vigor é o [A_INCLINACAO](HANDOFF_INTEGRACAO_line_sculpt3d_A_INCLINACAO_2026-10-02.md)
> (smoke dele ✅ aprovado a 03/10); o da etapa 4 escreve-se no fecho da linha.

## §0. Estado

| | |
|---|---|
| ramo · worktree | `line/sculpt3d` · `Worktrees/line-sculpt3d` · merge-base `1ad60a1ce` |
| plano | [`docs/3D/30`](../30_plano_camadas_e_efeitos_na_peca.md) — W1 = §10, **W2 = §11** (desenho, portas, medições, o que fica) |
| `SCULPT_DOC_VERSION` | `6` (W1) — a W2 não mudou o formato |
| `PROJECT_SCHEMA` · contratos §6 | intocados |
| fora da família | W1: `ph2d-painter-effects` (`reads_the_image_layout`) · `ph2d-tool-painter` (o compositor lê-a). **W2: `ph2d-mesh-colors` (`alfa.rs`, o canal de opacidade) · `ph2d-sculpt3d` (as leis de cor com opacidade: `tinta_fina`, `tinta_fina_anel`, `tela_na_malha_pousa`, `preenche`, `tela_semente`)** — nenhuma assinatura pública partida fora da família, salvo `TintaDoTraco::repinta`, cuja lei passa a receber e devolver `(cor, opacidade)` |
| a shell | não tocada |

## §1. O que a W2 deixou (detalhe no doc 30 §11)

- O traço pinta uma **cópia de trabalho da camada activa** (cor pré-multiplicada + opacidade); a
  peça (o `Tinta` composto) fica na peça e é o que a placa lê. Sobre `alfa = 1` toda lei é a de antes
  ao bit (gate `numa_camada_opaca_o_traco_e_o_de_hoje_ao_bit`).
- Cada quadro: `tinta_da_peca::pilha::desce_do_traco` → RGBA8 na camada, recomposição só das sujas.
- A pilha anda com o plano: `garante_com_pilha` (estaciona/volta), `acompanha` (nasce e RECOMPÕE o
  plano — invariante: **a peça é sempre a composição da pilha**), `empresta_da_peca`,
  `devolve_camada`, o balde (`preenche_camada`), o desfazer (`JanelaFina::do_traco_na_camada`,
  `PlanoInteiro::da_camada`, `troca_na_peca`), o ficheiro (`doc_camadas::encode`, `LoadedPiece::pilha`).
- Portas que ficaram sem chamador no produto e passaram a `#[cfg(test)]` (são o arnês dos gates da
  lei): `tinta_da_peca::empresta`, `garante_no_orcamento`, `PlanoInteiro::de`. As operações da porta da
  pilha (`nova_camada`, `apaga`, `duplica`, `nova_mascara`, `novo_ajuste`, `define_*`,
  `define_activa`) continuam `#[cfg(test)]` **até a W3 lhes dar o painel**.

## §2. Verde

| corrida | resultado |
|---|---|
| `ph2d-app-sculpt3d` (sem placa) | 386 + os 4 gates novos de §4 |
| `ph2d-sculpt3d` | 688 + 7 novos · `ph2d-mesh-colors` 63 + 1 |
| produto com placa (`--ignored tinta_no_produto`) | **42 / 42** (Painter, aquarela que escorre, balde, desfazer, e os 2 novos das camadas) |
| mutação W1+W2 (`docs/3D/ferramentas/muta_a_pilha_da_peca.sh`, **versionado**) | 24 mutações; pela corrida dos agentes 10/10 (W1) e 14/14 (W2, depois dos 7 gates que as 6 sobreviventes pediram). A corrida do arnês versionado (03/10, com placa): **23/24 + a M2 à mão = 24/24** — a M2 abortava por um defeito do próprio arnês (a substituição não compilava), âncora corrigida e provada sozinha (6 vermelhos) |
| pré-voo dos 18 arneses de escultura | limpos (6 re-ancorados: P6 P9 P24 L7 M30 M32 A1 A3) |

## §3. Medido (doc 30 §11)

Um quadro de traço (300 sujas) `0,038 ms` em todo degrau — o critério de desistência da W2 (`1 ms`
a `8x`) passa 26×. ⏳ O pen-down custa a cópia da camada: `0,30 ms` a `8x`, **`22,7 ms` a `64x`**.

## §4. O que a W3 tem de fazer

1. **O painel de Layers do Painter mostra a pilha da peça** quando se pinta nela
   (`ph2d-panel-painter-layers`: `set_current_layers`), e as suas operações vão à **porta da pilha**
   (tirar os `#[cfg(test)]` das operações; o desfazer delas é estrutural — a pilha inteira, metadado +
   planos tocados).
2. Os dois estados que o painel tem de DIZER (doc 30 §4): fora do Painter-na-peça a pilha não aparece;
   um efeito que lê a vizinhança aparece **desligado com a frase do porquê** (`RecusaDaPilha`).
3. A activa que não é raster: hoje o pen-down não empresta (o traço pinta só a cor por vértice) e o
   balde recusa no terminal — o painel tem de impedir ou dizer.
4. O seam test (`ph2d-ui-testkit`): «nova camada → pintar → baixar opacidade → `Ctrl+Z`» muda a cor da
   peça e volta ao bit.
5. ⚠️ Mudar a opacidade/modo de uma camada recompõe a PEÇA INTEIRA: `33,5 ms` a `64x` na CPU (doc 30
   §6) — é a W1b (compor na placa) que o resolve; a W3 pode viver com isso até lá, medindo.

## §5. Armadilhas pagas nesta onda

- O shell das ferramentas é **zsh**: `git commit -- $F` com `F` = lista não se parte em palavras —
  corra por `bash -c`.
- O `cargo-check-narrow.sh` só mostra ERROS; os avisos `dead_code` só num `cargo check` cru.
- Os censos de fiação (`tinta_fiacao_tests`, `a_voz_da_tinta_fina_e_armada_pela_porta`) e os arneses
  procuram TEXTO: mudar a linha que eles citam é reapontá-los no mesmo commit (foram 6 aqui).
- Uma agulha de censo que cite uma linha partida pelo `fmt` tem de citar a linha como o `fmt` a deixa.

## §6. Smoke e binário

Clippy `-D warnings --all-targets` limpo nas cinco crates tocadas. O binário do smoke está compilado
nesta worktree no HEAD da onda (`34a98497c`):

```
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.27s
```

Smoke da W2 ao dono (nada novo na tela — sem o painel não se cria camada): a pintura fina, o Painter
na peça, o impasto e o `Ctrl+Z` têm de se comportar EXACTAMENTE como antes (cena `52`).
