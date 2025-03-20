use tokio::time::{sleep, Duration};

use crate::utils::*;


const VALIDATE_TIMEOUT: u64 = 3000;


pub async fn task(appdata: WebAppData) -> TokioResult<()> {
    loop {
        sleep(Duration::from_millis(VALIDATE_TIMEOUT)).await;
        // println!("validate: {}", appdata.config.workers);
        // heavy_calculation();
        // println!("yes");
    }
}


// fn heavy_calculation() {
//     use std::sync::{Mutex, Arc};
//     let queue = Arc::new(Mutex::new((0..1000).collect::<Vec<u64>>()));
//     let count = Arc::new(Mutex::new(0));

//     let mut handlers = Vec::new();

//     for _ in 0..4 {
//         let queue = queue.clone();
//         let count = count.clone();
//         let handler = std::thread::spawn(move || {
//             while let Some(n) = queue.lock().unwrap().pop() {
//                 let mut flag = true;
//                 for d in 2..n {
//                     if n % d == 0 {
//                         flag = false;
//                         break;
//                     }
//                 }
//                 if flag {
//                     *count.lock().unwrap() += 1;
//                 }
//             }
//             println!("{:?}", count.lock().unwrap());
//         });
//         handlers.push(handler);
//     }
// }
