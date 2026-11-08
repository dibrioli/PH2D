# Handoff de continuação — Add Component (GEF 25, 08/11)

**Estado: smoke do dono OK (08/11).** Commits na `line/GameEngineFuncional` (`6727c42c71..158a9ea359`). Plano:
[25_add_component_plano.md](../25_add_component_plano.md). Decisões do dono (itens 1–6):
[memória](../../../project-memory/project_add_component_owner_decisions_2026_11_08.md). Não reconstruir.

## O que existe

- **11 assuntos** em `ph2d-game-dicas/src/componente.rs` (catálogo por baixo inalterado; é apresentação); 64→63
  dicas com desenho; cascata «+N» (`component_palette`); `PaletteItem.badge` pílula (`195731c805`).
- **Recibo** `ph2d-panel-inspector/src/recibo.rs`: Undo = `EditorAction::UndoStep`; sumiço por outra seleção/passo;
  **×** `INSP_RECEIPT_CLOSE` → `recibo::fecha` (`82177a20a3`).
- **Duplo clique real** (`pointer_down`, ramo modal, `record_pointer_down`) e **foco da busca** só após clique em
  `CMD_PALETTE_SEARCH` ou tecla (`ScopeState.search_focus`; clique em item desfoca) — `fa118a5c56`, `a10cefe30b`.
- **Câmera**: `GameCamera` Intrinsic, `ObjectKind::Camera`; Follow/Limits/Shake só em câmeras; Shake Emitter foi
  para «Game rules» (sem grupo de 1 entrada em imagem nova) — `edb897c7e4`, `9fa1c1f6fc`.
- **Ícone da câmera no canvas** (`crates/ph2d-camera-frame/src/tinta.rs`): selecionada pinta em `ObjCamera` (antes
  era sempre `Selection`) e leva a sombra escura (regra GEF02: `BgScrim` α0.75, `Thin` de cada lado, sombras antes
  das cores) — `156c9dc70a`, `158a9ea359`.
- Três vermelhos vizinhos curados em `709f4732c6`.

## Provas

Verdes por nome, mutações RED e restauradas: `componente_tests` (4 gates); `component_palette_tests`
(`a_cascata_e_um_mais_n…`, `os_grupos_seguem_os_onze_assuntos…`);
`a_busca_so_acende_o_foco_depois_de_um_clique_ou_de_uma_tecla`;
`um_clique_na_busca_da_lhe_o_foco_e_a_paleta_nova_nasce_sem`;
`o_recibo_vive_so_enquanto_o_passo_dele_e_o_topo` (agora com ×);
`a_camera_selecionada_pinta_o_icone_na_cor_dela`; `o_icone_da_camera_tem_a_sombra_por_baixo_da_cor`;
`a_game_camera_so_nasce_pelo_objeto…`; `add_object_dois_cliques_reais…`/`add_component_dois_cliques_reais…`;
`a_plaquinha_cabe_na_pilula…`.

## Smoke (fotos)

`PH2D_OBJECT_ADD_PICK=object_add.game.sprite PH2D_COMPONENT_ADD_SMOKE=1 [PH2D_OBJECT_ADD_QUERY=<texto>]
[PH2D_COMPONENT_ADD_PICK=ph2d::physics::NavAgent]`; câmera: `PH2D_OBJECT_ADD_PICK=object_add.game.camera`.
Foto: `docs/Components/ferramentas/fotografa_cena.sh` (variáveis extras antes do `bash`).

## Aberto

- ⚠️ LOC no limite: `card_field.rs` 499/500, `fase_object_add_tests.rs` 600/600 — não acrescentar.
- **Devido no fecho da LINHA** (não feito): mutações do Add Object desde o 1.º commit; a rodada anterior sem
  handoff; link do CLAUDE.md §5 (Game Engine) para o handoff de integração; `/pd-linha-fechar` (fmt/clippy das
  crates tocadas 1×, auditoria). Push/integração SÓ por ordem explícita do dono. A `line/GameEngineFuncional02`
  (física em cartões) é separada; a ordem é do integrador.
- **Fila** ([23_refundacao_plano.md](../23_refundacao_plano.md) «A casa»): Animated Sprite · várias folhas (dono
  08/11) · Inspector e Hierarchy · Prefab. O dono confirma o próximo.
