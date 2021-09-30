-- Your SQL goes here
CREATE TABLE workshopattachments
(
    workshop BIGINT UNSIGNED NOT NULL,
    attachment BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (workshop, attachment),
    FOREIGN KEY (workshop) REFERENCES workshops (id) ON DELETE CASCADE,
    FOREIGN KEY (attachment) REFERENCES attachments (id) ON DELETE CASCADE
);