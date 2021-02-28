-- noinspection SqlNoDataSourceInspectionForFile

CREATE TABLE file_details
(
    id        INTEGER PRIMARY KEY,
    file_path TEXT NOT NULL UNIQUE,
    data      BLOB NOT NULL,
    timestamp INT  NOT NULL
);

CREATE TABLE file_diffs
(
    id               INTEGER PRIMARY KEY,
    original_file_id INTEGER,
    file_path        TEXT NOT NULL,
    change_event     TEXT NOT NULL,
    data             BLOB NOT NULL,
    timestamp        INT  NOT NULL,
    FOREIGN KEY (original_file_id) REFERENCES file_details (id)
);