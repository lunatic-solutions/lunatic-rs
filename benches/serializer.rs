#![allow(clippy::let_unit_value)]
#![allow(clippy::unit_arg)]


use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lunatic::serializer::Serializer;
use serde::Serialize;

#[derive(Serialize)]
struct Age(i32);

impl<'de> serde::Deserialize<'de> for Age {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

const AGE: Age = Age(10);

#[derive(Serialize)]
struct Login {
    username: &'static str,
    password: &'static str,
    remember: bool,
}

impl<'de> serde::Deserialize<'de> for Login {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

const LOGIN: Login = Login {
    username: "johndoe@gmail.com",
    password: "JohnTheGod",
    remember: true,
};

#[derive(Serialize)]
struct User {
    a: &'static str,
    b: i32,
    c: bool,
    d: &'static str,
    e: i32,
    f: bool,
    h: &'static str,
    i: i32,
    j: bool,
    k: &'static str,
    l: i32,
    m: bool,
    n: &'static str,
    o: i32,
    p: bool,
    q: &'static str,
    r: i32,
    s: bool,
    t: &'static str,
    u: i32,
    v: bool,
    w: &'static str,
    x: i32,
    y: bool,
    z: &'static str,
    aa: i32,
    bb: bool,
    cc: &'static str,
    dd: i32,
    ee: bool,
    ff: &'static str,
    gg: i32,
    hh: bool,
    ii: &'static str,
    jj: i32,
    kk: bool,
    ll: &'static str,
    mm: i32,
    nn: bool,
    oo: &'static str,
    pp: i32,
    qq: bool,
    rr: &'static str,
    ss: i32,
    tt: bool,
    uu: &'static str,
    vv: i32,
    ww: bool,
    xx: &'static str,
    yy: i32,
    zz: bool,
    aaa: &'static str,
    aab: i32,
    aac: bool,
    aad: &'static str,
    aae: i32,
    aaf: bool,
    aah: &'static str,
    aai: i32,
    aaj: bool,
    aak: &'static str,
    aal: i32,
    aam: bool,
    aan: &'static str,
    aao: i32,
    aap: bool,
    aaq: &'static str,
    aar: i32,
    aas: bool,
    aat: &'static str,
    aau: i32,
    aav: bool,
    aaw: &'static str,
    aax: i32,
    aay: bool,
    aaz: &'static str,
    aaaa: i32,
    aabb: bool,
    aacc: &'static str,
    aadd: i32,
    aaee: bool,
    aaff: &'static str,
    aagg: i32,
    aahh: bool,
    aaii: &'static str,
    aajj: i32,
    aakk: bool,
    aall: &'static str,
    aamm: i32,
    aann: bool,
    aaoo: &'static str,
    aapp: i32,
    aaqq: bool,
    aarr: &'static str,
    aass: i32,
    aatt: bool,
    aauu: &'static str,
    aavv: i32,
    aaww: bool,
    aaxx: &'static str,
    aayy: i32,
    aazz: bool,
}

impl<'de> serde::Deserialize<'de> for User {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

const USER: User = User {
    a: "a",
    b: 123,
    c: true,
    d: "d",
    e: 123,
    f: true,
    h: "h",
    i: 123,
    j: true,
    k: "k",
    l: 123,
    m: true,
    n: "n",
    o: 123,
    p: true,
    q: "q",
    r: 123,
    s: true,
    t: "t",
    u: 123,
    v: true,
    w: "w",
    x: 123,
    y: true,
    z: "z",
    aa: 123,
    bb: true,
    cc: "cc",
    dd: 123,
    ee: true,
    ff: "ff",
    gg: 123,
    hh: true,
    ii: "ii",
    jj: 123,
    kk: true,
    ll: "ll",
    mm: 123,
    nn: true,
    oo: "oo",
    pp: 123,
    qq: true,
    rr: "rr",
    ss: 123,
    tt: true,
    uu: "uu",
    vv: 123,
    ww: true,
    xx: "xx",
    yy: 123,
    zz: true,
    aaa: "aaa",
    aab: 123,
    aac: true,
    aad: "aad",
    aae: 123,
    aaf: true,
    aah: "aah",
    aai: 123,
    aaj: true,
    aak: "aak",
    aal: 123,
    aam: true,
    aan: "aan",
    aao: 123,
    aap: true,
    aaq: "aaq",
    aar: 123,
    aas: true,
    aat: "aat",
    aau: 123,
    aav: true,
    aaw: "aaw",
    aax: 123,
    aay: true,
    aaz: "aaz",
    aaaa: 123,
    aabb: true,
    aacc: "aacc",
    aadd: 123,
    aaee: true,
    aaff: "aaff",
    aagg: 123,
    aahh: true,
    aaii: "aaii",
    aajj: 123,
    aakk: true,
    aall: "aall",
    aamm: 123,
    aann: true,
    aaoo: "aaoo",
    aapp: 123,
    aaqq: true,
    aarr: "aarr",
    aass: 123,
    aatt: true,
    aauu: "aauu",
    aavv: 123,
    aaww: true,
    aaxx: "aaxx",
    aayy: 123,
    aazz: true,
};

fn serialize_bincode_benchmark(c: &mut Criterion) {
    c.bench_function("encode_small", |b| {
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

    c.bench_function("encode_medium", |b| {
        b.iter(|| {
            unsafe {
                lunatic::host::api::message::create_data(0, 0);
            }
            black_box(lunatic::serializer::Bincode::encode(&LOGIN).unwrap());
            unsafe {
                lunatic::host::api::message::send(1337);
            }
        });
    });

    c.bench_function("encode_large", |b| {
        b.iter(|| {
            unsafe {
                lunatic::host::api::message::create_data(0, 0);
            }
            black_box(lunatic::serializer::Bincode::encode(&USER).unwrap());
            unsafe {
                lunatic::host::api::message::send(1337);
            }
        });
    });
}

fn serialize_json_benchmark(c: &mut Criterion) {
    c.bench_function("encode_small", |b| {
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

    c.bench_function("encode_medium", |b| {
        b.iter(|| {
            unsafe {
                lunatic::host::api::message::create_data(0, 0);
            }
            black_box(lunatic::serializer::Json::encode(&LOGIN).unwrap());
            unsafe {
                lunatic::host::api::message::send(1337);
            }
        });
    });

    c.bench_function("encode_large", |b| {
        b.iter(|| {
            unsafe {
                lunatic::host::api::message::create_data(0, 0);
            }
            black_box(lunatic::serializer::Json::encode(&USER).unwrap());
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
