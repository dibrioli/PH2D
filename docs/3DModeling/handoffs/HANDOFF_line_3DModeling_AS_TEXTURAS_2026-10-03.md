# Handoff — `line/3DModeling`: AS TEXTURAS (2026-10-03)

> **Wave, não integração.** Item 4 da ordem aprovada pelo dono (texturas triplanares + mapas de
> normal/rugosidade). Continua [O_SOL_E_A_SOMBRA](HANDOFF_line_3DModeling_O_SOL_E_A_SOMBRA_2026-10-03.md).
> Base da linha `1ad60a1ce` (main não andou). Commits da wave: `9e9e835f1` o instrumento do custo ·
> `ea5071a79` o oráculo · `c805f099a` a crate `ph2d-triplanar` · `da7db1c0d` o passo da normal ·
> `050a98a0b` a rugosidade do pixel (`ph2d-material`) · `50b87df92` o desenhista · `d36a58b4e` a
> paridade · `4b643d956` o documento (`FieldTexture`, schema 179) · `69f78d5b4` o modelador, o painel e
> a cena 39 · `a33208159` os gates do modelador · `bfe5b35b1` trocar não compila. **Smoke do dono: ⏳.**

## Decisão de PRODUTO (dono, 03/10)

*«De onde vêm as texturas?»* → **(c) as duas coisas**: um pacote livre embutido (CC0, como os céus)
**e** o artista importa as dele.

## §1 — O que existe

**`ph2d-triplanar` (NOVA: a lei na CPU + o gémeo WGSL + o pacote, sem wgpu)**

- `pesos(n, blend)` — a regra do «Blend» do Blender em três regimes (§2), nos eixos DELE.
  `vista(eixo, n_eixo)` → `Vista { t, b, a, k }` (as seis faces, cada uma lê a imagem de frente).
  `nivel(..)` = `log2` da maior pegada do pixel por vista (o `textureSampleLevel` explícito).
- `Mipmaps` (RGBA8, linha 0 = baixo, reamostra a `LADO = 1024`, mips com média em LINEAR para cor),
  `amostra(srgb, u, v, lod)` = o trilinear da placa com repetição. `Mapas { cor, nrh, tem_normal,
  tem_rugosidade }`: `nrh` = normal de tangente (OpenGL) em rgb + rugosidade no ALFA.
- `avalia(mapas, Triplanar { tamanho, aspecto, blend, relevo }, p, n, dx, dy)` → `Resultado { cor,
  rugosidade, normal }` (Whiteout por vista, Golus 2017). Relevo `k`: `(k·x, k·y, 1+(z−1)·k)`.
- `wgsl::fonte()` — `tri_avalia`, `TriParams`; quem chama escreve `tri_cor_ler`/`tri_nrh_ler`.
- `Embarcada` — 7 texturas Poly Haven CC0 a 1k, **sem modificação** (`texturas/LICENSE-CC0.txt`
  com o sha256 de cada; 10,6 MB): tijolo, madeira, pedra, chapa, ferrugem, couro, betão; `chave()`,
  `tamanho_real()` (m), `mapas()`. `junta_nrh`, `Imagem`.

**`ph2d-material`** — `por_pixel.rs`: `Surface::at_roughness(r)` (só o `main_alpha` depende dela) e o
gémeo `mx_at_roughness` + `mx_lobe_shrink` por TABELA de 65 em `√α` (`wgsl::por_pixel()`).

**`ph2d-mesh-forward`** — `gpu_triplanar.rs`: dois `texture_2d_array` (cor `Rgba8UnormSrgb`, `nrh`
`Rgba8Unorm`, mips, uma camada por textura) + amostrador trilinear com repetição; crescer copia as
camadas na placa. `Forward::sobe_textura(camada, &Mapas)`; `Cena::texturas: &[Option<TexturaMaterial
{ camada, triplanar, tem_normal, tem_rugosidade, mundo_para_folha }>]`. No `forward.wgsl`: as
derivadas no topo do `luz_de_cena` (fluxo uniforme), `tri_avalia` no espaço da FOLHA,
`mx_at_base_color(cor × textura)`, `mx_at_roughness`, a normal de volta ao mundo.

**`ph2d-field` / `ph2d-field-ecs`** — `Param::Texture(u8)`; `FieldTexture { source, tile, blend,
bump, color_file, normal_file, roughness_file }` (registado, `register_default`; `get`/`set` por
posição; a fonte «de ficheiro» só com a cor escolhida; os mapas só se APAGAM pelo número); a escrita
materializa como o material; o `copy_optional` leva-a.

**`ph2d-app-field3d`**

- `texturas.rs`: `das_folhas` (a ordem dos materiais; `mundo_para_folha` da `world_xform` da folha,
  refeito por quadro: mover não desliza nem extrai), `sync` (ao lado do `materials::sync`), `estado`
  (cache por fonte, decodificação noutra thread; o quadro espera, como o céu), `para_o_desenhista`,
  ficheiros por CAMINHO (PNG, JPG, WebP, TGA), o corredor do diálogo em três saltos:
  `pede_importar` (dreno) → `atende_pedido` (app, `rfd` pela porta `modal`) → `aplica_escolhidos`
  (ponte, no alcance do material, um passo de undo); `semeia`/`planta` (a semente da cena).
- `textura_painel.rs`: seis fileiras (Texture · Tile Size · Blend · Bumps · Normal Map · Roughness
  Map) **só no Render por malha**, logo depois do material da mesma folha; a «Roughness» do material
  apaga-se com `field.inert.roughness_from_texture` quando a textura a dá; «From File…» abre o
  diálogo e não escreve (`pede_ficheiro`).
- `scene_intents.rs`: o braço `Param::Texture(slot)` antes do genérico (espalha como o material).
- **Cena 39 «AS TEXTURAS»** (`smoke_scenes_textura.rs`): pedra, tijolo, madeira, chapa (metal),
  couro, materiais BRANCOS. `CENAS = 39`.

### Ids / variantes / consts novos (colisão na integração!)

| o quê | valor |
|---|---|
| `ph2d_field::Param::Texture(u8)` | variante nova no FIM do enum (`dims.rs`) |
| `ph2d_field::{TEXTURE_FIELDS, TEXTURE_SOURCES, TEXTURE_FROM_FILE}` | `6`, `9`, `8` |
| componente registado | `"ph2d::field::FieldTexture"` — o registo do campo `8 → 9` (gate `a_duplicate_carries_every_optional_component_of_a_node`) + catálogo `component.field_texture.name` |
| `PROJECT_SCHEMA` | `178 → 179` (escada + tripla `(179, 13, 22)`; o integrador **reconta** com `scripts/schema-recount.py`) |
| ligações do `g0` do desenhista | `12` (cor), `13` (`nrh`), `14` (amostrador) |
| tabela dos materiais | `MATERIAL_V4 + TEXTURA_V4` = `12 + 5` colunas |
| `smoke::scenes::CENAS` | `38 → 39` (o integrador **reconta** se outra linha criou a 39) |
| i18n | `crates/ph2d-i18n/src/model3d_texture.rs` (novo), registado depois do `model3d_sky` |
| crate nova | `ph2d-triplanar` (deps: `ph2d-imageio`, `-jpeg`; dev: `-png`) |
| deps novas do `ph2d-app-field3d` | `ph2d-triplanar`, `ph2d-imageio`, `-png`, `-jpeg`, `-webp`, `-tga` |
| fixturas | `crates/ph2d-triplanar/fixtures/{oraculo_triplanar.csv (7 344 linhas), teste_colorida.png, teste_normal.png}` |
| instrumentos | `docs/3DModeling/ferramentas/oraculo_triplanar_blender.py` · `ph2d-mesh-forward::tests_custo_textura::instrumento_custo_triplanar` |
| shell (`shells/desktop`) | `+11` linhas: o degrau do schema (`+10`) e `texturas::atende_pedido` (`+1`) |

## §2 — Medições

**Custo por pixel** (`instrumento_custo_triplanar`, RTX 5060 Ti, limites do WebGL2, 1080p,
`textureSampleLevel` trilinear com o nível das derivadas por eixo; base = o mesmo passe sem
leituras; load ~16–24, a placa sozinha):

| formato · lado | 1 vista × 2 mapas | 3 × 2 | 3 × 3 |
|---|---|---|---|
| RGBA8 sRGB · 512 / 1024 | +0,013 ms | +0,060 | +0,085 |
| RGBA8 sRGB · 2048 | +0,013 | +0,064 | +0,121 |
| Rgba16Float · 1024 | +0,014 | +0,065 | +0,112 |
| Rgba16Float · 2048 | +0,017 | +0,112 | +0,186 |

O ALU do nível por eixo é `+0,022` (3 eixos, 0 leituras). Celular de referência: Adreno 650
(Snapdragon 865) = **28,2 GTexel/s** contra **~370** aqui (144 TMUs × 2,57 GHz) ⇒ ×13: o pior caso
(3 vistas × 2 mapas) ≈ **0,8 ms** a 1080p, ~5 % de um quadro de 60 Hz. ⇒ **RGBA8, 1024², mipmaps,
2 mapas por material**, eixo de peso zero não lê. Memória: `2 × 5,6 MB` por textura subida.

**A lei do Blender** (`oraculo_triplanar_blender.py`, Blender 5.2.2 Cycles, esfera + caixa giradas,
coordenadas do OBJECTO), por passo:

- **Orientação** (eixos do Blender, `c` = coordenada mapeada): `+X: (c_y, c_z)` · `−X: (1−c_y, c_z)`
  · `+Y: (1−c_x, c_z)` · `−Y: (c_x, c_z)` · `+Z: (1−c_y, c_x)` · `−Z: (c_y, c_x)` — cada face lê a
  imagem de FRENTE (T × B = a normal de fora). O lado vem do sinal da normal.
- **Pesos**: `N = |n|/Σ|n|`, `L = (1+blend)/2`. Uma vista se `N_i > L·(N_i+N_j)` para os dois
  outros; senão (blend > 0) duas vistas se a terceira `< (1−L)·(soma das duas)`, com
  `w = clamp((N_a/(N_a+N_b) − (1−blend)/2)/blend, 0, 1)`; senão três: `w = ((2−L)·N + (L−1))/(2L−1)`.
  Blend 0 sem dominante: o X. Rust contra a rampa: **9,2e-7**.
- **Cor**: **7,1e-5** filtrando nos BYTES sRGB e decodificando depois (o Cycles); o produto (a placa,
  `Rgba8UnormSrgb`, e o Eevee) decodifica e filtra em linear — divergência declarada p50 `0,0024`,
  máx `0,265` (só em texel ampliado com borda dura).
- **Normal** (só a caixa: UV = a projecção do passo 1, nó Normal Map em tangente MikkTSpace):
  **4,4e-5** com o `z` do texel guardado.

**Encolhimento por tabela**: 65 entradas em `√α`, desvio `5,7e-5` (o piso é o salto da própria lei
no ramo `|α²−1| < 1e-3`). **`at_roughness` = `prepare` com aquela rugosidade**: `≤ 2e-7`.

**Paridade placa × CPU** (`a_textura_e_a_lei_da_casa`, quadrados planos sob o pôr do sol, 1 600 px
cada): minificação com 3 vistas · metal ampliado · verniz com 2 vistas · folha girada/escalada/
deslocada → **p50 0 B, p99 1 B, máx 1–4 B**.

**No app** (cena 39, foto `fotografa_cena.sh`): 59–60 fps, as cinco texturas nítidas, a rugosidade do
material apagada com a frase. ⚠️ Relógio do quadro NÃO medido com `load < 5` (re-medir).

## §3 — ⛔ Recusas MEDIDAS (não reconstrua)

| Recusado | Por quê |
|---|---|
| `Rgba16Float` para cor/normal | até 1,5× o custo e 2× a memória; 8 bits chega para cor e normal |
| Lado 2048 | +40 % no pior caso (cache) e 4× a memória; 1024 custa o mesmo que 512 |
| 3 mapas por material | +40 % sobre 2; normal + rugosidade cabem num RGBA8 |
| Reconstruir o `z` da normal de `xy` (rugosidade no azul) | `0,078` contra o Blender (filtrar `xy` encurta-os); o `z` guardado dá `4,4e-5` ao mesmo custo |
| Filtrar nos bytes sRGB (o Cycles) | errado fisicamente e não é o que a placa faz; divergência medida acima |
| `textureSampleGrad` / nível implícito | o nível implícito exige fluxo uniforme (os eixos de peso 0 não leem); o `Grad` é meia taxa no Mali — o nível explícito tem paridade com a CPU |
| Amostrador manual por `textureLoad` (como o céu) | 4 leituras × 2 níveis × 3 vistas × 2 mapas = 48 por pixel; a placa filtra RGBA8 no WebGL2 |
| Coordenadas no espaço da INSTÂNCIA (a malha extraída) | a textura deslizaria quando uma folha mexe dentro de um objecto fundido; a folha é o «Object» do Blender |
| Uma esfera facetada na paridade | o texel lido muda com a faceta (a posição é a da malha, não a analítica): quadrados planos dão posição e derivadas exactas |
| Imagens importadas DENTRO do projecto | o precedente da casa é o caminho (escultura importada); 1–3 MB por cópia por folha |

## §4 — Gates

| Crate | Gates |
|---|---|
| `ph2d-triplanar` | `os_pesos_e_as_vistas_sao_os_do_blender` · `a_cor_e_a_do_blender` (+ o controlo da divergência) · `o_mapa_de_normal_e_lido_como_o_blender` · `o_mapa_plano_devolve_a_normal_da_forma` · `o_mip_da_cor_faz_a_media_em_linear` · `o_pacote_embutido_decodifica` |
| `ph2d-material` | `a_rugosidade_do_pixel_e_o_prepare_com_ela` · `a_tabela_do_encolhimento_e_a_lei` |
| `ph2d-mesh-forward` | `a_textura_e_a_lei_da_casa` (GPU) · `trocar_de_textura_nao_compila_nem_acumula` (GPU) · os 11 de antes verdes · `o_shader_valida_sem_capacidades` |
| `ph2d-field-ecs` | `a_duplicate_carries_every_optional_component_of_a_node` (9 componentes, a textura viaja) |
| `ph2d-app-field3d` | `a_matriz_leva_o_mundo_a_folha` · `a_ordem_e_a_dos_materiais` · `os_ficheiros_viram_mapas_ou_dizem_porque` · `a_semente_planta_e_gasta` · `os_nomes_seguem_o_pacote` · `as_fileiras_seguem_o_material_e_apagam_a_rugosidade` · `a_textura_espalha_pela_seleccao` · `de_ficheiro_pede_o_dialogo_e_o_escolhido_chega_ao_mundo` |

**Mutações**: ⏳ (ver §4.1 quando corridas).

## §5 — ABERTO

- **Verniz a ESCURECER + textura**: o `mx_at_base_color` não re-deriva o `modulated_base_darkening`
  (fronteira declarada no `ph2d_material::wgsl`); com `coat_weight × coat_darkening > 0` a placa
  diverge da CPU numa peça texturizada. Cura: o `coat_darkening` na ranhura livre do `emissive.w`.
- **A normal na esfera** (as zonas de duas/três vistas) é a Whiteout publicada, gateada por
  propriedades (mapa plano devolve a forma; faces = o Blender); não há oráculo para a mistura.
- **Camadas na placa não se libertam** durante a sessão (o cache cresce); trocar o ficheiro no
  disco não recarrega até reabrir.
- **Ficheiro que sumiu**: a frase uma vez e a forma sem textura; o «religar» é escolher «From File…»
  outra vez (não há botão dedicado como na escultura).
- O Render TRAÇADO e o Matcap não desenham a textura (as fileiras só existem no Render por malha).
- Relógio do quadro com `load < 5`; mobile real; o binário cresce `10,6 MB` com o pacote.
- Herdados: §5 do [O_SOL_E_A_SOMBRA](HANDOFF_line_3DModeling_O_SOL_E_A_SOMBRA_2026-10-03.md#5--aberto).

## §6 — Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=39 cargo run -p ph2d-host-desktop --profile smoke`
2. **MODEL** → **Shading** → **Render**; clique numa forma; no painel da direita, secção **Texture**.
3. Deve acontecer: bola de pedra, caixa de tijolo, cilindro de madeira, anel de chapa, cápsula de
   couro; **Texture** troca (Bricks · Wood · Stone · Metal Plate · Rusty Metal · Leather · Concrete ·
   None); **Tile Size** aumenta/diminui o desenho; **Blend** amacia a costura na bola; **Bumps** 0 =
   liso, 2 = mais fundo; **From File…** abre a escolha de ficheiro e a forma mostra a imagem; mover a
   forma leva a textura junto; **Base Color** tinge; a **Roughness** do material diz «The texture
   gives the roughness».
4. Deu errado se: a textura escorrega ao mover a forma, a bola tem uma costura dura com Blend acima
   de 0, «From File…» não faz nada, ou engasga.

Nomes da tela = `crates/ph2d-i18n/src/model3d_texture.rs`. Foto do app:
`fotografa_cena.sh "PH2D_FIELD_SMOKE=39 PH2D_FIELD_SHADING=render" 1930 1040 <png> 18`.
Oráculo: `bash scripts/ph2d-run.sh blender -b -X --python docs/3DModeling/ferramentas/oraculo_triplanar_blender.py -- crates/ph2d-triplanar/fixtures/oraculo_triplanar.csv crates/ph2d-triplanar/fixtures/teste_colorida.png crates/ph2d-triplanar/fixtures/teste_normal.png`.

## §7 — Reports do dono

- ⏳ smoke.

## §8 — A PRÓXIMA ONDA: item 5 — **contacto entre peças + chão em paridade com o Render traçado**

O §5-d do [O_RENDER_POR_MALHA](HANDOFF_line_3DModeling_O_RENDER_POR_MALHA_2026-10-02.md): o chão
escurece pouco (alfa `19` medido contra `~60` da conta física) — gate de paridade contra
`ph2d_field_render::ground` em 4–5 pontos à volta de uma esfera e de uma caixa pousadas, e oclusão
de contacto barata entre peças.
