-- Your SQL goes here
CREATE TABLE users (
   id SERIAL PRIMARY KEY,
   username VARCHAR(100) NOT NULL,
   firstname VARCHAR(100) NOT NULL,
   lastname VARCHAR(100) NOT NULL,
   password VARCHAR(100) NOT NULL,
   role VARCHAR(100) NOT NULL,
   unit VARCHAR(100),
   UNIQUE (username)
)