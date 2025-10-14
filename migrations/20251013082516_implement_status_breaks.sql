-- Add migration script here
CREATE TABLE statusbreaks (
    id SERIAL PRIMARY KEY,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    year INT NOT NULL,
    reason TEXT
);
