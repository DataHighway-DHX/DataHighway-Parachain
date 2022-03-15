//! Benchmarking setup

use super::*;

#[allow(unused)]
use crate::Pallet as MiningClaimsToken;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {}

impl_benchmark_test_suite!(MiningClaimsToken, crate::mock::new_test_ext(), crate::mock::Test,);
