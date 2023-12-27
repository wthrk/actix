use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use actix::prelude::*;
use tokio::sync::oneshot;

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[derive(Debug)]
struct Ping(usize);

impl Message for Ping {
    type Result = ();
}

struct MyActor(Arc<AtomicUsize>);

impl Actor for MyActor {
    type Context = Context<Self>;
}

impl Handler<Ping> for MyActor {
    type Result = ();

    fn handle(&mut self, _: Ping, _: &mut actix::Context<MyActor>) {
        self.0
            .store(self.0.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
        //System::current().stop();
    }
}

#[wasm_bindgen_test]
async fn test_start_actor_message() {
    let count = Arc::new(AtomicUsize::new(0));
    let act_count = Arc::clone(&count);

    let (tx_fin, rx_fin) = oneshot::channel();

    actix_rt::System::new()
        .block_on(async move {
            actix_rt::spawn(async move {
                {
                    let (tx, rx) = oneshot::channel();

                    Arbiter::current().spawn_fn(move || {
                        let addr = MyActor(act_count).start();
                        tx.send(addr).ok().unwrap();
                    });

                    // TODO: investigate under CPU stress and/or with a drop impl
                    // original test used this line, but was buggy:
                    // rx.await.unwrap().do_send(Ping(1));
                    rx.await.unwrap().send(Ping(1)).await.unwrap();
                }

                tx_fin.send(()).unwrap();
            });
        })
        .await;

    rx_fin.await.unwrap();

    assert_eq!(count.load(Ordering::Relaxed), 1);
}
