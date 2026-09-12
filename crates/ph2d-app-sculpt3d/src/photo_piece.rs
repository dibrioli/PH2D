//! ⭐⭐⭐ **A ASSINATURA DA PEÇA** — irmã de [`super::rulers`] pelo teto de LOC da shell
//! (HR-18, 600), cortada por RESPONSABILIDADE: aquelas réguas perguntam *«esta malha é
//! boa?»*, e esta responde à pergunta ANTERIOR — *«estas duas malhas são a mesma peça?»*.

/// ⭐⭐⭐ **QUE PEÇA É ESTA?** — a assinatura que diz se dois ficheiros são o MESMO objecto.
///
/// # ⛔⛔⛔ Ela existe porque uma tabela desta linha comparou TRÊS peças como se fossem uma
///
/// O §8-octodecies do handoff de 30/08 pôs numa coluna só a entrada, duas retopologias de
/// referência e a nossa saída — e **três delas eram peças diferentes**. Daí saiu a
/// conclusão *«a entrada dele já tem a densidade certa e o botão deita-a fora»*, que é uma
/// afirmação sobre um par que nunca existiu.
///
/// ⚠️ **É a irmã exacta do ACHADO de 28/08** (a barra do oráculo lida a `1/9` da densidade
/// dele): ali a régua omitia a **contagem de faces** dos dois lados, aqui omitia a **peça**.
/// *Uma comparação entre A e B tem de dizer que A e B são comparáveis, e nenhuma régua deste
/// repo o dizia.*
///
/// # ⭐ Porquê ÁREA e VOLUME, e não a caixa
///
/// A caixa envolvente muda com a **rotação**, e um exportador que troca eixos devolve outra
/// caixa para a mesma peça — foi assim que os ficheiros do dono se leram como objectos
/// distintos e o mesmo objecto se leu como distinto de si. ⭐ **Área e volume são invariantes
/// a todo movimento rígido**, logo duas retopologias da mesma escultura concordam nelas a
/// poucos por cento, e duas esculturas diferentes não.
///
/// ⚠️ **O volume é assinado** (teorema da divergência sobre um leque de triângulos): numa
/// malha fechada e bem orientada ele é o volume real; numa aberta ele é lixo, e é por isso
/// que a linha do [`census`] — que diz se há bordo — corre **ao lado desta**.
pub(super) fn piece_signature(tag: &str, mesh: &ph2d_mesh::Mesh) {
    let pos = mesh.positions();
    let (mut area, mut vol) = (0.0f64, 0.0f64);
    for f in mesh.faces() {
        let v = f.verts();
        // ⚠️ **Leque a partir do vértice 0** — um quad não-planar dá dois valores conforme a
        // diagonal escolhida, e a diferença é de 2.ª ordem no desvio. Ela entra IGUAL nas
        // duas malhas comparadas, então não move a decisão «mesma peça?».
        for k in 1..v.len().saturating_sub(1) {
            let (a, b, c) = (
                pos[v[0] as usize],
                pos[v[k] as usize],
                pos[v[k + 1] as usize],
            );
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [
                u[1].mul_add(w[2], -(u[2] * w[1])),
                u[2].mul_add(w[0], -(u[0] * w[2])),
                u[0].mul_add(w[1], -(u[1] * w[0])),
            ];
            area += 0.5 * f64::from(n[0].mul_add(n[0], n[1].mul_add(n[1], n[2] * n[2])).sqrt());
            vol += f64::from(a[0].mul_add(n[0], a[1].mul_add(n[1], a[2] * n[2]))) / 6.0;
        }
    }
    // O alcance, para a linha ficar legível ao lado das sondas que já o imprimem.
    let n = pos.len().max(1) as f32;
    let mut c = [0.0f32; 3];
    for q in pos {
        for k in 0..3 {
            c[k] += q[k] / n;
        }
    }
    let reach = pos.iter().fold(0.0f32, |acc, q| {
        let d = [q[0] - c[0], q[1] - c[1], q[2] - c[2]];
        acc.max(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt())
    });
    eprintln!(
        "   {tag}: PECA area {area:.4} | volume {:.4} | alcance {reach:.4}  \
         (duas retopologias da MESMA escultura concordam a poucos %)",
        vol.abs(),
    );
}
