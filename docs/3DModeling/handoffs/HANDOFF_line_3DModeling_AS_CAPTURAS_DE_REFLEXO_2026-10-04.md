# Handoff — `line/3DModeling`: AS CAPTURAS DE REFLEXO (2026-10-04)

> **Wave, não integração.** Item 2 da ordem do dono de 03/10 (§8 do
> [O_CHAO_QUE_TAPA](HANDOFF_line_3DModeling_O_CHAO_QUE_TAPA_2026-10-03.md)): *«não temos reflexo dos objetos
> ao lado»*, no padrão Unreal/Fortnite. Base da linha `1ad60a1ce` (main não andou). Commits: `37fe1a374`
> o oráculo + a régua red-first · `47671f31b` a lei, os gates, o instrumento, a cena 42 · `2518c0b82`
> octaedro 256, passes juntos, grupo 0 dinâmico · `bc76c2d6c` o gate de girar PERTO · `1d9b6b161` o texto
> da cena 42 · o commit de docs desta wave. **Smoke do dono: PENDENTE** (o do item 1, O_RENDER_ANTIGO_SAI,
> também não voltou com report).

## ⛔ As premissas que a medição derrubou

| Premissa | O que se mediu |
|---|---|
| «6 faces = uma textura de 6 camadas» | o GLES do `wgpu-hal` (`get_info_from_desc`) faz CUBO de toda textura quadrada com camadas múltiplas de 6 ⇒ atlas `3 × 2` numa textura 2D, e a matriz das capturas com `2 · cap` camadas (`cap` potência de 2) |
| «o mapa de sombra do quadro serve as capturas» | ele enquadra-se pela VISTA (`vista_da_camera`): as capturas mudariam com a câmara. Enquadramento da cena inteira só para elas (`enquadra(…, pela_camera = false)`); a mutação que o trocava SOBREVIVIA com a cena toda à vista ⇒ gate `girar_perto_nao_refaz_as_capturas` |
| «paralaxe de 2 passos no mesmo nível» | onde a direcção crua não vê vizinha nenhuma o passo desistia: `19,1°` de erro. Do GROSSO ao fino resolve (tabela em `gpu_sondas::PARALAXE`) |
| «octaedro de 128 (o do Unreal móvel) basta» | o espelho PLANO perto da vizinha (a caixa) pede 256: miolo `0,060 → 0,036`, ao mesmo custo no desktop |
| «um passe por etapa e cópias para a cadeia (o padrão da cobertura)» | `4,3 ms` de pós-processamento a 16 peças (25 operações por captura) ⇒ cadeia com uma textura por nível, nível = UM passe (cadeia + pré-filtro): 7 operações, sem cópias |
| «o gate do contacto mede o contacto» | media o cinzento de omissão, COM realce — e o realce agora vê as vizinhas pela captura. O oráculo dele é difuso: o gate passa a desenhar a difusa (`0,0088 → 0,0085`) |

## §1 — O que existe

**`ph2d-mesh-forward`** (crate da linha):
- `gpu_sondas.rs` — recursos e pipelines: `LADO 256` (octaedro com 1 texel de borda que guarda a DOBRA),
  `FACE 128`, `NIVEIS 6` (`α = (k/5)²`, a pergunta do pré-filtro do céu `ph2d_sky::prefiltro`), `TAPS 64`
  (Hammersley filtrado, sem o viés `+1`), `PARALAXE [5, 2, 1, 0]`, `MAX 128` (256 camadas do WebGL2 ÷ 2).
  `face_vp`, `na_face` (tronco de 90° por face: corta as vizinhas fora), `Arranjo` (vistas de escrita
  pré-feitas por captura e nível).
- `gpu_sondas_passes.rs` — por quadro: `atribui_sondas` (cada instância visível, com ≥ 2 peças e
  `Rgba16Float`, ganha a camada `2i` e o centro da caixa), `planeia_sondas` (uniformes das 6 faces com
  `em_sonda = true`; a CHAVE = esses uniformes + bytes dos objetos + dos materiais + `geracao`), e
  `grava_sondas` (sombras da cena inteira → por captura: faces, octaedro, 5 níveis).
- `sondas.wgsl` (passes de ecrã: `fs_octa`, `fs_nivel`) · `sonda_le.wgsl` (ranhura `{SONDAS}`: `fs_sonda`
  = `luz_de_cena` + distância; a leitura com paralaxe, o lobo escalado pela razão das distâncias —
  Lagarde & Zanuttini 2012 —, memória por pixel da direcção e da leitura).
- `forward.wgsl`: `Objeto.sonda` (`80 → 96 B`), `@binding(20) sondas: texture_2d_array`; com captura o
  reflexo é `(cap.rgb + (1 − cap.a)·céu·(1 − chão))·ao + caixa` (o contacto mole sai do realce; a máscara
  `sombra_propria` só vale SEM captura). Dentro de uma captura (`olhar.w = 1`) ninguém lê capturas.
  O PCSS mudou-se para `pcss.wgsl` (ranhura `{PCSS}`, o tecto de 700 linhas).
- `geracao` sobe em `sobe`, `sobe_curvatura`, `sobe_contacto`, `esquece`, `sobe_ceu`, `sobe_textura`.
- **`ph2d-sky`**: `wgsl::OCT` (`sky_oct` + `sky_de_oct`, a porta do octaedro — o céu e as capturas).

**`ph2d-app-field3d`**: cena **42 «OS REFLEXOS»** (`smoke_scenes_reflexos.rs`): cromo no meio, bola
vermelha, caixa verde, bola amarela, caixa azul (foscas), alumínio escovado (`0,35`). Nada mais muda no
app: o desenhista faz as capturas sozinho.

## §2 — Medições

| o quê | resultado |
|---|---|
| **Oráculo** (Cycles, `oraculo_reflexo_vizinhas_blender.py` → `fixtures/oraculo_reflexo_vizinhas.csv`, 13 930 px; controlos: metal sozinho `1,0000`, direcção do céu `0,9999`, topo das foscas `0,2987` para `0,3`) | razão `viz/solo` no espelho, miolo do reflexo de uma vizinha: esfera nítida `0,6188 → 0,0051`, áspera `0,1606 → 0,0333`, caixa `0,6842 → 0,0364`; todos os px `0,0029 / 0,0123 / 0,0204`. Faixa do contorno `0,056 / 0,024 / 0,27` (o Cycles do oráculo é PONTUAL, o quadro filtrado — imprime-se) |
| Captura lida de volta contra a geometria (`a_captura_ve_as_vizinhas`, 64 000 texels) | cobertura errada `59` (0,09 %), distância `|Δ|/d` média `0,0005` |
| Paralaxe (`a_paralaxe_acerta_onde_o_raio_bate`) | esfera `0,01° / 0,10°`, caixa `0,09° / 2,45°` (89 raios rasantes leem «nada»; 39 batem num ponto que o centro não vê) |
| Resolução (tabela na doc de `gpu_sondas::LADO`) | 256/128 escolhido; faces 256 não ajudam a caixa e custam `+0,9 ms` |
| **Custo**, 1080p, RTX 5060 Ti, release, load `< 5` (`instrumento_custo_sondas`, A/B alternado) | girar (guardadas): `+0,02` (3 peças) · `+0,04` (8) · `+0,09 ms` (16). Arrastar (TODAS refeitas por quadro): `+0,61` · `+1,62` · `+3,84 ms` (quadro `6,5 ms` com 16 metal). Parte a 16: faces `~1 ms`, pós `~2,5 ms` (dos quais `~1 ms` são as 64 amostras) |
| App, cena 42 (sonda do render por malha, 6 objetos, 90 888 triângulos, a girar) | mediana `1,81 ms`, p90 `2,77 ms`; entrar `96 ms` |
| Celular | NÃO medido (regra ×13: arrastar 3 peças `~8 ms`) |

## §3 — ⛔ Recusas MEDIDAS (não reconstrua)

| Recusado | Por quê |
|---|---|
| Paralaxe de passos no mesmo nível (`[1, 1]`) | `19,1°` onde a direcção crua não vê nada |
| `[4, 2, 1, 0]` a 256 | o grosso já não é grosso: volta o `19,1°` (o gate apanhou) |
| `[5, 3, 1, 0]` | caixa `4,66°` no pior raio |
| Octaedro 128 | caixa `0,060` |
| Faces 256 | nada na caixa, `+0,9 ms` a arrastar 16 |
| Passe por etapa + cópias | `4,3 ms` de pós a 16 peças |
| Seis grupos de ligação por captura | `~0,3 ms` a 16 ⇒ uniforme do quadro DINÂMICO no grupo 0 |
| Textura de 6 camadas para as faces | o GLES faz cubo à força |

## §4 — Gates e prova de mutação

Novos (todos com placa): `tests_reflexo::o_espelho_mostra_as_vizinhas_como_no_cycles` (com o CONTROLO
sem capturas: `0,628 / 0,186 / 0,684`), `tests_sonda_cpu::{a_captura_ve_as_vizinhas,
a_paralaxe_acerta_onde_o_raio_bate}` (o gémeo da leitura na CPU sobre a captura lida de volta),
`tests_sondas::{as_capturas_so_se_refazem_quando_a_cena_muda, girar_perto_nao_refaz_as_capturas,
o_quadro_com_capturas_guardadas_e_o_de_um_desenhista_novo (ao byte), uma_peca_sozinha_nao_tem_capturas,
as_capturas_cabem_no_gles}`; instrumento `instrumento_custo_sondas`. Mudados: `o_reflexo_do_chao_e_o_do_cycles`
(fora da zona a régua AFIRMA contra o Cycles, `0,0039` < `0,01`, em vez de proibir a sombra de uma
vizinha), `o_contacto_na_placa_e_o_do_cycles` (a difusa do oráculo).

**Mutações (agente `mutacao`, controlos verdes): 11 de 11 VERMELHAS** — a chave congelada, sem materiais,
sem `geracao`; `sonda_camada = −1`; sem paralaxe (`0,215`); sem a razão no lobo (áspera `0,082`); a peça
na própria captura; o tronco só com caixas inteiras (`0,447`); a máscara da zona com captura
(`25` px escurecem contra `1 042`); a exposição no uniforme; e o enquadramento pela câmara — que
**sobrevivia** até ao gate de girar perto.

**Suítes:** `ph2d-mesh-forward` com placa `30/30` + o gate novo; `field3d` família Render `29/30` em
série (o `1` é a sonda `sonda_oclusao_da_malha`, que pede `PH2D_SONDA_DIR`); censo e roteador de cenas
`166` ✓. clippy `--all-targets` da crate limpo. ⚠️ `as_capturas_cabem_no_gles` NÃO correu: esta máquina
não tem adaptador GL (o `cabe_no_gles` antigo também sai cedo).

## §5 — ABERTO

- **O GLES não foi exercitado** (sem adaptador GL aqui): renderizar numa camada+nível de uma matriz, o
  grupo 0 dinâmico, as 4 saídas. O gate existe e corre onde houver GL.
- **Arrastar é N²**: cada peça movida refaz TODAS as capturas. 16 metal `+3,8 ms` no desktop; malhas de
  muitos triângulos multiplicam as faces (6 × vizinhas por captura, com o corte do tronco) — o LOD
  (item sem ordem do dono) é a alavanca. Amostras `64 → 16` tiram `~1 ms` (qualidade não medida).
- Espelho plano perto: a faixa do contorno da caixa lê `0,27` (resolução; o Cycles pontual); `39` raios
  batem num ponto que o centro da captura não vê (limite de uma captura por ponto).
- Sem `Rgba16Float` renderizável ou acima de `MAX = 128` peças: sem capturas, a lei antiga (máscara da zona).
- Memória `~1,4 MB` por captura (128 capturas `~180 MB`).
- O pré-filtro lê a cadeia até ao nível `k − 1` (o `k` nasce no mesmo passe): a esfera áspera `0,027 → 0,033`
  ao passar a 256 pode vir daí — não isolado.
- Herdados: §5 do [O_RENDER_ANTIGO_SAI](HANDOFF_line_3DModeling_O_RENDER_ANTIGO_SAI_2026-10-03.md#5--aberto)
  (a lâmpada pontual da malha sem gate) e do O_CHAO_QUE_TAPA (menos «o cromo não reflete as outras peças»).

## §6 — Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=42 cargo run -p ph2d-host-desktop --profile smoke`
2. **MODEL** → **Shading** → **Render**. A bola de cromo é a do meio.
3. Deve acontecer: no cromo aparecem a bola vermelha, a amarela e a caixa azul (a verde ao girar), cada
   uma do lado onde está; a bola de alumínio escovado (atrás) mostra-as borradas. Gire: os reflexos
   acompanham sem atraso. Mova uma bola: o reflexo segue-a na hora.
4. Deu errado se: o cromo mostrar só céu e chão, um reflexo aparecer do lado errado ou atrasar um quadro
   ao mover, ou a imagem tremer/granular ao girar.

Foto da câmara de fábrica (`fotografa_cena.sh "PH2D_FIELD_SMOKE=42 PH2D_FIELD_SHADING=render"`): o cromo
mostra a vermelha, a amarela e a azul — não precisa de arrastar.

## §7 — Para o integrador

ids/consts: `smoke::scenes::CENAS 41 → 42` (reconte), `QUEM_SEMEIA` 8 entradas, notas `1..42`
(`smoke.rs`, `lib.rs`). `ph2d-mesh-forward`: grupo 0 ganha `binding 20` (texturas amostradas no fragmento
`15` de `16`) e o `binding 0` passa a **dinâmico**; `Objeto` `96 B`; pipelines `15 → 18` (com
`Rgba16Float`; `10` sem). `ph2d-sky`: `wgsl::OCT` novo (pub), o `sky_oct` mudou-se para ele.
`PROJECT_SCHEMA` intacto; shell `0` linhas; contratos congelados: nenhum. Fixtura nova `~1,4 MB`.

## §8 — A PRÓXIMA ONDA

Sem ordem do dono (03/10): (c) mobile real · (d) dentes de 2–3 px na quina côncava · (e) LOD — que é
também a alavanca do custo de arrastar com capturas · o gate da lâmpada pontual da malha.
