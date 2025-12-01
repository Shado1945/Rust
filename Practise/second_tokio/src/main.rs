mod first_file;

use crate::first_file::FirstFile;

#[tokio::main]
async fn main() {
    let result = FirstFile::batching(10, 5).await;
    println!("Add: {}", result.sum);
    println!("Subtract: {}", result.dif);
    println!("Multiply: {}", result.multi);
}
