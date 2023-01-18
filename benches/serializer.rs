#![allow(clippy::let_unit_value)]
#![allow(clippy::unit_arg)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lunatic::serializer::CanSerialize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Age(i32);

const AGE: Age = Age(10);

#[derive(Serialize, Deserialize)]
struct Login {
    username: String,
    password: String,
    remember: bool,
}

#[inline(always)]
fn login_data() -> Login {
    Login {
        username: "johndoe@gmail.com".to_string(),
        password: "JohnTheGod".to_string(),
        remember: true,
    }
}

#[derive(Serialize, Deserialize)]
struct User {
    a: String,
    b: i32,
    c: bool,
    d: String,
    e: i32,
    f: bool,
    h: String,
    i: i32,
    j: bool,
    k: String,
    l: i32,
    m: bool,
    n: String,
    o: i32,
    p: bool,
    q: String,
    r: i32,
    s: bool,
    t: String,
    u: i32,
    v: bool,
    w: String,
    x: i32,
    y: bool,
    z: String,
    aa: i32,
    bb: bool,
    cc: String,
    dd: i32,
    ee: bool,
    ff: String,
    gg: i32,
    hh: bool,
    ii: String,
    jj: i32,
    kk: bool,
    ll: String,
    mm: i32,
    nn: bool,
    oo: String,
    pp: i32,
    qq: bool,
    rr: String,
    ss: i32,
    tt: bool,
    uu: String,
    vv: i32,
    ww: bool,
    xx: String,
    yy: i32,
    zz: bool,
    aaa: String,
    aab: i32,
    aac: bool,
    aad: String,
    aae: i32,
    aaf: bool,
    aah: String,
    aai: i32,
    aaj: bool,
    aak: String,
    aal: i32,
    aam: bool,
    aan: String,
    aao: i32,
    aap: bool,
    aaq: String,
    aar: i32,
    aas: bool,
    aat: String,
    aau: i32,
    aav: bool,
    aaw: String,
    aax: i32,
    aay: bool,
    aaz: String,
    aaaa: i32,
    aabb: bool,
    aacc: String,
    aadd: i32,
    aaee: bool,
    aaff: String,
    aagg: i32,
    aahh: bool,
    aaii: String,
    aajj: i32,
    aakk: bool,
    aall: String,
    aamm: i32,
    aann: bool,
    aaoo: String,
    aapp: i32,
    aaqq: bool,
    aarr: String,
    aass: i32,
    aatt: bool,
    aauu: String,
    aavv: i32,
    aaww: bool,
    aaxx: String,
    aayy: i32,
    aazz: bool,
}

#[inline(always)]
fn user_data() -> User {
    User {
        a: "a".to_string(),
        b: 123,
        c: true,
        d: "d".to_string(),
        e: 123,
        f: true,
        h: "h".to_string(),
        i: 123,
        j: true,
        k: "k".to_string(),
        l: 123,
        m: true,
        n: "n".to_string(),
        o: 123,
        p: true,
        q: "q".to_string(),
        r: 123,
        s: true,
        t: "t".to_string(),
        u: 123,
        v: true,
        w: "w".to_string(),
        x: 123,
        y: true,
        z: "z".to_string(),
        aa: 123,
        bb: true,
        cc: "cc".to_string(),
        dd: 123,
        ee: true,
        ff: "ff".to_string(),
        gg: 123,
        hh: true,
        ii: "ii".to_string(),
        jj: 123,
        kk: true,
        ll: "ll".to_string(),
        mm: 123,
        nn: true,
        oo: "oo".to_string(),
        pp: 123,
        qq: true,
        rr: "rr".to_string(),
        ss: 123,
        tt: true,
        uu: "uu".to_string(),
        vv: 123,
        ww: true,
        xx: "xx".to_string(),
        yy: 123,
        zz: true,
        aaa: "aaa".to_string(),
        aab: 123,
        aac: true,
        aad: "aad".to_string(),
        aae: 123,
        aaf: true,
        aah: "aah".to_string(),
        aai: 123,
        aaj: true,
        aak: "aak".to_string(),
        aal: 123,
        aam: true,
        aan: "aan".to_string(),
        aao: 123,
        aap: true,
        aaq: "aaq".to_string(),
        aar: 123,
        aas: true,
        aat: "aat".to_string(),
        aau: 123,
        aav: true,
        aaw: "aaw".to_string(),
        aax: 123,
        aay: true,
        aaz: "aaz".to_string(),
        aaaa: 123,
        aabb: true,
        aacc: "aacc".to_string(),
        aadd: 123,
        aaee: true,
        aaff: "aaff".to_string(),
        aagg: 123,
        aahh: true,
        aaii: "aaii".to_string(),
        aajj: 123,
        aakk: true,
        aall: "aall".to_string(),
        aamm: 123,
        aann: true,
        aaoo: "aaoo".to_string(),
        aapp: 123,
        aaqq: true,
        aarr: "aarr".to_string(),
        aass: 123,
        aatt: true,
        aauu: "aauu".to_string(),
        aavv: 123,
        aaww: true,
        aaxx: "aaxx".to_string(),
        aayy: 123,
        aazz: true,
    }
}

#[derive(Serialize, Deserialize)]
struct BigData {
    #[serde(with = "serde_bytes")]
    data1: Vec<u8>,
    #[serde(with = "serde_bytes")]
    data2: Vec<u8>,
    #[serde(with = "serde_bytes")]
    data3: Vec<u8>,
}

#[inline(always)]
fn big_data() -> BigData {
    BigData {
        data1: vec![0u8; 1024],
        data2: vec![0u8; 2048],
        data3: vec![0u8; 4098],
    }
}

fn serialize_bincode_benchmark(c: &mut Criterion) {
    c.bench_function("encode_small_struct_small_fields", |b| {
        b.iter(|| {
            unsafe {
                lunatic::host::api::message::create_data(0, 0);
            }
            black_box(lunatic::serializer::Bincode::encode(&AGE).unwrap());
            unsafe {
                lunatic::host::api::message::send(1337);
            }
        });
    });

    c.bench_function("encode_medium_struct_small_fields", |b| {
        b.iter(|| {
            unsafe {
                lunatic::host::api::message::create_data(0, 0);
            }
            black_box(lunatic::serializer::Bincode::encode(&login_data()).unwrap());
            unsafe {
                lunatic::host::api::message::send(1337);
            }
        });
    });

    c.bench_function("encode_medium_struct_big_fields", |b| {
        b.iter(|| {
            unsafe {
                lunatic::host::api::message::create_data(0, 0);
            }
            black_box(lunatic::serializer::Bincode::encode(&big_data()).unwrap());
            unsafe {
                lunatic::host::api::message::send(1337);
            }
        });
    });

    c.bench_function("encode_large_small_fields", |b| {
        b.iter(|| {
            unsafe {
                lunatic::host::api::message::create_data(0, 0);
            }
            black_box(lunatic::serializer::Bincode::encode(&user_data()).unwrap());
            unsafe {
                lunatic::host::api::message::send(1337);
            }
        });
    });
}

fn serialize_json_benchmark(c: &mut Criterion) {
    c.bench_function("encode_smal_struct_small_fields", |b| {
        b.iter(|| {
            unsafe {
                lunatic::host::api::message::create_data(0, 0);
            }
            black_box(lunatic::serializer::Json::encode(&AGE).unwrap());
            unsafe {
                lunatic::host::api::message::send(1337);
            }
        });
    });

    c.bench_function("encode_medium_struct_small_fields", |b| {
        b.iter(|| {
            unsafe {
                lunatic::host::api::message::create_data(0, 0);
            }
            black_box(lunatic::serializer::Json::encode(&login_data()).unwrap());
            unsafe {
                lunatic::host::api::message::send(1337);
            }
        });
    });

    c.bench_function("encode_medium_struct_big_fields", |b| {
        b.iter(|| {
            unsafe {
                lunatic::host::api::message::create_data(0, 0);
            }
            black_box(lunatic::serializer::Json::encode(&big_data()).unwrap());
            unsafe {
                lunatic::host::api::message::send(1337);
            }
        });
    });

    c.bench_function("encode_large_struct_small_fields", |b| {
        b.iter(|| {
            unsafe {
                lunatic::host::api::message::create_data(0, 0);
            }
            black_box(lunatic::serializer::Json::encode(&user_data()).unwrap());
            unsafe {
                lunatic::host::api::message::send(1337);
            }
        });
    });
}

criterion_group!(
    benches,
    serialize_bincode_benchmark,
    serialize_json_benchmark
);
criterion_main!(benches);
