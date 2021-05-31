-- Your SQL goes here
CREATE TABLE attachments
(
    id    SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    owner BIGINT UNSIGNED,
    FOREIGN KEY (owner) REFERENCES users (id) ON DELETE SET NULL
);

CREATE TABLE submissions
(
    id          SERIAL PRIMARY KEY,
    title       VARCHAR(255)    NOT NULL,
    comment     TEXT            NOT NULL,
    student     BIGINT UNSIGNED,
    workshop    BIGINT UNSIGNED NOT NULL,
    date        DATETIME        NOT NULL,
    locked      BOOL            NOT NULL,
    reviewsdone BOOL            NOT NULL,
    error       BOOl            NOT NULL,
    meanpoints  DOUBLE,
    maxpoint    DOUBLE,
    FOREIGN KEY (student) REFERENCES users (id) ON DELETE SET NULL,
    FOREIGN KEY (workshop) REFERENCES workshops (id) ON DELETE CASCADE
);

CREATE TABLE submissioncriteria
(
    submission BIGINT UNSIGNED NOT NULL,
    criterion  BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (submission, criterion),
    FOREIGN KEY (submission) REFERENCES submissions (id) ON DELETE CASCADE,
    FOREIGN KEY (criterion) REFERENCES criterion (id) ON DELETE CASCADE
);

CREATE TABLE submissionattachments
(
    submission BIGINT UNSIGNED NOT NULL,
    attachment BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (submission, attachment),
    FOREIGN KEY (submission) REFERENCES submissions (id) ON DELETE CASCADE,
    FOREIGN KEY (attachment) REFERENCES attachments (id) ON DELETE CASCADE
);

CREATE TABLE reviews
(
    id         SERIAL PRIMARY KEY,
    feedback   TEXT            NOT NULL,
    reviewer   BIGINT UNSIGNED,
    submission BIGINT UNSIGNED NOT NULL,
    workshop   BIGINT UNSIGNED NOT NULL,
    deadline   DATETIME        NOT NULL,
    done       BOOL            NOT NULL,
    locked     BOOL            NOT NULL,
    error      BOOl            NOT NULL,
    FOREIGN KEY (reviewer) REFERENCES users (id) ON DELETE SET NULL,
    FOREIGN KEY (submission) REFERENCES submissions (id) ON DELETE CASCADE,
    FOREIGN KEY (workshop) REFERENCES workshops (id) ON DELETE CASCADE
);

CREATE TABLE reviewpoints
(
    review    BIGINT UNSIGNED NOT NULL,
    criterion BIGINT UNSIGNED NOT NULL,
    points    DOUBLE,
    PRIMARY KEY (review, criterion),
    FOREIGN KEY (review) REFERENCES submissions (id) ON DELETE CASCADE,
    FOREIGN KEY (criterion) REFERENCES criterion (id) ON DELETE CASCADE
);