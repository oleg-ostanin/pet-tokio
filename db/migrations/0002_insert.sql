create table if not exists book (
  isbn varchar not null primary key,
  title varchar not null,
  author varchar not null
);
