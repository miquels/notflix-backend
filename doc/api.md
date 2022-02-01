## collections.
```
GET /api/v1/collections
[
  {
    "id": 1,
    "name": "TV Shows",
    "type: "series",
    "baseurl": "/data/1"
  },
  {
    "id": 2,
    "name": "Movies",
    "type: "movie",
    "baseurl": "/data/2"
  }
]
```

## items in a collection.
```
GET /api/v1/collection/2/items
[
  {
    "id": 8,
    "title": "Pippi Langkous (1957)",
    "path": "Pippi%20Langkous%20(1957)",
    "poster": "lifting-a-horse.jpg",
  },
  {
    "id": 66,
    "title": "Bassie en Andriaan en het Spook (1984)"
    "poster": "clowns.jpg",
  }
]
```

# movie details
```
GET /api/v1/collection/2/item/66
{
  "id": 66,
  "title": "Bassie en Andriaan en het Spook (1984)",
  "poster": "clowns.jpg",
  "plot": "Not remarkable",
  "year": 1984,
}
```

# series details
```
GET /api/v1/collection/1/item/22
{
  "id": 22,
  "title": "Night Rider",
  "poster": "car.jpg",
  "plot": "not really",
  "seasons"" [
    {
      season: 1,
      episodes: [
        {
	  episode: 1,
	  "video": "S01/and.so.it.begins.mp4",
	}
      ]
    }
  ]
}
```
