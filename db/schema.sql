PRAGMA foreign_keys = ON;

-- mirrors the data in the config file.
-- if at startup this collection is not defined in the config file, error out.
-- * unless this collection is empty (no items), then delete it.
-- if at startup this section in the config is not in the database, insert it.
CREATE TABLE collections(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name VARCHAR(255) NOT NULL,
  type VARCHAR(16) NOT NULL,
  directory TEXT NOT NULL
);

-- This is the base table for movies, tvshows, season, episodes.
-- It contains info generic for any media type.
CREATE TABLE mediaitems (
  id integer PRIMARY KEY AUTOINCREMENT,
  collection_id INTEGER NOT NULL,
  path VARCHAR(255),
  deleted INTEGER DEFAULT 0 NOT NULL,
  type VARCHAR(20) NOT NULL,
  title VARCHAR(255),
  plot TEXT,
  tagline TEXT,
  dateadded TEXT,
  rating JSON NOT NULL DEFAULT "[]",
  thumb JSON NOT NULL DEFAULT "[]",
  fanart JSON NOT NULL DEFAULT "[]",
  uniqueids JSON NOT NULL DEFAULT "{}"
);

-- Extra info for thumbwalls.
CREATE TABLE mediaitems_extra (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  mediaitem_id INTEGER NOT NULL,

  -- now the things we can sort on.
  sorttitle VARCHAR(255) NOT NULL,
  added BIGINT,
  year INTEGER,
  rating REAL,
  votes REAL,
  genres TEXT,

  -- and some images
  poster INTEGER,
  thumb INTEGER,

  FOREIGN KEY(mediaitem_id) REFERENCES mediaitems(id)
);


CREATE TABLE movies(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  mediaitem_id INTEGER NOT NULL,

  -- common to movies and tvshows
  originaltitle TEXT,
  sorttitle TEXT,
  country JSON NOT NULL DEFAULT "[]",
  genre JSON NOT NULL DEFAULT "[]",
  studio JSON NOT NULL DEFAULT "[]",
  premiered TEXT,
  mpaa TEXT,
  actors JSON NOT NULL DEFAULT "[]",

  -- movie
  runtime INTEGER,
  credits JSON NOT NULL DEFAULT "[]",
  director JSON NOT NULL DEFAULT "[]",

  FOREIGN KEY(mediaitem_id) REFERENCES mediaitems(id)
);

CREATE TABLE tvshows(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  mediaitem_id INTEGER NOT NULL,

  -- common to movies and tvshows
  originaltitle TEXT,
  sorttitle TEXT,
  country JSON NOT NULL DEFAULT "[]",
  genre JSON NOT NULL DEFAULT "[]",
  studio JSON NOT NULL DEFAULT "[]",
  premiered TEXT,
  mpaa TEXT,
  actors JSON NOT NULL DEFAULT "[]",

  -- tvshow
  seasons INTEGER,
  episodes INTEGER,
  status TEXT,

  FOREIGN KEY(id) REFERENCES mediaitems(id)
);

CREATE TABLE episodes(
  id INTEGER PRIMARY KEY NOT NULL,
  mediaitem_id INTEGER NOT NULL,
  tvshow_id INTEGER NOT NULL,

 -- episode
  aired TEXT,
  runtime INTEGER,
  season INTEGER,
  episode INTEGER,
  displayseason INTEGER,
  displayepisode INTEGER,
  actors JSON NOT NULL DEFAULT "[]",
  credits JSON NOT NULL DEFAULT "[]",
  director JSON NOT NULL DEFAULT "[]",

  FOREIGN KEY(tvshow_id) REFERENCES mediaitems(id)
);

-- The seasons table exists for season-specific info.
-- For now, just images.
CREATE TABLE seasons(
  id INTEGER PRIMARY KEY NOT NULL,
  tvshow_id INTEGER NOT NULL,
  season INTEGER NOT NULL,

  FOREIGN KEY(tvshow_id) REFERENCES mediaitems(id)
);

CREATE TABLE images(
  id INTEGER PRIMARY KEY NOT NULL,
  mediaitem_id INTEGER NOT NULL,

  -- non-unique id (resized images have the same id).
  image_id INTEGER,

  -- art type (poster, thumb, fanart).
  arttype TEXT NOT NULL,

  -- dimensions
  width NOT NULL,
  height NOT NULL,

  -- location, and inode/size to detect changes.
  path TEXT,
  inode BIGINT,
  size INTEGER,

  FOREIGN KEY(mediaitem_id) REFERENCES mediaitems(id)
);
CREATE INDEX idx_images_image_id ON images(image_id);

CREATE TABLE uniqueids(
  id INTEGER PRIMARY KEY NOT NULL,
  mediaitem_id INTEGER NOT NULL,

  -- type is imdb, or tvdb, etc
  type TEXT NOT NULL,

  -- uniqueid is a imdb-id, or tvdb-id, etc.
  uniqueid TEXT NOT NULL,

  -- default INTEGER DEFAULT 0 NOT NULL,

  FOREIGN KEY(mediaitem_id) REFERENCES mediaitems(id)
);

CREATE TABLE actors_in_item(
  id INTEGER PRIMARY KEY NOT NULL,
  mediaitem_id INTEGER NOT NULL,

  name TEXT NOT NULL,
  role TEXT,
  order_prio INTEGER,
  thumb_id INTEGER,

  FOREIGN KEY(mediaitem_id) REFERENCES mediaitems(id)
);

/*
CREATE TABLE genres(
);

CREATE TABLE actors(
);

CREATE TABLE genres(
);
*/
