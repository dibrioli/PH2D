//! Binário só com os kernels: o `nm` dele diz que matemática do sistema eles chamam.
fn main() {
    println!("kernels {:016x}", sonda_w0::hash_kernels());
}
