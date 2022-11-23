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

-- This is the base table for movies, tvshows, season, episodes.
-- It contains info generic for any media type.
CREATE TABLE mediaitems (
  -- AUTOINCREMENT is important, we should not re-use mediaitems.id.
  id TEXT PRIMARY KEY NOT NULL,
  collection_id TEXT NOT NULL,
  -- unix timestamp of anything contained in this item.
  lastmodified BIGINT NOT NULL,
  -- directory is a FileInfo, path + inode + size.
  directory JSON NOT NULL,
  deleted INTEGER DEFAULT 0 NOT NULL,
  type VARCHAR(20) NOT NULL,
  nfofile JSON,
  title VARCHAR(255),
  plot TEXT,
  tagline TEXT,
  dateadded TEXT,
  -- ratings MIGHT move to the ratings table
  ratings JSON NOT NULL DEFAULT "[]",
  -- thumb and fanart will move to the `images` table which will backref
  thumbs JSON NOT NULL DEFAULT "[]",
  -- uniqueids will move to the `uniqueids` table which will backref
  uniqueids JSON NOT NULL DEFAULT "{}",
  -- this might all move to a `credits` table
  actors JSON NOT NULL DEFAULT "[]",
  credits JSON NOT NULL DEFAULT "[]",
  directors JSON NOT NULL DEFAULT "[]"
);

CREATE TABLE movies(
  id TEXT PRIMARY KEY NOT NULL,
  mediaitem_id TEXT NOT NULL,

  -- common to movies and tvshows
  originaltitle TEXT,
  sorttitle TEXT,
  countries JSON NOT NULL DEFAULT "[]",
  genres JSON NOT NULL DEFAULT "[]",
  studios JSON NOT NULL DEFAULT "[]",
  premiered TEXT,
  mpaa TEXT,

  -- movie
  video JSON NOT NULL,
  runtime INTEGER,

  FOREIGN KEY(mediaitem_id) REFERENCES mediaitems(id)
);

CREATE TABLE tvshows(
  id TEXT PRIMARY KEY NOT NULL,
  mediaitem_id TEXT NOT NULL,

  -- common to movies and tvshows
  originaltitle TEXT,
  sorttitle TEXT,
  countries JSON NOT NULL DEFAULT "[]",
  genres JSON NOT NULL DEFAULT "[]",
  studios JSON NOT NULL DEFAULT "[]",
  premiered TEXT,
  mpaa TEXT,

  -- tvshow
  seasons INTEGER,
  episodes INTEGER,
  status TEXT,

  FOREIGN KEY(mediaitem_id) REFERENCES mediaitems(id)
);

CREATE TABLE episodes(
  id TEXT PRIMARY KEY NOT NULL,
  mediaitem_id TEXT NOT NULL,
  tvshow_id TEXT NOT NULL,

 -- episode
  video JSON NOT NULL,
  aired TEXT,
  runtime INTEGER,
  season INTEGER NOT NULL,
  episode INTEGER NOT NULL,
  displayseason INTEGER,
  displayepisode INTEGER,
  thumbs JSON NOT NULL DEFAULT "[]",

  FOREIGN KEY(mediaitem_id) REFERENCES mediaitems(id),
  FOREIGN KEY(tvshow_id) REFERENCES mediaitems(id)
);

CREATE TABLE images(
  id TEXT PRIMARY KEY NOT NULL,
  collection_id TEXT NOT NULL,
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

/*
CREATE TABLE actors_in_item(
  id INTEGER PRIMARY KEY,
  mediaitem_id INTEGER NOT NULL,

  name TEXT NOT NULL,
  role TEXT,
  order_prio INTEGER,
  thumb_id INTEGER,

  FOREIGN KEY(mediaitem_id) REFERENCES mediaitems(id)
);
*/

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
