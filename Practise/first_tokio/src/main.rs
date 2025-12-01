use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    println!("Hello from async!");

    let task1 = tokio::spawn(async {
        sleep(Duration::from_secs(1)).await;
        println!("Task 1 Completed");
    });

    let task2 = tokio::spawn(async {
        sleep(Duration::from_millis(500)).await;
        println!("Task 2 Completed");
    });

    let _res = tokio::join!(task1, task2);
    println!("All tasks done!");
}
