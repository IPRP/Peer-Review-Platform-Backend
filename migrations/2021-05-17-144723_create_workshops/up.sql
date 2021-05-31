-- Your SQL goes here
CREATE TABLE workshops
(
    id        SERIAL PRIMARY KEY,
    title     VARCHAR(100) NOT NULL,
    content   TEXT         NOT NULL,
    end       DATETIME     NOT NULL,
    anonymous BOOLEAN      NOT NULL
);

CREATE TABLE criterion
(
    id      SERIAL PRIMARY KEY,
    title   VARCHAR(100)                                       NOT NULL,
    content TEXT                                               NOT NULL,
    weight  DOUBLE,
    kind    enum ('point', 'grade', 'percentage', 'truefalse') NOT NULL
);

CREATE TABLE criteria
(
    workshop  BIGINT UNSIGNED NOT NULL,
    criterion BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (workshop, criterion),
    FOREIGN KEY (workshop) REFERENCES workshops (id) ON DELETE CASCADE,
    FOREIGN KEY (criterion) REFERENCES criterion (id) ON DELETE CASCADE
);

CREATE TABLE workshoplist
(
    workshop BIGINT UNSIGNED             NOT NULL,
    user     BIGINT UNSIGNED             NOT NULL,
    role     enum ('student', 'teacher') NOT NULL,
    PRIMARY KEY (workshop, user),
    FOREIGN KEY (workshop) REFERENCES workshops (id) ON DELETE CASCADE,
    FOREIGN KEY (user) REFERENCES users (id) ON DELETE CASCADE
);