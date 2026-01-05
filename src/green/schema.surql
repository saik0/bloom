-- GREEN NODES (content-addressed)
DEFINE TABLE node SCHEMAFULL;
DEFINE FIELD content_hash ON node TYPE string;
DEFINE FIELD syntax_id ON node TYPE int;
DEFINE FIELD span ON node TYPE object;
DEFINE FIELD data ON node TYPE object;

-- Indices
DEFINE INDEX content_hash_idx ON node FIELDS content_hash UNIQUE;
DEFINE INDEX syntax_id_idx ON node FIELDS syntax_id;

-- GRAPH EDGES (tree structure)
DEFINE TABLE child_of SCHEMAFULL TYPE RELATION FROM node TO node;
DEFINE FIELD index ON child_of TYPE int;

-- Indices
DEFINE INDEX parent_idx ON child_of FIELDS in;
DEFINE INDEX child_idx ON child_of FIELDS out;

-- INTERNED STRINGS
DEFINE TABLE intern_string SCHEMAFULL;
DEFINE FIELD text ON intern_string TYPE string;

-- Index
DEFINE INDEX text_unique ON intern_string FIELDS text UNIQUE;
