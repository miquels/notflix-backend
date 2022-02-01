# Database model

See [database.sql](database.sql).

We maintain a list of movies / tv-series in the database. The
specific info stored is the minimum we need to:

- show the thumbs on the thumbwall
- sort the thumbs
- filter the thumbs

When more specific info for a tv show or movie is needed, we
load it on demand from the filesystem (and LRU cache it in memory).

Each item has a 'lastupdate' value. With that we can optimize checking
entries in the 'seen' table. Only if an entry in the 'seen' table has
a lower value than the item we need to update.

We can in fact use this on the client to poll for updates from the server,
in a 'last-modified-since' sense.

# Image resizer.

The image resizing service also uses a few tables in the database.

The `images` table keeps a list of original images, based on their
dev/ino/size/mtime. This means that if a request comes for a resized
image, we do have to stat() the original file once to get at the
database 'key' (an index exists in dev/ino/size/mtime).

Then we check in the `rsimages table if we already have an image resized
to the width/height/quality of the request. If so - and the file
exists(!) fine, return the contents of that file. If not, create
a new resized image, store it on the file system and in the database,
and return it.

