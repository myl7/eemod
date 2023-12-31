use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dcf::prg::Aes256HirosePrg;
use dcf::{BoundState, CmpFn, Dcf, DcfImpl};
use group_math::int::U128Group;
use rand::prelude::*;

fn from_item_num(c: &mut Criterion) {
    let prg_key: [u8; 32] = thread_rng().gen();
    let prg = Aes256HirosePrg::new([&prg_key]);
    let dcf = DcfImpl::new(prg);
    let f = CmpFn::<16, 16, U128Group> {
        alpha: 10u128.to_le_bytes(),
        beta: U128Group(1),
    };

    let item_num_iter = [1, 2, 5, 10, 12].into_iter().map(|x| x * 3600 * 11);
    item_num_iter.for_each(|item_num| {
        c.bench_with_input(
            BenchmarkId::new("gen_compare", item_num),
            &item_num,
            |b, _| {
                b.iter(|| {
                    black_box({
                        let s00 = thread_rng().gen();
                        let s01 = thread_rng().gen();
                        let kappa_shares: Vec<_> = (0..item_num)
                            .map(|_| {
                                let mut share0 = dcf.gen(&f, [&s00, &s01], BoundState::GtBeta);
                                let mut share1 = share0.clone();
                                share0.s0s.remove(1);
                                share1.s0s.remove(0);
                                (share0, share1)
                            })
                            .collect();

                        let mut gammas = vec![0; item_num];
                        thread_rng().fill_bytes(&mut gammas);

                        (kappa_shares, gammas)
                    })
                });
            },
        );
    });
}

// criterion_group!(benches, from_item_num);
criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = from_item_num
}
criterion_main!(benches);
