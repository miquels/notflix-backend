CREATE TABLE items(
  name TEXT NOT NULL PRIMARY KEY,
  votes BIGINT,
  year BIGINT,
  genre TEXT NOT NULL,
  rating REAL,
  nfotime BIGINT NOT NULL,
  firstvideo BIGINT NOT NULL,
  lastvideo BIGINT NOT NULL
);
