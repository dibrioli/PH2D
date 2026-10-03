# Handoff — `line/3DModeling`: AS TEXTURAS (2026-10-03) — ⏳ EM CURSO

> **Wave, não integração.** Item 4 da ordem aprovada pelo dono (texturas triplanares + mapas de
> normal/rugosidade). Continua [O_SOL_E_A_SOMBRA](HANDOFF_line_3DModeling_O_SOL_E_A_SOMBRA_2026-10-03.md).
> Base `1ad60a1ce` (main não andou). ⚠️ Rascunho vivo: o plano e as medições já feitas.

## Decisão de PRODUTO (dono, 03/10)

*«De onde vêm as texturas?»* → **(c) as duas coisas**: um pacote livre embutido (CC0, como os céus)
**e** o artista importa as dele.

## §2 — Medições (até aqui)

**Custo por pixel** (`ph2d-mesh-forward::tests_custo_textura::instrumento_custo_triplanar`, RTX 5060
Ti, limites do WebGL2, 1080p, `textureSampleLevel` trilinear com o nível das derivadas por eixo;
base = o mesmo passe sem leituras; load ~16–24, a placa sozinha):

| formato · lado | 1 vista × 2 mapas | 3 × 2 | 3 × 3 |
|---|---|---|---|
| RGBA8 sRGB · 512 / 1024 | +0,013 ms | +0,060 | +0,085 |
| RGBA8 sRGB · 2048 | +0,013 | +0,064 | +0,121 |
| Rgba16Float · 1024 | +0,014 | +0,065 | +0,112 |
| Rgba16Float · 2048 | +0,017 | +0,112 | +0,186 |

O ALU do nível por eixo é `+0,022` (3 eixos, 0 leituras). Celular de referência: Adreno 650
(Snapdragon 865) = **28,2 GTexel/s** contra **~370** aqui (144 TMUs × 2,57 GHz) ⇒ ×13: o pior caso
(3 vistas × 2 mapas) ≈ **0,8 ms** a 1080p, ~5 % de um quadro de 60 Hz. ⇒ **RGBA8, 1024², mipmaps,
2 mapas por material** (cor sRGB · normal xy + rugosidade), eixo de peso zero não lê.

**A lei do Blender** (`docs/3DModeling/ferramentas/oraculo_triplanar_blender.py` →
`crates/ph2d-triplanar/fixtures/oraculo_triplanar.csv`, Blender 5.2.2 Cycles, esfera + caixa
giradas, coordenadas do OBJECTO):

- **Orientação** (eixos do Blender, `c` = coordenada já mapeada): `+X: (c_y, c_z)` · `−X: (1−c_y, c_z)`
  · `+Y: (1−c_x, c_z)` · `−Y: (c_x, c_z)` · `+Z: (1−c_y, c_x)` · `−Z: (c_y, c_x)` — cada face lê a
  imagem de FRENTE (T × B = a normal de fora nas seis). O lado vem do sinal da normal.
- **Pesos**: `N = |n|/Σ|n|`, `L = (1+blend)/2`. Uma vista se `N_i > L·(N_i+N_j)` para os dois outros;
  senão (blend > 0) duas vistas se a terceira `< (1−L)·(soma das duas)`, com
  `w = clamp((N_a/(N_a+N_b) − (1−blend)/2)/blend, 0, 1)`; senão três: `w = ((2−L)·N + (L−1))/(2L−1)`.
  Blend 0 sem dominante: o X. Erro contra a rampa: **≤ 7e-7** (rampa) em todos os blends.
- **Cor**: bate a **6e-5** se se filtra nos BYTES sRGB e se decodifica depois (o Cycles); a placa
  (`Rgba8UnormSrgb`) decodifica primeiro e filtra em linear (o Eevee, o correcto). A diferença entre
  as duas: p50 `0,0024`, máx `0,26` (só em texel ampliado com borda dura). ⇒ o produto filtra em
  LINEAR; o gate prova a geometria com o passo «filtrar como o Cycles» e mede a divergência.

## Desenho (decidido — técnico)

- Crate nova **`ph2d-triplanar`**: a lei (CPU) + o gémeo WGSL + o pacote embutido.
- Normal: Whiteout por eixo (Ben Golus, *Normal Mapping for a Triplanar Shader*, 2017) na base
  (T, B, A) de cada vista; mapa no padrão OpenGL (verde = +v).
- Rugosidade por pixel: porta nova `Surface::at_roughness` (CPU) + `mx_at_roughness` (WGSL); o
  encolhimento do lóbulo por pixel vem de uma TABELA lida igual nos dois lados.
- Coordenadas da textura = espaço da FOLHA (`world_xform` da folha⁻¹ × posição de mundo), por
  material, refeito a cada quadro: mover não desliza, não extrai.
- Documento: componente novo `FieldTexture` (registado) por folha + `Param::Texture(u8)`; fileiras
  só no Render por MALHA (como o céu); ficheiros importados por CAMINHO (precedente da escultura
  importada: chave + religar). `PROJECT_SCHEMA` +1.

## ⛔ Recusas MEDIDAS (até aqui)

| Recusado | Por quê |
|---|---|
| `Rgba16Float` para cor/normal | até 1,5× o custo e 2× a memória; 8 bits chega para cor e normal |
| Lado 2048 | +40 % no pior caso (cache) e 4× a memória; 1024 custa o mesmo que 512 |
| 3 mapas por material | +40 % sobre 2 mapas; normal xy + rugosidade cabem num RGBA8 |
| Filtrar nos bytes sRGB (o Cycles) | errado fisicamente e não é o que a placa faz; divergência medida acima |
