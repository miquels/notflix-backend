PRAGMA foreign_keys = ON;

-- mirrors the data in the config file.
-- if at startup this collection is not defined in the config file, error out.
-- * unless this collection is empty (no items), then delete it.
-- if at startup this section in the config is not in the database, insert it.
CREATE TABLE collections(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  directory TEXT NOT NULL,
);

-- movie or series.
CREATE TABLE items(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  collection_id INTEGER NOT NULL,
  deleted INTEGER DEFAULT 0 NOT NULL,

  -- data for the thumb wall
  title TEXT NOT NULL,
  path TEXT NOT NULL,
  poster TEXT

  -- now the things we can sort on.
  firstvideo BIGINT,
  lastvideo BIGINT,
  year INTEGER,
  rating REAL,
  genres TEXT,

  -- need this for 'seen'.
  lastupdate BIGINT NOT NULL,

  FOREIGN KEY(collection_id) REFERENCES collections(id)
);

-- external IDs to internal ID mapping. This helps when an item
-- is renamed, or removed and later restored.
CREATE TABLE uniqueid(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  item_id INTEGER NOT NULL,
  -- imdb, tvdb, etc
  ext_name TEXT NOT NULL,
  -- id as defined by imdb, tvdb, etc
  ext_id TEXT NOT NULL,

  FOREIGN KEY(item_id) REFERENCES items(id)
);

-- user.
CREATE TABLE users(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  username TEXT NOT NULL,
);

-- movie or tv series marked as favorite.
CREATE TABLE favorites(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT_NULL,
  item_id INTEGER NOT NULL,

  FOREIGN KEY(user_id) REFERENCES users(id),
  FOREIGN KEY(item_id) REFERENCES items(id)
);

-- Which items we've (partly) seen.
CREATE TABLE seen(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  item_id INTEGER NOT NULL,
  season INTEGER,
  episode INTEGER,
  paused_at INTEGER,
  ended integer DEFAULT FALSE NOT NULL,
  lastupdate BIGINT NOT NULL,

  FOREIGN KEY(item_id) REFERENCES items(id)
);

-- Our image resizing service.
-- first the original images.
CREATE TABLE images(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ino BIGINT NOT NULL,
  dev BIGINT NOT NULL,
  size BIGINT NOT NULL,
  mtime BIGINT NOT NULL,
  width INTEGER NOT NULL,
  height INTEGER NOT NULL
);
CREATE INDEX images_idx ON images(ino, dev, size, mtime);

-- then the resized images.
CREATE TABLE rsimages(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  image_id INTEGER NOT NULL,
  width INTEGER NOT NULL,
  height INTEGER NOT NULL,
  quality INTEGER DEFAULT 100 NOT NULL,
  path TEXT NOT NULL,

  FOREIGN KEY(image_id) REFERENCES images(id)
);
