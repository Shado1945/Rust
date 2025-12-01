use tokio::time::{Duration, sleep};

#[derive(Debug)]
pub struct FirstFile {
    pub sum: i32,
    pub dif: i32,
    pub multi: i32,
}

impl FirstFile {
    pub async fn batching(a: i32, b: i32) -> FirstFile {
        let (sum, dif, multi) = tokio::join!(Self::add(a, b), Self::dif(a, b), Self::multi(a, b));

        FirstFile { sum, dif, multi }
    }

    pub async fn add(a: i32, b: i32) -> i32 {
        sleep(Duration::from_millis(150)).await;
        a + b
    }

    pub async fn dif(a: i32, b: i32) -> i32 {
        sleep(Duration::from_millis(50)).await;
        a - b
    }

    pub async fn multi(a: i32, b: i32) -> i32 {
        sleep(Duration::from_millis(130)).await;
        a * b
    }
}
