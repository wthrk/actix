use std::ops::Mul;

use actix::prelude::*;
use tokio::sync::oneshot;

use wasm_bindgen_test::wasm_bindgen_test as test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[derive(Message)]
#[rtype(result = "()")]
struct Empty;

struct EmptyActor;

impl Actor for EmptyActor {
    type Context = Context<Self>;
}

impl Handler<Empty> for EmptyActor {
    type Result = ();

    fn handle(&mut self, _message: Empty, _context: &mut Context<Self>) {}
}

#[test]
#[allow(clippy::unit_cmp)]
async fn response_derive_empty() {
    System::new()
        .block_on(async {
            let addr = EmptyActor.start();
            let (tx, rx) = oneshot::channel::<()>();

            actix_rt::spawn(async move {
                let res = addr.send(Empty);
                match res.await {
                    Ok(result) => assert!(result == ()),
                    _ => panic!("Something went wrong"),
                }
                tx.send(()).unwrap();
            });

            rx.await.unwrap();
        })
        .await;
}

#[derive(Message)]
#[rtype(result = "Result<usize, ()>")]
struct SumResult(usize, usize);

struct SumResultActor;

impl Actor for SumResultActor {
    type Context = Context<Self>;
}

impl Handler<SumResult> for SumResultActor {
    type Result = Result<usize, ()>;

    fn handle(&mut self, message: SumResult, _context: &mut Context<Self>) -> Self::Result {
        Ok(message.0 + message.1)
    }
}

#[test]
async fn derive_result() {
    System::new()
        .block_on(async {
            let addr = SumResultActor.start();
            let (tx, rx) = oneshot::channel::<()>();

            actix_rt::spawn(async move {
                let res = addr.send(SumResult(10, 5));
                match res.await {
                    Ok(result) => assert!(result == Ok(10 + 5)),
                    _ => panic!("Something went wrong"),
                }
                tx.send(()).unwrap();
            });

            rx.await.unwrap();
        })
        .await;
}

#[derive(Message)]
#[rtype(usize)]
struct SumOne(usize, usize);

struct SumOneActor;

impl Actor for SumOneActor {
    type Context = Context<Self>;
}

impl Handler<SumOne> for SumOneActor {
    type Result = usize;

    fn handle(&mut self, message: SumOne, _context: &mut Context<Self>) -> Self::Result {
        message.0 + message.1
    }
}

#[test]
async fn response_derive_one() {
    actix_rt::System::new()
        .block_on(async {
            let addr = SumOneActor.start();
            let (tx, rx) = oneshot::channel::<()>();

            actix_rt::spawn(async move {
                let res = addr.send(SumOne(10, 5));
                match res.await {
                    Ok(result) => assert!(result == 10 + 5),
                    _ => panic!("Something went wrong"),
                }
                tx.send(()).unwrap();
            });

            rx.await.unwrap();
        })
        .await;
}

#[derive(MessageResponse, PartialEq)]
struct MulRes(usize);

#[derive(Message)]
#[rtype(MulRes)]
struct MulOne(usize, usize);

struct MulOneActor;

impl Actor for MulOneActor {
    type Context = Context<Self>;
}

impl Handler<MulOne> for MulOneActor {
    type Result = MulRes;

    fn handle(&mut self, message: MulOne, _context: &mut Context<Self>) -> Self::Result {
        MulRes(message.0 * message.1)
    }
}

#[test]
async fn derive_response_one() {
    actix_rt::System::new()
        .block_on(async {
            let addr = MulOneActor.start();
            let (tx, rx) = oneshot::channel::<()>();

            actix_rt::spawn(async move {
                let res = addr.send(MulOne(10, 5));
                match res.await {
                    Ok(result) => assert!(result == MulRes(10 * 5)),
                    _ => panic!("Something went wrong"),
                }
                tx.send(()).unwrap();
            });

            rx.await.unwrap();
        })
        .await;
}

#[derive(MessageResponse, PartialEq)]
struct MulAny<T: 'static + Mul>(T);

#[derive(Message)]
#[rtype(result = "MulAny<usize>")]
struct MulAnyOne(usize, usize);

struct MulAnyOneActor;

impl Actor for MulAnyOneActor {
    type Context = Context<Self>;
}

impl Handler<MulAnyOne> for MulAnyOneActor {
    type Result = MulAny<usize>;

    fn handle(&mut self, message: MulAnyOne, _context: &mut Context<Self>) -> Self::Result {
        MulAny(message.0 * message.1)
    }
}

#[test]
async fn derive_response_two() {
    actix_rt::System::new()
        .block_on(async {
            let addr = MulAnyOneActor.start();
            let (tx, rx) = oneshot::channel::<()>();

            actix_rt::spawn(async move {
                let res = addr.send(MulAnyOne(10, 5));
                match res.await {
                    Ok(result) => assert!(result == MulAny(10 * 5)),
                    _ => panic!("Something went wrong"),
                }
                tx.send(()).unwrap();
            });

            rx.await.unwrap();
        })
        .await;
}
