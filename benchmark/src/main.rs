use redis::RedisClient;
use rstore::RStoreClient;

pub mod redis;
pub mod rstore;

pub trait KeyValueStore {
    fn set_key_value(&mut self, key: &str, value: &str) -> anyhow::Result<()>;
    fn get_key_value(&mut self, key: &str) -> anyhow::Result<String>;
    fn clear_all(&mut self) -> anyhow::Result<()>;
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
// Case 2. 여러개의 스레드로 동시 요청을 보낼 때, 최대 레이턴시와 평균 레이턴시, 완료되기까지의 총 소요 시간을 측정합니다.

const CASE_1_KEY_COUNT: usize = 1_000_000;

fn benchmark_redis() {
    let mut connection = RedisClient::new().unwrap();

    connection.clear_all().unwrap();

    // Case 1
    {
        // 1. Set 1 million keys
        let timer = Timer::new();

        for i in 0..CASE_1_KEY_COUNT {
            let key = format!("key{}", i);
            let value = format!("value{}", i);

            connection.set_key_value(&key, &value).unwrap();
        }

        let total_elapsed = timer.elapsed();

        println!("Total elapsed time for 1 million keys: {:?}", total_elapsed);
    }

    // Case 2
    {
        // 2. Get 1 million keys
        let timer = Timer::new();

        for i in 0..CASE_1_KEY_COUNT {
            let key = format!("key{}", i);

            let value = connection.get_key_value(&key).unwrap();
            assert_eq!(value, format!("value{}", i));
        }

        let total_elapsed = timer.elapsed();

        println!(
            "Total elapsed time for getting 1 million keys: {:?}",
            total_elapsed
        );
    }
}

fn benchmark_rstore() {
    let mut client = RStoreClient::new().unwrap();

    client.clear_all().unwrap();
}

fn main() {
    println!("------------------------------");
    println!("Benchmarking Redis...");
    benchmark_redis();
    println!("Benchmarking Redis completed.");
    println!("------------------------------");

    println!("");
    println!("");

    println!("------------------------------");
    println!("Benchmarking RStore...");
    benchmark_rstore();
    println!("Benchmarking RStore completed.");
    println!("------------------------------");
}
