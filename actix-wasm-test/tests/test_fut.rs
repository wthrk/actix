use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use actix::{clock::sleep, prelude::*};

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

struct MyActor {
    timeout: Arc<AtomicBool>,
}

impl Actor for MyActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        async {
            sleep(Duration::from_millis(20)).await;
            System::current().stop();
        }
        .into_actor(self)
        .timeout(Duration::from_millis(1))
        .map(|e, act, _| {
            if e == Err(()) {
                act.timeout.store(true, Ordering::Relaxed);
                System::current().stop();
            }
        })
        .wait(ctx);
    }
}

#[wasm_bindgen_test]
async fn test_fut_timeout() {
    let timeout = Arc::new(AtomicBool::new(false));
    let timeout2 = Arc::clone(&timeout);

    let sys = System::new();
    sys.block_on(async {
        let _addr = MyActor { timeout: timeout2 }.start();
        actix_rt::time::sleep(Duration::from_millis(200)).await;
    })
    .await;

    assert!(timeout.load(Ordering::Relaxed), "Not timeout");
}

struct MyStreamActor {
    timeout: Arc<AtomicBool>,
}

impl Actor for MyStreamActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut s = futures_util::stream::FuturesOrdered::new();
        s.push_back(sleep(Duration::from_millis(20)));
        s.push_back(sleep(Duration::from_millis(20)));

        s.into_actor(self)
            .timeout(Duration::from_millis(1))
            .then(|res, act, _| {
                // Additional waiting time to test `then` call as well
                async move {
                    sleep(Duration::from_millis(20)).await;
                    res
                }
                .into_actor(act)
            })
            .map(|res, act, _| {
                assert!(
                    res.is_err(),
                    "MyStreamActor should return error when timed out"
                );
                act.timeout.store(true, Ordering::Relaxed);
                System::current().stop();
            })
            .finish()
            .wait(ctx)
    }
}

struct MyStreamActor2 {
    counter: usize,
}

impl Actor for MyStreamActor2 {
    type Context = actix::Context<Self>;
}

struct TakeWhileMsg(usize);

impl Message for TakeWhileMsg {
    type Result = ();
}

impl Handler<TakeWhileMsg> for MyStreamActor2 {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: TakeWhileMsg, _: &mut Context<Self>) -> Self::Result {
        let num = msg.0;
        let result = futures_util::stream::repeat(num)
            .into_actor(self)
            .take_while(move |n, act, ctx| {
                ctx.spawn(actix_rt::time::sleep(Duration::from_millis(1)).into_actor(act));
                assert_eq!(*n, num);
                assert!(act.counter < 10);
                act.counter += 1;
                futures_util::future::ready(act.counter < 10)
            })
            .finish()
            .boxed_local();

        return result;
    }
}

struct SkipWhileMsg(usize);

impl Message for SkipWhileMsg {
    type Result = ();
}

impl Handler<SkipWhileMsg> for MyStreamActor2 {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: SkipWhileMsg, _: &mut Context<Self>) -> Self::Result {
        let num = msg.0;
        futures_util::stream::repeat(num)
            .into_actor(self)
            .take_while(|_, act, _| {
                let cond = act.counter < 10;
                act.counter += 1;
                futures_util::future::ready(cond)
            })
            .skip_while(move |n, act, ctx| {
                ctx.spawn(actix_rt::time::sleep(Duration::from_millis(1)).into_actor(act));
                assert_eq!(*n, num);
                act.counter -= 1;
                futures_util::future::ready(act.counter > 0)
            })
            .finish()
            .boxed_local()
    }
}

struct CollectMsg(usize);

impl Message for CollectMsg {
    type Result = Vec<usize>;
}

impl Handler<CollectMsg> for MyStreamActor2 {
    type Result = ResponseActFuture<Self, Vec<usize>>;

    fn handle(&mut self, msg: CollectMsg, _: &mut Context<Self>) -> Self::Result {
        let num = msg.0;
        futures_util::stream::repeat(num)
            .into_actor(self)
            .take_while(|_, act, _| {
                let cond = act.counter < 5;
                act.counter += 1;
                futures_util::future::ready(cond)
            })
            .map(|_, act, _| act.counter)
            .collect()
            .boxed_local()
    }
}

struct TryFutureMsg(usize);

impl Message for TryFutureMsg {
    type Result = Result<isize, u32>;
}

impl Handler<TryFutureMsg> for MyStreamActor2 {
    type Result = ResponseActFuture<Self, Result<isize, u32>>;

    fn handle(&mut self, msg: TryFutureMsg, _: &mut Context<Self>) -> Self::Result {
        let num = msg.0;
        async move {
            assert_eq!(num, 5);
            Ok::<usize, usize>(num * 2)
        }
        .into_actor(self)
        .map_ok(|res, _, _| {
            assert_eq!(10usize, res);
            res as isize
        })
        .and_then(|res, _, _| {
            assert_eq!(10isize, res);
            fut::err::<isize, usize>(996)
        })
        .map_err(|e, _, _| {
            assert_eq!(996usize, e);
            e as u32
        })
        .boxed_local()
    }
}

#[wasm_bindgen_test]
#[ignore]
async fn test_stream_timeout() {
    let timeout = Arc::new(AtomicBool::new(false));
    let timeout2 = Arc::clone(&timeout);

    let sys = System::new();
    sys.block_on(async {
        let _addr = MyStreamActor { timeout: timeout2 }.start();
        sleep(Duration::from_millis(200)).await;
    })
    .await;

    assert!(timeout.load(Ordering::Relaxed), "Not timeout");
}

#[wasm_bindgen_test]
async fn test_stream_take_while() {
    System::new()
        .block_on(async {
            let addr = MyStreamActor2 { counter: 0 }.start();
            addr.send(TakeWhileMsg(5)).await.unwrap();
        })
        .await;
}

#[wasm_bindgen_test]
async fn test_stream_skip_while() {
    System::new()
        .block_on(async {
            let addr = MyStreamActor2 { counter: 0 }.start();
            addr.send(SkipWhileMsg(5)).await.unwrap();
        })
        .await;
}

#[wasm_bindgen_test]
async fn test_stream_collect() {
    System::new()
        .block_on(async {
            let addr = MyStreamActor2 { counter: 0 }.start();
            let res = addr.send(CollectMsg(3)).await.unwrap();

            assert_eq!(res, vec![1, 2, 3, 4, 5]);
        })
        .await;
}

#[wasm_bindgen_test]
async fn test_try_future() {
    System::new()
        .block_on(async {
            let addr = MyStreamActor2 { counter: 0 }.start();
            let res = addr.send(TryFutureMsg(5)).await.unwrap();
            assert_eq!(res.err().unwrap(), 996u32);
        })
        .await;
}
