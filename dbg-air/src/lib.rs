use std::borrow::{Borrow, BorrowMut};

use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::PrimeField64;
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrix;
use rand::Rng;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ToyCols<T> {
    pub cond_a: T,
    pub cond_b: T,
    pub c: T,
}

pub const NUM_TOY_COLS: usize = size_of::<ToyCols<u8>>();

impl<T> Borrow<ToyCols<T>> for [T] {
    fn borrow(&self) -> &ToyCols<T> {
        debug_assert_eq!(self.len(), NUM_TOY_COLS);
        let (prefix, shorts, suffix) = unsafe { self.align_to::<ToyCols<T>>() };
        debug_assert!(prefix.is_empty(), "Alignment should match");
        debug_assert!(suffix.is_empty(), "Alignment should match");
        debug_assert_eq!(shorts.len(), 1);
        &shorts[0]
    }
}

impl<T> BorrowMut<ToyCols<T>> for [T] {
    fn borrow_mut(&mut self) -> &mut ToyCols<T> {
        debug_assert_eq!(self.len(), NUM_TOY_COLS);
        let (prefix, shorts, suffix) = unsafe { self.align_to_mut::<ToyCols<T>>() };
        debug_assert!(prefix.is_empty(), "Alignment should match");
        debug_assert!(suffix.is_empty(), "Alignment should match");
        debug_assert_eq!(shorts.len(), 1);
        &mut shorts[0]
    }
}

pub struct ToyAir;

impl<F> BaseAir<F> for ToyAir {
    fn width(&self) -> usize {
        NUM_TOY_COLS
    }
}

impl<AB: AirBuilder> Air<AB> for ToyAir {
    #[inline]
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let &local: &ToyCols<AB::Var> = (*local).borrow();
        builder
            .when_transition()
            .when(local.cond_a)
            .when(local.cond_b)
            .assert_zero(local.c);
    }
}

pub fn generate_trace<F: PrimeField64>(len: usize) -> RowMajorMatrix<F> {
    let rng = &mut rand::thread_rng();
    let mut data = vec![];
    for i in 0..len {
        let cond_a = rng.gen_bool(0.5);
        let cond_b = rng.gen_bool(0.5);
        let c = if i != len - 1 {
            if cond_a && cond_b {
                0
            } else {
                rng.gen_range(1..1 << 20)
            }
        } else if cond_a && cond_b {
            rng.gen_range(1..1 << 20)
        } else {
            0
        };
        data.extend(
            [cond_a as u64, cond_b as u64, c]
                .into_iter()
                .map(F::from_canonical_u64),
        );
    }
    RowMajorMatrix::new(data, NUM_TOY_COLS)
}
