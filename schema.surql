-- GREEN NODES (content-addressed)
-- Node ID is the content hash - same content = same ID = automatic deduplication
DEFINE TABLE nodes SCHEMAFULL;
DEFINE FIELD kind ON nodes TYPE string;
DEFINE FIELD text ON nodes TYPE string;
DEFINE FIELD span_start ON nodes TYPE int;
DEFINE FIELD span_len ON nodes TYPE int;
DEFINE FIELD parent ON nodes TYPE option<string>;
DEFINE FIELD children ON nodes TYPE array<string>;
DEFINE FIELD child_index ON nodes TYPE int;

-- Note: The record ID itself is the content hash, so no separate content_hash field needed
-- This enables deduplication: inserting the same content twice is a no-op

-- INTERNED STRINGS (for future use - token text deduplication)
DEFINE TABLE intern_string SCHEMAFULL;
DEFINE FIELD text ON intern_string TYPE string;
DEFINE INDEX text_unique ON intern_string FIELDS text UNIQUE;
