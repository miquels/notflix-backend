use std::collections::HashMap;
use once_cell::sync::Lazy;

static GENRES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| HashMap::from([
    ( "absurdist",            "Absurdist" ),
    ( "action",               "Action" ),
    ( "adventure",            "Adventure" ),
    ( "animation",            "Animation" ),
    ( "biography",            "Biography" ),
    ( "children",             "Children" ),
    ( "comedy",               "Comedy" ),
    ( "crime",                "Crime" ),
    ( "disaster",             "Disaster" ),
    ( "documentary",          "Documentary" ),
    ( "drama",                "Drama" ),
    ( "erotic",               "Erotic" ),
    ( "family",               "Family" ),
    ( "fantasy",              "Fantasy" ),
    ( "film noir",            "Film Noir" ),
    ( "film-noir",            "Film Noir" ),
    ( "foreign",              "Foreign" ),
    ( "game show",            "Game Show" ),
    ( "game-show",            "Game Show" ),
    ( "historical",           "Historical" ),
    ( "history",              "History" ),
    ( "holiday",              "Holiday" ),
    ( "horror",               "Horror" ),
    ( "indie",                "Indie" ),
    ( "mini series",          "Mini Series" ),
    ( "mini-series",          "Mini Series" ),
    ( "music",                "Music" ),
    ( "musical",              "Musical" ),
    ( "mystery",              "Mystery" ),
    ( "news",                 "News" ),
    ( "philosophical",        "Philosophical" ),
    ( "political",            "Political" ),
    ( "reality",              "Reality" ),
    ( "romance",              "Romance" ),
    ( "satire",               "Satire" ),
    ( "sci fi",               "Sci-Fi" ),
    ( "sci-fi",               "Sci-Fi" ),
    ( "science fiction",      "Sci-Fi" ),
    ( "science-fiction",      "Sci-Fi" ),
    ( "short",                "Short" ),
    ( "soap",                 "Soap" ),
    ( "sport",                "Sports" ),
    ( "sports",               "Sports" ),
    ( "sports film",          "Sports" ),
    ( "sports-film",          "Sports" ),
    ( "surreal",              "Surreal" ),
    ( "suspense",             "Suspense" ),
    ( "tv movie",             "TV Movie" ),
    ( "tv-movie",             "TV Movie" ),
    ( "talk show",            "Talk Show" ),
    ( "talk-show",            "Talk Show" ),
    ( "telenovela",           "Telenovela" ),
    ( "thriller",             "Thriller" ),
    ( "urban",                "Urban" ),
    ( "war",                  "War" ),
    ( "western",              "Western" ),
]));

pub fn normalize_genre(genre: &str) -> &'_ str {
    GENRES.get(genre.to_lowercase().as_str()).map(|s| *s).unwrap_or(genre)
}

pub fn normalize_genres(genres: &[String]) -> Vec<String> {
    let mut v = genres.iter().map(|g| normalize_genre(g).to_string()).collect::<Vec<_>>();
    v.sort();
    v.dedup();
    v
}
