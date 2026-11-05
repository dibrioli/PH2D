//! Binário só com o FunDSP: o `nm` dele diz que matemática do sistema ele chama.
fn main() {
    println!("fundsp_tick {:016x}", sonda_w0::hash_fundsp_tick());
    println!("fundsp_bloco {:016x}", sonda_w0::hash_fundsp_bloco());
}
