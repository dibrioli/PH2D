# HANDOFF DE CONTINUAÇÃO — linha Pixel Art reaberta (02/11)

> Base: `main` `3df5894489` (a P6h–P6l já dentro; `git cherry main HEAD` = 0, sem rebase). Anterior:
> [integração P6h–P6l](HANDOFF_INTEGRACAO_line_PixelArt_2026-11-01_P6h_a_P6l.md). Esta nota **corrige o §8 dele**.

## 1. Pendências do fecho anterior (§8.1) — feitas

- `cargo machete` nas 8 crates do Pixel (`ph2d-pixel-{core,io,render}`, `ph2d-app-pixel`, `ph2d-panel-pixel-{tools,timeline,preview}`, `shells/pixel-scenes`): zero deps sem uso.
- `cargo audit`: **RUSTSEC-2026-0433** (`pulldown-cmark` 0.13.5, parsing quadrático de wikilinks; publicada 28/10). Único consumidor `ph2d-kanban-md`. Cura: `Cargo.lock` 0.13.5 → **0.13.6** (patch, sem tocar no `Cargo.toml`); `cargo test -p ph2d-kanban-md` 13/13 verde. Fica só o aviso já permitido `RUSTSEC-2026-0186` (`memmap2` 0.5.10, *unsound*). `cargo deny` advisories passa a verde pelo mesmo motivo.
- `agent-loop-profile.sh` e `rm -rf target/*/incremental`: corridos no fim desta jornada.

## 2. Itens do §8.5 que JÁ estavam fechados (não reconstruir)

| Item da lista | Estado real | Prova |
|---|---|---|
| Gravar com forma viva a assenta (P6d §6) | **feito na P6f** | `crates/ph2d-app-pixel/src/persist.rs:20` (`viva::assenta` antes de serializar), chamado de `shells/desktop/src/project_save.rs:162`; teste `tests/it/ajustes.rs:159` |
| Etiqueta «Points: n», só-contorno a meio do arrasto, barra da moldura com a viva, Ctrl+Z na viva (resto do P6d §6) | **feito na P6f** | `viva_ajustes.rs:198`, `viva.rs:125`, `panel.rs:214`, `input_teclas.rs:112`; testes `tests/it/ajustes.rs:60/76/110/179` |
| `LengthDisplay` na etiqueta do Pixel | **DECIDIDO na P6g: fica «{n} px» de imagem** | [P6g §4](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6g.md) + emenda P6g do ADR-0196: `LengthDisplay` é para comprimento de MUNDO; a célula não tem tamanho fixo no mundo (o sprite escala). Não reabrir sem pedido do dono. |
| Paletas Tint/Shade/Tone, Spectrum, rodas RGB/RYB | **existem** | `crates/ph2d-panel-pixel-tools/src/seletor.rs:170-290`, ids em `ids.rs:370-382`; testes `tests/it/seletor_leis.rs`, `seletor_arrasto.rs` |

## 3. ABERTO (na ordem)

1. Por medir no Trial (P6j §5): cópia com Ctrl numa camada contínua; bloco de cels largado fora da grelha; Paste as New Sprite em si.
2. Plano P7–P13 ([02_plano_modulo.md](../02_plano_modulo.md)): P7 pincéis + dinâmica (`ph2d-pen`) · P8 gradiente/texto/ajustes/FX · P9 export/import (sprite sheet, GIF/APNG/WebP, `.ase` ler/escrever) · P10 tilemaps · P11 slices · P12 ergonomia · P13 rampas OKLCH, color cycling, etc.
3. GPU dos 19 modos de mistura: por medir (P3 §6).
4. RotSprite e empates do Fast: recusas MEDIDAS (`../ferramentas/oraculo_rotacao.md`), só com medição nova.
5. Outras listas reordenáveis do app herdam voo + barra + queda (memória `feedback_every_drag_reorder_flies_like_the_kanban_card.md`).

## 4. Para o integrador

Superfície desta jornada: `Cargo.lock` (2 linhas, `pulldown-cmark` versão + checksum) e este ficheiro. Outra linha que também suba o `pulldown-cmark` ⇒ regenerar o lock, nunca à mão.
