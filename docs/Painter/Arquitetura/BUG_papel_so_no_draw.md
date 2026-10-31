# BUG (dono 31/10) — o papel texturizado só aparece ao entrar no Draw

> *«Há um bug que ao criar o papel texturizado, ele só fica texturizado ao entrar no modo Draw.»*

## Mecanismo

O Object mostra os píxeis da **sprite**; o Draw mostra a composição do **documento** do Painter. O papel
texturizado só existia no segundo.

1. `shells/desktop/src/render_loop/fase_new_image_modal.rs:36` — o *New Image…* chama
   `spawn_blank_canvas(…, bg = 3, …)`.
2. `crates/ph2d-image-import/src/lib.rs:256-258` — o fundo `3` («Textured Paper») enche a sprite de
   **branco liso** (`2 | 3 => [255, 255, 255, 255]`, *«whose paper the Painter gives when it opens it»*).
3. `crates/ph2d-app-painter/src/paint_mode.rs:82` (`new_image_nasceu`) só **agenda** o papel:
   `PainterTool::agenda_papel_texturizado` (`crates/ph2d-tool-painter/src/tool/papel_texturizado.rs`).
4. O papel é servido só quando a sprite vira documento: `bind_document_corpo` →
   `serve_papel_texturizado` (`crates/ph2d-tool-painter/src/tool/documents.rs:164`), e o bind corre na
   fase do Painter (`painter_bridge_phases::bind_document`), que só acontece com o Painter em mãos — o Draw.
5. Ao sair do Draw, `aplica_papel_preset` já tinha levantado o `edited_since_bind` (`invalidate_composite`),
   e o `fase_painter_persist` assa a composição na sprite — por isso, depois da 1.ª ida ao Draw, o Object
   passava a mostrar o papel. Antes dela, branco liso.

Não é geração preguiçosa do tile nem upload por invalidar: o tile e o compositor estão certos; a sprite é que
nascia com outra coisa.

## Teste

`ph2d-tool-painter` · `tool::paint::tests::papel_texturizado_tests::o_papel_texturizado_ja_se_ve_no_object_sem_entrar_no_draw`
(`crates/ph2d-tool-painter/src/tool/paint/papel_texturizado_tests.rs`):

- os píxeis com que a sprite nasce (`PainterTool::pixeis_do_papel_texturizado`, o que o Object mostra) têm
  desvio de luminância > 1 nível — **vermelho visto** com o comportamento de hoje (sprite branca);
- abrir essa sprite no Draw (`bind_document`) compõe o **mesmo bit** (0 bytes diferentes) e o papel é o do
  dono (`#E6D9CC`);
- reabrir num Painter sem a agenda (sair sem pintar e voltar, ou outra sessão) dá o mesmo bit — o dente não se
  dobra (camada com o assado + papel por cima).

Verde depois da cura (os outros dois testes do ficheiro continuam verdes).

## Cura

- `PainterTool::pixeis_do_papel_texturizado(w, h)` — o assado (`run_full`) do documento novo com o papel do
  dono, num Painter descartável: a sprite NASCE com ele.
- `PainterTool::abre_a_sprite_nova` (chamada no `bind_document_corpo`) reconhece a sprite nascida e ainda
  não pintada (1.º píxel + hash do assado por tamanho; sem memória do tamanho, só assa para comparar se o
  1.º píxel é do tom do papel) e abre-a como o documento de que ela é o assado: fundo vazio + o papel.
- `ph2d_app_painter::paint_mode::pixeis_da_imagem_nova(size, bg)` — a porta da shell.

✅ **Shell ligada pelo coordenador:** `fase_new_image_modal.rs` chama `ph2d_app_painter::paint_mode::spawn_imagem_nova`
(a escolha abaixo vive na família, a shell não cresce). O esboço original era:

```rust
let spawned = match ph2d_app_painter::paint_mode::pixeis_da_imagem_nova(size, bg) {
    Some(px) => crate::image_import::spawn_rgba(
        sim, renderer, asset_db, cell, size, size, px,
        ph2d_core::Vec2::new(0.0, 0.0), ppm, atlas_asset_map, "Canvas",
    ),
    None => crate::image_import::spawn_blank_canvas(
        sim, renderer, asset_db, cell, size, bg,
        ph2d_core::Vec2::new(0.0, 0.0), ppm, atlas_asset_map,
    ),
};
match spawned { … }
```

Fica de fora (pré-existente, outra família): uma sprite JÁ pintada que se reabre noutra sessão continua a
trazer o papel assado na camada e o papel do cartão por cima.

## O portão que estava verde

`o_papel_texturizado_nasce_com_os_valores_do_dono` media só os valores do cartão Paper DEPOIS do
`bind_document` — o lado do Draw. Nenhum teste lia os píxeis da sprite (o lado do Object), e o
`every_new_image_choice_is_alive_under_the_mouse` (editor-core) prova só que o pedido `(size, bg)` chega à
shell. As duas metades eram medidas cada uma no seu modo; a régua nova compara-as ao bit.
