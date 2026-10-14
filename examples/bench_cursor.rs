//! Compares a flat `[u16; 64]` with field based models accessed via
//! [ReadCursor] / [WriteCursor], using the layout of the SunSpec 802
//! battery base model (header + 62 registers, 58 points).
//!
//! Variants:
//! - `flat`:   `[u16; 64]`
//! - `dyn`:    58 `RegXxx` fields, visited through an array of `&dyn` registers
//! - `direct`: same fields, visited one after another
//! - `prim`:   58 primitive fields (`u16`, `u32`, `i16`), serialized on access
//! - `view`:   like `prim`, but the fields are references into a state owned elsewhere
//! - `blocks`: largest possible read-only blocks, only interrupted by writable registers
//!
//! Run with `cargo run --release --example bench_cursor [iterations]`.

use std::hint::black_box;
use std::time::Instant;

use modbus_stack::Registers;
use modbus_stack::register::{
    RO, RW, ReadCursor, ReadRegisterData, RegI16, RegU16, RegU16Array, RegU32, RegisterData,
    RegisterResult, WriteCursor, WriteRegisterData, read_words, write_words,
};

const SPAN: usize = 64;
const ITERATIONS: u32 = 1_000_000;

/// Defines a model struct whose fields are visited one after another by the cursors.
macro_rules! model {
    (@impls $name:ident $(<$lt:lifetime>)? { $($field:ident),* }) => {
        impl$(<$lt>)? RegisterData for $name$(<$lt>)? {
            fn register_span(&self) -> usize {
                0 $(+ self.$field.register_span())*
            }
        }

        impl$(<$lt>)? ReadRegisterData for $name$(<$lt>)? {
            fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
                let mut cursor = ReadCursor::new(offset, buf);
                $(cursor.visit(&self.$field)?;)*
                cursor.finish()
            }
        }

        impl$(<$lt>)? WriteRegisterData for $name$(<$lt>)? {
            fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
                let mut cursor = WriteCursor::new(offset, values);
                $(cursor.visit(&mut self.$field)?;)*
                cursor.finish()
            }
        }
    };

    ($view:ident<$lt:lifetime> of $state:ident { $($field:ident: $access:ident $ty:ty),* $(,)? }) => {
        /// Owns the values, like the application would.
        #[derive(Default)]
        struct $state {
            $($field: $ty),*
        }

        impl $state {
            /// Loads `words` into the fields.
            fn load(&mut self, words: &[u16]) {
                let mut offset = 0;
                $(
                    let span = self.$field.register_span();
                    self.$field
                        .write(0, Registers::from(&words[offset..offset + span]))
                        .unwrap();
                    offset += span;
                )*
                assert_eq!(offset, words.len());
            }
        }

        /// References the values of a state, adding access rights.
        struct $view<$lt> {
            $($field: view!(@type $access $lt $ty)),*
        }

        impl<$lt> $view<$lt> {
            fn new(state: &$lt mut $state) -> $view<$lt> {
                $view {
                    $($field: view!(@new $access state.$field)),*
                }
            }
        }

        model!(@impls $view<$lt> { $($field),* });
    };

    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Default)]
        struct $name {
            $($field: $ty),*
        }

        #[allow(dead_code)]
        impl $name {
            /// Loads `words` into the fields, bypassing access rights.
            fn load(&mut self, words: &[u16]) {
                let mut offset = 0;
                $(
                    let span = self.$field.register_span();
                    (*self.$field)
                        .write(0, Registers::from(&words[offset..offset + span]))
                        .unwrap();
                    offset += span;
                )*
                assert_eq!(offset, words.len());
            }

            /// Like [ReadRegisterData::read], but visits the fields through an array of `&dyn`.
            fn read_dyn(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
                let mut cursor = ReadCursor::new(offset, buf);
                for reg in [$(&self.$field as &dyn ReadRegisterData),*] {
                    cursor.visit(reg)?;
                }
                cursor.finish()
            }

            /// Like [WriteRegisterData::write], but visits the fields through an array of `&mut dyn`.
            fn write_dyn(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
                let mut cursor = WriteCursor::new(offset, values);
                for reg in [$(&mut self.$field as &mut dyn WriteRegisterData),*] {
                    cursor.visit(reg)?;
                }
                cursor.finish()
            }
        }

        model!(@impls $name { $($field),* });
    };
}

/// Maps the access rights of a view field to its type and constructor.
macro_rules! view {
    (@type RO $lt:lifetime $ty:ty) => { RO<&$lt $ty> };
    (@type RW $lt:lifetime $ty:ty) => { RW<&$lt mut $ty> };
    (@new RO $value:expr) => { RO::new(&$value) };
    (@new RW $value:expr) => { RW::new(&mut $value) };
}

model!(Battery802 {
    id: RO<RegU16>,
    len: RO<RegU16>,
    ah_rtg: RO<RegU16>,
    wh_rtg: RO<RegU16>,
    w_cha_rte_max: RO<RegU16>,
    w_dis_cha_rte_max: RO<RegU16>,
    dis_cha_rte: RO<RegU16>,
    soc_np_max_pct: RW<RegU16>,
    soc_np_min_pct: RW<RegU16>,
    max_rsv_pct: RW<RegU16>,
    min_rsv_pct: RW<RegU16>,
    soc: RO<RegU16>,
    dod: RO<RegU16>,
    soh: RO<RegU16>,
    n_cyc: RO<RegU32>,
    cha_st: RO<RegU16>,
    loc_rem_ctl: RO<RegU16>,
    hb: RO<RegU16>,
    ctrl_hb: RW<RegU16>,
    alm_rst: RW<RegU16>,
    typ: RO<RegU16>,
    state: RO<RegU16>,
    state_vnd: RO<RegU16>,
    warr_dt: RO<RegU32>,
    evt1: RO<RegU32>,
    evt2: RO<RegU32>,
    evt_vnd1: RO<RegU32>,
    evt_vnd2: RO<RegU32>,
    v: RO<RegU16>,
    v_max: RO<RegU16>,
    v_min: RO<RegU16>,
    cell_v_max: RO<RegU16>,
    cell_v_max_str: RO<RegU16>,
    cell_v_max_mod: RO<RegU16>,
    cell_v_min: RO<RegU16>,
    cell_v_min_str: RO<RegU16>,
    cell_v_min_mod: RO<RegU16>,
    cell_v_avg: RO<RegU16>,
    a: RO<RegI16>,
    a_cha_max: RO<RegU16>,
    a_dis_cha_max: RO<RegU16>,
    w: RO<RegI16>,
    req_inv_state: RO<RegU16>,
    req_w: RO<RegI16>,
    set_op: RW<RegU16>,
    set_inv_state: RW<RegU16>,
    ah_rtg_sf: RO<RegI16>,
    wh_rtg_sf: RO<RegI16>,
    w_cha_dis_cha_max_sf: RO<RegI16>,
    dis_cha_rte_sf: RO<RegI16>,
    soc_sf: RO<RegI16>,
    dod_sf: RO<RegI16>,
    soh_sf: RO<RegI16>,
    v_sf: RO<RegI16>,
    cell_v_sf: RO<RegI16>,
    a_sf: RO<RegI16>,
    a_max_sf: RO<RegI16>,
    w_sf: RO<RegI16>,
});

model!(Battery802Prim {
    id: RO<u16>,
    len: RO<u16>,
    ah_rtg: RO<u16>,
    wh_rtg: RO<u16>,
    w_cha_rte_max: RO<u16>,
    w_dis_cha_rte_max: RO<u16>,
    dis_cha_rte: RO<u16>,
    soc_np_max_pct: RW<u16>,
    soc_np_min_pct: RW<u16>,
    max_rsv_pct: RW<u16>,
    min_rsv_pct: RW<u16>,
    soc: RO<u16>,
    dod: RO<u16>,
    soh: RO<u16>,
    n_cyc: RO<u32>,
    cha_st: RO<u16>,
    loc_rem_ctl: RO<u16>,
    hb: RO<u16>,
    ctrl_hb: RW<u16>,
    alm_rst: RW<u16>,
    typ: RO<u16>,
    state: RO<u16>,
    state_vnd: RO<u16>,
    warr_dt: RO<u32>,
    evt1: RO<u32>,
    evt2: RO<u32>,
    evt_vnd1: RO<u32>,
    evt_vnd2: RO<u32>,
    v: RO<u16>,
    v_max: RO<u16>,
    v_min: RO<u16>,
    cell_v_max: RO<u16>,
    cell_v_max_str: RO<u16>,
    cell_v_max_mod: RO<u16>,
    cell_v_min: RO<u16>,
    cell_v_min_str: RO<u16>,
    cell_v_min_mod: RO<u16>,
    cell_v_avg: RO<u16>,
    a: RO<i16>,
    a_cha_max: RO<u16>,
    a_dis_cha_max: RO<u16>,
    w: RO<i16>,
    req_inv_state: RO<u16>,
    req_w: RO<i16>,
    set_op: RW<u16>,
    set_inv_state: RW<u16>,
    ah_rtg_sf: RO<i16>,
    wh_rtg_sf: RO<i16>,
    w_cha_dis_cha_max_sf: RO<i16>,
    dis_cha_rte_sf: RO<i16>,
    soc_sf: RO<i16>,
    dod_sf: RO<i16>,
    soh_sf: RO<i16>,
    v_sf: RO<i16>,
    cell_v_sf: RO<i16>,
    a_sf: RO<i16>,
    a_max_sf: RO<i16>,
    w_sf: RO<i16>,
});

model!(Battery802View<'a> of BatteryState {
    id: RO u16,
    len: RO u16,
    ah_rtg: RO u16,
    wh_rtg: RO u16,
    w_cha_rte_max: RO u16,
    w_dis_cha_rte_max: RO u16,
    dis_cha_rte: RO u16,
    soc_np_max_pct: RW u16,
    soc_np_min_pct: RW u16,
    max_rsv_pct: RW u16,
    min_rsv_pct: RW u16,
    soc: RO u16,
    dod: RO u16,
    soh: RO u16,
    n_cyc: RO u32,
    cha_st: RO u16,
    loc_rem_ctl: RO u16,
    hb: RO u16,
    ctrl_hb: RW u16,
    alm_rst: RW u16,
    typ: RO u16,
    state: RO u16,
    state_vnd: RO u16,
    warr_dt: RO u32,
    evt1: RO u32,
    evt2: RO u32,
    evt_vnd1: RO u32,
    evt_vnd2: RO u32,
    v: RO u16,
    v_max: RO u16,
    v_min: RO u16,
    cell_v_max: RO u16,
    cell_v_max_str: RO u16,
    cell_v_max_mod: RO u16,
    cell_v_min: RO u16,
    cell_v_min_str: RO u16,
    cell_v_min_mod: RO u16,
    cell_v_avg: RO u16,
    a: RO i16,
    a_cha_max: RO u16,
    a_dis_cha_max: RO u16,
    w: RO i16,
    req_inv_state: RO u16,
    req_w: RO i16,
    set_op: RW u16,
    set_inv_state: RW u16,
    ah_rtg_sf: RO i16,
    wh_rtg_sf: RO i16,
    w_cha_dis_cha_max_sf: RO i16,
    dis_cha_rte_sf: RO i16,
    soc_sf: RO i16,
    dod_sf: RO i16,
    soh_sf: RO i16,
    v_sf: RO i16,
    cell_v_sf: RO i16,
    a_sf: RO i16,
    a_max_sf: RO i16,
    w_sf: RO i16,
});

model!(Battery802Blocks {
    // ID .. DisChaRte
    ro0: RO<RegU16Array<7>>,
    soc_np_max_pct: RW<RegU16>,
    soc_np_min_pct: RW<RegU16>,
    max_rsv_pct: RW<RegU16>,
    min_rsv_pct: RW<RegU16>,
    // SoC .. Hb
    ro1: RO<RegU16Array<8>>,
    ctrl_hb: RW<RegU16>,
    alm_rst: RW<RegU16>,
    // Typ .. ReqW
    ro2: RO<RegU16Array<29>>,
    set_op: RW<RegU16>,
    set_inv_state: RW<RegU16>,
    // scale factors
    ro3: RO<RegU16Array<12>>,
});

/// Number of iterations per measurement: the first argument, or [ITERATIONS].
fn iterations() -> u32 {
    std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(ITERATIONS)
}

/// Runs `f` [iterations] times and returns the mean duration in nanoseconds.
fn measure(mut f: impl FnMut()) -> f64 {
    let iterations = iterations();
    // warm up
    for _ in 0..iterations / 10 {
        f();
    }
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    start.elapsed().as_nanos() as f64 / iterations as f64
}

fn report(name: &str, times: [f64; 6]) {
    print!("{name:<28}");
    for time in times {
        print!(" {time:>8.1}");
    }
    println!();
}

/// Reads all registers of `model`.
fn snapshot(model: &impl ReadRegisterData) -> [u16; SPAN] {
    let mut buf = [0u16; SPAN];
    model.read(0, &mut buf).unwrap();
    buf
}

fn main() {
    let mut init = [0u16; SPAN];
    for (i, word) in init.iter_mut().enumerate() {
        *word = (i as u16).wrapping_mul(0x0101) ^ 0x8000;
    }

    let mut flat = init;
    let mut direct = Battery802::default();
    direct.load(&init);
    let mut prim = Battery802Prim::default();
    prim.load(&init);
    let mut state = BatteryState::default();
    state.load(&init);
    let mut view = Battery802View::new(&mut state);
    let mut blocks = Battery802Blocks::default();
    blocks.load(&init);

    assert_eq!(direct.register_span(), SPAN);
    assert_eq!(prim.register_span(), SPAN);
    assert_eq!(view.register_span(), SPAN);
    assert_eq!(blocks.register_span(), SPAN);
    assert_eq!(snapshot(&direct), init);
    assert_eq!(snapshot(&prim), init);
    assert_eq!(snapshot(&view), init);
    assert_eq!(snapshot(&blocks), init);

    println!(
        "{:<28} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "request [ns]", "flat", "dyn", "direct", "prim", "view", "blocks"
    );

    // (name, offset, count)
    let reads = [
        ("read all (64)", 0, SPAN),
        ("read first (1)", 0, 1),
        ("read middle Evt1 (2)", 26, 2),
        ("read last (1)", SPAN - 1, 1),
    ];
    for (name, offset, count) in reads {
        let mut bufs = [[0u16; SPAN]; 6];
        let [b_flat, b_dyn, b_direct, b_prim, b_view, b_blocks] = &mut bufs;
        let times = [
            measure(|| {
                read_words(black_box(&flat), black_box(offset), &mut b_flat[..count]).unwrap();
                black_box(&b_flat);
            }),
            measure(|| {
                black_box(&direct)
                    .read_dyn(black_box(offset), &mut b_dyn[..count])
                    .unwrap();
                black_box(&b_dyn);
            }),
            measure(|| {
                black_box(&direct)
                    .read(black_box(offset), &mut b_direct[..count])
                    .unwrap();
                black_box(&b_direct);
            }),
            measure(|| {
                black_box(&prim)
                    .read(black_box(offset), &mut b_prim[..count])
                    .unwrap();
                black_box(&b_prim);
            }),
            measure(|| {
                black_box(&view)
                    .read(black_box(offset), &mut b_view[..count])
                    .unwrap();
                black_box(&b_view);
            }),
            measure(|| {
                black_box(&blocks)
                    .read(black_box(offset), &mut b_blocks[..count])
                    .unwrap();
                black_box(&b_blocks);
            }),
        ];
        for buf in &bufs[1..] {
            assert_eq!(buf[..count], bufs[0][..count]);
        }
        report(name, times);
    }

    // SetOp and SetInvState
    let values = [1u16, 2];
    let offset = 50;
    let times = [
        measure(|| {
            write_words(
                black_box(&mut flat),
                black_box(offset),
                Registers::from(&values[..]),
            )
            .unwrap();
        }),
        measure(|| {
            black_box(&mut direct)
                .write_dyn(black_box(offset), Registers::from(&values[..]))
                .unwrap();
        }),
        measure(|| {
            black_box(&mut direct)
                .write(black_box(offset), Registers::from(&values[..]))
                .unwrap();
        }),
        measure(|| {
            black_box(&mut prim)
                .write(black_box(offset), Registers::from(&values[..]))
                .unwrap();
        }),
        measure(|| {
            black_box(&mut view)
                .write(black_box(offset), Registers::from(&values[..]))
                .unwrap();
        }),
        measure(|| {
            black_box(&mut blocks)
                .write(black_box(offset), Registers::from(&values[..]))
                .unwrap();
        }),
    ];
    report("write SetOp+SetInvState (2)", times);

    assert_eq!(flat[offset..offset + 2], values);
    assert_eq!(snapshot(&direct), flat);
    assert_eq!(snapshot(&prim), flat);
    assert_eq!(snapshot(&view), flat);
    assert_eq!(snapshot(&blocks), flat);

    // the view wrote through to the state (its borrow ends after its last use above)
    assert_eq!((state.set_op, state.set_inv_state), (1, 2));
}
