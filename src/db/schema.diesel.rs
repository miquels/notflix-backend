table! {
    images (id) {
        id -> Nullable<Integer>,
        ino -> BigInt,
        dev -> BigInt,
        size -> BigInt,
        mtime -> BigInt,
        width -> Integer,
        height -> Integer,
    }
}

table! {
    rsimages (id) {
        id -> Nullable<Integer>,
        image_id -> BigInt,
        width -> Integer,
        height -> Integer,
        quality -> Integer,
        path -> Text,
    }
}

joinable!(rsimages -> images (image_id));

allow_tables_to_appear_in_same_query!(
    images,
    rsimages,
);
