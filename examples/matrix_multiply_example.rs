use template::matrix::{multiply, Matrix};
// cargo run --example matrix_multiply_example
fn main() {
    // let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
    // let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
    let a = Matrix::new([1, 2, 3, 4], 2, 2);
    let b = Matrix::new([1, 2, 3, 4], 2, 2);
    let c = multiply(&a, &b);
    println!("{c}")
}
