# F31 — a identidade das camadas do Vetor (plano, 25/10)

Fila: [F30 handoff §6.1](handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-25_F30.md) · decisão técnica (não reabrir):
[F30_plano §A2](F30_plano.md) · lei: [o Auto Key grava tudo + decisões de 20/10](../../project-memory/project_autokey_records_everything_2026_10_19.md).
Base: ponta `36c29f1e5e` sobre o main `c3662cd8e4` (o main não andou).

## 0 — Medição: o mecanismo de cada defeito (uma família, CLAUDE.md §0.10)

A chave VETOR (`FieldKey { familia, campo, i, j }`, `ph2d-ecs/src/canal.rs:21`) endereça POR POSIÇÃO em três listas; apagar ou
mover desloca as animações para a vizinha.

| # | lista | endereço hoje | onde se mexe na lista | porque a chave muda de dono |
|---|---|---|---|---|
| 1 | **tintas da pilha** `p.paints` (`PaintEntry`, `ph2d-vec-scene/src/paint_stack.rs:98`) | `i = k+1` (`0` = tinta de base) — `canais_campos.rs:119-150,171-176`; campos `TINTA_*`, `TRACO_*`, `PINCEL_*`, `PILHA_*` (`por_i`) | `ph2d-app-vec/src/paint_stack.rs` `add :203` · `remove :221` · `shift :233` | apagar `k` faz `k+1` responder pelo `i` dele |
| 1b | **efeitos da pilha** `p.effects` (`FxEntry`, `effect.rs:148`) | `i = e` — `canais_campos.rs:177-182`; `EFEITO_LIGADO`/`EFEITO_PARAM` | `fx_bridge.rs` `add :95` · `remove :106` · `reorder :117` | idem |
| 2 | **filtros da forma** `VecFilter::ops` (`ph2d-ecs/src/vec_filter.rs:23`, componente ECS) | `i = op` — `canais_filtros.rs:72-90,100,157` | `fase_filter_commands.rs:14` (Add/Remove) · `move_up/move_down` (`vec_filter.rs:63,72`) | idem |
| 3 | **contornos** (`p.verts` + `p.subpaths`) | `i = contorno` (forma inteira = um grupo, `por_i = false`) | `VecPath::remove_contour` (`compound.rs:216`), `make/release_compound`, cortes | medido (F30 §A2): apagar o 0 ⇒ `propagar` = `None` (só compara o nº de pontos POR `i`), o `i=1` passa a tocar as chaves do apagado e as do último ficam órfãs |

Os leitores da posição fora do funil: os losangos do painel (`marcas.rs:236-330`, `Contexto` só tem `aberta`/`camadas`),
o nome da linha no dope-sheet (`ph2d-panel-timeline/src/linhas.rs:~208`, `«{nome} {i+1}.{j+1}»`).

## 1 — Desenho

**Identidade = `ItemId` (u32, `ph2d-item-id`), do `StableIdCounter` do projecto** — persistido, monotónico, fora do undo
(`stable_id.rs:122`); a mesma lei da F30 §A1. **Único por OBJECTO** (dentro de um caminho; a cópia de um caminho leva os
ids e é OUTRA entidade — a chave é `(entidade, FieldKey)`).

### A — a chave (ph2d-ecs + timeline)

1. `FieldKey` ganha `item: ItemId` (`NONE` em toda família que não o usa). `FieldKey::new` mantém a forma (item `NONE`);
   construtor novo `FieldKey::do_item(familia, campo, item, j)` (i = 0). `grupo()`: `por_i` com item ⇒ o sub é o item.
2. **Endereço VETOR durável:** campo de tinta/traço/pincel/pilha — base = `i 0, item NONE`; entrada = `i 0, item = id`.
   Efeito e filtro — sempre `i 0, item = id`. Contorno/vértice/forma/texto/param: inalterados (`item NONE`).
3. **Formato:** `DOC_VERSION` 23 → 24. Só `TargetBinding.prop` leva a chave (medido); a v23 lê-se por um âmbito de
   leitura da chave antiga (precedente literal: `with_stroke_layout_v22`, `ph2d-vec-scene/src/stroke_wire.rs:28`) —
   fixture `timeline_doc_v23.bin` + gate.
4. **Migração semântica** (`i` → id): a chave antiga é reconhecível PELA FORMA (VETOR, item `NONE`, e: campo de entrada
   com `i ≥ 1`, ou campo de efeito/filtro) ⇒ não precisa de bandeira e cura-se a si própria se um doc meio-migrado for
   gravado. Corre no `upkeep` de `ph2d-timeline-persist` (`lib.rs:124`, tem o `World`) contra o **espelho**
   `IdsDoVetor` — recurso em `ph2d-ecs`, `entidade → { tintas, efeitos, filtros: Vec<ItemId> }`, publicado pelo
   `ph2d-app-vec` a cada quadro (gémeo de `PosicoesDosItens`; uma entidade com algum id por atribuir não se publica).
   Índice fora da lista ⇒ fica órfã (lei existente). Alvo já ligado ⇒ não funde, avisa.
5. **Nome da linha:** `publish_object_names` publica em `channel_names` o nome com a POSIÇÃO tirada do espelho
   (o mesmo texto que o dope-sheet dava com `i`).

### B — as listas (ph2d-vec-scene + ph2d-app-vec + ph2d-ecs)

1. `PaintEntry.id`, `FxEntry.id` (`ItemId`, nascem `NONE`); `VEC_SCENE_SCHEMA` 23 → 24, a 23 lê-se pelo âmbito de leitura
   (ids `NONE`) — fixture + gate.
2. `VecFilter` ganha ids por op **sem poder divergir do comprimento**: `ops`/`ids` privados, leitura `ops()`, valores
   `ops_mut() -> &mut [FxOp]`, estrutura por `push`/`remove`/`move_up`/`move_down` (a cura do «reconciliar por
   COMPRIMENTO», F30 §6.2, não se repete aqui). Componente gravado ⇒ `PROJECT_SCHEMA` 195 → 196 com a forma 195 congelada
   (padrão `item_id_v194.rs`); os ids atribuem-se no `assign_missing_item_ids` (`item_id.rs:170`) que já reconcilia.
3. **Passe de atribuição do Vetor** (`ph2d-app-vec`), no MESMO quadro da edição e ANTES da captura do undo
   (`fase_autokey` corre em `fase_hero_commits`, antes de `post_frame_undo`, `main.rs:937`) ⇒ o id nasce no passo de
   undo da criação, e o redo devolve o mesmo id. Antes de alocar **reconcilia o contador com o máximo dos ids da cena**
   (a VecScene não está no mundo: o `reconcile_at_least` do load não a vê). Repetido no mesmo caminho ⇒ o 2.º recebe
   novo. Depois publica `IdsDoVetor`.
4. `canais_campos` / `canais_filtros`: a porta de tradução `posição ↔ id` fica num sítio só por lista; `campos` emite
   a chave durável, `ler`/`escrever` resolvem o id; entrada com id `NONE` não tem canal.
5. **Losango:** `marcas::Contexto` leva os ids da forma mostrada (entrada `k` ⇒ `tintas[k]`, linha de efeito `r` ⇒
   `efeitos[r]`), lidos do espelho.

### C — contornos (`canais_topologia::propagar`, lei do dono 20/10)

Apagar/inserir/reordenar um CONTORNO re-chaveia todas as chaves da forma: cada contorno de hoje casa com o de ontem de
âncoras IGUAIS bit a bit (os que a edição não tocou ficam iguais — a mesma lei do alinhamento de pontos); casado ⇒ os
valores dele em cada chave mudam de `i`; novo ⇒ entra com o valor de hoje em todas as chaves; trilhas de `i` que deixaram
de existir saem. `mudou()` passa a ver também o nº de contornos.

## 2 — Gates (cada um visto VERMELHO por mutação — agente `mutacao`)

| gate | mutação que o tem de pôr vermelho |
|---|---|
| `apagar_uma_tinta_da_pilha_nao_muda_a_chave_da_vizinha` | `campos` volta a `i = k+1` |
| `mover_um_efeito_leva_as_chaves_com_ele` | chave do efeito volta a `i = e` |
| `apagar_um_filtro_nao_muda_a_chave_do_vizinho` | chave do filtro volta a `i = op` |
| `apagar_o_contorno_0_leva_as_chaves_do_1` · `inserir_um_contorno_nao_mexe_nos_outros` | sem o casamento de contornos |
| `o_doc_v23_le_e_migra_para_o_id` (fixture v23) | a migração mantém a posição / o âmbito antigo desligado |
| `a_vec_scene_v23_le_com_ids_por_atribuir` (fixture v23) | o âmbito antigo desligado |
| `o_projeto_195_le_o_vec_filter_sem_ids` | a forma 195 lida pela nova |
| `o_passe_reconcilia_o_contador_com_os_ids_da_cena` | sem a reconciliação |
| `um_id_repetido_no_mesmo_caminho_e_refeito` | sem a deduplicação |
| `o_losango_da_entrada_aberta_e_o_do_id` | `Contexto` volta à posição |
| `o_nome_da_linha_da_entrada_e_a_posicao` | o nome volta a ler `i` |

## 3 — Smoke

Cena nova `PH2D_AUTOKEY_SMOKE=36` (conta no roteador: vai até 35): uma forma com duas tintas animadas e dois filtros
animados; o dono apaga a de baixo e o primeiro filtro e vê as animações das outras ficarem. Fotografada antes.

## 4 — Formato: contadores (DELTA para a integração)

`DOC_VERSION` +1 (23→24) · `VEC_SCENE_SCHEMA` +1 (23→24) · `PROJECT_SCHEMA` +1 (195→196) · `PH2D_AUTOKEY_SMOKE` +1 (`..36`).
