# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, P14 (O sprite vive na cena) — 2026-11-09

> Leitor: a próxima janela da linha, que abre a **P13 — O que só a PH2D tem**. A linha continua; não integra nem faz push (o dono não ordenou).
> Anterior: [P11](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-08_P11.md). Plano da onda (mecanismo de cada bloco, números, vermelhos): [P14_plano.md](../P14_plano.md) §11 — este handoff NÃO o repete.

## Estado

- Branch `line/PixelArt`, worktree `Worktrees/line-PixelArt`, 103 commits à frente do `main`; a onda P14 é `1eb5eff0a4^..5aa6af20bf` (+ o commit deste handoff). **Nada integrado.**
- **Smoke do dono (09/11): cenas 26 e 27 OK** (`PH2D_PIXEL_SMOKE=26|27`), depois de duas curas achadas no próprio smoke (abaixo). Fotos em [handoffs/](.) `smoke_P14_cena26_{editor,play,correr}_2026-11-09.png` e `smoke_P14_cena27_{editor,play}_2026-11-09.png` (os `.log` não se commitam).
- **Fecho da onda:** `cargo fmt` + `clippy -D warnings` das 40 crates tocadas limpos; 21 crates de testes da onda verdes POR NOME (lista por bloco no §11). ⚠️ **Não correu a suíte completa** (é do integrador).

## As duas curas do smoke

1. **O idle do Pixel Sprite nunca tocava em Play** (`current` vazio + Rules ⇒ `vai_para_o_inicio` retornava cedo; o Sprite do motor tocava desde o import). Cura em `crates/ph2d-app-sprite-anim/src/tick.rs::vai_para_o_inicio`: com Rules a animação continua a `current`, mas a FASE rebobina no Play como a de todo Autoplay. Commit `5aa6af20bf`; gate `os_dois_herois_comecam_no_mesmo_quadro_ao_entrar_em_play` (visto vermelho: 720 ms diferentes; verde: 0).
2. **«Pulinhos»**: trocar a forma de um corpo dinâmico pousado injetava subida de 2 px (o `try_update_contacts` do parry reaproveita os pontos de contacto guardados no referencial LOCAL da forma antiga; o rapier só zera o `workspace` no `ColliderChanges::SHAPE`). Cura `6fe79965df`: **rapier2d 0.36.1 VENDORIZADO** em `crates/ph2d-physics/vendor/rapier2d` (`VENDOR.md`; Apache-2.0, © Dimforge), ligado por `[patch.crates-io]` no `Cargo.toml` da raiz (linhas ~271–273, molde do `wgpu-core`); UMA mudança em `src/geometry/narrow_phase/pair_update.rs`: `ContactManifold::mark_shapes_deformed()` em cada manifold quando há `SHAPE`. Gate `ph2d-physics forma_encolhe::trocar_a_forma_de_um_corpo_pousado_nao_o_faz_pular` (vermelho 2,000 px; verde 0,000 px).
   - ⚠️ **MUDANÇA DE DEPENDÊNCIA: o INTEGRADOR tem de a aprovar** e conferir `cargo deny`, licença (`pacman`/crates.io, triagem por artefacto), `bash scripts/stack-audit.sh --tetos` e que o vendor continua fora do workspace. A fixture das cenas passou a um herói com pernas/olhos e slice `body` estável (`241446c058`, `ph2d_pixel_cenas::heroi::heroi_da_cena`); a `heroi_p14` dos gates ficou igual.

## Decisões do dono (09/11)

- **Play NÃO liga a física sozinho** («ainda não»): o interruptor Physics da barra de transporte continua desligado por omissão (ADR-0131). A cena 26/27 arma-o ao montar.
- **No editor, o Sprite importado anima e o Pixel Sprite fica parado — «deixe como está».** No Play os dois partem do quadro 0 no mesmo tique.

## Decisões técnicas (com o porquê)

- **A normal é do ASSET de imagem**, não do sprite (`normal_map`: cor → normal por `AssetId` no `LibraryDoc`; undo e ficheiro): é o desenho da CanvasTexture do Godot e da secondary texture da Unity — dois sprites da mesma imagem acendem igual. `ph2d_sprite_edit::normais` é a única verdade; a normal sem nome chama-se `<cor>_n`. Outro tamanho é recusado por chave i18n.
- **Pivô e collider por CÉLULA no Sprite do motor** (`SpriteCells`, `SegueCelula`, `campos_derivados_quando_algum`): o padrão da Unity; um pivô por sprite não segue a key do quadro.
- **`ph2d.collider` é DERIVADO do mundo ao gravar** (`ase_do_documento`), nunca guardado no documento Pixel; o leitor (`ler_com_colliders`) tira a marca do user data e o Open recria a filha `SegueSlice` no mesmo passo do nascimento.
- **Merge Down Normal×cor recusado; Flatten em duas camadas** (a cor e uma Normal Map à parte): fundir deitava a normal fora da cor. **Normal Map só em RGB** (em Indexed a normal passaria pela paleta; `SetLayerNormal` e a conversão recusados; `.ase` Indexed de fora com `ph2d.normal` entra sem a bandeira).
- **«Key held» = laço enquanto segura; soltar volta ao clip de entrada.**
- **Godot troca de quadro 1 passo depois** (convenção medida no oráculo 4.7.2): o produto segue o **Aseprite**, não o Godot; o `.tres` assa o pivô por quadro na `margin`; fps = 1000/mdc das durações; ping-pong desenrolado.
- **Família não depende de família** (`cbfdd16f28`): `ph2d-app-pixel` e `ph2d-pixel-luz` deixam de ver `ph2d-app-light2d`; a banda normal é composta na shell (`fase_vector_bands_doc.rs`); os gates de placa do Pixel na luz moram em `shells/pixel-scenes/tests/it/pixel_na_luz.rs`.
- **Recusado MEDIDO (não reconstruir):** `set_enabled(false)` no collider escondido (tira a massa, o corpo nem cai ⇒ `set_colliders_muted`); reancorar o centro de massa contra o pulinho (2,000 px igual); M pelas sondas da c0 de F (pior); normal de 16 bits em toda célula do átlas sem medir o custo.

## Superfície de colisão e contadores (DELTA do que esta linha traz)

- **`PROJECT_SCHEMA` 225 (main) → 229:** 226 animação · 227 `SegueSlice` · 228 normal (reescrito sem `SavedAsset.normal`; a normal vive no `LibraryDoc`) · 229 `SpriteCells`/`SegueCelula`. O integrador reconta: `python3 scripts/schema-recount.py`. **Blob Pixel MINOR 8** (`Layer.normal`, `Op::LayerNormal`; MINOR 7 = pivô). Sem leitura dos anteriores (§0.11).
- **Smokes:** `PH2D_PIXEL_SMOKE=1..27` (`max_level` 27 em `crates/ph2d-app-pixel/src/smoke.rs`); `PH2D_PIXEL_SMOKE_PLAY=1` (entra em Play) e `=2` (Play + segura → no quadro Play+60, por `ph2d_app_host::smoke_drive::seta_pedida` ⇒ `janela_do_jogo.rs::aperta_seta`). **O `CLAUDE.md` §5 diz `1..25`: o integrador acerta para `1..27`** (e a próxima cena é a 28).
- **Crates novas/partidas:** `ph2d-pixel-cena`, `ph2d-pixel-cenas` (`heroi`, `heroi27`, `patch::op`), `ph2d-pixel-luz`, `ph2d-pixel-ponte`, `ph2d-sprite-ase-import`; `region_doc` saiu da `ph2d-app-sprite-anim` para `ph2d-anim-editor`. Catracas de LOC: `ph2d-app-pixel` 11 261 (tecto 11 278), `ph2d-app-sprite-anim` abaixo do tecto depois de `b864de1feb`/`f8ce9c4d9c`; os 5 ficheiros partidos por assunto em `4acbc266d5`.
- **Tocados fora da pasta (append-only):** `ph2d-light2d-gpu` (momento `M`: `solver_momento.rs`, `hrc_momento.wgsl`, `PassList::com_momento`), `ph2d-light2d-ecs` (`CenaDeLuz::com_normal`), `ph2d-render` (`sprite.wgsl` `fs_main_n`, `TexeisDaNormal`, `definir_normal_texeis`, `caixas_com_normal`), `ph2d-physics` (+ vendor), `ph2d-ecs` (`FormaDesligada`), `ph2d-physics-ecs` (`set_colliders_muted`), `ph2d-sprite-sheet` (`JsonSlice.collider`), shell: `present_luz.rs`, `fase_vector_bands_doc.rs`, `janela_do_jogo.rs` (+4), `import_router`, `input_drop`.

## O que um merge pode partir

- A cadeia de `[patch.crates-io]` (wgpu-core + rapier2d): outra linha que mexa no `Cargo.toml` raiz ou no `Cargo.lock` conflita aí.
- Escada do `PROJECT_SCHEMA` e a tripla irmã (três sítios): recontar, nunca escolher.
- Censos do Inspector (a linha Normal Map conta: Animated 18→19 à vista, 49→50 no total) e `the_shell_only_shrinks`/`architecture_no_dependency_climbs_a_layer`.
- `sprite.wgsl`/`spriteRenderer`: quem parte o renderer (`renderer_normal.rs`) colide com a porta `definir_normal_texeis`.
- Os gates de placa (RADV) do Pixel na luz e do Sprite com normal precisam da vez da placa ([A_VEZ_DA_PLACA.md](../../DevOps/A_VEZ_DA_PLACA.md)).

## ABERTO (ordenado; só o que continua aberto)

1. **Dependência vendorizada do rapier:** aprovação, `cargo deny`/licença/`stack-audit` e, se o crates.io publicar a cura, remover o `[patch]` (do integrador).
2. **Defeito medido, não curado:** o `retune_body` da `heroi_p14` (a slice `body` de várias keys encolhe a forma por quadro; a cena usa 1 key, o vendor cura a física, mas a key por quadro de um corpo dinâmico continua a pedir cuidado).
3. **Luz com normal — M e entorno:** `(1,0)` dentro do opaco 0,17–0,18 (face de COSTAS; medir em absoluto antes de mexer); vértice do opaco 0,30 já no `F`; `M` em translúcidos (sem `transmitida`); miolo de `M` com região < grade; **entorno W5d (`PH2D_LIGHT2D_BORDA=0`) sem `M` de fora**.
4. **Normal de 16 bits só em textura PRÓPRIA** (`Rgba16Float`); a célula do átlas continua em 8 (medir o custo da irmã inteira em 16 antes). Medido: 8 bits erra até 2 níveis só em F/2π = 2.
5. **«À vista» por quad da instância:** malha POSADA que sai do quad e `LiftedInstances` (`gpu_extra` do Motion) não entram nas caixas ⇒ `M` não liga (a luz de hoje, nunca errada por excesso). `pele_j` interpolado por vértice, medido só com osso rígido.
6. **Sprite do motor sem número de luz pela placa:** a razão esquerda/direita do Sprite do MOTOR (`SpriteRenderer`+`sincronizar`+`LuzDoQuadro` num teste) não existe; a sonda da 27 prova que cor e normal chegam iguais e a foto mostra os dois acesos em espelho. Razão do Pixel: 3,036 (sem normal 2,119). Na 27 os 6 quadros da `heroi_p14` têm os mesmos píxeis (só fita vê a troca).
7. **Folhas extra do Animated Sprite:** nenhum gesto acrescenta um `.ase`/folha como folha EXTRA (`liga_celulas_da_folha` espera o produtor); `importa_folha` com atlas sem `duration` dá N sprites; a deteção de Split lê os NOMES (`--filename-format` sem `{frame}` com uma tag só passa como folha normal).
8. **Provas só «mesma chamada»:** um passo de undo do Open (nascimento + filha) e o ramo do `input_drop` da shell (gate da crate, não do clique); gate de ponteiro da queda de FICHEIRO do sistema nas ranhuras Image/Normal Map (hoje vai ao canvas como sprite nova).
9. **Godot:** pivô que muda por quadro só vai no JSON (o `AnimatedSprite2D` pede `frame_changed` do lado do jogo); Split de camadas/tags com Godot é recusado, não exportado; **smoke do dono por fazer do Export** (Split slices + Godot SpriteFrames, abrir `heroi_body.tres` no Godot) e do **largar `heroi.json`** (+`.png`/`_n.png`) na janela ⇒ um sprite a andar, com forma e luz.
10. **Costuras:** Pivot do cartão só na saída em cartões (`PH2D_CARDS=0`); oclusão do Pixel compõe a cor inteira por quadro (não medido em cena cheia); receptor DENTRO do sprite que bloqueia; o Pixel no palco sem faixas desenha acima de todo o documento e o quadro inteiro da shell no palco nunca foi medido em janela real; aviso do Color Mode recusado e pintor do botão desligado sem gate de pixel.
11. **Do dono, por decidir:** a cena 26 enquadra o herói a ~26 % da janela (o anel do objecto Luz manda no *Frame All*); a normal importada de fora aparece no Assets como `<cor>_n`.

## Abrir a P13 por aqui

**P13 — O que só a PH2D tem** ([02_plano_modulo.md](../02_plano_modulo.md) §5 P13): ainda SEM plano. A próxima janela abre `docs/PixelArt/P13_plano.md` com: passo 0 de medição (o que a casa já exprime, CLAUDE §5.0), estado da arte, oráculos com licença triada por ARTEFACTO (§0.9), recusas medidas, e mostra as decisões ao dono no formato da P11/P14 antes de código. Memória do dono: `project-memory/project_pixel_no_aseprite_license_trial_plus_free_oracles_2026_11_09.md`.

## Regras que continuam

UI só em inglês, cartões compactos; zero hex e zero literal de UI; teste por nome, suíte só na integração; código novo vai para crate (a shell só encolhe); integrar e push só por ordem do dono; handoff e prompt só depois do smoke dele.
