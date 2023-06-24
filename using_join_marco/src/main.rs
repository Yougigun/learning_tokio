use tokio::join;
#[tokio::main]
async fn main() {
    join!(
        async {
            println!("Hello");
        },
        async {
            // sleep 10s
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            println!("World");
        }
    );
}
