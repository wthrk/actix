//#![cfg(feature = "macros")]

use std::time::Duration;

use actix::prelude::*;
use actix_rt::time::sleep;

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

struct MyActor;

impl Actor for MyActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_millis(100), |_this, ctx| {
            if !ctx.connected() {
                ctx.stop();
            }
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        System::current().stop()
    }
}

#[wasm_bindgen_test]
async fn test_connected() {
    actix_rt::spawn(async move {
        let addr = MyActor::start(MyActor);
        sleep(Duration::from_millis(350)).await;
        drop(addr);
    });

    actix_rt::time::sleep(Duration::from_millis(500)).await;
}
