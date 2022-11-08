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
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  mediaitem_id INTEGER NOT NULL,

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
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  mediaitem_id INTEGER NOT NULL,

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

  FOREIGN KEY(id) REFERENCES mediaitems(id)
);

CREATE TABLE episodes(
  id INTEGER PRIMARY KEY NOT NULL,
  mediaitem_id INTEGER NOT NULL,
  tvshow_id INTEGER NOT NULL,

 -- episode
  video JSON NOT NULL,
  aired TEXT,
  runtime INTEGER,
  season INTEGER NOT NULL,
  episode INTEGER NOT NULL,
  displayseason INTEGER,
  displayepisode INTEGER,
  thumbs JSON NOT NULL DEFAULT "[]",

  FOREIGN KEY(id) REFERENCES mediaitems(id),
  FOREIGN KEY(tvshow_id) REFERENCES mediaitems(id)
);

CREATE TABLE images(
  id INTEGER PRIMARY KEY NOT NULL,
  mediaitem_id INTEGER NOT NULL,

  -- non-unique id (resized images have the same id).
  image_id INTEGER NOT NULL,

  -- art type (poster, thumb, fanart).
  aspect TEXT NOT NULL,

  -- dimensions
  width NOT NULL,
  height NOT NULL,

  -- location, and inode/size to detect changes.
  path JSON NOT NULL,

  -- extra info. E.g. for seasons, season thumb or name.
  extra JSON,

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
