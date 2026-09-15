# Fixtures — as corridas do oráculo do GESTO DE CORTE

Insumo da [`SPEC_trim_gesture.md`](../../SPEC_trim_gesture.md). Cada número com selo **M** naquela
espec sai de uma linha de `corridas_do_oraculo.json`.

## Proveniência (§5 da SKILL_Cleanroom — a da ENTRADA decide a da SAÍDA)

| | |
|---|---|
| **Malhas de entrada** | ⭐ **NOSSAS** — geradas por `ph2d_mesh::shapes` e `ph2d_mesh::shapes_open` (receita abaixo) |
| **Quem computou as saídas** | o binário instalado do alvo, corrido **fora da árvore** como oráculo, por um arnês **nosso** que só usa a API pública dele |
| **Estatuto legal** | ⭐ **dados.** A saída de um programa não é coberta pela licença do programa — é **texto de licença**, não opinião (SKILL_Cleanroom §1.1; GPLv2 §0) |
| **Data** | 2026-09-15 · oráculo `5.2.1 LTS` |
| **Regenerar** | acto de **E** (a janela-I ⛔ não corre o oráculo). O arnês vive em `~/Referencias/blender-trim/oracle/` |

⛔ **Não há aqui nenhum asset de terceiros** — nem como entrada. As malhas são todas nossas, e a
peça densa de 98 306 vértices **não é distribuída** (6 MB de OBJ por zero informação nova): ela
reproduz-se com a mesma receita.

## As peças, e por que cada uma está aqui

| ficheiro | peça | V / F | por que ela existe nesta lista |
|---|---|---|---|
| `cube.obj` | cubo | 8 / 6 | o caso mínimo fechado, e o único cuja face é grande em relação ao corte |
| `uv_sphere_16x32.obj` | esfera | 482 / 512 | ⭐ a peça de trabalho da matriz inteira |
| `torus_32x16.obj` | toro | 512 / 512 | **género 1** — a peça que apanha perda de asa |
| `cylinder_32.obj` | cilindro | 66 / 96 | tampas planas ⇒ o caso **coplanar**, que é onde os solucionadores se separam |
| `open_tube3.obj` | tubo **aberto** | 18 / 12 | ⭐ bordo — §11.1 da espec |
| `open_disc.obj` | disco **aberto** | 19 / 24 | bordo, com miolo |
| `pillow.obj` | peça degenerada | 3 / **2** | o caso mínimo que ainda é uma malha. ⚠️ **O ficheiro tem `2` faces e o oráculo mediu `1`**: são os mesmos três vértices com enrolamento oposto, e a importação do alvo coalesce-as **em silêncio** (espec §11.1) |
| *(não distribuída)* | esfera de escultura | 98 306 / 98 304 | a medição de densidade longe do corte, e a nota pública dos 100 k |

## Receita das entradas (determinística, e é NOSSA)

```rust
use ph2d_mesh::{write_obj, ExportPiece, Pose, shapes, shapes_open};
// cube            = shapes::cube(1.0)
// uv_sphere_16x32 = shapes::uv_sphere(16, 32, 1.0)
// torus_32x16     = shapes::torus(32, 16, 1.0, 0.35)
// cylinder_32     = shapes::cylinder(32, 0.7, 1.6)
// sculpt_sphere   = shapes::sculpt_sphere(1.0)      // a não distribuída
// open_tube3      = shapes_open::open_tube3()
// open_disc       = shapes_open::open_disc()
// pillow          = shapes_open::pillow()
let piece = ExportPiece { name: Some(nome), mesh: &malha, pose: Pose::default() };
std::fs::write(caminho, write_obj(&[piece])).unwrap();
```

## O formato de `corridas_do_oraculo.json`

**56 corridas, com NOME ÚNICO** — e a unicidade é **verificada**, não prometida:
`python3 verifica_corridas.py` (piso de população, nome único, e toda recusa com motivo de
domínio). ⛔ **Corra-o antes de citar qualquer número daqui.**

⚠️⚠️ **Porque ele existe:** este corpus já teve **dois nomes repetidos**, com registos
**contraditórios** sob o mesmo nome — um par sem medição nenhuma (defeito do arnês, não do
oráculo) e o par bom. *Um arnês indexado por nome apanha o que calhar, e a discordância é muda.*
As duas corridas fantasma foram **apagadas**: uma corrida que nunca produziu medição não é fixtura.

Cada corrida tem as opções que a produziram e, quando não foi recusada, o estado da malha `antes`
e `depois` nas mesmas colunas:

| chave | o que é |
|---|---|
| `vertices` · `faces` · `triangulos` · `quadrilateros` · `poligonos_maiores` | contagens |
| `arestas_de_bordo` · `arestas_nao_manifold` | a saúde topológica — ⭐ as colunas que provam que o corte fecha |
| `aresta_p01` · `aresta_p50` · `aresta_p99` | a **densidade**, em comprimento de aresta |
| `volume_com_sinal` | ⭐ positivo = nada ficou com a normal virada; e é a **única** coluna que distingue dois bolsos de profundidades diferentes com a mesma contagem |
| `vertices_de_entrada_preservados_ao_bit` | ⭐⭐ **a régua que separa a booleana da rota por voxel** (espec §1) |
| `lado_oposto` | a mesma pergunta restrita aos vértices longe do corte |
| `veredito` · `mensagem` | `FINISHED` / `CANCELLED` / `RECUSADO`. ⛔ A `mensagem` é o motivo **em vocabulário de domínio** — ⚠️ **nunca** o texto que o alvo imprime (ver o parágrafo abaixo) |

⚠️ **As chaves estão em vocabulário do DOMÍNIO, não no do alvo** (SKILL §5): nenhum nome interno
dele aparece neste ficheiro, nem nos nomes dos ficheiros desta pasta.

⛔⛔ **E a coluna `mensagem` NÃO traz o texto que o alvo imprime.** Uma mensagem de erro é **texto
do programa**, não dado (SKILL §5: *dump é dado; texto do programa é programa*), e o sweep final
apanhou-a aqui. Cada recusa está **traduzida para o domínio**, preservando a informação: qual foi
o motivo. ⚠️ *Foi o próprio instrumento de parede que cobrou isto — a primeira versão destas
fixturas publicava as mensagens cruas.*

⚠️ **O `raio_do_cursor` é `null` em quase todas as linhas de propósito** — só as corridas que
**varrem** esse knob o fixam; as outras usam o valor de fábrica, e escrever um número ali sugeriria
uma medição que não foi feita.
