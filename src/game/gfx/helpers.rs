/// Return a deterministic pseudo-random value in [0,1) from a seed.
pub(crate) fn hash_to_unit(seed: u32) -> f32 {
    let mut value = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    value = (value >> ((value >> 28) + 4)) ^ value;
    value = value.wrapping_mul(277_803_737);
    (((value >> 22) ^ value) as f32) / (u32::MAX as f32)
}

