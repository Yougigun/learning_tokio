#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
use bytes::Bytes;
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::ptr::hash;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

type DB = Arc<Mutex<HashMap<String, Bytes>>>;

// 当同步锁的竞争变成一个问题时，使用 Tokio 提供的异步锁几乎并不能帮你解决问题，此时可以考虑如下选项：
// 创建专门的任务并使用消息传递的方式来管理状态
// 将锁进行分片
// 重构代码以避免锁
type ShardDB = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

// 在 .await 期间持有锁
// - 提前釋放鎖
// - 重構代碼，在重構期間不持有鎖
// - 使用異步任務和消息傳遞來管理狀態
use tokio::sync::Mutex as asyncMutex; // 注意，这里使用的是 Tokio 提供的锁
type AsyncShardDB = Arc<Vec<asyncMutex<HashMap<String, Bytes>>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db = Arc::new(Mutex::new(HashMap::<String, Bytes>::new()));
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // 将 handle 克隆一份
        let db = db.clone();
        // 为每一条连接都生成一个新的任务，
        // `socket` 的所有权将被移动到新的任务中，并在那里进行处理 << 新的任務會在隊列中等待executor處理
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

// async fn process(socket: TcpStream) {
//     // `Connection` 对于 redis 的读写进行了抽象封装，因此我们读到的是一个一个数据帧frame(数据帧 = redis命令 + 数据)，而不是字节流
//     // `Connection` 是在 mini-redis 中定义
//     let mut connection = Connection::new(socket);

//     if let Some(frame) = connection.read_frame().await.unwrap() {
//         println!("GOT: {:?}", frame);

//         // 回复一个错误
//         let response = Frame::Error("unimplemented".to_string());
//         connection.write_frame(&response).await.unwrap();
//     }
// }

async fn process(socket: TcpStream, db: DB) {
    use mini_redis::Command::{self, Get, Set};

    // 使用 hashmap 来存储 redis 的数据

    // `mini-redis` 提供的便利函数，使用返回的 `connection` 可以用于从 socket 中读取数据并解析为数据帧
    let mut connection = Connection::new(socket);

    // 使用 `read_frame` 方法从连接获取一个数据帧：一条redis命令 + 相应的数据
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                // 值被存储为 `Bytes` 的形式
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    // `Frame::Bulk` 期待数据的类型是 `Bytes`， 该类型会在后面章节讲解
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // 将请求响应返回给客户端
        connection.write_frame(&response).await.unwrap();
    }
}
