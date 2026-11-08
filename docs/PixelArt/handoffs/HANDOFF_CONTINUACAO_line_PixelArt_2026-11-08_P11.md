# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, P11 (Slices) — 2026-11-08

> Leitor: a próxima janela da linha, que abre a **P12 — Ergonomia**. A linha continua; não integra nem faz push (o dono não ordenou).
> Anterior: [P10c](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-07_P10c.md). Plano da onda: [P11_plano.md](../P11_plano.md). Oráculos: [arquivo](../P11_oraculos_arquivo.md) · [ferramenta](../P11_oraculos_ferramenta.md).

## Estado

- Branch `line/PixelArt`, worktree `Worktrees/line-PixelArt`, 31 commits à frente do `main` (`79989e9e0b` … `558b01b127`, mais o deste handoff). Árvore limpa.
- **Smoke do dono: cena 24 (`PH2D_PIXEL_SMOKE=24`), duas rodadas.** 1.ª (08/11): virar/girar seguindo a arte OK, G4 OK, e pedidos novos. 2.ª (08/11, depois da auditoria): **«smoke OK»**. Fotos: `P11_smoke2_01..07_*.png` nesta pasta.
- P11a + P11b fechadas juntas (§7 do plano). Nada de push.

## O que existe (não reconstruir)

- **Dados e formato:** slices no documento com keys por quadro, `Op` com desfazer, `.ase` (pivot misto com `ph2d_aseprite::NO_PIVOT`), `meta.slices` no JSON, `Split slices` no Export Sheet, `Area` no Export As (`ph2d-pixel-core/src/slice.rs`, `ph2d-pixel-io/src/sheet/{fatias,nomes}.rs`).
- **Leis do dono (§4.3–4.4, §9):** a slice segue a arte em quadros (`after_frames` com mapa novo→origem: a cópia leva a slice da origem) e geometria (key escondida incluída); Transform numa cel ligada move a key em todos os quadros que mostram aquela arte.
- **Ferramenta:** `Shift+C`, gesto puro em `ph2d-pixel-slices` (`gesto`, `alcas`, `seta`, `transforma`, `campos`). Pairar e premir partilham `seta::sob()`: realce em `ph2d-pixel-render/src/overlay.rs`, cursor Ew/Ns/Nwse/Nesw/Move. Transform mostra os pixels ao vivo (`transforma::previa`).
- **Janela SLICES** (`ph2d-panel-pixel-slices`): lista, All/None, Transform, cartão `Slice Properties` com campos Figma (letra dentro, arrastar a letra, 1 desfazer por arrasto, X Y / W H em duas linhas), seleção mista com traço, menu do botão direito.
- **Borda das janelas acessórias** (SLICES e Tilemap): `editor-core` `panel/cards_borda.rs` + `interaction/state/accessory_width_ops.rs`; largura em `~/.ph2d/sections.txt`; a janela fica fora da lista docada (`clear_of_left_list`).
- **Esc num campo de texto** larga o foco e o desenho do campo em todo o app (`editor-core/src/interaction/dispatch/key.rs`).
- **Drivers Xvfb** (scratchpad da sessão, não versionados): passo por linha `foto nome` / `espera s` / xdotool; teclado só chega com `windowfocus --sync`.

## Provas

- Testes por nome verdes ao longo da onda: `ph2d-pixel-slices` (`tests/ao_vivo.rs`), `shells/pixel-scenes` (`slices.rs`, `slices_janela.rs`, `camadas.rs`), `ph2d-panel-registry-init` (`the_pixel_slices_answers_a_real_pointer`, `the_slices_window_widens_by_its_outer_edge`), `ph2d-panel-pixel-tools` (`seam_pixel*`), `the_shell_only_shrinks`.
- Mutação: fixtures que separam as hipóteses (`7e055973e3`, `e67b12e679`, `ec64e3bb3b`, `52ae993cd6`).
- Auditoria de 2 lentes: 10 achados, todos curados (`a7a9446538` … `8e66a82d8e`).
- ⚠️ **Não correu:** suíte completa (é do integrador). O clippy das crates tocadas só correu até `51dc224051`. **Antes de integrar, o fecho da linha (`/pd-linha-fechar`) corre fmt + clippy 1× sobre tudo o que a P11 tocou.**
- `ph2d-app-pixel` está em ~11 412 de 11 415 linhas: a P12 **não cabe** lá. Código novo vai para crate.

## Abertos (conhecidos, o dono não pediu)

- O tooltip do campo F tapa a linha de baixo. É do app inteiro; ficou como está.
- A cura E (offset do Tilemap) não tem teste vermelho.
- Os números do cartão não acompanham ao vivo durante um arrasto na tela; atualizam ao soltar.
- Abrir o Export Sheet esconde a janela SLICES.

## Abrir a P12 por aqui

**P12 — Ergonomia** (`02_plano_modulo.md` §5, linha 188): atalhos por omissão iguais aos do Aseprite e editáveis, Run Command, Undo History, autosave e recuperação, user data/propriedades, preferências do módulo (grade, onion, xadrez).

- ⛔ Exceção do dono (07/11): balde = `P`, Gradient = `Shift+P`; o `G` é da grade do canvas em todo o app. Slice = `Shift+C`.
- O plano diz «Smoke 13», mas está velho: a próxima cena do Pixel é **25** (confira `max_level` em `crates/ph2d-app-pixel/src/smoke.rs`).
- A P12 **não tem plano próprio**. Comece por `docs/PixelArt/P12_plano.md`: confira no código o que o app já tem (atalhos editáveis, autosave, histórico de undo: a casa pode já ter peças, CLAUDE.md §5.0) e meça o Aseprite Trial como oráculo que se CORRE (§0.9: lista de atalhos, Run Command, Undo History, recuperação). Mostre as decisões ao dono no formato da P11 (§9 do plano) antes de código.

## Regras que continuam

UI só em inglês, cartões compactos; zero hex e zero literal de UI; teste por nome, suíte só na integração; mover código para crate, nunca subir o tecto; integrar e push só por ordem do dono; handoff e prompt só depois do smoke dele.
