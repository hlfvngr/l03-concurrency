use std::{
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Mul},
    sync::mpsc::{self, Sender},
    thread,
};

const NUM_THREADS: usize = 4;
// [1,2, 3,4, 5,6]
pub struct Matrix<T> {
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

impl<T> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>, rows: usize, cols: usize) -> Self {
        Matrix {
            data: data.into(),
            rows,
            cols,
        }
    }
}

pub struct Msg<T> {
    input: MsgInput<T>,
    sender: oneshot::Sender<MsgOutput<T>>,
}

impl<T> Msg<T> {
    pub fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Msg { input, sender }
    }
}

pub struct MsgInput<T> {
    idx: usize,
    a: Vec<T>,
    b: Vec<T>,
}

impl<T> MsgInput<T> {
    pub fn new(idx: usize, a: Vec<T>, b: Vec<T>) -> Self {
        MsgInput { idx, a, b }
    }
}

pub struct MsgOutput<T> {
    idx: usize,
    result: T,
}

impl<T> MsgOutput<T> {
    pub fn new(idx: usize, result: T) -> Self {
        MsgOutput { idx, result }
    }
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Matrix<T>
where
    T: Copy + Mul<Output = T> + AddAssign + Debug + Default,
{
    assert_eq!(a.rows, b.cols);

    let mut data = vec![T::default(); a.rows * b.cols];
    for i in 0..a.rows {
        for j in 0..b.cols {
            for k in 0..a.cols {
                data[i * a.rows + j] += a.data[i * a.cols + k] * b.data[k * b.cols + j];
            }
        }
    }
    Matrix::new(data, a.rows, b.cols)
}

pub fn dot_product<T>(a: &Vec<T>, b: &Vec<T>) -> T
where
    T: Copy + Mul<Output = T> + Add<Output = T> + Debug + Default,
{
    assert_eq!(a.len(), b.len());

    let sum = a
        .iter()
        .zip(b.iter())
        .fold(T::default(), |acc, (m, n)| acc + *m * *n);

    sum
}

pub fn multiply_02<T>(a: &Matrix<T>, b: &Matrix<T>) -> Matrix<T>
where
    T: Copy + Mul<Output = T> + AddAssign + Debug + Default + Add<Output = T>,
{
    assert_eq!(a.rows, b.cols);

    let mut data = vec![T::default(); a.rows * b.cols];
    for i in 0..a.rows {
        for j in 0..b.cols {
            let m = a.data[i * a.cols..(i + 1) * a.cols].to_vec();
            let n = b.data[j..]
                .iter()
                .step_by(b.cols)
                .map(|x| x.clone())
                .collect::<Vec<T>>();
            data[i * a.rows + j] = dot_product(&m, &n);
        }
    }
    Matrix::new(data, a.rows, b.cols)
}

pub fn multiply_multi<T>(a: &Matrix<T>, b: &Matrix<T>) -> Matrix<T>
where
    T: Copy + Mul<Output = T> + AddAssign + Debug + Default + Add<Output = T> + Send + 'static,
{
    assert_eq!(a.rows, b.cols);

    let senders = (0..NUM_THREADS)
        .into_iter()
        .map(|_| {
            let (tx, rx) = mpsc::channel::<Msg<T>>();
            thread::spawn(move || {
                for msg in rx {
                    let res = dot_product(&msg.input.a, &msg.input.b);
                    msg.sender
                        .send(MsgOutput::new(msg.input.idx, res))
                        .expect("return result message fail!");
                }
            });
            tx
        })
        .collect::<Vec<Sender<Msg<T>>>>();

    let mut receivers = Vec::new();
    let mut data = vec![T::default(); a.rows * b.cols];
    for i in 0..a.rows {
        for j in 0..b.cols {
            let m = a.data[i * a.cols..(i + 1) * a.cols].to_vec();
            let n = b.data[j..]
                .iter()
                .step_by(b.cols)
                .map(|x| x.clone())
                .collect::<Vec<T>>();

            let idx = i * a.rows + j;

            let (tx, rx) = oneshot::channel();
            let msg_input = MsgInput::new(idx, m, n);
            let msg = Msg::new(msg_input, tx);

            senders[idx % NUM_THREADS]
                .send(msg)
                .expect("send calculation message fail!");

            receivers.push(rx);
        }

        for r in receivers.iter() {
            if let Ok(a) = r.recv_ref() {
                data[a.idx] = a.result;
            }
        }
    }
    Matrix::new(data, a.rows, b.cols)
}

impl<T> Display for Matrix<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::from("{{");
        for i in 0..self.rows {
            for j in 0..self.cols {
                res.push_str(format!("{}", self.data[i * self.rows + j]).as_str());
                res.push_str(" ");
            }
            let _ = res.split_off(res.len() - 1);
            res.push_str(", ");
        }
        let _ = res.split_off(res.len() - 2);
        res.push_str("}}");
        f.write_str(res.as_str())
    }
}

impl<T> Debug for Matrix<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}", self).as_str())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn matrix_multiply() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
        let c = multiply(&a, &b);
        println!("{c}")
    }

    #[test]
    fn matrix_multiply_multi() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
        let c = multiply_multi(&a, &b);
        println!("{c}")
    }
}
