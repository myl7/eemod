use std::env::var;

use criterion::BenchmarkId;
use criterion::{black_box, Criterion};
use criterion::{criterion_group, criterion_main};
use rand::prelude::*;
use tokio::runtime::Builder;
use uuid::Uuid;

use eemod::crypto;
use eemod::crypto::prelude::*;
use eemod::grpc::eems::eems_for_send_client::EemsForSendClient;
use eemod::user::{Receiver, Sender};

fn from_body_size(c: &mut Criterion) {
    let rt = Builder::new_multi_thread().enable_all().build().unwrap();

    let sender_id_s = "f7584d00-6948-4e9c-b444-ff757f4dd9c1";
    let sender_id = Uuid::parse_str(sender_id_s).unwrap();
    let sender_sk: SK = hex::decode(var("EEMOD_EVAL_SENDER_SK").unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    let sender_id_sign = crypto::pk_sign(&sender_sk, sender_id.as_bytes());
    let eems_addr = "127.0.0.1:8000";
    let eems_url = format!("http://{eems_addr}");
    let eems_client = rt.block_on(async { EemsForSendClient::connect(eems_url).await.unwrap() });
    let sender = Sender::new(sender_id, sender_id_sign, eems_client);

    let eems_pk: PK = hex::decode(var("EEMOD_EVAL_EEMS_PK").unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    let receiver = Receiver::new(eems_pk);

    let body_size_iter = (3..13).into_iter().map(|x| 2usize.pow(x));
    body_size_iter.for_each(|body_size| {
        let msg_key: SymK = thread_rng().gen();
        let mut body = vec![0; body_size];
        thread_rng().fill_bytes(&mut body);
        let msg_id = rt.block_on(async { sender.gen_id(&body, msg_key).await.unwrap() });

        c.bench_with_input(
            BenchmarkId::new("verify_msg", body_size),
            &body_size,
            |b, _| {
                b.iter(|| black_box(receiver.verify_msg(&body, &msg_id).unwrap()));
            },
        );
    });

    let body_size_iter = (1300..=4000).step_by(300);
    body_size_iter.for_each(|body_size| {
        let msg_key: SymK = thread_rng().gen();
        let mut body = vec![0; body_size];
        thread_rng().fill_bytes(&mut body);
        let msg_id = rt.block_on(async { sender.gen_id(&body, msg_key).await.unwrap() });

        c.bench_with_input(
            BenchmarkId::new("verify_msg", body_size),
            &body_size,
            |b, _| {
                b.iter(|| black_box(receiver.verify_msg(&body, &msg_id).unwrap()));
            },
        );
    });
}

criterion_group!(benches, from_body_size);
criterion_main!(benches);
