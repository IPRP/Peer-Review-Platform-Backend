-- This file should undo anything in `up.sql`
ALTER TABLE workshops
    DROP COLUMN reviewtimespan;

ALTER TABLE submissions
    DROP COLUMN deadline;