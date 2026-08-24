# TRIAGEM — o estado da arte em quad remesh, degrau a degrau (§2 da SKILL_Cleanroom)

> Papel **E** (Especificador), 2026-08-24. Ledger: [`LEDGER_quadwild.md`](LEDGER_quadwild.md).
> ⚠️ **Este documento é a saída do PASSO 1 do BLOCO-E**, e o passo 1 tem uma ordem embutida:
> *«Achou porta mais barata? PARE e reporte.»* — foi o que aconteceu. **A obra seguinte não é T2.**
>
> ⛔ Nenhum identificador interno de alvo copyleft aparece aqui (§4.2). Nomes de projeto e de
> **API pública** de biblioteca permissiva são uso nominativo, lícito e necessário.

---

## §1 — O achado em uma linha

> ⭐⭐⭐ **ATUALIZADO 2026-08-24, depois da medição:** a decisão final está no
> **[ADR-0164](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)**
> — *a extração é clean-room dos papers; a biblioteca MPL-2.0 fica FORA, como oráculo.*
> A espec funcional completa está em
> [`SPEC_extracao_de_malha_quad.md`](SPEC_extracao_de_malha_quad.md).
> A triagem abaixo é o que **levou** lá, e continua válida como mapa de licenças.

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
| `blossom5-cmake` | ⛔⛔ **wrapper Unlicense, ALGORITMO NÃO-LIVRE** — *avaliação e pesquisa* apenas, **redistribuição proibida**, licença comercial à parte (por isso o repositório dele só guarda um *patch*, nunca o fonte) | **T4** | emparelhamento perfeito de custo mínimo — ⚠️ **o «solver exato» que a nossa `ph2d-quantize` nomeia como *a cura*** |
| `lpsolve` | **LGPL** (conferida) | T0½ | solver linear |

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

## §5-bis — ⭐ A Rota 0 FOI EXECUTADA: o que a medição disse (2026-08-24)

Arnês fora da árvore (`~/Referencias/directional-bench/`), a biblioteca clonada em
`~/Referencias/directional/`. Régua: espelho em Python de `ph2d_quadfill::QuadShape`.

### §5-bis.1 — O que se aprendeu ANTES de qualquer resultado

| achado | consequência |
|---|---|
| ⛔ **A extração não compila sem GMP** — o inteiro-grande embutido não oferece a interface que aquele passo chama | ⚠️ **e o guarda tem o nome trocado**: o `CMake` do próprio projeto define um símbolo, o código testa **outro** ⇒ o caminho com GMP **nunca liga** pela via oficial. ⭐ Irrelevante para nós: em Rust o inteiro/racional exacto é **MIT/Apache** |
| ⚠️ **A integração exige campo de CURL REDUZIDO** | um campo apenas liso **não serve**; os próprios tutoriais de integração leem sempre o campo curl-corrigido |
| ⛔ **Todo o nosso corpus é de QUADS** (só o cubo e o toro são puros; as outras misturam) | uma biblioteca de triângulos lê lixo. *O oráculo de produção tolera porque triangula sozinho* |

### §5-bis.2 — ⭐⭐ A qualidade, com a NOSSA régua, no caso que terminou

Malha **deles**, campo **deles**, extração completa em segundos: **1124 faces, 97,9% quads**.

| grandeza | ⭐ biblioteca MPL-2.0 | oráculo de produção (orelha) | ⛔ o nosso F5 hoje (orelha) |
|---|---|---|---|
| **enviesamento p50** | **`5,0°`** | `6°` | `27°` |
| enviesamento p99 | `30,3°` | `20°` | — |
| **faces com canto pior que 60°** | **`0`** | `0` | **`9 159`** |
| aspecto p50 | `1,13` | `1,08` | `1,98` |
| ⚠️ aspecto p99 · max · `>4×` | `15,45` · `187` · **`43`** | `1,4` · — · **`0`** | — · `122,7` · — |
| espalhamento de área | `1,28` | — | — |

⭐⭐⭐ **A leitura que importa:** na grandeza que perseguimos — **enviesamento** — uma cadeia
por **extração** aterra em `5,0°`, a classe do oráculo, e **5× melhor** que os `27°` do nosso
preenchimento por patch. ⇒ *a cura que a caça por eliminação nomeou tem agora um número
independente por trás.*
⚠️ **E o preço vem junto:** a cauda de **aspecto** dela é **pior** que a do oráculo — `43`
faces acima de `4×` contra `0`. *Ela não é o oráculo; é outra troca.*
⛔ **Comparação entre peças DIFERENTES** (a jarra é deles, a orelha é nossa). Vale como
**classe**, nunca como placar.

### §5-bis.3 — ⛔ E a extração sobre o NOSSO corpus ficou INCONCLUSIVA — por culpa do insumo

| corrida | campo | resultado |
|---|---|---|
| jarra deles · campo **deles** | — | ⭐ **extração completa**, segundos |
| jarra deles · campo **meu** | ⛔ `curl max = 5724` | **cai** (SIGFPE) |
| gancho nosso · campo **meu** | `curl max = 0,70` (plausível) | integração **ok**; extração **> 420 s sem terminar** |
| peça furada nossa (38 arestas de bordo) | — | cai **antes da 1ª linha** |

⇒ ⛔ **Não se pode concluir «a extração é frágil».** O único insumo que produziu extração
completa foi o **deles**; o meu produziu `5724` de curl numa malha onde o deles funciona.
*Toda falha a jusante está confundida com um campo em que não se pode confiar.*

⭐ **O que ISSO entrega, e é o mais accionável da medição:** o contrato de entrada da
extração é **um campo de baixo curl**, e satisfazê-lo é sub-problema próprio.
⚠️⚠️ **E ele apanha-nos:** o nosso F2 foi **ilibado** por contagem de singularidades
(8 = mínimo de Poincaré–Hopf), e **ninguém mediu o curl dele**. Se for alto, a extração
falha para nós pela mesma porta. ⇒ **medição barata e decisiva, antes de qualquer porte.**

### §5-bis.3-bis — ⭐⭐⭐ E ENTÃO O EXPERIMENTO CERTO CORREU: o NOSSO campo, na extração DELES

⚠️ **O que faltava não era afinar o meu campo — era não usar campo meu nenhum.** A biblioteca
lê o campo **por arquivo**, e formato não é expressão protegida (§4.1.4). ⇒ escrevi um
exportador em Rust (`~/Referencias/directional-bench/rustfield/`, harness fora da árvore) que
publica o campo do `ph2d-crossfield` no formato de intercâmbio dela.

**Mesma malha, mesma extração, só o campo muda:**

| grandeza | campo **deles** | ⭐ campo **NOSSO** |
|---|---|---|
| **enviesamento p50** | `5,0°` | ⭐⭐ **`3,0°`** |
| enviesamento p99 | `30,3°` | **`24,6°`** |
| **enviesamento máx** | `43,3°` | **`29,6°`** |
| faces com canto pior que 60° | `0` | **`0`** |
| aspecto p50 | `1,13` | **`1,06`** |
| ⚠️ aspecto máx | `187` | ⚠️ `3 639` (uma lasca) |
| quads | `97,9%` | `92,0%` |

⭐⭐⭐ **Duas coisas ficam provadas de uma vez:**

1. **A rota da extração ultrapassa o oráculo de produção** na grandeza que perseguimos —
   `3,0°` contra `6°` — e é **9× melhor** que os `27°` do nosso preenchimento por patch.
2. ⭐ **O nosso campo é MELHOR que o da biblioteca de referência.** O F2 estava ilibado por
   *contagem de singularidades*; agora está ilibado por **resultado**.
   ⇒ ⛔ **a medição de curl proposta na §5-bis.5 deixa de ser um portão** — ela vira
   diagnóstico útil, não pré-condição. *A pergunta que ela ia responder foi respondida por
   uma via mais forte.*

⚠️ **O que fica em aberto, medido:** a extração dela é **lenta na nossa escala** — segundos
numa peça de `2 404` triângulos, **minutos sem terminar** numa de `6 768`. ⛔ Isso não
invalida a qualidade; **é razão para não portar** (ADR-0164, razão 5).

### §5-bis.3-ter — ⛔⛔⛔ E O CORPUS INTEIRO INVERTEU A CONCLUSÃO (mesma tarde)

⚠️ **A varredura sobre o nosso corpus terminou DEPOIS de eu ter escrito o §5-bis.3-bis, e
desmente-o no ponto que decidia.** Três peças nossas extraíram — e em **segundos**, não em
minutos. Medidas com a **mesma** régua, contra a saída do **oráculo de produção na MESMA
peça**:

| peça | | ⭐ oráculo de produção | ⛔ a cadeia que montei | ⛔ o nosso F5 hoje |
|---|---|---|---|---|
| **enrugada** | enviesamento p50 · máx · `>60°` | **`4,8°`** · `34,6°` · **`0`** | `11,1°` · `82,1°` · `5` | `27°` · — · `9 159` |
| **estriada** | idem | **`7,1°`** · `41,1°` · **`0`** | `12,4°` · `83,6°` · `5` | — |
| **esfera uv** | idem | **`5,9°`** · `38,4°` · **`0`** | `9,1°` · `86,8°` · `6` | — |
| | faces · quads | `3 352`–`4 696` · **`100%`** | `2 178`–`2 225` · `99,7–99,9%` | — |

⛔⛔ **RETRACTADO:** a frase *«a rota da extracção ULTRAPASSA o oráculo de produção»* estava
errada. Ela saiu de **uma** peça — **a deles** — e o corpus **nosso** diz o contrário: a
cadeia que montei é **1,6× a 2,3×** o enviesamento mediano do oráculo, e produz faces com
canto acima de `60°` onde ele produz **zero**.

⭐ **O que SOBREVIVE, e continua a decidir:** contra o **nosso** preenchimento por patch a
cadeia é **2–3× melhor** (`9–12°` contra `27°`), e as faces péssimas caem de **`9 159` para
`5–6`**. ⇒ *a direcção continua certa; o que caiu foi a margem.*

⚠️ **E a correcção reabre um portão que eu tinha fechado cedo demais.** O §5-bis.3-bis
dizia que a medição de curl *«deixa de ser um portão»*, com base na jarra. ⛔ **Errado.** A
diferença entre os nossos `9–12°` e os `5–7°` do oráculo é exactamente a classe de defeito
que um campo **não integrável** produz — e o nosso campo é liso, mas **nunca teve o curl
reduzido**. ⇒ **a §5-bis.5 volta a ser pré-condição.**

⚠️ **A robustez também tem número agora:** de **7** peças nossas, **4** extraíram
(`8–15 s`), **1** recusou, **1** estourou o tecto de `900 s` e **1** caiu com falha de
segmentação (o toro, género 1 — ⭐ **cujo MAPA saiu bem**, logo o defeito é da extração, não
da integração). ⛔ *Isto refuta a minha própria frase «é lenta na nossa escala»: em peças de
27 360 triângulos ela leva 8–10 segundos.* **O problema não é velocidade, é robustez** — e
robustez é precisamente o que o método promete resolver.

### §5-bis.4 — ⛔ Os QUATRO erros desta medição (a parte reutilizável)

1. ⛔ **Alimentei uma biblioteca de triângulos com malhas de quadriláteros** — o corpus
   inteiro. Invalidou as três primeiras corridas.
2. ⛔ **Usei o campo sem curl-correcção** e li a queda como fragilidade da biblioteca; o
   tutorial dela lê sempre o irmão corrigido.
3. ⛔⛔ **Li `curl max = 1,47e-15` como «excelente»** numa peça com **zero** restrições —
   era o **balde que ninguém encheu**, sobre uma malha que já era lixo (erro 1).
4. ⛔ **`pkill -f` matou a própria janela que o executava** (o texto do script estava na
   linha de comando dela), e o `| tail` mascarou três códigos de saída.

⭐⭐ **A lei que os atravessa:** *antes de medir uma ferramenta alheia, REPRODUZA o resultado
DELA com os insumos DELA — e só então troque **um** insumo de cada vez.* Eu troquei malha,
campo e formato ao mesmo tempo, e passei horas a acusar a ferramenta.

### §5-bis.5 — ⭐⭐ O passo seguinte, ENDEREÇADO (e não é meu: exige escrever produto)

⛔ **Meça o curl do NOSSO campo antes de qualquer porte.** É a única medição barata que
pode **matar ou confirmar** a rota inteira, e hoje ela **não existe**: `ph2d-crossfield`
não tem nenhuma noção de curl (conferido — zero ocorrências na crate).

**A lei, e ela é aritmética de aresta dual:** um campo é livre de curl (localmente
integrável) sse, para **toda** aresta interior, as duas faces concordam sobre quanto a
direção **avança ao longo daquela aresta**:

```
curl(e) = ⟨d_f , v_e⟩ − ⟨d_g , v_e⟩
```

com `v_e` o vetor da aresta partilhada, e `d_f`/`d_g` os ramos **emparelhados** do campo
nas duas faces (o emparelhamento é o `period` que a `CrossField` já guarda — ⚠️ usar o
ramo cru dá desacordo por construção, que foi exactamente o defeito da régua de holonomia
em §4-septemetquinquagies do [`PLAN.md`](../quad-remesh/PLAN.md)).

**As peças já existem todas:** `Dual::edges()` dá `f`, `g` por aresta ·
`CrossField::direction(dual, f)` dá o vetor · `CrossField::period(e)` dá o salto.

| o que reportar | por quê |
|---|---|
| `curl` p50 · p99 · **max**, por peça do corpus | ⚠️ **percentis, nunca média** — o defeito é uma faixa |
| **a contagem de arestas medidas**, ao lado | ⛔ senão um balde vazio lê-se como perfeito (§5-bis.4.3) |
| normalizado pelo comprimento da aresta | senão a grandeza mede o tamanho da malha |

⭐ **A barra sai de graça:** o campo que faz a extração alheia terminar e o que a faz cair
foram **os dois medidos** por este arnês — `0,70` e `5724` na mesma unidade da biblioteca.
⚠️ Não são a nossa unidade; ⇒ **o controlo positivo é correr a régua nova sobre um campo
que sabemos ruim** (um campo liso sem restrições) e ver o número subir.

⇒ Se o nosso campo já for de baixo curl, a Rota A fica com **um** problema (a extração).
Se não for, ela fica com **dois**, e o segundo vem primeiro.

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
| ⛔ **Não portar a implementação de referência do emparelhamento (Blossom)** | ela **não é livre** — redistribuição proibida, licença comercial. É **T4**: reimplementar do *paper* é a única rota | §7.1 |
| ⛔ **Não concluir «a extração é frágil»** | todas as quedas medidas estão **confundidas com um campo em que não se pode confiar** (o meu deu `5724` de curl onde o deles funciona) | §5-bis.3 |
| ⛔ **Não usar a leitura de `curl` de uma peça sem restrições como prova** | balde vazio lê-se como perfeito (`1,47e-15`) | §5-bis.4 |
| ⛔⛔ **Não anunciar um resultado medido numa peça DELES como se fosse do nosso corpus** | a jarra deu `3,0°` e o nosso corpus deu `9–12°`; a conclusão inverteu-se | §5-bis.3-ter |
| ⛔ **Não dizer que a extração é «lenta na nossa escala»** | 8–10 s em peças de 27 360 triângulos; o defeito é **robustez** (3 de 7 falham), não velocidade | §5-bis.3-ter |

---

## §7 — O que ficou por conferir (dito, não escondido)

1. ✅ **RESOLVIDO — e virou o achado mais afiado da triagem.** O *blossom5* empacotado
   **não é software livre**: o autor publica-o para *avaliação e pesquisa*, **proíbe a
   redistribuição** e vende licença comercial à parte (o repositório de empacotamento
   guarda **só um patch**, nunca o fonte — é a assinatura da restrição).
   ⛔⛔ **Por que dói:** a nossa [`ph2d-quantize`](../../../crates/ph2d-quantize/src/solve.rs)
   nomeia, em **dois** doc-comments, *«o solver exato por matching (Blossom)»* como **a cura
   da meia-integralidade**. ⇒ *uma cerca que nomeia a cura tem de dizer se a cura está
   disponível.* O fato de licença foi colado aos dois sítios; o caminho lícito é
   **escrevê-lo do *paper*** (público, citado no próprio empacotamento), **nunca portá-lo**.
   ⭐ O nosso oráculo é construído **sem** ele — conferido no binário, que não linka um
   único símbolo dessa família.
2. ✅ **RESOLVIDO:** o solver linear embutido é **LGPL**.
3. ⚠️ **ABERTO:** o arquivo do mesher de N-funções **sem banner** de licença — a declaração
   de repositório cobre, mas quem tomar a Rota A deve pedir confirmação ao autor.
4. ⛔ **A vassoura de identificadores do alvo GPL NÃO foi gerada** para a travessia (§6);
   a que existe foi montada **só do que esta janela de facto viu**, e é o que varre os
   artefatos. A vassoura completa é o **primeiro acto** da Rota B, se a Rota B abrir.

---

## §8 — ⚠️ O achado que dói, e fica registrado

A **quantização Bi-MDF** — o nosso **F4**, com óptimo demonstrado — foi reimplementada por
clean-room a partir do *paper*. A biblioteca que a implementa é **MIT**, e sempre foi.

⇒ *A triagem que a skill manda fazer no passo 1 não existia quando aquela fase foi
construída.* **Não é trabalho perdido** (o nosso F4 é nosso, sem obrigação nenhuma), mas é a
medida exacta do que este documento poupa: **uma fase inteira**.
