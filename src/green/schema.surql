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

-- IDENTITY (Salsa/NodeId ↔ content hash)
DEFINE TABLE identity SCHEMAFULL;
DEFINE FIELD salsa_id ON identity TYPE string;
DEFINE FIELD content_hash ON identity TYPE string;
DEFINE FIELD path ON identity TYPE array;
DEFINE FIELD parent ON identity TYPE option<string>;
DEFINE FIELD file ON identity TYPE option<string>;

-- Indices
DEFINE INDEX salsa_id_unique ON identity FIELDS salsa_id UNIQUE;
DEFINE INDEX content_hash_idx ON identity FIELDS content_hash;
DEFINE INDEX file_idx ON identity FIELDS file;

-- SOURCE ROOT (file → current root content hash)
DEFINE TABLE source_root SCHEMAFULL;
DEFINE FIELD path ON source_root TYPE string;
DEFINE FIELD root_hash ON source_root TYPE string;
DEFINE INDEX source_root_unique ON source_root FIELDS path UNIQUE;

-- IDENTITY ↔ NODE link
DEFINE TABLE identity_of SCHEMAFULL TYPE RELATION FROM identity TO node;
