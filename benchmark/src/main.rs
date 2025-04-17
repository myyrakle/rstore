use std::thread;

use redis::RedisClient;

pub mod redis;
pub mod rstore_http;
pub mod rstore_tcp;

#[async_trait::async_trait]
pub trait KeyValueStore {
    async fn set_key_value(&mut self, key: &str, value: &str) -> anyhow::Result<()>;
    async fn get_key_value(&mut self, key: &str) -> anyhow::Result<String>;
    async fn clear_all(&mut self) -> anyhow::Result<()>;
}

pub struct Timer {
    start: std::time::Instant,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            start: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

// Case 1. 100만개의 Key를 무작위로 생성할 때, 최대 레이턴시와 평균 레이턴시를 측정합니다.
const CASE_1_KEY_COUNT: usize = 1_000_000;

async fn case_1(client: &mut impl KeyValueStore) {
    // 1. Set 1 million keys

    let mut total_elapsed = 0_u128;
    let total_count = CASE_1_KEY_COUNT as u128;
    let mut min_elapsed = u128::MAX;
    let mut max_elapsed = 0_u128;

    for i in 0..CASE_1_KEY_COUNT {
        let key = format!("key{}", i);
        let value = format!("value{}", i);

        let timer = Timer::new();
        client.set_key_value(&key, &value).await.unwrap();
        let elapsed = timer.elapsed().as_micros();

        if elapsed < min_elapsed {
            min_elapsed = elapsed;
        }
        if elapsed > max_elapsed {
            max_elapsed = elapsed;
        }
        total_elapsed += elapsed;
    }

    let avg_elapsed = total_elapsed / total_count;
    println!("AVERAGE ELAPSED TIME: {} microseconds", avg_elapsed);
    println!("MIN ELAPSED TIME: {} microseconds", min_elapsed);
    println!("MAX ELAPSED TIME: {} microseconds", max_elapsed);
}

// Case 2. 여러개의 스레드로 동시 요청을 보낼 때, 최대 레이턴시와 평균 레이턴시, 완료되기까지의 총 소요 시간을 측정합니다.

const CASE_2_THREAD_COUNT: usize = 10;
const CASE_2_KEY_COUNT_PER_THREAD: usize = 100_000;

async fn case_2<T>(client: &mut T)
where
    T: KeyValueStore + Clone + Send + Sync + 'static,
{
    let mut total_elapsed = 0_u128;
    let total_count = CASE_2_THREAD_COUNT as u128;
    let mut min_elapsed = u128::MAX;
    let mut max_elapsed = 0_u128;

    let handles: Vec<_> = (0..CASE_2_THREAD_COUNT)
        .map(|i| {
            let mut client = client.clone();
            thread::spawn(async move || {
                let mut total_elapsed = 0_u128;
                let total_count = CASE_2_KEY_COUNT_PER_THREAD as u128;
                let mut min_elapsed = u128::MAX;
                let mut max_elapsed = 0_u128;

                for j in 0..CASE_2_KEY_COUNT_PER_THREAD {
                    let key = format!("key{}-{}", i, j);
                    let value = format!("value{}-{}", i, j);

                    let timer = Timer::new();
                    client.set_key_value(&key, &value).await.unwrap();
                    let elapsed = timer.elapsed().as_micros();

                    if elapsed < min_elapsed {
                        min_elapsed = elapsed;
                    }
                    if elapsed > max_elapsed {
                        max_elapsed = elapsed;
                    }
                    total_elapsed += elapsed;
                }

                total_elapsed / total_count
            })
        })
        .collect();

    for handle in handles {
        let elapsed = handle.join().unwrap().await;

        if elapsed < min_elapsed {
            min_elapsed = elapsed;
        }
        if elapsed > max_elapsed {
            max_elapsed = elapsed;
        }
        total_elapsed += elapsed;
    }

    let avg_elapsed = total_elapsed / total_count;
    println!("AVERAGE ELAPSED TIME: {} microseconds", avg_elapsed);
    println!("MIN ELAPSED TIME: {} microseconds", min_elapsed);
    println!("MAX ELAPSED TIME: {} microseconds", max_elapsed);
}

async fn benchmark_redis() {
    let mut client = RedisClient::new().unwrap();

    client.clear_all().await.unwrap();
    thread::sleep(std::time::Duration::from_secs(1));

    // Case 1
    {
        case_1(&mut client).await;
    }

    thread::sleep(std::time::Duration::from_secs(1));
    client.clear_all().await.unwrap();

    // Case 2
    {
        case_2(&mut client).await;
    }
}

async fn benchmark_rstore_http() {
    let mut client = rstore_http::RStoreClient::new().unwrap();

    client.clear_all().await.unwrap();
    thread::sleep(std::time::Duration::from_secs(1));

    // Case 1
    {
        case_1(&mut client).await;
    }

    thread::sleep(std::time::Duration::from_secs(1));
    client.clear_all().await.unwrap();

    // Case 2
    {
        case_2(&mut client).await;
    }
}

async fn benchmark_rstore_tcp() {
    let mut client = rstore_tcp::RStoreClient::new().unwrap();

    client.clear_all().await.unwrap();
    thread::sleep(std::time::Duration::from_secs(1));

    // Case 1
    {
        case_1(&mut client).await;
    }

    thread::sleep(std::time::Duration::from_secs(1));
    client.clear_all().await.unwrap();

    // Case 2
    {
        case_2(&mut client).await;
    }
}

#[tokio::main]
async fn main() {
    println!("------------------------------");
    println!("Benchmarking Redis...");
    benchmark_redis().await;
    println!("Benchmarking Redis completed.");
    println!("------------------------------");

    println!("");
    println!("");

    println!("------------------------------");
    println!("Benchmarking RStore HTTP...");
    benchmark_rstore_http().await;
    println!("Benchmarking RStore HTTP completed.");
    println!("------------------------------");

    println!("");
    println!("");

    println!("------------------------------");
    println!("Benchmarking RStore TCP...");
    benchmark_rstore_tcp().await;
    println!("Benchmarking RStore TCP completed.");
    println!("------------------------------");

    println!("");
    println!("All benchmarks completed.");
}
