CREATE TABLE IF NOT EXISTS "book_storage" (
  book_id BIGINT UNIQUE NOT NULL REFERENCES book_info(id),
  quantity BIGINT NOT NULL,
  updated_at timestamp with time zone NOT NULL DEFAULT now()
);
