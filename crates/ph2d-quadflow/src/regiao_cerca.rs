//! ⭐⭐⭐⭐ **A CERCA DA FORMA da retícula** — o que impede o passe de afinar um
//! triângulo, e a prova de que a pergunta tem de ser feita com **todos** os
//! movimentos aplicados.
//!
//! ⚠️ Ele saiu da [`crate::regiao`] por **TECTO DE LOC** (`746` contra `700`),
//! e o corte é por responsabilidade: ali fica quem decide PARA ONDE, aqui quem
//! decide SE.

use ph2d_mesh::Mesh;

use super::norm;

/// Quantas rondas de veto.
///
/// ⭐ **O laço só REMOVE movimentos, logo ele TERMINA sozinho** — o tecto não é
/// uma condição de paragem, é uma guarda de **CUSTO** contra uma pegada
/// patológica (cada ronda é `O(faces da pegada)`).
///
/// ⚠️ **Medido no traço da cena `=49`, contando quantas rondas de facto
/// disparam:** a ronda `0` veta em `84` dos dabs, a `1` em **um**, e a `2` em
/// **nenhum**. ⇒ `3` é uma de folga sobre o máximo observado, e a segunda ronda
/// existe porque **tirar um movimento muda os triângulos vizinhos** — ela é
/// rara, não é decorativa.
pub(super) const RONDAS_DO_VETO: usize = 3;

/// ⭐⭐⭐⭐ **A CERCA DA FORMA, JULGADA COM TODOS OS MOVIMENTOS APLICADOS.**
///
/// # ⛔⛔⛔⛔ A cerca por-vértice não era FRACA — era ERRADA, e ela FABRICAVA as
/// lascas que existia para impedir
///
/// A antiga perguntava *«pôr ESTE vértice aqui afina um triângulo do anel
/// dele?»* e vetava-o sozinho. ⚠️ **Mas o laço é Jacobi e os alvos são
/// mutuamente CONSISTENTES** — são todos nós da MESMA grade. Vetar um
/// subconjunto deixa a malha **meio-movida**, que não é a entrada nem o alvo:
/// *uma configuração que nenhum dos dois lados tem*.
///
/// **Medido pela porta do produto** (média dos quatro rumos, `4` rondas, knob
/// no topo), e a tabela é o diagnóstico inteiro:
///
/// | cerca | lascas | pior ângulo | portão da cena |
/// |---|---|---|---|
/// | **por-vértice** (a antiga) | **`4`** | `3,81°` | ⛔ **VERMELHO** |
/// | nenhuma | `0` | `19,58°` | ✅ |
/// | **combinada** (esta) | `0` | `5,07°` | ✅ |
///
/// ⇒ *o que bloqueava o degrau seguinte das varreduras não era o número, era a
/// cerca* — e a dívida que 20/09 escreveu (*«a cura é uma cerca que veja
/// PARES»*) apontava para o sítio certo pela razão errada.
///
/// # ⚠️⚠️ E ela é um GUARDA: no corpus do produto é INDISTINGUÍVEL de não ter
/// cerca nenhuma
///
/// A tabela acima diz isso à letra, e a prova de mutação diz o mesmo: **apagar
/// a chamada do caminho do produto deixa o portão da cena VERDE**. Ela fica —
/// e a razão não é conforto:
///
/// - sem cerca, um movimento de retícula é **incondicional**: o vértice vai
///   onde a grade manda, seja o que for que isso faça ao triângulo. Numa esfera
///   lisa com dyntopo uniforme isso é inofensivo; numa peça **esculpida** — com
///   vincos, pontas e paredes finas — é exactamente o que esta linha já pagou
///   meia dúzia de vezes;
/// - e ela é a **única** que pode recusar uma conspiração real, o que está
///   gateado em fixtura construída para isso
///   (`dois_vizinhos_nao_conspiram_numa_lasca`), porque o corpus do produto
///   **não contém o caso**.
///
/// ⚠️ **Custo medido: `4,172 → 4,180 ms`** a `4` rondas na maior peça (`0,2 %`)
/// — ela é `O(faces da pegada)` × [`RONDAS_DO_VETO`], ao lado de dois campos
/// suavizados.
///
/// # Como
///
/// Decide-se tudo (o laço acima), e só então cada triângulo do anel de um
/// movido é medido **duas** vezes: com as posições de entrada e com **todos** os
/// destinos aprovados. Se a segunda for pior **e** abaixo do chão, **vetam-se
/// todos os vértices movidos daquele triângulo** — e repete-se, porque tirar um
/// movimento muda os triângulos vizinhos.
///
/// # As DUAS metades da lei
///
/// ⚠️ **É `pior && abaixo do chão`, nunca só uma das duas.** Só *«pior»*
/// congelaria a malha (a retícula reforma triângulos de propósito); só *«abaixo
/// do chão»* prenderia para sempre um vértice cujo anel **já nasceu** com uma
/// lasca — e essa é a metade que o produto encontra numa peça **esculpida**, e
/// que uma **mutação sobreviveu** a matar no portão da cena (uma esfera lisa não
/// tem lascas prévias). Gate: `uma_lasca_que_ja_la_estava_nao_prende_o_vertice`.
///
/// ⚠️ **Sem transcendental:** o menor ângulo é o de maior COSSENO, e *abaixo de
/// `θ`* é *cosseno acima de `cos θ`*. Materializar o ângulo seria pagar um
/// `acos` por canto para responder o que a comparação já responde.
///
/// ⚠️ **Vetar o triângulo INTEIRO e não «o pior dos dois» é o que torna isto
/// determinístico:** escolher um culpado entre dois precisaria de um desempate,
/// e um desempate por índice faria a saída depender da numeração da malha — o
/// defeito que o Jacobi existe para não ter. Ele é conservador de propósito.
///
/// ⚠️ **O conjunto vetado em cada ronda não depende da ORDEM** (só se lê, e
/// escreve-se no fim), e o laço **só remove** ⇒ termina.
pub(super) fn veta_combinado(mesh: &Mesh, aprovados: Vec<(u32, [f32; 3])>) -> Vec<(u32, [f32; 3])> {
    if aprovados.is_empty() {
        return aprovados;
    }
    let p = mesh.positions();
    let faces = mesh.faces();
    let adj = mesh.adjacency();

    let mut destino: std::collections::BTreeMap<u32, [f32; 3]> =
        aprovados.iter().copied().collect();
    let mut a_ver: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for (v, _) in &aprovados {
        for &f in adj.vert_faces.neighbours(*v as usize) {
            a_ver.insert(f);
        }
    }

    for _ in 0..RONDAS_DO_VETO {
        let mut vetados: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
        for &f in &a_ver {
            let face = faces[f as usize];
            for t in 0..face.tri_count() {
                let tri = face.tri_at(t);
                if !tri.iter().any(|i| destino.contains_key(i)) {
                    continue;
                }
                let antes =
                    maior_cosseno([p[tri[0] as usize], p[tri[1] as usize], p[tri[2] as usize]]);
                let leia = |i: u32| destino.get(&i).copied().unwrap_or(p[i as usize]);
                let depois = maior_cosseno([leia(tri[0]), leia(tri[1]), leia(tri[2])]);
                if depois > antes && depois > CHAO_DA_LASCA {
                    for i in tri {
                        if destino.contains_key(&i) {
                            vetados.insert(i);
                        }
                    }
                }
            }
        }
        if vetados.is_empty() {
            break;
        }
        for v in &vetados {
            destino.remove(v);
        }
    }

    aprovados
        .into_iter()
        .filter(|(v, _)| destino.contains_key(v))
        .collect()
}

/// `cos(5°)` — o chão da lasca.
///
/// ⚠️ **O número é o do gate da cena** (`LIMIAR_DA_LASCA`, `5°`), e não um valor
/// escolhido aqui: é ali que o dono julga o resultado, e duas respostas à
/// pergunta *«isto é uma lasca?»* divergiriam no dia em que uma delas mudasse.
pub(super) const CHAO_DA_LASCA: f32 = 0.996_194_7;

/// O **maior cosseno** dos três cantos — ou seja, o cosseno do MENOR ângulo.
pub(super) fn maior_cosseno(t: [[f32; 3]; 3]) -> f32 {
    let mut pior = -1.0f32;
    for k in 0..3 {
        let (a, b, c) = (t[k], t[(k + 1) % 3], t[(k + 2) % 3]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let (lu, lw) = (norm(u), norm(w));
        if lu <= 0.0 || lw <= 0.0 {
            // Uma aresta de comprimento zero não tem canto: ela é a própria
            // degenerescência, e devolver `1` (ângulo nulo) é dizê-lo.
            return 1.0;
        }
        let c = u[0].mul_add(w[0], u[1].mul_add(w[1], u[2] * w[2])) / (lu * lw);
        pior = pior.max(c.clamp(-1.0, 1.0));
    }
    pior
}
