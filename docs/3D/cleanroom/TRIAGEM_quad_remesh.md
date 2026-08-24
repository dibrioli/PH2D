# TRIAGEM — o estado da arte em quad remesh, degrau a degrau (§2 da SKILL_Cleanroom)

> Papel **E** (Especificador), 2026-08-24. Ledger: [`LEDGER_quadwild.md`](LEDGER_quadwild.md).
> ⚠️ **Este documento é a saída do PASSO 1 do BLOCO-E**, e o passo 1 tem uma ordem embutida:
> *«Achou porta mais barata? PARE e reporte.»* — foi o que aconteceu. **A obra seguinte não é T2.**
>
> ⛔ Nenhum identificador interno de alvo copyleft aparece aqui (§4.2). Nomes de projeto e de
> **API pública** de biblioteca permissiva são uso nominativo, lícito e necessário.

---

## §1 — O achado em uma linha

⭐⭐⭐ **A família GPL não é uma coisa só, e as DUAS fases que nos bloqueiam hoje —
o arredondamento inteiro e a extração — existem sob MPL-2.0**, que é a licença
**já permitida** pelo [`deny.toml`](../../../deny.toml) desta casa.

⇒ O degrau da obra seguinte é **T0½ (porte fiel com copyleft por-ARQUIVO)**, não
**T2 (clean-room de semanas)**. A diferença medida na escada do §2 é *horas–dias*
contra *dias–semanas*.

---

## §2 — A tabela de licenças, MEDIDA

Lida arquivo a arquivo em 2026-08-24, de cada `LICENSE`/`COPYING` e de cada cabeçalho
de fonte, no clone local e via API pública.

### §2.1 — O oráculo que usamos hoje (a família `quadwild`) é um MOSAICO

| dependência | licença **medida** | degrau | fase que ela serve |
|---|---|---|---|
| umbrella `quadwild` / `quadwild-bimdf` | **GPL-3.0** | T2 | o pipeline inteiro |
| `vcglib` | **GPL-3.0** | T2 | malha, I/O, utilitários |
| `xfield_tracer` | **GPL-3.0** | T2 | ⚠️ **o traçado de separatrizes (o nosso F3)** |
| `CoMISo` | **GPL-3.0** | T2 | solver misto-inteiro |
| `quadretopology` | ⚠️ **sem licença própria** — nenhum `LICENSE`, nenhum cabeçalho nos fontes ⇒ herda a GPL-3.0 do umbrella | T2 | preenchimento por padrões (o nosso F5) |
| `libigl` | MPL-2.0, **com um sub-diretório `copyleft/` GPL** | T0½ / T2 | campo, parametrização |
| `libsatsuma` | ⭐ **MIT** | **T0** | ⚠️ **quantização Bi-MDF (o nosso F4)** |
| `lemon` | **Boost** | **T0** | fluxo de custo mínimo |
| `OpenMesh` | **BSD-3** | **T0** | half-edge |
| `libTimekeeper` · `nlohmann/json` | **MIT** | T0 | instrumentação, serialização |
| `eigen` | **MPL-2.0** | T0½ | álgebra linear |
| `glew` | **BSD-3** | T0 | só visualização |
| `blossom5-cmake` | Unlicense (o **wrapper**) | ⚠️ | ⚠️ o *blossom5* empacotado tem licença própria, **não conferida** — e o build do próprio projeto o desliga por default (`SATSUMA_ENABLE_BLOSSOM5=0`) |
| `lpsolve` | ⚠️ **não conferida** (tipicamente LGPL) | ⚠️ | solver linear |

⚠️ **A leitura que a tabela obriga:** o copyleft da família entra por **três** submódulos
(`vcglib`, `xfield_tracer`, `CoMISo`) mais um sem licença (`quadretopology`). **Não** por
todo o resto.

### §2.2 — ⭐⭐ O irmão permissivo que fecha as duas fases abertas

**Directional** (Amir Vaxman) — **MPL-2.0**, conferida em **cabeçalho de arquivo**, não
por reputação:

> `// This Source Code Form is subject to the terms of the Mozilla Public License v. 2.0.`

| propriedade | valor medido |
|---|---|
| licença | **MPL-2.0** por-arquivo, confirmada em 6 dos 7 módulos da rota de extração |
| ⚠️ exceção | **um** arquivo da rota (o mesher de N-funções) **não traz banner** — só a declaração de repositório cobre; ambiguidade a nomear se a rota for tomada |
| dependências | **Eigen** (MPL-2.0) + PolyScope (só visualização), **ambas embutidas**; `GMP` (LGPL) é **opcional e dispensável** — a biblioteca traz o inteiro-grande próprio, também MPL-2.0 |
| ⛔ o que **não** arrasta | zero GPL. Ela **deixou de depender** de libigl e de CGAL |
| forma | header-only, um arquivo por função |

**Os módulos públicos que ela expõe, contra as nossas fases:**

| a nossa fase | o módulo público dela | estado nosso |
|---|---|---|
| G1 — cortar em discos | `cut_mesh_with_singularities.h` | ✅ construído |
| G2 — pentear + salto de período | `combing.h` | ✅ construído |
| — | `index_prescription.h` | (F2 já ilibado por medição) |
| G3 — solver global | `setup_integration.h` · `integrate.h` | ✅ construído |
| ⛔ **G3-bis — o ARREDONDAMENTO INTEIRO** | ⭐ **`iterative_rounding`**, com `integralSeamless` (*seamless translacional pleno* = translações inteiras) e `roundSeams` (arredondar costuras **ou** singularidades) | ⛔ **é exactamente o nosso bloqueador** |
| ⛔ **a EXTRAÇÃO** | ⭐⭐ `branched_isolines.h` · `setup_mesher.h` · `mesher.h` + o mesher de N-funções, com predicados **exactos** | ⛔ **é exactamente a nossa obra seguinte** |

⭐ **E ela é alcançável, não teórica:** o repositório traz tutoriais numerados que
**demonstram** as duas — um de *integração seamless*, um de *costuras/singularidades/
arredondamento*, um de *meshing*.

---

## §3 — Patente (§8.1) — o checkpoint incondicional

Buscado em **2026-08-24**. Termos: *quad mesh extraction · cross field · integer grid map ·
quadrilateral remeshing · global parametrization*, cruzados com Autodesk, Pixologic, Maxon,
Adobe, Dassault, Siemens, Ansys e universidades.

| patente | dono | estado **medido** | lê sobre o nosso caminho? |
|---|---|---|---|
| **US 8.531.456** — remalhamento automático por mapeamento de grade 2D em malhas de género g | Technion R&D Foundation | ⭐ **EXPIRADA** (*expired — fee related*) | ⇒ **divulgação pública total + domínio público**: é espec de graça (§1.5.4) |
| **US 11.017.597** — redução de singularidades em malhas quadrilaterais | (concedida 2021, viva) | **VIVA** | ⛔ **NÃO** — ela cobre substituir sub-malhas de uma malha quad **já existente** por gabaritos de singularidade mínima. É pós-processamento reactivo; **não** alcança campo direccional nem mapa de grade inteira. ⚠️ **Cerca a nomear:** se algum dia construirmos *«troque um pedaço com muitas singularidades por um gabarito»*, isso **lê** sobre ela |
| **US 9.349.216** — geração e edição de malhas quad **por esboço** | ETH Zurich + Disney Enterprises | **VIVA até 2034** | ⛔ **NÃO** — a reivindicação é sobre rede de curvas **desenhada pelo utilizador**. ⚠️ **Cerca a nomear:** os autores são os do *paper* de quadrangulação por PADRÕES de retalhos de n lados — que é a família do nosso **F5**. A rota automática não lê sobre ela; uma rota **autorada** por curvas leria |

⇒ ⭐ **Nenhuma patente viva bloqueia o caminho campo→mapa inteiro→extração.** Duas cercas
ficam **nomeadas**, e nenhuma delas está no caminho de hoje.

---

## §4 — O que o estado da arte de 2024–2026 mudou (e por que quase nada disso é nosso)

| linha | o que ela melhora | serve-nos? |
|---|---|---|
| Instant Meshes (BSD) · QuadriFlow (BSD) | família **local** | ⛔ já medida e **rejeitada por classe** — é o que motivou o pivô |
| quadwild / Bi-MDF (GPL) | família **global**, referência de produção | é o nosso oráculo |
| **NeurCross** (2024) · **CrossGen** (2025) · NeurFrame (2026) | ⭐ o **CAMPO CRUZADO**, por rede neural — menor erro angular, e ordens de grandeza mais rápido | ⛔ **NÃO nos serve**: o nosso campo (F2) já foi **ilibado com número** — 8 singularidades contra as 8 do oráculo, que é o mínimo de Poincaré–Hopf. *Melhorar o que já está certo não move o produto* |
| QuadGPT (2025) · QuadLink (2026) | geração **autoregressiva** de malha | ⛔ outra classe inteira; sem código no caso do QuadGPT |
| *On Quad Mesh Extraction From Messy Grid Preserving Maps* (Ray, 2025) | ⭐ **a EXTRAÇÃO** a partir de mapas imperfeitos — a nossa fase | ⚠️ **é fundação, não algoritmo**: o próprio *paper* diz que **abre** a pesquisa por um extractor robusto. Sem código. **Leitura obrigatória** para quem especificar a extração |

⭐⭐ **A leitura que atravessa a tabela:** *todo o avanço público de 2024–2026 aconteceu na
fase que nós já temos correcta.* A fase que nos falta — extrair de um mapa de grade inteira —
está estável desde 2013, publicada, e **disponível a MPL-2.0**.

---

## §5 — As três rotas, com o preço de cada uma

### ⭐ Rota A — porte fiel T0½ do irmão MPL-2.0 (**recomendada**)

- **Custo:** horas–dias (a escada do §2).
- **Sem parede:** porte é acto **licenciado**; não há espec, nem ledger de contaminação, nem
  janela queimada. Só **atribuição**.
- **O preço real, dito inteiro:** MPL-2.0 é copyleft **do ARQUIVO**. Um arquivo Rust que
  seja tradução de um arquivo deles **permanece MPL-2.0**, com o fonte *dele* disponível.
  ⭐ **Não contamina o resto do repositório** — a MPL-2.0 §3.3 permite explicitamente
  combinar com código proprietário numa obra maior. ⇒ o custo é **publicar N arquivos**,
  e `MPL-2.0` **já está na lista de licenças aceites** desta casa.
- ⚠️ **Não é «copiar e colar»:** é C++ header-only sobre Eigen e sobre as estruturas de
  malha deles; a tradução para a `ph2d-mesh` é obra real.
- ⚠️ **Quem executa é OUTRA janela** (§2 da skill): porte fiel não se mistura com quem leu
  alvo copyleft.

### Rota B — clean-room T2 da família GPL

- **Custo:** dias–semanas, mais 4 janelas (E → R-pré → I → R-pós), vassoura, sweep, ledger.
- **Ganho sobre a Rota A:** **nenhuma obrigação de licença**.
- ⛔ **Só se justifica** para o que a Rota A **não** cobre: hoje isso é o **F3** (traçado) e
  o **F5** (preenchimento por padrões) — e nenhum dos dois é o bloqueador de agora.

### ⭐⭐ Rota 0 — o SEGUNDO ORÁCULO, antes de qualquer porte

⚠️ **Nenhuma das duas rotas acima tem um número atrás.** Ninguém mediu se a saída da
biblioteca MPL-2.0 é melhor que a nossa — ela é biblioteca de **pesquisa**, o nosso oráculo
é remalhador de **produção**.

⇒ Compilá-la (permissiva, header-only, Eigen embutido), correr o mesmo corpus de 10 peças e
medir com a **nossa** régua por-face (`QuadShape`: aspecto · enviesamento · área) contra os
`1,08 / 6° / ZERO` do oráculo de produção.

⭐ **É a lei que esta linha já pagou três vezes em 23/08:** *meça o alcance antes de
construir*. Custa horas, e responde se a Rota A vale alguma coisa **antes** de alguém
traduzir uma linha.

---

## §6 — ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| ⛔ **Não abrir clean-room T2 para o arredondamento inteiro** | existe sob **MPL-2.0**, licença **já aceite** pelo `deny.toml` — semanas contra horas | §2.2, §5 |
| ⛔ **Não abrir clean-room T2 para a extração** | idem, com tutorial que a demonstra | §2.2 |
| ⛔ **Não perseguir o campo neural (NeurCross/CrossGen/NeurFrame)** | o nosso campo já dá **8 singularidades = o mínimo de Poincaré–Hopf**, igual ao oráculo | §4 |
| ⛔ **Não construir a partir do *paper* de extração de 2025** | o próprio *paper* se declara **fundação**, não algoritmo; sem código | §4 |
| ⛔ **Não gerar a vassoura de identificadores do alvo GPL agora** | gerá-la **exige ler o fonte GPL**, e isso aumenta contaminação por uma rota que pode nunca abrir | §7 |
| ⛔ **Não tratar a família GPL como bloco único** | o copyleft entra por **3 submódulos + 1 sem licença**; a quantização que reimplementámos era **MIT** o tempo todo | §2.1 |

---

## §7 — O que ficou por conferir (dito, não escondido)

1. ⚠️ A licença real do *blossom5* empacotado (o **wrapper** é Unlicense; o algoritmo tem
   licença própria). **Sem urgência:** o build do próprio projecto o desliga por omissão.
2. ⚠️ A licença do solver linear embutido (tipicamente LGPL).
3. ⚠️ O arquivo do mesher de N-funções **sem banner** de licença — a declaração de
   repositório cobre, mas quem tomar a Rota A deve pedir confirmação ao autor.
4. ⛔ **A vassoura de identificadores do alvo GPL NÃO foi gerada**, de propósito (§6).
   Ela é o **primeiro acto** da Rota B, se a Rota B abrir.

---

## §8 — ⚠️ O achado que dói, e fica registrado

A **quantização Bi-MDF** — o nosso **F4**, com óptimo demonstrado — foi reimplementada por
clean-room a partir do *paper*. A biblioteca que a implementa é **MIT**, e sempre foi.

⇒ *A triagem que a skill manda fazer no passo 1 não existia quando aquela fase foi
construída.* **Não é trabalho perdido** (o nosso F4 é nosso, sem obrigação nenhuma), mas é a
medida exacta do que este documento poupa: **uma fase inteira**.
