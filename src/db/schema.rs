table! {
    items (name) {
        name -> Text,
        votes -> Nullable<BigInt>,
        year -> Nullable<BigInt>,
        genre -> Text,
        rating -> Nullable<Float>,
        nfotime -> BigInt,
        firstvideo -> BigInt,
        lastvideo -> BigInt,
    }
}
