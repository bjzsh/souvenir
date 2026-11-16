-- Runs once, when the container initializes an empty data directory.
-- template1 first, so databases created later (including by tests) start
-- with the extension; then the database the entrypoint created.
\set target_db :DBNAME
\connect template1
CREATE EXTENSION IF NOT EXISTS souvenir;
\connect :"target_db"
CREATE EXTENSION IF NOT EXISTS souvenir;
