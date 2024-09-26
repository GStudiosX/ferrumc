#![feature(portable_simd)]

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use fastanvil::Region;
use fastnbt::Value;
use ferrumc_macros::NBTDeserialize;
use nbt as hematite_nbt;
use std::io::Cursor;

fn bench_ferrumc_nbt(data: &[u8]) {
    /*let mut parser = ferrumc_nbt::de::borrow::NbtTape::new(data);
    parser.parse();

    black_box(parser);*/

    #[derive(NBTDeserialize)]
    struct Chunk<'a> {
        #[nbt(rename = "xPos")]
        x_pos: i32,
        #[nbt(rename = "zPos")]
        z_pos: i32,
        #[nbt(rename = "Heightmaps")]
        heightmaps: Heightmaps<'a>
    }

    #[derive(NBTDeserialize)]
    struct Heightmaps<'a> {
        #[nbt(rename = "MOTION_BLOCKING")]
        motion_blocking: &'a [i64],
    }


    let chunk = Chunk::from_bytes(data).unwrap();
    assert_eq!(chunk.x_pos, 0);
    assert_eq!(chunk.z_pos, 32);
    assert_eq!(chunk.heightmaps.motion_blocking.len(), 37);
}

fn bench_simdnbt(data: &[u8]) {
    let nbt = simdnbt::borrow::read(&mut std::io::Cursor::new(data)).unwrap();

    let nbt = nbt.unwrap();
    let nbt = nbt.as_compound();
    let x_pos = nbt.get("xPos").unwrap().int().unwrap();
    let z_pos = nbt.get("zPos").unwrap().int().unwrap();

    let motion_blocking = nbt.get("Heightmaps").unwrap().compound().unwrap().get("MOTION_BLOCKING").unwrap().long_array().unwrap();

    assert_eq!(x_pos, 0);
    assert_eq!(z_pos, 32);
    assert_eq!(motion_blocking.len(), 37);
}

fn bench_simdnbt_owned(data: &[u8]) {
    let nbt = simdnbt::owned::read(&mut Cursor::new(data)).unwrap();
    assert!(nbt.is_some());
}

fn ussr_nbt_borrow(data: &[u8]) {
    let nbt = black_box(ussr_nbt::borrow::Nbt::read(&mut Cursor::new(data)).unwrap());
    black_box(nbt);
}

fn ussr_nbt_owned(data: &[u8]) {
    let nbt = black_box(ussr_nbt::owned::Nbt::read(&mut Cursor::new(data)).unwrap());
    black_box(nbt);
}

fn fastnbt(data: &[u8]) {
    let nbt: Value = black_box(fastnbt::from_reader(&mut Cursor::new(data)).unwrap());
    black_box(nbt);
}

fn crab_nbt(data: &[u8]) {
    let nbt = crab_nbt::Nbt::read(&mut Cursor::new(data)).unwrap();
    black_box(nbt);
}

fn hematite_nbt(data: &[u8]) {
    let nbt = hematite_nbt::Blob::from_reader(&mut Cursor::new(data)).unwrap();
    black_box(nbt);
}

fn criterion_benchmark(c: &mut Criterion) {
    // let cursor = Cursor::new(include_bytes!("../../../../../.etc/benches/region/r.0.0.mca"));
    // let file = std::fs::File::open(r#"D:\Minecraft\framework\ferrumc\ferrumc-2_0\ferrumc\.etc\benches\region\r.0.0.mca"#).unwrap();

    let cursor = include_bytes!("../../../../../.etc/benches/region/r.0.1.mca");

    let buf_reader = std::io::Cursor::new(cursor);

    let mut region = Region::from_stream(buf_reader).unwrap();
    let next = region.read_chunk(0, 0).unwrap().unwrap();

    // let data = chunk.data.as_slice();
    let data = next.as_slice();

    // let data = include_bytes!("../../../../../.etc/benches/registry_data.nbt");
    let data = ferrumc_nbt::decompress_gzip(data).unwrap();
    let data = data.as_slice();

    let mut group = c.benchmark_group("Chunk Data NBT Parsing");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function("FerrumC NBT", |b| {
        b.iter(|| bench_ferrumc_nbt(black_box(data)))
    });
    group.bench_function("simdnbt borrow", |b| {
        b.iter(|| bench_simdnbt(black_box(data)))
    });
    group.bench_function("simdnbt owned", |b| {
        b.iter(|| bench_simdnbt_owned(black_box(data)))
    });
    group.bench_function("fastnbt", |b| b.iter(|| fastnbt(black_box(data))));
    group.bench_function("ussr_nbt owned", |b| {
        b.iter(|| ussr_nbt_owned(black_box(data)))
    });
    group.bench_function("ussr_nbt borrow", |b| {
        b.iter(|| ussr_nbt_borrow(black_box(data)))
    });
    group.bench_function("crab_nbt", |b| b.iter(|| crab_nbt(black_box(data))));
    group.bench_function("hematite_nbt", |b| b.iter(|| hematite_nbt(black_box(data))));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);