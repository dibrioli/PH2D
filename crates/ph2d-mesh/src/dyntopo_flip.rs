//! **O FLIP DE ARESTA** — o operador que devolve a qualidade que o corte tirou.
//!
//! Irmão do [`crate::dyntopo`]: o corte e o flip são as duas metades de UM refino, e
//! separá-los em crates diferentes seria convidar alguém a rodar só o primeiro.
//!
//! # Por que ele existe, com o número ao lado
//!
//! Um refino que só PARTE não pode manter a forma dos triângulos, e a razão é
//! estrutural: para não deixar rachadura, a vizinha de uma face escolhida tem de
//! aprender o vértice novo — e uma face com **uma** aresta partida vira dois
//! triângulos com metade do ângulo daquele canto (o corte "verde"). Isso não é
//! um caso raro: é o ANEL inteiro em volta de tudo o que o pincel toca, a cada
//! dab.
//!
//! Medido num traço de 24 dabs sobre a esfera de smoke, pelo pior ângulo mínimo
//! de triângulo — as duas saídas que existem sem este operador:
//!
//! | esquema | pior ângulo | abaixo de 10° | vértices no hemisfério NÃO tocado |
//! |---|---|---|---|
//! | controle (sem refino) | 21,21° | 0,0% | 57 |
//! | escolha por ARESTA | 1,53° | 15,0% | 57 |
//! | escolha por FACE | 0,59° | 48,0% | 57 |
//! | + promover a vizinha a 1→4 | **20,47°** | 0,0% | **846** |
//!
//! ⚠️ **A última linha é a tentação, e ela foi MEDIDA e REJEITADA:** dar à
//! vizinha a mesma garantia da escolhida cascateia pela malha inteira — 846
//! vértices onde o artista nunca passou, ou seja o **oposto exato** da promessa
//! deste modo. Qualidade é global; a pegada não pode ser.
//!
//! O flip fecha a tensão porque ele é **local e não propaga**: ele não cria nem
//! remove vértice, só troca a diagonal de dois triângulos vizinhos quando isso
//! deixa o pior ângulo dos dois melhor. É o operador de sempre do remalhamento
//! incremental (Botsch–Kobbelt: *split · collapse · **flip** · smooth*), e a
//! metade `collapse` é a wave seguinte.
//!
//! # As quatro recusas — três são geometria, e a quarta é a RODADA
//!
//! 1. **Aresta de borda** (uma face só) não tem diagonal a trocar.
//! 2. **A diagonal nova já existe** — flipar criaria uma segunda aresta entre o
//!    mesmo par, e a malha deixaria de ser variedade. Acontece de verdade num
//!    tetraedro e em qualquer anel de valência 3.
//! 3. **A troca DOBRARIA a superfície** — se uma das faces novas aponta para o
//!    lado oposto do par antigo, o flip é uma dobra e não uma melhoria. Sem esta
//!    guarda um vinco afiado é "melhorado" para um bico invertido.
//! 4. ⭐ **A diagonal nova já foi criada NESTA rodada** — e esta é a única que a
//!    adjacência de entrada não sabe responder.
//!
//! ## Por que a recusa 4 existe, com o número que a nomeou
//!
//! A recusa 2 pergunta ao anel de `c` se `d` já lá está. ⚠️ **Esse anel é o de
//! ANTES da rodada.** Duas trocas da mesma rodada, sobre pares de faces
//! **disjuntos** — logo fora do alcance do `spent`, que só protege a face —,
//! podem produzir a mesma diagonal `c—d`: nenhuma das duas a vê, e a malha sai
//! com **duas arestas entre o mesmo par**.
//!
//! ⚠️ **É raro, silencioso, e apodrece tudo o que vem depois.** Medido em
//! 2026-08-21 (`ph2d-remesh-iso/tests/it/manifold_probe.rs`): **UMA** ocorrência em
//! 9 968 trocas de uma única rodada sobre a esfera de 13 682 vértices — e a
//! aresta ofensora sai com **quatro** faces, que é a assinatura de *criada duas
//! vezes* (uma diagonal criada por cima de outra que já existia teria três).
//!
//! O preço não é a aresta: é que o remesh isotrópico chama esta porta em laço, e
//! o defeito **acumula**. O cubo saía do passe com **18 anéis abertos**, e três
//! fases adiante isso virava **33 singularidades falsas** e uma soma de índices
//! de `−1` onde a topologia exige `+8`. *A invariante de Poincaré–Hopf deixava de
//! bater sem que nada no campo estivesse errado.*
//!
//! # Ele pergunta pela REGIÃO, e o número que obrigou isso
//!
//! A primeira versão varria **toda aresta da malha** por rodada. Medido num dab
//! a 98k vértices (340k arestas), isso é **33,2 de 66,1 ms — metade do dab** —, e
//! a segunda rodada gastava 17,3 ms para achar **zero** flips.
//!
//! ⚠️ **E o desperdício era de VARREDURA, não de mudança:** medindo por faixa de
//! distância ao centro do dab, **todas** as faces alteradas caem dentro da esfera
//! do pincel e **nenhuma** fora (3648 dentro, 0 fora, a 28k). Ou seja, a
//! varredura global já respondia *"o trabalho é todo local"* — pagando `O(malha)`
//! para dizê-lo.
//!
//! Então o operador recebe as faces que o corte mexeu e olha as arestas DELAS; a
//! rodada seguinte recebe as faces que a anterior flipou, porque é só ali que a
//! forma mudou de novo.
//!
//! ⚠️ **A ORDEM de visita muda, e com ela o conjunto de flips** — `spent` deixa
//! uma face entrar numa troca só por rodada, então percorrer por *(face,canto)*
//! não escolhe exatamente as mesmas trocas que percorrer por id de aresta. O que
//! é gateado não é a rota e sim a **propriedade**: o pior ângulo mínimo depois de
//! um traço (`a_moving_dab_does_not_shred_the_triangles`). Um gate de identidade
//! aqui pinaria uma heurística; este pina o que ela existe para entregar.
//!
//! ⚠️ **E ele não constrói mais o [`crate::Edges`]:** as duas perguntas que ele
//! fazia àquele grafo — *quem é a outra face desta aresta* e *a diagonal nova já
//! existe?* — são as duas respondidas pela [`crate::Adjacency`], que a malha já
//! carrega. `id_of` era, literalmente, uma varredura do anel de `c` atrás de `d`.

use std::collections::BTreeSet;

use crate::adjacency::Adjacency;
use crate::face::Face;
use crate::mesh::{Mesh, RegionScratch};

/// Quantas rodadas de flip um refino pode gastar.
///
/// ⚠️ **É um teto de RECURSO, e o recurso é o quadro** — o mesmo argumento do
/// `MAX_PASSES` do corte. Medido, a segunda rodada já quase não acha o que
/// trocar (o critério é estritamente melhorar, então o processo drena); três é
/// folga para o caso em que um dab inteiro nasce de uma vez.
pub(crate) const MAX_ROUNDS: usize = 3;

/// O ganho mínimo, em cosseno do ângulo, para uma troca valer a pena.
///
/// ⚠️ **Um limiar estritamente positivo é o que garante que isto TERMINA**: cada
/// troca aceita aumenta o pior ângulo do par, e um empate não é aceito — então
/// não há ciclo de duas arestas trocando uma para a outra para sempre.
const MIN_GAIN: f32 = 1e-4;

/// ⭐⭐⭐ **O GANHO MÍNIMO DE ALINHAMENTO — e é ele que faz o BOTÃO ser um botão.**
///
/// ⛔⛔⛔ **Com o [`MIN_GAIN`] (de `1e-4`) este passe era QUASE BINÁRIO no
/// knob**, e a razão é estrutural: o critério é uma COMPARAÇÃO entre duas
/// preferências, e a preferência do pente é `k · cos 4α` — o `k` escala os dois
/// lados e **cancela-se**. ⇒ a `6 %` do curso o flip trocava quase tanto como no
/// tecto, e medido na chapa isso deixava o pior triângulo da faixa em
/// **`0,24°`** (contra `11,57°` por pentear), *com o `Q` alto*.
///
/// ⭐ A cura não é uma cerca nova: é o limiar ser comparado contra a preferência
/// **já escalada**, o que faz o ganho exigido em `cos 4α` valer
/// `GANHO / k` — ele **cresce** quando o botão desce, e abaixo de um certo ponto
/// passa de `2`, que é o alcance inteiro da função. *O passe desliga-se sozinho
/// no curso baixo, sem um segundo número a dizer onde.*
const GANHO_DO_ALINHAMENTO: f32 = 0.20;

/// ⭐⭐⭐ **O CHÃO DE QUALIDADE do passe por DIRECÇÃO** — em `−cos(ângulo)`, que
/// é o que a [`worst_angle`] guarda (maior é MELHOR).
///
/// ⛔⛔⛔ **A cerca MONÓTONA (`novo >= velho`) foi construída e medida QUASE
/// INERTE:** na bola da cena de smoke ela move o `Q` de `+0,0179` para `+0,0208`
/// num rumo e **piora** noutro (`+0,0175 → +0,0149`) — *uma troca que só aceita
/// melhorar a forma quase nunca acontece, porque o critério de forma já drenou o
/// que havia*. ⇒ a cerca que fica é **ABSOLUTA**: a troca pode piorar a forma
/// desde que o par continue acima deste ângulo.
///
/// ⛔⛔⛔ **ELE ESTAVA EM `24°` E O DONO REPROVOU-O COM FOTO** (*«pouca ou
/// nenhuma diferença»*, 18/09) — e a medição deu-lhe razão: desenhado o arame
/// dos dois lados do controlo, no regime da cena `=49`, as duas imagens são
/// **indistinguíveis**. ⭐⭐⭐ **Este chão é a alavanca, e ele era a única das
/// três constantes do passe que o era** (`docs/3D/ferramentas/varre_as_constantes_do_flip.py`): as rondas
/// **convergem** (`8` e `20` dão a MESMA malha) e o ganho vale `+1` ponto de
/// grade, enquanto o chão vale `+6`.
///
/// # A régua que faltava, e é ela que decide
///
/// ⚠️⚠️ **O `Q` é uma MÉDIA e o olho lê uma CONTAGEM.** A régua que escolhe este
/// número é a `grade_da_faixa` da `ph2d-sculpt3d` — *que fracção das arestas corre
/// a menos de `15°` da grade do traço* —, e uma malha sem direcção nenhuma lê
/// `33 %` nela, porque o desvio à grade é uniforme em `[0°, 45°]`.
///
/// # A janela, MEDIDA em graus e nas DUAS peças
///
/// Bola da cena `=49`, pior dos quatro rumos, com o pente no tecto (a coluna
/// `lascas` conta triângulos abaixo de `5°` em ~2 000 da faixa):
///
/// | chão | grade | `Q` | pior ângulo | lascas |
/// |---|---|---|---|---|
/// | desligado | `32,2 %` | `−0,046` | `22,8°` | `0` |
/// | `24°` (o de ontem) | `37,6 %` | `+0,066` | `22,5°` | `0` |
/// | `20°` | `38,8 %` | `+0,109` | `17,3°` | `0` |
/// | `18°` | `40,1 %` | `+0,168` | `5,5°` | `0` |
/// | **`16°`** | **`41,4 %`** | **`+0,214`** | **`8,0°`** | **`0`** |
/// | `14°` | `43,4 %` | `+0,245` | `5,1°` | `0` |
/// | `12°` | `45,1 %` | `+0,266` | `3,2°` | **`1`** ⛔ |
/// | `8°` | `49,1 %` | `+0,297` | `1,0°` | `7` ⛔ |
///
/// ⇒ **o fundo da janela é `14°`**: no degrau seguinte nasce a primeira lasca.
/// **O topo é `18°`**: acima dele o desenho volta a ser indistinguível do lado
/// desligado, que é o report. O meio é **`16°`**.
///
/// ⭐⭐ **E a chapa da bancada escolhe o MESMO número, por outro caminho**
/// (`diag_a_escada_do_pente`, pior ângulo MÍNIMO do curso alcançável do botão,
/// contra a barra de `2°` do `o_pente_nao_compra_alinhamento_com_lascas`):
///
/// | chão | `Q` no tecto | pior ângulo mínimo do curso |
/// |---|---|---|
/// | `24°` | `+0,089` | `11,57°` |
/// | `18°` | `+0,128` | `7,27°` |
/// | **`16°`** | **`+0,153`** | **`11,57°`** |
/// | `14°` | `+0,165` | `7,33°` |
/// | `12°` | `+0,190` | `5,33°` |
///
/// ⭐ A `16°` o mínimo do curso é **exactamente o da malha por pentear** — ali o
/// passe nunca deixa a chapa pior do que ela já estava sem pente nenhum, e é o
/// único degrau da janela de que isso se pode dizer.
///
/// # ⛔⛔ E a janela de ontem foi medida FORA do alcance do botão
///
/// A tabela anterior dizia `12° → 0,17×` e fechava a janela em `[20°, 28°]`.
/// Re-medida hoje em graus absolutos, essa queda **não existe dentro do curso**:
/// a `12°` a chapa lê `5,33°` contra uma barra de `2°`, e só desce abaixo de
/// `3°` em `pente ≥ 1,25` — que o slider **não produz** (a faixa é `0..1` e o
/// [`k`](ph2d_rake::k_do_pente) satura). *Um limite escolhido num regime que o
/// produto não alcança é um limite sobre outro programa.*
///
/// # ⛔ O que ficou REFUTADO ao mesmo tempo
///
/// A hipótese natural era a outra metade — **pregar a NORMALIZAÇÃO do campo de
/// tamanho pela média em vez de por um extremo**, para o viés poder fazer o
/// passe trabalhar MAIS. Medida (`diag_a_normalizacao_contra_a_grade`), ela
/// multiplica os cortes por **`23×`** (`2 100 → 47 624`), leva o pior triângulo
/// a `0,64°` com `18` lascas — e a grade fica em `40,7 %` contra `39,5 %`.
/// ⇒ *o tecto do campo de tamanho é `~41 %`, e ele não é a alavanca.*
///
/// ⚠️ **A cerca MONÓTONA (`novo >= velho`) foi construída e medida QUASE
/// INERTE:** na bola da cena de smoke ela move o `Q` de `+0,0179` para `+0,0208`
/// num rumo e **piora** noutro (`+0,0175 → +0,0149`) — *uma troca que só aceita
/// melhorar a forma quase nunca acontece, porque o critério de forma já drenou o
/// que havia*. ⇒ a cerca que fica é **ABSOLUTA**: a troca pode piorar a forma
/// desde que o par continue acima deste ângulo.
///
/// ⚠️ **O preço está medido e cabe:** as trocas por traço passam de `1 221` para
/// `2 760` e o dab de `1,11` para `1,37 ms` em `--release`, contra o orçamento
/// de `8 ms` do passe.
const CHAO_DO_ALINHAMENTO: f32 = -0.961262; // -cos(16°)

/// **Relaxa a REGIÃO por troca de diagonal.** Devolve quantas arestas trocaram.
///
/// `seeds` são as faces que o corte mexeu. Não move vértice nenhum e não muda a
/// contagem — só a ligação. É por isso que ele pode rodar depois do corte sem
/// falar com o traço em voo: os índices dos vértices ficam exatamente onde
/// estavam, que é a mesma premissa de que o `SculptStroke::grow_with` depende.
///
/// ⚠️ **Os índices de FACE também sobrevivem a uma rodada** — ela reescreve dois
/// slots do vetor e o devolve com o mesmo comprimento —, e é isso que deixa as
/// faces flipadas serem as sementes da rodada seguinte sem remapeamento.
/// **A MESMA TROCA, sobre a malha INTEIRA** — a porta que o remesh isotrópico usa.
///
/// ⚠️ **Ela existe porque o [`relax`] é por REGIÃO de propósito**: no traço, as
/// sementes são as faces que o corte mexeu, e varrer o modelo todo a cada dab
/// seria pagar o modelo para detalhar uma unha. O remesh isotrópico é o caso
/// oposto — ele é um passe global, sob comando — e derivar a lista de sementes
/// no chamador seria a mesma resposta escrita num segundo sítio.
///
/// ⚠️ **Ponto de extensão APPEND-ONLY** (`CLAUDE.md` §0.2): ela não muda o
/// [`relax`] nem o que o traço vê; ela só nomeia o caso "todas as faces".
///
/// Devolve quantas trocas aconteceram.
/// ⭐⭐⭐ **A MESMA TROCA, mas escolhida por DIRECÇÃO** — a terceira metade do
/// pente de topologia.
///
/// `preferencia` recebe a **direcção unitária** de uma diagonal e devolve quanto
/// ela é desejada (maior é melhor). A troca acontece quando a diagonal NOVA é
/// mais desejada que a ANTIGA **e** o pior ângulo do par não piora.
///
/// # ⛔⛔⛔ Porque ela existe, e porque as outras duas metades não bastavam
///
/// Partir uma aresta para alinhar CRIA arestas — e medido, toda configuração que
/// compra `Q` suficiente por corte entrega triângulos de `0,2°`–`2,1°` (sem
/// normal utilizável) ou adensa a malha até `5,5×`. **Uma troca de diagonal não
/// cria nada:** ela muda a direcção de uma aresta a contagem constante, logo não
/// paga densidade *nem* forma.
///
/// ⚠️ **A cerca da FORMA é o que a separa de uma destruidora:** `novo >= velho`
/// sobre o pior ângulo do par, sem folga. ⇒ *este passe não pode piorar um
/// triângulo*, e a segunda coluna da régua do pente é satisfeita **por
/// construção** em vez de por calibração.
///
/// ⚠️ **DIVERGÊNCIA DECLARADA:** a espec §3.1 mede que o alvo **não troca uma
/// única diagonal** — mas isso é com o operador de topologia PARADO, que é
/// exactamente o regime em que não há passe nenhum a correr. *Esta lei é NOSSA e
/// está declarada como tal*, como a isometria do controlador de topo.
///
/// ⚠️ **Ponto de extensão APPEND-ONLY:** ela não muda o [`relax`] nem o
/// [`relax_valence`], e com `preferencia = None` o caminho é o de sempre **ao
/// bit**.
pub fn alinha_arestas(
    mesh: &mut Mesh,
    center: [f32; 3],
    radius: f32,
    preferencia: &(dyn Fn([f32; 3]) -> f32 + Sync),
    scratch: &mut RegionScratch,
) -> usize {
    if radius <= 0.0 {
        return 0;
    }
    let mut faces = Vec::new();
    mesh.octree().faces_in_sphere(center, radius, &mut faces);
    if faces.is_empty() {
        return 0;
    }
    relax_com(mesh, &faces, scratch, Some(preferencia))
}

pub fn relax_valence(mesh: &mut Mesh, scratch: &mut RegionScratch) -> usize {
    let all: Vec<u32> = (0..mesh.face_count() as u32).collect();
    relax(mesh, &all, scratch)
}

pub(crate) fn relax(mesh: &mut Mesh, seeds: &[u32], scratch: &mut RegionScratch) -> usize {
    relax_com(mesh, seeds, scratch, None)
}

/// O miolo dos dois: `preferencia = None` é o critério de sempre, **ao bit**.
fn relax_com(
    mesh: &mut Mesh,
    seeds: &[u32],
    scratch: &mut RegionScratch,
    preferencia: Option<&(dyn Fn([f32; 3]) -> f32 + Sync)>,
) -> usize {
    let mut total = 0;
    let mut work: Vec<u32> = seeds.to_vec();
    for _ in 0..MAX_ROUNDS {
        if work.is_empty() {
            break;
        }
        let (n, next) = one_round(mesh, &work, scratch, preferencia);
        total += n;
        if n == 0 {
            break;
        }
        work = next;
    }
    total
}

/// Uma rodada. Devolve quantas trocas aconteceram e as faces que mudaram.
fn one_round(
    mesh: &mut Mesh,
    seeds: &[u32],
    scratch: &mut RegionScratch,
    preferencia: Option<&(dyn Fn([f32; 3]) -> f32 + Sync)>,
) -> (usize, Vec<u32>) {
    let mut changes: Vec<(usize, Face)> = Vec::new();
    let mut next: Vec<u32> = Vec::new();
    {
        let src = mesh.faces();
        let adj = mesh.adjacency();
        let pos = mesh.positions();
        // Uma face só pode entrar numa troca por rodada: as duas trocas leriam a
        // mesma face pela versão ANTIGA e a segunda escreveria por cima da
        // primeira. ⚠️ E é ele que torna seguro procurar a face vizinha na
        // adjacência de ENTRADA: uma face já trocada some do anel que a lista
        // descreve, e o guard a recusa antes de alguém ler o par errado.
        let mut spent = vec![false; src.len()];
        // ⭐ **As diagonais que ESTA rodada já criou** — a recusa 4, e ela é a
        // única das quatro que não se responde à adjacência de entrada. Ver o
        // cabeçalho do módulo.
        //
        // ⚠️ `BTreeSet` e não `HashSet`: o conjunto não decide *o quê*, mas
        // percorrê-lo em ordem de inserção arbitrária tornaria um dia um
        // diagnóstico irreprodutível — e esta crate é a espinha do determinismo
        // (`CLAUDE.md` §5.1, Física).
        let mut made: BTreeSet<(u32, u32)> = BTreeSet::new();
        for &f in seeds {
            let i0 = f as usize;
            if i0 >= src.len() || !src[i0].is_tri() {
                continue;
            }
            for k in 0..3 {
                if spent[i0] {
                    break;
                }
                let v0 = src[i0].verts();
                let (ea, eb) = (v0[k], v0[(k + 1) % 3]);
                let Some(f1) = other_face(adj, src, f, ea, eb) else {
                    continue;
                };
                let i1 = f1 as usize;
                if spent[i1] || !src[i1].is_tri() {
                    continue;
                }
                let Some((a, b, c, d)) = quad(src, i0, i1) else {
                    continue;
                };
                // A diagonal nova já existe em outro lugar da malha? Ver a
                // recusa 2 — e a pergunta é o anel de `c`, não um id de aresta.
                if adj.vert_verts.neighbours(c as usize).contains(&d) {
                    continue;
                }
                let p = |v: u32| pos[v as usize];
                let old = worst_angle(p(a), p(b), p(c)).min(worst_angle(p(b), p(a), p(d)));
                let new = worst_angle(p(a), p(d), p(c)).min(worst_angle(p(d), p(b), p(c)));
                match preferencia {
                    // O critério de sempre: só troca quem melhora a FORMA.
                    None => {
                        if new <= old + MIN_GAIN {
                            continue;
                        }
                    }
                    // ⭐⭐⭐ **Por DIRECÇÃO, com a forma como CERCA.** A diagonal
                    // antiga é `a—b` (a aresta partilhada) e a nova é `c—d`.
                    // ⛔ A cerca `new >= old` é sem folga de propósito: *este
                    // passe não pode piorar um triângulo*, e é isso que o torna
                    // grátis nas duas colunas da régua do pente.
                    Some(pref) => {
                        let Some(antiga) = unitaria(p(a), p(b)) else {
                            continue;
                        };
                        let Some(nova) = unitaria(p(c), p(d)) else {
                            continue;
                        };
                        if pref(nova) <= pref(antiga) + GANHO_DO_ALINHAMENTO
                            || new < old.min(CHAO_DO_ALINHAMENTO)
                        {
                            continue;
                        }
                    }
                }
                if folds(p(a), p(b), p(c), p(d)) {
                    continue;
                }
                // ⭐ **Recusa 4 — e ela é INVISÍVEL para a adjacência de entrada.**
                // Duas trocas desta mesma rodada, sobre pares de faces DISJUNTOS
                // (logo fora do alcance do `spent`), podem produzir a MESMA
                // diagonal `c—d`. Nenhuma das duas a vê no anel de `c`, porque
                // esse anel é o de ANTES da rodada; a malha sairia com duas
                // arestas entre o mesmo par — **quatro faces numa aresta** — e
                // deixaria de ser variedade.
                //
                // ⚠️ **A reserva é feita AQUI e não junto da recusa 2**, e a
                // diferença é medível: reservar antes das guardas de ângulo e de
                // dobra faria um candidato REJEITADO bloquear uma troca válida
                // que viesse depois com a mesma diagonal.
                if !made.insert((c.min(d), c.max(d))) {
                    continue;
                }
                changes.push((i0, Face::tri(a, d, c)));
                changes.push((i1, Face::tri(d, b, c)));
                spent[i0] = true;
                spent[i1] = true;
                next.push(f);
                next.push(f1);
            }
        }
    }

    if changes.is_empty() {
        return (0, Vec::new());
    }
    let flips = changes.len() / 2;
    // ⚠️ **Uma troca de diagonal é uma EDIÇÃO, não uma malha nova.** O caminho
    // antigo montava a lista inteira de faces e chamava um `Mesh::rebuild` —
    // `O(malha)` medido em **10,4 ms a 98k** para reescrever duas faces. Aqui a
    // malha recebe as duas e refresca a região; nenhum vértice é criado nem
    // movido, então a partição do octree e a contagem não mudam.
    let edits: Vec<(u32, Face)> = changes
        .into_iter()
        .map(|(i, f)| (u32::try_from(i).unwrap_or(u32::MAX), f))
        .collect();
    mesh.relink_faces(&edits, scratch);
    (flips, next)
}

/// A outra face que divide a aresta `(a, b)` com `fi` — `None` se for borda.
///
/// ⚠️ **Numa aresta não-manifold (três ou mais faces) ela devolve a primeira que
/// achar**, o que é tão arbitrário quanto os dois primeiros slots que a versão
/// por tabela guardava. A malha que o refino aceita é de triângulos, e uma
/// aresta com três faces já é malformada na entrada.
fn other_face(adj: &Adjacency, faces: &[Face], fi: u32, a: u32, b: u32) -> Option<u32> {
    adj.vert_faces
        .neighbours(a as usize)
        .iter()
        .copied()
        .find(|&fj| fj != fi && faces[fj as usize].verts().contains(&b))
}

/// Os quatro cantos do quadrilátero que duas faces vizinhas formam:
/// `(a, b, c, d)` com a diagonal atual em `a—b`, `c` oposto na primeira face e
/// `d` na segunda. O contorno é `a → d → b → c`.
fn quad(faces: &[Face], i0: usize, i1: usize) -> Option<(u32, u32, u32, u32)> {
    let (t0, t1) = (faces[i0].verts(), faces[i1].verts());
    // O canto de `t0` que não está em `t1` é o `c`; os outros dois são a aresta.
    let k = (0..3).find(|&k| !t1.contains(&t0[k]))?;
    let c = t0[k];
    let (a, b) = (t0[(k + 1) % 3], t0[(k + 2) % 3]);
    // ⚠️ A ordem importa: em `t0` o percurso é `c → a → b`, então a aresta
    // compartilhada corre `a → b` aqui e `b → a` na vizinha, e é isso que dá o
    // contorno `a → d → b → c` do quadrilátero.
    let d = *t1.iter().find(|v| **v != a && **v != b)?;
    Some((a, b, c, d))
}

/// O menor ângulo do triângulo, em COSSENO invertido — devolvemos o cosseno do
/// maior ângulo negado para comparar sem `acos`, que é transcendental e não
/// muda a ordem. Maior valor = triângulo melhor.
fn worst_angle(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> f32 {
    let pts = [p0, p1, p2];
    let mut worst = f32::INFINITY;
    for k in 0..3 {
        let (o, u, v) = (pts[k], pts[(k + 1) % 3], pts[(k + 2) % 3]);
        let a = sub(u, o);
        let b = sub(v, o);
        let (la, lb) = (norm(a), norm(b));
        if la < 1e-12 || lb < 1e-12 {
            return -1.0;
        }
        // O ângulo cresce quando o cosseno cai, então o PIOR canto é o de menor
        // `-cos` — é a mesma ordem de `min(ângulo)`, sem chamar `acos`.
        worst = worst.min(-(dot(a, b) / (la * lb)));
    }
    worst
}

/// A troca dobraria a superfície? Ver a recusa 3.
fn folds(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> bool {
    let before = add(tri_normal(a, b, c), tri_normal(b, a, d));
    let (n0, n1) = (tri_normal(a, d, c), tri_normal(d, b, c));
    dot(n0, before) <= 0.0 || dot(n1, before) <= 0.0
}

fn tri_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    cross(sub(p1, p0), sub(p2, p0))
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

/// A direcção unitária de `a` para `b`, ou `None` se a aresta é degenerada.
///
/// ⚠️ **O sentido sai do par como ele vem**, e isso é seguro porque a lei que o
/// lê é PAR (ver [`crate::Sizing`]): uma preferência ímpar faria a troca depender
/// de qual face propôs o quad primeiro.
fn unitaria(a: [f32; 3], b: [f32; 3]) -> Option<[f32; 3]> {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    (l > 0.0).then(|| [d[0] / l, d[1] / l, d[2] / l])
}
