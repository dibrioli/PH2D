# Handoff — `line/3DModeling`: o CÉU DE VERDADE (HDRI) no render por malha (2026-10-03)

> **Wave, não integração.** Continua [O_BRILHO_E_O_ESTILO](HANDOFF_line_3DModeling_O_BRILHO_E_O_ESTILO_2026-10-02.md)
> — item 2 da ordem aprovada pelo dono ([O_RENDER_POR_MALHA §8](HANDOFF_line_3DModeling_O_RENDER_POR_MALHA_2026-10-02.md#8--a-próxima-onda-a-ordem-aprovada-pelo-dono-0210--qualidade-unrealfortnite-em-mobile)).
> Base da linha `1ad60a1ce` (main não andou). Commits da wave: `de88d5d42` lei CPU (`ph2d-sky`) ·
> `f80205667` desenhista · `f5123169e` modelador (painel, cena 38) · `b1c61b16b` desfoque + sonda ·
> `3320d5d67` painel `MAX_CHOICES 4 → 9`. **Smoke do dono: OK (03/10).**

## §1 — O que existe

**`crates/ph2d-sky` (NOVA, a lei na CPU + o gémeo WGSL, sem wgpu)**

- `Panorama` (equiretangular, linha 0 = polo `+y`; `de_exr`, `media`, `radiancia`). Orientação **= a
  do Blender** com `y` para cima: `u = 0,5 + atan2(z, x)/2π`, `v = acos(y)/π`.
- `Ceu::novo(&Panorama)` → atlas **octaédrico `f16`** (`Rgba16Float`, lido por `textureLoad`, nunca
  amostrador) em prateleiras `ATLAS_W × ATLAS_H` = `1028 × 838` (6,9 MB). **17 níveis** em
  `√α = k/16`, lados `512, 512, 256, 128, 128, 64×12`, margem de 1 texel pela dobra octaédrica.
  - nível 0: o panorama bilinear; `α < 0,09`: Hammersley **filtrada** (Colbert & Křivánek, viés de
    nível `0`), `amostras(α)` = 256/512/1024; `α ≥ 0,09`: **convolução EXACTA** sobre o nível da
    pirâmide com altura `≥ max(20/α, 64)`.
  - ⭐ **A irradiância `E(n)/π` É o nível `α = 1`** (com `α = 1` a NDF do GGX é `1/π` e o núcleo
    `(R·l)·D(h)` vira o cosseno). Não há mapa de irradiância.
- `Ceu::radiance(dir, α)` / `irradiance(n)` / `media()`; `Orientado { ceu, giro, forca }` implementa
  `ph2d_material::Environment`. `gira(d, giro)` = rotação em `y`.
- `wgsl::fonte()` — `sky_gira`, `sky_oct`, `sky_bilinear`, `sky_radiance`, `sky_irradiance`, com as
  constantes do atlas geradas da CPU. O chamador escreve `fn sky_atlas(x: u32, y: u32) -> vec3<f32>`.
- `Embarcado` — os 8 céus do Blender (Poly Haven, **CC0**, `ceus/LICENSE-CC0.txt`, 1,8 MB,
  `include_bytes!`); `TODOS` na ordem do painel, `chave()` = nome do ficheiro.
- Oráculo versionado: [`ferramentas/oraculo_ceu_blender.py`](../ferramentas/oraculo_ceu_blender.py)
  → `crates/ph2d-sky/fixtures/oraculo_blender.csv` (Cycles, esfera difusa e espelho, 3 céus).

**`ph2d-mesh-forward`**

- `Cena.foto: Option<Foto { giro, forca, caixa, fundo: Option<α> }>`: substitui a parte **SEM
  caixa** do céu de quem chama (`ceu_rad_sem` / `ceu_irr_sem` no `forward.wgsl`); a **DA caixa**
  (luz-chave com sombra) fica, pesada por `Foto::caixa`. O chão (só recebe) usa o mesmo despacho.
- `Forward::sobe_ceu(&Ceu)` / `tem_ceu()` — textura criada na 1.ª vez, trocar só escreve. Ligação
  `@binding(8)`; sem céu, um texel vazio e a `Foto` é ignorada.
- **Fundo**: passe de ecrã cheio (`vs_fundo`/`fs_fundo`) com `inv_view_proj` no uniforme do quadro;
  fora do brilho (alvo cena-linear a zero — a lei da CPU: só a peça brilha). Pipelines **7 → 12**
  com brilho (eram 6/10).
- `gpu_texturas.rs` (irmão do `gpu.rs` pelo tecto de LOC): `sobe_ceu`, `vazia`, a porta de paridade
  do brilho, `textura_de_floats`.

**`ph2d-app-field3d`**

- `ceu_foto.rs`: `Ceu { qual (0 = estúdio de sempre, 1..8 = Embarcado::TODOS[qual−1]), giro°,
  forca (stops), caixa, fundo, desfoque (√α) }`, `pack/unpack/sanitized`; **é VISTA** (`View::ceu`,
  `Smoke::ceu`, `Smoke::set_ceu` mora AQUI) — não entra no documento, `PROJECT_SCHEMA` intacto.
  - **Força 0 = a luz média do estúdio**: `normalizacao = AMBIENT·luma(ENV_BASE)·(1−share) / luma(média)`.
  - `pronto(e)` monta o atlas **noutra thread** (cache por céu); o quadro devolve `Feito::Espera`
    até lá, como a curvatura.
  - Fábrica: `fundo = true`, `desfoque = 0,15` (fotografado — §2).
  - `PH2D_FIELD_SKY=<chave>` abre com aquele céu (foto); a sonda lê `PH2D_SONDA_CEU=<chave>`.
- `ceu_painel.rs`: secção **Sky** — 6 linhas, `Param::Sky(slot)`; **só no Render por MALHA**
  (`render && malha` no `scene_panel.rs`; o traçado desenha o estúdio de sempre). Razões de linha
  apagada: `field.inert.sky_is_studio`, `field.inert.sky_background_is_off`.
- `malha_render_quadro.rs`: `Assinatura.ceu`, `Desenhista.ceu_subido`, `Cena.foto`.
- **Cena 38 «O CÉU»** (`smoke_scenes_malha::cena_38`, `materiais_38`): cromo, ouro, azul brilhante,
  vermelho fosco, nó branco. `CENAS = 38`.

**`ph2d-panel-model3d`**: `MAX_CHOICES 4 → 9` (o céu tem 9) + `debug_assert!` no pintor.

### Ids / variantes / consts novos (colisão na integração!)

| o quê | valor |
|---|---|
| `ph2d_field::Param::Sky(u8)` | variante nova no fim do enum (`dims.rs`), braço `BadRoot` no `edit_params_write.rs` |
| `ph2d_panel_model3d::populate::MAX_CHOICES` | `4 → 9` (registo `912 → 1497` entradas) |
| `smoke::scenes::CENAS` | `37 → 38` (o integrador **reconta** se outra linha também criou a 38) |
| i18n | `crates/ph2d-i18n/src/model3d_sky.rs` (novo), registado no `lib.rs` depois do `model3d_bloom` |
| `Cargo.toml` raiz | `[profile.dev.package.ph2d-sky]` e `[profile.dev.package.exr]` `opt-level = 2` |
| crate nova | `ph2d-sky` (deps: `ph2d-material`, `ph2d-imageio`, `ph2d-imageio-exr`, `rayon`, `half`) |

## §2 — Medições (release; máquina partilhada, load indicado)

**Pré-filtro × a convolução EXACTA sobre o panorama cru** (não a quadratura por amostras — ver §3):
`α ≥ 0,09` mediana `≤ 0,74 %`, máximo `≤ 5,5 %`; `α = 0,03` mediana `≤ 1,3 %`, máximo `≤ 30 %`
(borda de lâmpada do interior). Irradiância × soma crua: mediana `0,3 %`, máx `7 %` (o nível `α = 1`).
Espelho × panorama: mediana `≤ 0,5 %`; a folhagem da floresta `7,9 %` (octaédrico 512 × 1K).

**Oráculo Cycles**: difusa mediana `0,4–0,7 %` (o ruído de 2048 amostras); espelho `0,4–0,6 %`,
floresta `8,6 %`. Controlos (céu espelhado, de pernas para o ar, girado 90°): `≥ 49 %`.

**Paridade placa × CPU** (`s.indirect` com o `Orientado`, pelo olhar): 3 materiais coloridos × 324
px: **p99 1 B, máx 1 B, centro 0 B**. Fundo em perspectiva: **0 B**.

**Atlas**: ~0,7 s por céu em release (load 14); dev com `opt-2`: `11,0 s → 2,9 s` (3 céus).

**Quadro** 1920×1080 a girar, intercalado 2× a load ~5,5: cena 38 `1,76–1,84 → 1,88–1,97 ms`,
cena 28 `1,43–1,45 → 1,56–1,58 ms` ⇒ **+0,12 ms**. Entrar no Render: inalterado.

**Desfoque do fundo** (fotos da sonda, cena 38, pôr do sol): `0` = o chão da foto ampliado ~6× com
degraus nas bordas · `0,3` apaga a foto · **`0,15`** lê-se.

## §3 — ⛔ Recusas MEDIDAS (não reconstrua)

| Recusado | Por quê |
|---|---|
| Quadratura por AMOSTRAS como régua | no interior (lâmpadas pequenas) 16 k e 64 k amostras discordavam **8 %** na mediana — a régua mentia; a verdade é a soma exacta por texel (`tests::verdade`) |
| Viés `+1` no nível da amostragem filtrada | dobrava o erro sobre o equiretangular |
| 9 níveis | entre o espelho e o 1.º nível o erro chegava a 280 %; 17 níveis |
| Espelho a 256 | a floresta perdia 17 % contra o Cycles; 512 |
| Mapa de irradiância próprio (32²) | redundante (é o nível `α = 1`) e menos fiel (máx 7,2 % × 2,6 %); a mutação «irradiância := radiância a α 1» sobrevivia ao byte |
| SH de 2.ª ordem para a irradiância | trunca o sol; a convolução exacta custa nada |
| Amostrador da placa / `Rgba32Float` filtrado | WebGL2 não filtra `f32`; a CPU lê os MESMOS meios-floats |
| 64 amostras nos níveis finos | máximo 5 % → 29 % na borda de lâmpada; 256/512/1024 |
| Fundo nítido de fábrica | o HDRI `1K` ampliado a 1080p mostra degraus; `0,15` |
| SKY no Render traçado | o traçado sai quando o dono aprovar; ali seriam controlos mortos |

## §4 — Gates (novos)

| Crate | Gates |
|---|---|
| `ph2d-sky` | `um_ceu_chapado_devolve_l` (forno, tolerância `f16`) · `o_pre_filtro_e_a_convolucao` · `o_espelho_e_o_panorama` · `a_irradiancia_e_a_soma_crua` · `a_orientacao_e_a_do_blender` (+3 controlos) · `os_embarcados_decodificam` · instrumento `instrumento_decomposicao` (`ignored`) |
| `ph2d-mesh-forward` | `o_ceu_foto_e_a_lei_da_casa` · `o_fundo_e_o_ceu` · `nada_compila_ao_editar` com troca de céu (GPU, `--release -- --ignored`) |
| `ph2d-app-field3d` | `ceu_foto::tests` (força 0 = estúdio: `0,00 %`) · `ceu_painel::tests` (5) · costura `in_the_mesh_render_the_panel_offers_the_sky_and_choosing_one_changes_the_frame` |
| `ph2d-panel-model3d` | `a_click_on_the_ninth_choice_reaches_the_intent` (vermelho com `MAX_CHOICES = 4`) |

**Mutações**: 12 vermelhas (giro, força, nível único, offset do atlas, orientação, peso `N·L`,
convolução desligada, céu nunca ligado, fundo sem giro, fundo sem α, irradiância lendo o espelho
→ 86 B) + a equivalente explicada; costura: sem foto no quadro → 0 px de fundo; sem fileiras → 0/6.

**Suítes**: `field3d` lib 508 · `mesh-forward` 13/13 na placa · `panel-model3d` 47 + 19 ·
`ph2d-field`, `field-ecs`, `i18n`, `sky` verdes · clippy `-D warnings` limpo nos 7 crates ·
`editor-core` it: o tecto de LOC voltou a verde depois do corte (`gpu.rs` 849 → 659, `smoke_state.rs` 699).

## §5 — ABERTO

Herda o §5 **b..h** do [handoff O_RENDER_POR_MALHA](HANDOFF_line_3DModeling_O_RENDER_POR_MALHA_2026-10-02.md#5--aberto)
e o §5 do [O_BRILHO_E_O_ESTILO](HANDOFF_line_3DModeling_O_BRILHO_E_O_ESTILO_2026-10-02.md#5--aberto). Acrescenta:

- **O sol da foto ainda não faz sombra**: ele fica DENTRO da parte sem caixa (tapado só pela oclusão
  assada); a sombra continua a vir da caixa do estúdio (`Key Light`). É o item 3.
- Interpolação perto do espelho (`α ≈ 0,001`, rugosidade 0,03): mediana 11 % na floresta — limite
  de 17 níveis uniformes em `√α`; não medido se o artista vê.
- Importar um `.hdr`/`.exr` do disco (só os 8 embarcados). A `ph2d-imageio-hdr-radiance` existe.
- Céu fotográfico no Render traçado e no Matcap: não (por decisão, §3).
- GLES real / celular real: sem adaptador aqui (o `cabe_no_gles` não corre).

## §6 — Smoke (aprovado pelo dono, 03/10)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=38 cargo run -p ph2d-host-desktop --profile smoke`
2. **Model** → **Shading** → **Render**; secção **Sky** → linha **Sky** → **Sunset**.
3. Deve acontecer: o pôr do sol atrás das peças (desfocado), o cromo espelha-o, o ouro tinge; **Rotation**,
   **Strength**, **Key Light**, **Background**, **Background Blur**; **Studio** volta ao de sempre.
4. Deu errado se: um céu não muda nada, um dos 9 botões não responde, imagem branca/preta, engasgo.

Nomes da tela = `crates/ph2d-i18n/src/model3d_sky.rs` (nunca os do código).

## §7 — Reports do dono

- ✅ Smoke OK (03/10).

## §8 — ⭐ A PRÓXIMA ONDA: item 3 — **Sol direccional com sombras em cascata**

O que este handoff deixa pronto para ele, e o que ele tem de decidir com medição:

- **De onde vem o sol**: num céu fotográfico, do próprio HDRI — achar o disco (o pico de luminância
  e o ângulo sólido dele no `Panorama`), **tirá-lo da parte sem caixa** (o atlas pré-filtrado sem o
  disco) e devolvê-lo como a parte DA caixa direccional (`ceu_*_da_caixa` passa a ser o sol). No
  estúdio, o sol é a caixa de hoje (de cima). ⚠️ Conservação: céu sem disco + sol = o céu inteiro
  (gate de forno como o `as_duas_partes_somam_o_ceu`).
- **A sombra hoje é VERTICAL** (`gpu_quadro::enquadra_sombra`: mapa ortográfico «a olhar a direito
  para baixo»), e o chão lê o MESMO mapa: a cobertura (`gpu_cobertura`, penumbra) e o horizonte
  (`forward.wgsl::ceu_do_chao`) assumem vista de cima. Um sol inclinado precisa de: `sombra_vp` na
  direcção do sol (CSM: 2–4 cascatas no frustum da câmara, PCF/PCSS por cascata) **e** um mapa de
  alturas vertical à parte para o céu do chão — medir se o horizonte vertical sobrevive.
- Inegociáveis: `Features::empty()` + limites WebGL2 (`2048`, texturas e não armazenamento);
  nada acumulado entre quadros; pipelines compilados UMA vez.
- Oráculo: o Cycles já corre aqui (o script desta onda é o molde) — sombra de um sol de ângulo
  conhecido sobre um plano, comparada por passo.

Depois: 4 texturas triplanares · 5 contacto entre peças + chão em paridade com o traçado.

## Estado ao fechar a janela

HEAD `3320d5d67` (+ o commit deste handoff), árvore limpa, **NÃO integrado, NÃO enviado**,
`incremental/` NÃO reclamado (a linha continua). Binário `smoke` compilado nesta worktree — 2.ª corrida:

```
▸ linha line_3dmodeling · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.23s
```
