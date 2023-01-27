PRAGMA foreign_keys = ON;

-- mirrors the data in the config file.
-- if at startup this collection is not defined in the config file, error out.
-- * unless this collection is empty (no items), then delete it.
-- if at startup this section in the config is not in the database, insert it.
CREATE TABLE collections(
  id INTEGER PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  type VARCHAR(16) NOT NULL,
  directory TEXT NOT NULL
);

-- This table contains the info for a movie, tvshow or episode.
CREATE TABLE mediaitems (
  id TEXT PRIMARY KEY NOT NULL,
  collection_id TEXT NOT NULL,
  -- movie / tvshow / episode.
  type VARCHAR(20) NOT NULL,
  -- unix timestamp of the file/item with latest modification date.
  lastmodified BIGINT NOT NULL,
  -- date added in YYYY-MM-DD.
  dateadded VARCHAR(10) NOT NULL,
  -- directory relative to collection directory (FileInfo).
  -- can be NUL because episodes don't have a specific directory.
  directory JSON,
  -- was this item deleted.
  deleted INTEGER DEFAULT 0 NOT NULL,

  -- title.
  title TEXT,

  -- nfo
  nfo_file JSON,
  nfo_info JSON,

  -- images.
  thumbs JSON NOT NULL DEFAULT "[]",

  -- subtitles.
  subtitles JSON NOT NULL DEFAULT "[]",

  -- video. can be NULL if this is a tvshow.
  video_file JSON,
  video_info JSON,

  -- for episodes.
  season INTEGER,
  episode INTEGER,
  tvshow_id TEXT
);

CREATE TABLE images(
  id INTEGER PRIMARY KEY,
  collection_id INTEGER NOT NULL,
  mediaitem_id TEXT NOT NULL,

  -- Variants have the same image_id. Original has id == image_id.
  image_id TEXT NOT NULL,

  -- path, mtime, size.
  fileinfo JSON NOT NULL,

  -- art type (poster, thumb, fanart).
  aspect TEXT NOT NULL,

  -- dimensions
  width INTEGER NOT NULL,
  height INTEGER NOT NULL,
  quality INTEGER NOT NULL DEFAULT 100,

  -- extra info. E.g. for seasons, season thumb or name.
  extra JSON,

  FOREIGN KEY(mediaitem_id) REFERENCES mediaitems(id)
  FOREIGN KEY(image_id) REFERENCES images(id)
);
CREATE INDEX idx_images_image_id ON images(image_id);

CREATE TABLE uniqueids(
  id INTEGER PRIMARY KEY,
  mediaitem_id TEXT NOT NULL,

  -- type is imdb, or tvdb, etc
  idtype TEXT NOT NULL,

  -- uniqueid is a imdb-id, or tvdb-id, etc.
  uniqueid TEXT NOT NULL,

  is_default INTEGER DEFAULT 0 NOT NULL

  -- FOREIGN KEY(mediaitem_id) REFERENCES mediaitems(id)
);
CREATE UNIQUE INDEX uniqueids_idx ON uniqueids(idtype, uniqueid);

CREATE TABLE users(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  username TEXT NOT NULL,
  password TEXT NOT NULL,
  email TEXT
);

CREATE TABLE sessions(
  id INTEGER PRIMARY KEY,
  user_id INTEGER NOT NULL,
  sessionid TEXT NOT NULL,
  created TEXT NOT NULL,
  updated TEXT NOT NULL,
  data TEXT,

  FOREIGN KEY(user_id) REFERENCES users(id)
);
